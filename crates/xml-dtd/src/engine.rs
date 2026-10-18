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

use crate::{compile_dtd, DtdCompileError, DtdElementDeclaration};

#[derive(Clone, Copy)]
pub struct DtdIndexRequest<'a> {
    pub catalog: &'a CatalogResolver,
    pub system_id: &'a str,
    pub public_id: Option<&'a str>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DtdIndexReport {
    pub index: SchemaIndex,
    pub approved_grammar_resources: usize,
}

pub type DtdIndexFuture<'a> =
    Pin<Box<dyn Future<Output = Result<DtdIndexReport, DtdIndexError>> + Send + 'a>>;

pub trait DtdIndexEngine: Send + Sync {
    fn name(&self) -> &'static str;
    fn compile<'a>(&'a self, request: DtdIndexRequest<'a>) -> DtdIndexFuture<'a>;
}

#[derive(Clone, Debug)]
pub struct Libxml2DtdIndexEngine {
    executable: PathBuf,
    deadline: Duration,
    max_resource_bytes: u64,
    max_graph_bytes: u64,
    max_resources: usize,
    max_stdout_bytes: u64,
    max_stderr_bytes: u64,
}

impl Libxml2DtdIndexEngine {
    pub fn new(executable: impl AsRef<Path>) -> Result<Self, DtdIndexError> {
        let executable =
            std::fs::canonicalize(executable.as_ref()).map_err(DtdIndexError::Spawn)?;
        if !executable.is_file() {
            return Err(DtdIndexError::InvalidExecutable(executable));
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
        request: DtdIndexRequest<'_>,
    ) -> Result<DtdIndexReport, DtdIndexError> {
        let started = Instant::now();
        let result = self.run(request).await;
        let (result_name, resource_count, element_count, error_class) = match &result {
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
            action = "dtd.index",
            result = result_name,
            resource = "schema",
            engine = "libxml2-process",
            duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
            grammar_resources = resource_count,
            element_count,
            error_class,
        );
        result
    }

    async fn run(&self, request: DtdIndexRequest<'_>) -> Result<DtdIndexReport, DtdIndexError> {
        let root = request
            .catalog
            .resolve_external(Some(request.system_id), request.public_id)?
            .ok_or(DtdIndexError::GrammarNotFound)?;
        let mut command = Command::new(&self.executable);
        command
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        let mut child = command.spawn().map_err(DtdIndexError::Spawn)?;
        let mut stdin = child.stdin.take().ok_or(DtdIndexError::MissingPipe)?;
        let stdout = child.stdout.take().ok_or(DtdIndexError::MissingPipe)?;
        let stderr = child.stderr.take().ok_or(DtdIndexError::MissingPipe)?;
        let header = serde_json::to_vec(&HelperRequest {
            mode: "dtd_index",
            grammar_uri: request.system_id,
            public_id: request.public_id,
            document_bytes: 0,
        })
        .map_err(|error| DtdIndexError::Protocol(error.to_string()))?;
        let send = async {
            stdin.write_all(&header).await?;
            stdin.write_all(b"\n\n").await?;
            stdin.flush().await
        };
        timeout(self.deadline, send)
            .await
            .map_err(|_| DtdIndexError::Timeout(self.deadline))?
            .map_err(DtdIndexError::Io)?;

        let stderr_limit = self.max_stderr_bytes;
        let stderr_task = tokio::spawn(async move { read_bounded(stderr, stderr_limit).await });
        let stdout = BufReader::new(stdout.take(self.max_stdout_bytes));
        let interaction = broker_resources(
            stdout,
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
                return Err(DtdIndexError::Timeout(self.deadline));
            }
        };
        drop(stdin);
        let status = match timeout(self.deadline, child.wait()).await {
            Ok(status) => status.map_err(DtdIndexError::Io)?,
            Err(_) => {
                stderr_task.abort();
                let _ = child.kill().await;
                return Err(DtdIndexError::Timeout(self.deadline));
            }
        };
        let (_stderr, stderr_truncated) = stderr_task.await.map_err(DtdIndexError::Task)??;
        if stderr_truncated {
            return Err(DtdIndexError::OutputTooLarge);
        }
        if !status.success() || !broker.valid {
            return Err(DtdIndexError::Rejected);
        }
        let declarations = broker
            .dtd
            .ok_or_else(|| DtdIndexError::Protocol("helper omitted DTD declarations".to_owned()))?;
        let index = compile_dtd(declarations)?;
        Ok(DtdIndexReport {
            index,
            approved_grammar_resources: broker.approved_resources,
        })
    }
}

impl DtdIndexEngine for Libxml2DtdIndexEngine {
    fn name(&self) -> &'static str {
        "libxml2-process"
    }

    fn compile<'a>(&'a self, request: DtdIndexRequest<'a>) -> DtdIndexFuture<'a> {
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
        dtd: Option<Vec<DtdElementDeclaration>>,
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
    dtd: Option<Vec<DtdElementDeclaration>>,
    approved_resources: usize,
}

