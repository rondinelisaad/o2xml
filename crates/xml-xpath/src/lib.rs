use std::{
    collections::BTreeMap,
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    process::Stdio,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    process::Command,
    time::timeout,
};
use tracing::info;
use xml_document::LineIndex;
use xml_domain::{Revision, SourceSpan};

#[derive(Clone, Copy)]
pub struct XPathRequest<'a> {
    pub source: &'a str,
    pub revision: Revision,
    pub expression: &'a str,
    pub namespaces: &'a BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XPathReport {
    pub revision: Revision,
    pub value: XPathValue,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum XPathValue {
    Nodes {
        items: Vec<XPathNode>,
        truncated: bool,
    },
    Boolean(bool),
    Number(String),
    String(String),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XPathNode {
    pub path: String,
    pub span: SourceSpan,
}

pub type XPathFuture<'a> =
    Pin<Box<dyn Future<Output = Result<XPathReport, XPathError>> + Send + 'a>>;

pub trait XPathEngine: Send + Sync {
    fn name(&self) -> &'static str;
    fn evaluate<'a>(&'a self, request: XPathRequest<'a>) -> XPathFuture<'a>;
}

#[derive(Debug, Error)]
pub enum XPathError {
    #[error("XPath helper executable is not a file: {0}")]
    InvalidExecutable(PathBuf),
    #[error("document exceeds the {limit}-byte XPath limit")]
    DocumentTooLarge { limit: usize },
    #[error("XPath expression exceeds the {limit}-byte limit")]
    ExpressionTooLarge { limit: usize },
    #[error("XPath namespace binding count exceeds the limit of {limit}")]
    TooManyNamespaces { limit: usize },
    #[error("XPath namespace prefix or URI exceeds the {limit}-byte limit")]
    NamespaceTooLarge { limit: usize },
    #[error("cannot start XPath helper: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("XPath helper pipe is unavailable")]
    MissingPipe,
    #[error("cannot communicate with XPath helper: {0}")]
    Io(#[source] std::io::Error),
    #[error("XPath helper task failed: {0}")]
    Task(#[source] tokio::task::JoinError),
    #[error("XPath helper exceeded its {0:?} deadline")]
    Timeout(Duration),
    #[error("XPath helper output exceeded its limit")]
    OutputTooLarge,
    #[error("XPath helper rejected the document or expression")]
    Rejected,
    #[error("invalid XPath helper protocol: {0}")]
    Protocol(String),
}

#[derive(Clone, Debug)]
pub struct Libxml2XPathEngine {
    executable: PathBuf,
    deadline: Duration,
    max_document_bytes: usize,
    max_expression_bytes: usize,
    max_namespaces: usize,
    max_namespace_bytes: usize,
    max_output_bytes: u64,
    max_results: usize,
}

impl Libxml2XPathEngine {
    pub fn new(executable: impl AsRef<Path>) -> Result<Self, XPathError> {
        let executable = std::fs::canonicalize(executable.as_ref()).map_err(XPathError::Spawn)?;
        if !executable.is_file() {
            return Err(XPathError::InvalidExecutable(executable));
        }
        Ok(Self {
            executable,
            deadline: Duration::from_secs(5),
            max_document_bytes: 128 * 1024 * 1024,
            max_expression_bytes: 64 * 1024,
            max_namespaces: 128,
            max_namespace_bytes: 8 * 1024,
            max_output_bytes: 8 * 1024 * 1024,
            max_results: 10_000,
        })
    }

    #[must_use]
    pub fn with_deadline(mut self, deadline: Duration) -> Self {
        self.deadline = deadline.max(Duration::from_millis(1));
        self
    }

    async fn evaluate_inner(&self, request: XPathRequest<'_>) -> Result<XPathReport, XPathError> {
        let started = Instant::now();
        let result = self.run(request).await;
        let (result_name, result_count, value_kind, error_class) = match &result {
            Ok(report) => (
                "success",
                report.value.result_count(),
                Some(report.value.kind()),
                None,
            ),
            Err(error) => ("failure", 0, None, Some(error.class())),
        };
        info!(
            audit = true,
            action = "xpath.evaluate",
            result = result_name,
            resource = "document",
            revision = request.revision.0,
            duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
            result_count,
            value_kind,
            error_class,
        );
        result
    }

    async fn run(&self, request: XPathRequest<'_>) -> Result<XPathReport, XPathError> {
        if request.source.len() > self.max_document_bytes {
            return Err(XPathError::DocumentTooLarge {
                limit: self.max_document_bytes,
            });
        }
        if request.expression.len() > self.max_expression_bytes {
            return Err(XPathError::ExpressionTooLarge {
                limit: self.max_expression_bytes,
            });
        }
        if request.namespaces.len() > self.max_namespaces {
            return Err(XPathError::TooManyNamespaces {
                limit: self.max_namespaces,
            });
        }
        if request.namespaces.iter().any(|(prefix, uri)| {
            prefix.len() > self.max_namespace_bytes || uri.len() > self.max_namespace_bytes
        }) {
            return Err(XPathError::NamespaceTooLarge {
                limit: self.max_namespace_bytes,
            });
        }
        let mut command = Command::new(&self.executable);
        command
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        let mut child = command.spawn().map_err(XPathError::Spawn)?;
        let mut stdin = child.stdin.take().ok_or(XPathError::MissingPipe)?;
        let stdout = child.stdout.take().ok_or(XPathError::MissingPipe)?;
        let stderr = child.stderr.take().ok_or(XPathError::MissingPipe)?;
        let header = serde_json::to_vec(&HelperRequest {
            mode: "xpath",
            grammar_uri: "",
            public_id: None,
            document_bytes: request.source.len(),
            expression: request.expression,
            namespaces: request.namespaces,
            max_results: self.max_results,
        })
        .map_err(|error| XPathError::Protocol(error.to_string()))?;
        let send = async {
            stdin.write_all(&header).await?;
            stdin.write_all(b"\n").await?;
            stdin.write_all(request.source.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            stdin.flush().await
        };
        timeout(self.deadline, send)
            .await
            .map_err(|_| XPathError::Timeout(self.deadline))?
            .map_err(XPathError::Io)?;
        drop(stdin);

        let output_limit = self.max_output_bytes;
        let stdout_task = tokio::spawn(async move { read_bounded(stdout, output_limit).await });
        let stderr_task = tokio::spawn(async move { read_bounded(stderr, 1024 * 1024).await });
        let status = match timeout(self.deadline, child.wait()).await {
            Ok(status) => status.map_err(XPathError::Io)?,
            Err(_) => {
                stdout_task.abort();
                stderr_task.abort();
                let _ = child.kill().await;
                return Err(XPathError::Timeout(self.deadline));
            }
        };
        let (stdout, stdout_truncated) = stdout_task.await.map_err(XPathError::Task)??;
        let (_stderr, _stderr_truncated) = stderr_task.await.map_err(XPathError::Task)??;
        if stdout_truncated {
            return Err(XPathError::OutputTooLarge);
        }
        if !status.success() {
            return Err(XPathError::Rejected);
        }
        let message: HelperMessage = serde_json::from_slice(&stdout)
            .map_err(|error| XPathError::Protocol(error.to_string()))?;
        let HelperMessage::Result {
            valid: true,
            xpath: Some(value),
        } = message
        else {
            return Err(XPathError::Rejected);
        };
        Ok(XPathReport {
            revision: request.revision,
            value: value.into_public(request.source),
        })
    }
}

impl XPathEngine for Libxml2XPathEngine {
    fn name(&self) -> &'static str {
        "libxml2-xpath-1.0"
    }

    fn evaluate<'a>(&'a self, request: XPathRequest<'a>) -> XPathFuture<'a> {
        Box::pin(async move { self.evaluate_inner(request).await })
    }
}

impl XPathValue {
    fn kind(&self) -> &'static str {
        match self {
            Self::Nodes { .. } => "nodes",
            Self::Boolean(_) => "boolean",
            Self::Number(_) => "number",
            Self::String(_) => "string",
        }
    }

    fn result_count(&self) -> usize {
        match self {
            Self::Nodes { items, .. } => items.len(),
            _ => 1,
        }
    }
}

impl XPathError {
    fn class(&self) -> &'static str {
        match self {
            Self::InvalidExecutable(_) => "invalid_executable",
            Self::DocumentTooLarge { .. } => "document_too_large",
            Self::ExpressionTooLarge { .. } => "expression_too_large",
            Self::TooManyNamespaces { .. } => "too_many_namespaces",
            Self::NamespaceTooLarge { .. } => "namespace_too_large",
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

#[derive(Serialize)]
struct HelperRequest<'a> {
    mode: &'static str,
    grammar_uri: &'static str,
    public_id: Option<&'static str>,
    document_bytes: usize,
    expression: &'a str,
    namespaces: &'a BTreeMap<String, String>,
    max_results: usize,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum HelperMessage {
    Result {
        valid: bool,
        xpath: Option<WireValue>,
    },
}

#[derive(Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
enum WireValue {
    Nodes {
        items: Vec<WireNode>,
        truncated: bool,
    },
    Boolean(bool),
    Number(String),
    String(String),
}

#[derive(Deserialize)]
struct WireNode {
    path: String,
    line: usize,
}

impl WireValue {
    fn into_public(self, source: &str) -> XPathValue {
        match self {
            Self::Nodes { items, truncated } => XPathValue::Nodes {
                items: items
                    .into_iter()
                    .map(|node| XPathNode {
                        path: node.path,
                        span: approximate_line_span(source, node.line),
                    })
                    .collect(),
                truncated,
            },
            Self::Boolean(value) => XPathValue::Boolean(value),
            Self::Number(value) => XPathValue::Number(value),
            Self::String(value) => XPathValue::String(value),
        }
    }
}

fn approximate_line_span(source: &str, requested_line: usize) -> SourceSpan {
    let line = requested_line.max(1);
    let byte = if line == 1 {
        0
    } else {
        source
            .match_indices('\n')
            .nth(line.saturating_sub(2))
            .map_or(source.len(), |(byte, _)| byte.saturating_add(1))
    };
    let index = LineIndex::new(source);
    let mut span = index.span(source, byte, byte);
    span.approximate = true;
    span
}

async fn read_bounded<R: AsyncRead + Unpin>(
    mut reader: R,
    limit: u64,
) -> Result<(Vec<u8>, bool), XPathError> {
    let capacity = usize::try_from(limit.min(64 * 1024)).unwrap_or(64 * 1024);
    let mut kept = Vec::with_capacity(capacity);
    let mut buffer = [0_u8; 8192];
    let mut truncated = false;
    loop {
        let read = reader.read(&mut buffer).await.map_err(XPathError::Io)?;
        if read == 0 {
            return Ok((kept, truncated));
        }
        let remaining = usize::try_from(limit)
            .unwrap_or(usize::MAX)
            .saturating_sub(kept.len());
        let retained = remaining.min(read);
        kept.extend_from_slice(&buffer[..retained]);
        truncated |= retained < read;
    }
}
