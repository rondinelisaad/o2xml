use std::{
    collections::HashMap,
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    process::Stdio,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWriteExt, BufReader},
    process::Command,
    time::timeout,
};
use tracing::info;
use xml_resource::{CatalogError, CatalogResolver, ResolvedResource};
use xml_schema::SchemaIndex;

use crate::{compile_xsd, XsdCompileError, XsdElementDeclaration};

#[derive(Clone, Copy)]
pub struct XsdIndexRequest<'a> {
    pub catalog: &'a CatalogResolver,
    pub uri: &'a str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XsdIndexReport {
    pub index: SchemaIndex,
    pub approved_grammar_resources: usize,
}

pub type XsdIndexFuture<'a> =
    Pin<Box<dyn Future<Output = Result<XsdIndexReport, XsdIndexError>> + Send + 'a>>;

pub trait XsdIndexEngine: Send + Sync {
    fn name(&self) -> &'static str;
    fn compile<'a>(&'a self, request: XsdIndexRequest<'a>) -> XsdIndexFuture<'a>;
}

#[derive(Clone, Debug)]
pub struct Libxml2XsdIndexEngine {
    executable: PathBuf,
    deadline: Duration,
    max_resource_bytes: u64,
    max_graph_bytes: u64,
    max_resources: usize,
    max_stdout_bytes: u64,
    max_stderr_bytes: u64,
}

impl Libxml2XsdIndexEngine {
    pub fn new(executable: impl AsRef<Path>) -> Result<Self, XsdIndexError> {
        let executable =
            std::fs::canonicalize(executable.as_ref()).map_err(XsdIndexError::Spawn)?;
        if !executable.is_file() {
            return Err(XsdIndexError::InvalidExecutable(executable));
        }
        Ok(Self {
            executable,
            deadline: Duration::from_secs(10),
            max_resource_bytes: 8 * 1024 * 1024,
            max_graph_bytes: 64 * 1024 * 1024,
            max_resources: 256,
            max_stdout_bytes: 32 * 1024 * 1024,
            max_stderr_bytes: 1024 * 1024,
        })
    }

    #[must_use]
    pub fn with_deadline(mut self, deadline: Duration) -> Self {
        self.deadline = deadline.max(Duration::from_millis(1));
        self
    }

    async fn compile_inner(
        &self,
        request: XsdIndexRequest<'_>,
    ) -> Result<XsdIndexReport, XsdIndexError> {
        let started = Instant::now();
        let result = self.run(request).await;
        let (result_name, resources, elements, error_class) = match &result {
            Ok(report) => (
                "success",
                report.approved_grammar_resources,
                report.index.len(),
                None,
            ),
            Err(error) => ("failure", 0, 0, Some(error.class())),
        };
        info!(
            audit = true,
            action = "xsd.index",
            result = result_name,
            resource = "schema",
            engine = "libxml2-process",
            duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
            grammar_resources = resources,
            element_count = elements,
            error_class,
        );
        result
    }

    async fn run(&self, request: XsdIndexRequest<'_>) -> Result<XsdIndexReport, XsdIndexError> {
        let root = request
            .catalog
            .resolve_uri(request.uri)?
            .ok_or(XsdIndexError::GrammarNotFound)?;
        let mut command = Command::new(&self.executable);
        command
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        let mut child = command.spawn().map_err(XsdIndexError::Spawn)?;
        let mut stdin = child.stdin.take().ok_or(XsdIndexError::MissingPipe)?;
        let stdout = child.stdout.take().ok_or(XsdIndexError::MissingPipe)?;
        let stderr = child.stderr.take().ok_or(XsdIndexError::MissingPipe)?;
        let header = serde_json::to_vec(&HelperRequest {
            mode: "xsd_index",
            grammar_uri: request.uri,
            public_id: None,
            document_bytes: 0,
        })
        .map_err(|error| XsdIndexError::Protocol(error.to_string()))?;
        let send = async {
            stdin.write_all(&header).await?;
            stdin.write_all(b"\n\n").await?;
            stdin.flush().await
        };
        timeout(self.deadline, send)
            .await
            .map_err(|_| XsdIndexError::Timeout(self.deadline))?
            .map_err(XsdIndexError::Io)?;

        let stderr_limit = self.max_stderr_bytes;
        let stderr_task = tokio::spawn(async move { read_bounded(stderr, stderr_limit).await });
        let interaction = broker_resources(
            BufReader::new(stdout.take(self.max_stdout_bytes)),
            &mut stdin,
            request,
            root,
            self.max_resource_bytes,
            self.max_graph_bytes,
            self.max_resources,
        );
        let broker = match timeout(self.deadline, interaction).await {
            Ok(Ok(result)) => result,
            Ok(Err(error)) => {
                stderr_task.abort();
                let _ = child.kill().await;
                return Err(error);
            }
            Err(_) => {
                stderr_task.abort();
                let _ = child.kill().await;
                return Err(XsdIndexError::Timeout(self.deadline));
            }
        };
        drop(stdin);
        let status = match timeout(self.deadline, child.wait()).await {
            Ok(status) => status.map_err(XsdIndexError::Io)?,
            Err(_) => {
                stderr_task.abort();
                let _ = child.kill().await;
                return Err(XsdIndexError::Timeout(self.deadline));
            }
        };
        let (_stderr, truncated) = stderr_task.await.map_err(XsdIndexError::Task)??;
        if truncated {
            return Err(XsdIndexError::OutputTooLarge);
        }
        if !status.success() || !broker.valid {
            return Err(XsdIndexError::Rejected);
        }
        let declarations = broker
            .xsd
            .ok_or_else(|| XsdIndexError::Protocol("helper omitted XSD declarations".into()))?;
        Ok(XsdIndexReport {
            index: compile_xsd(declarations)?,
            approved_grammar_resources: broker.approved_resources,
        })
    }
}