async fn broker_resources<R: AsyncRead + Unpin>(
    mut stdout: BufReader<R>,
    stdin: &mut tokio::process::ChildStdin,
    request: DtdIndexRequest<'_>,
    root: ResolvedResource,
    max_resource_bytes: u64,
    max_graph_bytes: u64,
    max_resources: usize,
) -> Result<BrokerResult, DtdIndexError> {
    let mut line = String::new();
    let mut cached = HashMap::<PathBuf, Vec<u8>>::new();
    let mut total_bytes = 0_u64;
    let mut first_error = None;
    loop {
        line.clear();
        if stdout
            .read_line(&mut line)
            .await
            .map_err(DtdIndexError::Io)?
            == 0
        {
            return Err(first_error.unwrap_or_else(|| {
                DtdIndexError::Protocol("helper closed stdout before reporting a result".into())
            }));
        }
        let message = serde_json::from_str::<HelperMessage>(line.trim_end())
            .map_err(|error| DtdIndexError::Protocol(error.to_string()))?;
        match message {
            HelperMessage::Resource { url, public_id } => {
                let resolution = resolve_resource(request, &root, &url, public_id.as_deref())
                    .and_then(|resource| {
                        if let Some(bytes) = cached.get(&resource.path) {
                            return Ok((resource, bytes.clone()));
                        }
                        let metadata = std::fs::metadata(&resource.path).map_err(|source| {
                            DtdIndexError::GrammarIo {
                                path: resource.path.clone(),
                                source,
                            }
                        })?;
                        if metadata.len() > max_resource_bytes {
                            return Err(DtdIndexError::GrammarTooLarge {
                                path: resource.path.clone(),
                                limit: max_resource_bytes,
                            });
                        }
                        if cached.len().saturating_add(1) > max_resources {
                            return Err(DtdIndexError::TooManyResources {
                                limit: max_resources,
                            });
                        }
                        let next_total = total_bytes.saturating_add(metadata.len());
                        if next_total > max_graph_bytes {
                            return Err(DtdIndexError::GrammarGraphTooLarge {
                                limit: max_graph_bytes,
                            });
                        }
                        let bytes = std::fs::read(&resource.path).map_err(|source| {
                            DtdIndexError::GrammarIo {
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
            HelperMessage::Result { valid, dtd } => {
                if let Some(error) = first_error {
                    return Err(error);
                }
                return Ok(BrokerResult {
                    valid,
                    dtd,
                    approved_resources: cached.len(),
                });
            }
        }
    }
}

fn resolve_resource(
    request: DtdIndexRequest<'_>,
    root: &ResolvedResource,
    url: &str,
    public_id: Option<&str>,
) -> Result<ResolvedResource, DtdIndexError> {
    if url == request.system_id || (public_id.is_some() && public_id == request.public_id) {
        return Ok(root.clone());
    }
    if let Some(resource) = request.catalog.resolve_external(Some(url), public_id)? {
        return Ok(resource);
    }
    let normalized = if Path::new(url).is_absolute() {
        url::Url::from_file_path(url)
            .map_err(|()| DtdIndexError::Protocol("invalid absolute resource path".into()))?
            .to_string()
    } else {
        url.to_owned()
    };
    request
        .catalog
        .resolve_uri_or_direct(&normalized)
        .map_err(DtdIndexError::from)
}

async fn write_resource_reply(
    stdin: &mut tokio::process::ChildStdin,
    uri: Option<&str>,
    bytes: &[u8],
) -> Result<(), DtdIndexError> {
    let header = serde_json::to_vec(&ResourceReply {
        allowed: uri.is_some(),
        uri,
        bytes: bytes.len(),
    })
    .map_err(|error| DtdIndexError::Protocol(error.to_string()))?;
    stdin.write_all(&header).await.map_err(DtdIndexError::Io)?;
    stdin.write_all(b"\n").await.map_err(DtdIndexError::Io)?;
    stdin.write_all(bytes).await.map_err(DtdIndexError::Io)?;
    stdin.write_all(b"\n").await.map_err(DtdIndexError::Io)?;
    stdin.flush().await.map_err(DtdIndexError::Io)
}

async fn read_bounded<R: AsyncRead + Unpin>(
    mut reader: R,
    limit: u64,
) -> Result<(Vec<u8>, bool), DtdIndexError> {
    let capacity = usize::try_from(limit.min(64 * 1024)).unwrap_or(64 * 1024);
    let mut kept = Vec::with_capacity(capacity);
    let mut buffer = [0_u8; 8192];
    let mut truncated = false;
    loop {
        let read = reader.read(&mut buffer).await.map_err(DtdIndexError::Io)?;
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
pub enum DtdIndexError {
    #[error(transparent)]
    Catalog(#[from] CatalogError),
    #[error(transparent)]
    Compile(#[from] DtdCompileError),
    #[error("catalog did not resolve the requested DTD")]
    GrammarNotFound,
    #[error("cannot inspect DTD resource {path}: {source}")]
    GrammarIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("DTD resource exceeds the {limit}-byte limit: {path}")]
    GrammarTooLarge { path: PathBuf, limit: u64 },
    #[error("DTD graph exceeds the {limit}-resource limit")]
    TooManyResources { limit: usize },
    #[error("DTD graph exceeds the {limit}-byte aggregate limit")]
    GrammarGraphTooLarge { limit: u64 },
    #[error("DTD helper executable is not a file: {0}")]
    InvalidExecutable(PathBuf),
    #[error("cannot start DTD helper: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("DTD helper pipe is unavailable")]
    MissingPipe,
    #[error("cannot communicate with DTD helper: {0}")]
    Io(#[source] std::io::Error),
    #[error("DTD helper task failed: {0}")]
    Task(#[source] tokio::task::JoinError),
    #[error("DTD helper exceeded its {0:?} deadline")]
    Timeout(Duration),
    #[error("DTD helper output exceeded its limit")]
    OutputTooLarge,
    #[error("DTD helper rejected the grammar")]
    Rejected,
    #[error("invalid DTD helper protocol: {0}")]
    Protocol(String),
}

impl DtdIndexError {
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
