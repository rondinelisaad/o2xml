use std::{collections::HashMap, fs, path::PathBuf, process::Stdio, time::Instant};

use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::Command,
    time::timeout,
};
use tracing::info;
use xml_domain::Revision;
use xml_resource::{CatalogResolver, ResolvedResource};

use crate::{capture_bounded_stderr, TransformError, TransformReport, MAX_OUTPUT_BYTES};

const MAX_RESOURCE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_RESOURCE_GRAPH_BYTES: u64 = 64 * 1024 * 1024;
const MAX_RESOURCES: usize = 64;

#[derive(Clone, Copy)]
pub struct LibxsltRequest<'a> {
    pub source: &'a str,
    pub revision: Revision,
    pub catalog: &'a CatalogResolver,
    pub stylesheet_uri: &'a str,
}

#[derive(Clone, Debug)]
pub struct LibxsltProcessEngine {
    executable: PathBuf,
    deadline: std::time::Duration,
}

impl LibxsltProcessEngine {
    pub fn new(executable: impl Into<PathBuf>) -> Result<Self, TransformError> {
        let executable = fs::canonicalize(executable.into()).map_err(TransformError::Io)?;
        if !executable.is_file() {
            return Err(TransformError::InvalidAsset(executable));
        }
        Ok(Self {
            executable,
            deadline: std::time::Duration::from_secs(30),
        })
    }

    #[must_use]
    pub fn with_deadline(mut self, deadline: std::time::Duration) -> Self {
        self.deadline = deadline.max(std::time::Duration::from_millis(1));
        self
    }

    pub async fn transform(
        &self,
        request: LibxsltRequest<'_>,
    ) -> Result<TransformReport, TransformError> {
        let started = Instant::now();
        let result = self.run(request).await;
        info!(
            audit = true,
            action = "transform.run",
            result = if result.is_ok() { "success" } else { "failure" },
            resource = "document",
            revision = request.revision.0,
            engine = "libxslt-process",
            duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        );
        result
    }

    async fn run(&self, request: LibxsltRequest<'_>) -> Result<TransformReport, TransformError> {
        let root = request
            .catalog
            .resolve_uri_or_direct(request.stylesheet_uri)?;
        let mut child = Command::new(&self.executable)
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(TransformError::Spawn)?;
        let mut stdin = child.stdin.take().ok_or(TransformError::MissingStdin)?;
        let stdout = child.stdout.take().ok_or(TransformError::MissingStdout)?;
        let stderr = child.stderr.take().ok_or(TransformError::MissingStderr)?;
        let header = serde_json::to_vec(&HelperRequest {
            mode: "transform",
            grammar_uri: request.stylesheet_uri,
            public_id: None,
            document_bytes: request.source.len(),
            expression: None,
        })
        .map_err(|error| TransformError::Protocol(error.to_string()))?;
        stdin
            .write_all(&header)
            .await
            .map_err(TransformError::Write)?;
        stdin
            .write_all(b"\n")
            .await
            .map_err(TransformError::Write)?;
        stdin
            .write_all(request.source.as_bytes())
            .await
            .map_err(TransformError::Write)?;
        stdin
            .write_all(b"\n")
            .await
            .map_err(TransformError::Write)?;
        stdin.flush().await.map_err(TransformError::Write)?;

        let stderr_reader = tokio::spawn(capture_bounded_stderr(stderr));
        let output = match timeout(
            self.deadline,
            broker_transform(
                BufReader::new(stdout),
                &mut stdin,
                request.catalog,
                request.stylesheet_uri,
                root,
            ),
        )
        .await
        {
            Ok(result) => result?,
            Err(_) => {
                stderr_reader.abort();
                let _ = child.kill().await;
                return Err(TransformError::Timeout(self.deadline));
            }
        };
        drop(stdin);
        let status = match timeout(self.deadline, child.wait()).await {
            Ok(status) => status.map_err(TransformError::Wait)?,
            Err(_) => {
                stderr_reader.abort();
                let _ = child.kill().await;
                return Err(TransformError::Timeout(self.deadline));
            }
        };
        let (stderr, stderr_truncated) = stderr_reader.await.map_err(TransformError::Task)??;
        if !status.success() {
            return Err(TransformError::EngineFailed(
                String::from_utf8_lossy(&stderr)
                    .chars()
                    .take(8 * 1024)
                    .collect(),
            ));
        }
        Ok(TransformReport {
            revision: request.revision,
            output,
            stderr_truncated,
        })
    }
}

#[derive(Serialize)]
struct HelperRequest<'a> {
    mode: &'static str,
    grammar_uri: &'a str,
    public_id: Option<&'a str>,
    document_bytes: usize,
    expression: Option<&'a str>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum HelperMessage {
    Resource {
        url: String,
        public_id: Option<String>,
    },
    Result {
        valid: bool,
        output_bytes: Option<usize>,
    },
}

#[derive(Serialize)]
struct ResourceReply<'a> {
    allowed: bool,
    uri: Option<&'a str>,
    bytes: usize,
}