impl XsdIndexEngine for Libxml2XsdIndexEngine {
    fn name(&self) -> &'static str {
        "libxml2-process"
    }

    fn compile<'a>(&'a self, request: XsdIndexRequest<'a>) -> XsdIndexFuture<'a> {
        Box::pin(async move { self.compile_inner(request).await })
    }
}

#[derive(Serialize)]
struct HelperRequest<'a> {
    mode: &'static str,
    grammar_uri: &'a str,
    public_id: Option<&'a str>,
    document_bytes: usize,
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
        xsd: Option<Vec<XsdElementDeclaration>>,
    },
}

#[derive(Serialize)]
struct ResourceReply<'a> {
    allowed: bool,
    uri: Option<&'a str>,
    bytes: usize,
}

struct BrokerResult {
    valid: bool,
    xsd: Option<Vec<XsdElementDeclaration>>,
    approved_resources: usize,
}

#[allow(clippy::too_many_arguments)]
async fn broker_resources<R: AsyncRead + Unpin>(
    mut stdout: BufReader<R>,
    stdin: &mut tokio::process::ChildStdin,
    request: XsdIndexRequest<'_>,
    root: ResolvedResource,
    max_resource_bytes: u64,
    max_graph_bytes: u64,
    max_resources: usize,
) -> Result<BrokerResult, XsdIndexError> {
    let mut line = String::new();
    let mut cached = HashMap::<PathBuf, Vec<u8>>::new();
    let mut total_bytes = 0_u64;
    let mut first_error = None;
    loop {
        line.clear();
        if stdout
            .read_line(&mut line)
            .await
            .map_err(XsdIndexError::Io)?
            == 0
        {
            return Err(first_error.unwrap_or_else(|| {
                XsdIndexError::Protocol("helper closed stdout before reporting a result".into())
            }));
        }
        match serde_json::from_str::<HelperMessage>(line.trim_end())
            .map_err(|error| XsdIndexError::Protocol(error.to_string()))?
        {
            HelperMessage::Resource { url, public_id } => {
                let resolution = resolve_resource(request, &root, &url, public_id.as_deref())
                    .and_then(|resource| {
                        if let Some(bytes) = cached.get(&resource.path) {
                            return Ok((resource, bytes.clone()));
                        }
                        let metadata = std::fs::metadata(&resource.path).map_err(|source| {
                            XsdIndexError::GrammarIo {
                                path: resource.path.clone(),
                                source,
                            }
                        })?;
                        if metadata.len() > max_resource_bytes {
                            return Err(XsdIndexError::GrammarTooLarge {
                                path: resource.path.clone(),
                                limit: max_resource_bytes,
                            });
                        }
                        if cached.len().saturating_add(1) > max_resources {
                            return Err(XsdIndexError::TooManyResources {
                                limit: max_resources,
                            });
                        }
                        let next_total = total_bytes.saturating_add(metadata.len());
                        if next_total > max_graph_bytes {
                            return Err(XsdIndexError::GrammarGraphTooLarge {
                                limit: max_graph_bytes,
                            });
                        }
                        let bytes = std::fs::read(&resource.path).map_err(|source| {
                            XsdIndexError::GrammarIo {
                                path: resource.path.clone(),
                                source,
                            }
                        })?;
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
            HelperMessage::Result { valid, xsd } => {
                if let Some(error) = first_error {
                    return Err(error);
                }
                return Ok(BrokerResult {
                    valid,
                    xsd,
                    approved_resources: cached.len(),
                });
            }
        }
    }
}

fn resolve_resource(
    request: XsdIndexRequest<'_>,
    root: &ResolvedResource,
    url: &str,
    public_id: Option<&str>,
) -> Result<ResolvedResource, XsdIndexError> {
    if url == request.uri {
        return Ok(root.clone());
    }
    if let Some(resource) = request.catalog.resolve_external(Some(url), public_id)? {
        return Ok(resource);
    }
    let normalized = if Path::new(url).is_absolute() {
        url::Url::from_file_path(url)
            .map_err(|()| XsdIndexError::Protocol("invalid absolute resource path".into()))?
            .to_string()
    } else {
        url.to_owned()
    };
    request
        .catalog
        .resolve_uri_or_direct(&normalized)
        .map_err(Into::into)
}

async fn write_resource_reply(
    stdin: &mut tokio::process::ChildStdin,
    uri: Option<&str>,
    bytes: &[u8],
) -> Result<(), XsdIndexError> {
    let header = serde_json::to_vec(&ResourceReply {
        allowed: uri.is_some(),
        uri,
        bytes: bytes.len(),
    })
    .map_err(|error| XsdIndexError::Protocol(error.to_string()))?;
    stdin.write_all(&header).await.map_err(XsdIndexError::Io)?;
    stdin.write_all(b"\n").await.map_err(XsdIndexError::Io)?;
    stdin.write_all(bytes).await.map_err(XsdIndexError::Io)?;
    stdin.write_all(b"\n").await.map_err(XsdIndexError::Io)?;
    stdin.flush().await.map_err(XsdIndexError::Io)
}

async fn read_bounded<R: AsyncRead + Unpin>(
    mut reader: R,
    limit: u64,
) -> Result<(Vec<u8>, bool), XsdIndexError> {
    let mut kept = Vec::new();
    let mut buffer = [0_u8; 8192];
    let mut truncated = false;
    loop {
        let read = reader.read(&mut buffer).await.map_err(XsdIndexError::Io)?;
        if read == 0 {
            return Ok((kept, truncated));
        }
        let remaining = usize::try_from(limit)
            .unwrap_or(usize::MAX)
            .saturating_sub(kept.len());
        let retain = remaining.min(read);
        kept.extend_from_slice(&buffer[..retain]);
        truncated |= retain < read;
    }
}

#[derive(Debug, Error)]
pub enum XsdIndexError {
    #[error(transparent)]
    Catalog(#[from] CatalogError),
    #[error(transparent)]
    Compile(#[from] XsdCompileError),
    #[error("catalog did not resolve the requested XSD")]
    GrammarNotFound,
    #[error("cannot inspect XSD resource {path}: {source}")]
    GrammarIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("XSD resource exceeds the {limit}-byte limit: {path}")]
    GrammarTooLarge { path: PathBuf, limit: u64 },
    #[error("XSD graph exceeds the {limit}-resource limit")]
    TooManyResources { limit: usize },
    #[error("XSD graph exceeds the {limit}-byte aggregate limit")]
    GrammarGraphTooLarge { limit: u64 },
    #[error("XSD helper executable is not a file: {0}")]
    InvalidExecutable(PathBuf),
    #[error("cannot start XSD helper: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("XSD helper pipe is unavailable")]
    MissingPipe,
    #[error("cannot communicate with XSD helper: {0}")]
    Io(#[source] std::io::Error),
    #[error("XSD helper task failed: {0}")]
    Task(#[source] tokio::task::JoinError),
    #[error("XSD helper exceeded its {0:?} deadline")]
    Timeout(Duration),
    #[error("XSD helper output exceeded its limit")]
    OutputTooLarge,
    #[error("XSD helper rejected the grammar")]
    Rejected,
    #[error("invalid XSD helper protocol: {0}")]
    Protocol(String),
}

impl XsdIndexError {
    fn class(&self) -> &'static str {
        match self {
            Self::Catalog(_) => "catalog",
            Self::Compile(_) => "compile",
            Self::GrammarNotFound => "grammar_not_found",
            Self::GrammarIo { .. } => "grammar_io",
            Self::GrammarTooLarge { .. } => "grammar_too_large",
            Self::TooManyResources { .. } => "too_many_resources",
            Self::GrammarGraphTooLarge { .. } => "grammar_graph_too_large",
            Self::InvalidExecutable(_) => "invalid_executable",
            Self::Spawn(_) => "spawn",
            Self::MissingPipe => "missing_pipe",
            Self::Io(_) => "io",
            Self::Task(_) => "task",
            Self::Timeout(_) => "timeout",
            Self::OutputTooLarge => "output_too_large",
            Self::Rejected => "rejected",
            Self::Protocol(_) => "protocol",
        }
    }
}