async fn broker_transform(
    mut stdout: BufReader<tokio::process::ChildStdout>,
    stdin: &mut tokio::process::ChildStdin,
    catalog: &CatalogResolver,
    stylesheet_uri: &str,
    root: ResolvedResource,
) -> Result<Vec<u8>, TransformError> {
    let mut line = String::new();
    let mut cached = HashMap::<PathBuf, Vec<u8>>::new();
    let mut total_bytes = 0_u64;
    let mut first_error = None;
    loop {
        line.clear();
        if stdout
            .read_line(&mut line)
            .await
            .map_err(TransformError::Read)?
            == 0
        {
            return Err(first_error.unwrap_or_else(|| {
                TransformError::Protocol(
                    "helper closed stdout before reporting a transformation result".into(),
                )
            }));
        }
        if line.len() > 64 * 1024 {
            return Err(TransformError::Protocol(
                "helper message is too large".into(),
            ));
        }
        match serde_json::from_str::<HelperMessage>(line.trim_end())
            .map_err(|error| TransformError::Protocol(error.to_string()))?
        {
            HelperMessage::Resource { url, public_id } => {
                let resolution =
                    resolve_resource(catalog, stylesheet_uri, &root, &url, public_id.as_deref())
                        .and_then(|resource| {
                            if let Some(bytes) = cached.get(&resource.path) {
                                return Ok((resource, bytes.clone()));
                            }
                            let metadata =
                                fs::metadata(&resource.path).map_err(TransformError::Io)?;
                            if metadata.len() > MAX_RESOURCE_BYTES {
                                return Err(TransformError::ResourceTooLarge);
                            }
                            if cached.len().saturating_add(1) > MAX_RESOURCES {
                                return Err(TransformError::TooManyResources);
                            }
                            let next_total = total_bytes.saturating_add(metadata.len());
                            if next_total > MAX_RESOURCE_GRAPH_BYTES {
                                return Err(TransformError::ResourceGraphTooLarge);
                            }
                            let bytes = fs::read(&resource.path).map_err(TransformError::Io)?;
                            total_bytes = next_total;
                            cached.insert(resource.path.clone(), bytes.clone());
                            Ok((resource, bytes))
                        });
                match resolution {
                    Ok((resource, bytes)) => {
                        write_resource_reply(stdin, Some(resource.uri.as_str()), &bytes).await?;
                    }
                    Err(error) => {
                        if first_error.is_none() {
                            first_error = Some(error);
                        }
                        write_resource_reply(stdin, None, &[]).await?;
                    }
                }
            }
            HelperMessage::Result {
                valid,
                output_bytes,
            } => {
                if let Some(error) = first_error {
                    return Err(error);
                }
                if !valid {
                    return Err(TransformError::EngineFailed(
                        "libxslt reported a failed transformation".into(),
                    ));
                }
                let bytes = output_bytes.ok_or_else(|| {
                    TransformError::Protocol("transform result has no output length".into())
                })?;
                if u64::try_from(bytes).unwrap_or(u64::MAX) > MAX_OUTPUT_BYTES {
                    return Err(TransformError::OutputTooLarge);
                }
                let mut output = vec![0_u8; bytes];
                stdout
                    .read_exact(&mut output)
                    .await
                    .map_err(TransformError::Read)?;
                let mut delimiter = [0_u8; 1];
                stdout
                    .read_exact(&mut delimiter)
                    .await
                    .map_err(TransformError::Read)?;
                if delimiter != *b"\n" {
                    return Err(TransformError::Protocol(
                        "transform result frame is missing its delimiter".into(),
                    ));
                }
                return Ok(output);
            }
        }
    }
}

fn resolve_resource(
    catalog: &CatalogResolver,
    stylesheet_uri: &str,
    root: &ResolvedResource,
    url: &str,
    public_id: Option<&str>,
) -> Result<ResolvedResource, TransformError> {
    if url == stylesheet_uri {
        return Ok(root.clone());
    }
    if let Some(resource) = catalog.resolve_external(Some(url), public_id)? {
        return Ok(resource);
    }
    let normalized = if std::path::Path::new(url).is_absolute() {
        url::Url::from_file_path(url)
            .map_err(|()| TransformError::Protocol("invalid absolute resource path".into()))?
            .to_string()
    } else {
        url.to_owned()
    };
    Ok(catalog.resolve_uri_or_direct(&normalized)?)
}

async fn write_resource_reply(
    stdin: &mut tokio::process::ChildStdin,
    uri: Option<&str>,
    bytes: &[u8],
) -> Result<(), TransformError> {
    let header = serde_json::to_vec(&ResourceReply {
        allowed: uri.is_some(),
        uri,
        bytes: bytes.len(),
    })
    .map_err(|error| TransformError::Protocol(error.to_string()))?;
    stdin
        .write_all(&header)
        .await
        .map_err(TransformError::Write)?;
    stdin
        .write_all(b"\n")
        .await
        .map_err(TransformError::Write)?;
    stdin
        .write_all(bytes)
        .await
        .map_err(TransformError::Write)?;
    stdin
        .write_all(b"\n")
        .await
        .map_err(TransformError::Write)?;
    stdin.flush().await.map_err(TransformError::Write)
}
