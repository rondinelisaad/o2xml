use std::{
    borrow::Cow,
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
use tracing::{info, warn};
use xml_document::LineIndex;
use xml_domain::{Diagnostic, Revision, Severity, SourceSpan};
use xml_resource::{CatalogError, CatalogResolver, ResolvedResource};

mod jing;

pub use jing::JingProcessEngine;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GrammarReference<'a> {
    Dtd {
        system_id: &'a str,
        public_id: Option<&'a str>,
    },
    Xsd {
        uri: &'a str,
    },
    RelaxNg {
        uri: &'a str,
    },
    RelaxNgCompact {
        uri: &'a str,
    },
    Schematron {
        uri: &'a str,
    },
}

impl GrammarReference<'_> {
    fn engine_name(self) -> &'static str {
        match self {
            Self::Dtd { .. } => "libxml2-dtd",
            Self::Xsd { .. } => "libxml2-xsd",
            Self::RelaxNg { .. } => "libxml2-relax-ng",
            Self::RelaxNgCompact { .. } => "jing-rnc",
            Self::Schematron { .. } => "libxml2-schematron",
        }
    }
}

#[derive(Clone, Copy)]
pub struct ValidationRequest<'a> {
    pub source: &'a str,
    pub revision: Revision,
    pub file: Option<&'a str>,
    pub catalog: &'a CatalogResolver,
    pub grammar: GrammarReference<'a>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationReport {
    pub engine: &'static str,
    pub revision: Revision,
    pub diagnostics: Vec<Diagnostic>,
    pub stderr_truncated: bool,
    pub approved_grammar_resources: usize,
}

pub type ValidationFuture<'a> =
    Pin<Box<dyn Future<Output = Result<ValidationReport, ValidationError>> + Send + 'a>>;

pub trait ValidationEngine: Send + Sync {
    fn name(&self) -> &'static str;
    fn validate<'a>(&'a self, request: ValidationRequest<'a>) -> ValidationFuture<'a>;
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error(transparent)]
    Catalog(#[from] CatalogError),
    #[error("catalog did not resolve the requested grammar")]
    GrammarNotFound,
    #[error("cannot inspect grammar {path}: {source}")]
    GrammarIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("grammar exceeds the {limit}-byte validation limit: {path}")]
    GrammarTooLarge { path: PathBuf, limit: u64 },
    #[error("grammar dependency graph exceeds the {limit}-resource limit")]
    TooManyGrammarResources { limit: usize },
    #[error("grammar dependency graph exceeds the {limit}-byte aggregate limit")]
    GrammarGraphTooLarge { limit: u64 },
    #[error("document exceeds the {limit}-byte validation limit")]
    DocumentTooLarge { limit: usize },
    #[error("standalone adapter rejected an XML document with an external DOCTYPE")]
    ExternalDocumentType,
    #[error("cannot start validation engine: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("validation engine stdin is unavailable")]
    MissingStdin,
    #[error("validation engine stdout is unavailable")]
    MissingStdout,
    #[error("validation engine stderr is unavailable")]
    MissingStderr,
    #[error("validation helper protocol failed: {0}")]
    Protocol(String),
    #[error("cannot send XML to validation engine: {0}")]
    Write(#[source] std::io::Error),
    #[error("cannot read validation engine output: {0}")]
    Read(#[source] std::io::Error),
    #[error("validation output reader failed: {0}")]
    ReadTask(#[source] tokio::task::JoinError),
    #[error("cannot wait for validation engine: {0}")]
    Wait(#[source] std::io::Error),
    #[error("validation engine exceeded its {0:?} deadline")]
    Timeout(Duration),
    #[error("validation engine executable is not a file: {0}")]
    InvalidExecutable(PathBuf),
    #[error("validation engine support file is not a file: {0}")]
    InvalidEngineAsset(PathBuf),
    #[error("validation engine does not support the requested grammar type")]
    UnsupportedGrammar,
    #[error("compact grammar contains an unsupported or malformed resource directive")]
    CompactGrammarResourcesUnsupported,
    #[error("cannot stage validation input: {0}")]
    Staging(#[source] std::io::Error),
}

#[derive(Clone, Debug)]
pub struct Libxml2ProcessEngine {
    executable: PathBuf,
    deadline: Duration,
    max_document_bytes: usize,
    max_grammar_bytes: u64,
    max_grammar_graph_bytes: u64,
    max_grammar_resources: usize,
    max_stderr_bytes: u64,
    max_diagnostics: usize,
}

impl Libxml2ProcessEngine {
    pub fn new(executable: impl AsRef<Path>) -> Result<Self, ValidationError> {
        let executable =
            std::fs::canonicalize(executable.as_ref()).map_err(ValidationError::Spawn)?;
        if !executable.is_file() {
            return Err(ValidationError::InvalidExecutable(executable));
        }
        Ok(Self {
            executable,
            deadline: Duration::from_secs(10),
            max_document_bytes: 128 * 1024 * 1024,
            max_grammar_bytes: 8 * 1024 * 1024,
            max_grammar_graph_bytes: 64 * 1024 * 1024,
            max_grammar_resources: 256,
            max_stderr_bytes: 1024 * 1024,
            max_diagnostics: 1_000,
        })
    }

    #[must_use]
    pub fn with_deadline(mut self, deadline: Duration) -> Self {
        self.deadline = deadline.max(Duration::from_millis(1));
        self
    }

    async fn validate_inner(
        &self,
        request: ValidationRequest<'_>,
    ) -> Result<ValidationReport, ValidationError> {
        let started = Instant::now();
        let engine = request.grammar.engine_name();
        let result = self.run(request).await;
        let (result_name, diagnostic_count, grammar_resources, error_class) = match &result {
            Ok(report) if report.diagnostics.is_empty() => {
                ("success", 0, report.approved_grammar_resources, None)
            }
            Ok(report) => (
                "failure",
                report.diagnostics.len(),
                report.approved_grammar_resources,
                None,
            ),
            Err(error) => ("failure", 0, 0, Some(error_class(error))),
        };
        info!(
            audit = true,
            action = "validation.run",
            result = result_name,
            resource = "document",
            revision = request.revision.0,
            engine,
            duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
            diagnostic_count,
            grammar_resources,
            error_class,
        );
        result
    }

    async fn run(
        &self,
        request: ValidationRequest<'_>,
    ) -> Result<ValidationReport, ValidationError> {
        let engine = request.grammar.engine_name();
        if matches!(request.grammar, GrammarReference::RelaxNgCompact { .. }) {
            return Err(ValidationError::UnsupportedGrammar);
        }
        if request.source.len() > self.max_document_bytes {
            return Err(ValidationError::DocumentTooLarge {
                limit: self.max_document_bytes,
            });
        }
        let grammar = resolve_grammar(request.catalog, request.grammar)?;
        let document_source = mask_document_type(request.source);
        let mut command = Command::new(&self.executable);
        command
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = command.spawn().map_err(ValidationError::Spawn)?;
        let mut stdin = child.stdin.take().ok_or(ValidationError::MissingStdin)?;
        let stdout = child.stdout.take().ok_or(ValidationError::MissingStdout)?;
        let header =
            serde_json::to_vec(&HelperRequest::new(request.grammar, document_source.len()))
                .map_err(|error| ValidationError::Protocol(error.to_string()))?;
        let send_document = async {
            stdin.write_all(&header).await?;
            stdin.write_all(b"\n").await?;
            stdin.write_all(document_source.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            stdin.flush().await
        };
        timeout(self.deadline, send_document)
            .await
            .map_err(|_| ValidationError::Timeout(self.deadline))?
            .map_err(ValidationError::Write)?;

        let stderr = child.stderr.take().ok_or(ValidationError::MissingStderr)?;
        let max_output = self.max_stderr_bytes;
        let reader = tokio::spawn(async move { read_bounded(stderr, max_output).await });
        let interaction = broker_resources(
            BufReader::new(stdout),
            &mut stdin,
            request.catalog,
            request.grammar,
            grammar,
            self.max_grammar_bytes,
            self.max_grammar_graph_bytes,
            self.max_grammar_resources,
        );
        let helper_result = match timeout(self.deadline, interaction).await {
            Ok(result) => result?,
            Err(_) => {
                reader.abort();
                let _ = child.kill().await;
                return Err(ValidationError::Timeout(self.deadline));
            }
        };
        drop(stdin);
        let status = match timeout(self.deadline, child.wait()).await {
            Ok(status) => status.map_err(ValidationError::Wait)?,
            Err(_) => {
                reader.abort();
                let _ = child.kill().await;
                return Err(ValidationError::Timeout(self.deadline));
            }
        };
        let (bytes, stderr_truncated) = reader.await.map_err(ValidationError::ReadTask)??;

        if stderr_truncated {
            warn!(
                action = "validation.output_truncated",
                result = "limited",
                engine,
            );
        }
        let stderr = String::from_utf8_lossy(&bytes);
        let mut diagnostics = normalize_diagnostics(
            &stderr,
            request.source,
            request.revision,
            request.file,
            engine,
            self.max_diagnostics,
        );
        if (!status.success() || !helper_result.valid) && diagnostics.is_empty() {
            let index = LineIndex::new(request.source);
            let mut span = index.span(request.source, 0, 0);
            span.approximate = true;
            diagnostics.push(Diagnostic {
                severity: Severity::Error,
                message: "validation engine exited without a structured diagnostic".to_owned(),
                source: engine.to_owned(),
                rule: None,
                file: request.file.map(ToOwned::to_owned),
                span,
                revision: request.revision,
            });
        }
        Ok(ValidationReport {
            engine,
            revision: request.revision,
            diagnostics,
            stderr_truncated,
            approved_grammar_resources: helper_result.approved_grammar_resources,
        })
    }
}

#[derive(Serialize)]
struct HelperRequest<'a> {
    mode: &'static str,
    grammar_uri: &'a str,
    public_id: Option<&'a str>,
    document_bytes: usize,
}

impl<'a> HelperRequest<'a> {
    fn new(grammar: GrammarReference<'a>, document_bytes: usize) -> Self {
        let (mode, grammar_uri, public_id) = match grammar {
            GrammarReference::Dtd {
                system_id,
                public_id,
            } => ("dtd", system_id, public_id),
            GrammarReference::Xsd { uri } => ("xsd", uri, None),
            GrammarReference::RelaxNg { uri } => ("relax_ng", uri, None),
            GrammarReference::RelaxNgCompact { .. } => ("unsupported", "", None),
            GrammarReference::Schematron { uri } => ("schematron", uri, None),
        };
        Self {
            mode,
            grammar_uri,
            public_id,
            document_bytes,
        }
    }
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
    approved_grammar_resources: usize,
}

#[allow(clippy::too_many_arguments)]
async fn broker_resources(
    mut stdout: BufReader<tokio::process::ChildStdout>,
    stdin: &mut tokio::process::ChildStdin,
    catalog: &CatalogResolver,
    grammar_reference: GrammarReference<'_>,
    root_grammar: ResolvedResource,
    max_resource_bytes: u64,
    max_graph_bytes: u64,
    max_resources: usize,
) -> Result<BrokerResult, ValidationError> {
    let mut line = String::new();
    let mut cached = HashMap::<PathBuf, Vec<u8>>::new();
    let mut total_bytes = 0_u64;
    let mut first_error = None;

    loop {
        line.clear();
        if stdout
            .read_line(&mut line)
            .await
            .map_err(ValidationError::Read)?
            == 0
        {
            return Err(first_error.unwrap_or_else(|| {
                ValidationError::Protocol("helper closed stdout before reporting a result".into())
            }));
        }
        let message = serde_json::from_str::<HelperMessage>(line.trim_end())
            .map_err(|error| ValidationError::Protocol(error.to_string()))?;
        match message {
            HelperMessage::Resource { url, public_id } => {
                let resolution = resolve_broker_resource(
                    catalog,
                    grammar_reference,
                    &root_grammar,
                    &url,
                    public_id.as_deref(),
                )
                .and_then(|resource| {
                    if let Some(bytes) = cached.get(&resource.path) {
                        return Ok((resource, bytes.clone()));
                    }
                    let metadata = std::fs::metadata(&resource.path).map_err(|source| {
                        ValidationError::GrammarIo {
                            path: resource.path.clone(),
                            source,
                        }
                    })?;
                    if metadata.len() > max_resource_bytes {
                        return Err(ValidationError::GrammarTooLarge {
                            path: resource.path.clone(),
                            limit: max_resource_bytes,
                        });
                    }
                    if cached.len().saturating_add(1) > max_resources {
                        return Err(ValidationError::TooManyGrammarResources {
                            limit: max_resources,
                        });
                    }
                    let next_total = total_bytes.saturating_add(metadata.len());
                    if next_total > max_graph_bytes {
                        return Err(ValidationError::GrammarGraphTooLarge {
                            limit: max_graph_bytes,
                        });
                    }
                    let bytes = std::fs::read(&resource.path).map_err(|source| {
                        ValidationError::GrammarIo {
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
                        let uri = resource.uri.as_str();
                        write_resource_reply(stdin, Some(uri), &bytes).await?;
                    }
                    Err(error) => {
                        if first_error.is_none() {
                            first_error = Some(error);
                        }
                        write_resource_reply(stdin, None, &[]).await?;
                    }
                }
            }
            HelperMessage::Result { valid } => {
                if let Some(error) = first_error {
                    return Err(error);
                }
                return Ok(BrokerResult {
                    valid,
                    approved_grammar_resources: cached.len(),
                });
            }
        }
    }
}

fn resolve_broker_resource(
    catalog: &CatalogResolver,
    grammar_reference: GrammarReference<'_>,
    root: &ResolvedResource,
    url: &str,
    public_id: Option<&str>,
) -> Result<ResolvedResource, ValidationError> {
    let is_root = match grammar_reference {
        GrammarReference::Dtd {
            system_id,
            public_id: expected_public,
        } => url == system_id || (public_id.is_some() && public_id == expected_public),
        GrammarReference::Xsd { uri } => url == uri,
        GrammarReference::RelaxNg { uri } => url == uri,
        GrammarReference::RelaxNgCompact { uri } => url == uri,
        GrammarReference::Schematron { uri } => url == uri,
    };
    if is_root {
        return Ok(root.clone());
    }
    if let Some(resource) = catalog.resolve_external(Some(url), public_id)? {
        return Ok(resource);
    }
    let normalized = if Path::new(url).is_absolute() {
        url::Url::from_file_path(url)
            .map_err(|()| ValidationError::Protocol("invalid absolute resource path".into()))?
            .to_string()
    } else {
        url.to_owned()
    };
    catalog
        .resolve_uri_or_direct(&normalized)
        .map_err(ValidationError::from)
}

async fn write_resource_reply(
    stdin: &mut tokio::process::ChildStdin,
    uri: Option<&str>,
    bytes: &[u8],
) -> Result<(), ValidationError> {
    let header = serde_json::to_vec(&ResourceReply {
        allowed: uri.is_some(),
        uri,
        bytes: bytes.len(),
    })
    .map_err(|error| ValidationError::Protocol(error.to_string()))?;
    stdin
        .write_all(&header)
        .await
        .map_err(ValidationError::Write)?;
    stdin
        .write_all(b"\n")
        .await
        .map_err(ValidationError::Write)?;
    stdin
        .write_all(bytes)
        .await
        .map_err(ValidationError::Write)?;
    stdin
        .write_all(b"\n")
        .await
        .map_err(ValidationError::Write)?;
    stdin.flush().await.map_err(ValidationError::Write)
}

async fn read_bounded<R: AsyncRead + Unpin>(
    mut reader: R,
    limit: u64,
) -> Result<(Vec<u8>, bool), ValidationError> {
    let capacity = usize::try_from(limit.min(64 * 1024)).unwrap_or(64 * 1024);
    let mut kept = Vec::with_capacity(capacity);
    let mut buffer = [0_u8; 8192];
    let mut truncated = false;
    loop {
        let read = reader
            .read(&mut buffer)
            .await
            .map_err(ValidationError::Read)?;
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

impl ValidationEngine for Libxml2ProcessEngine {
    fn name(&self) -> &'static str {
        "libxml2-process"
    }

    fn validate<'a>(&'a self, request: ValidationRequest<'a>) -> ValidationFuture<'a> {
        Box::pin(async move { self.validate_inner(request).await })
    }
}

fn resolve_grammar(
    catalog: &CatalogResolver,
    grammar: GrammarReference<'_>,
) -> Result<ResolvedResource, ValidationError> {
    let resolved = match grammar {
        GrammarReference::Dtd {
            system_id,
            public_id,
        } => catalog.resolve_external(Some(system_id), public_id)?,
        GrammarReference::Xsd { uri } => catalog.resolve_uri(uri)?,
        GrammarReference::RelaxNg { uri } => catalog.resolve_uri(uri)?,
        GrammarReference::RelaxNgCompact { uri } => catalog.resolve_uri(uri)?,
        GrammarReference::Schematron { uri } => catalog.resolve_uri(uri)?,
    };
    resolved.ok_or(ValidationError::GrammarNotFound)
}

fn mask_document_type(source: &str) -> Cow<'_, str> {
    let Some((start, end)) = document_type_span(source) else {
        return Cow::Borrowed(source);
    };
    let mut masked = source.as_bytes().to_vec();
    for byte in &mut masked[start..end] {
        if !matches!(*byte, b'\n' | b'\r') {
            *byte = b' ';
        }
    }
    Cow::Owned(String::from_utf8(masked).unwrap_or_else(|_| source.to_owned()))
}

fn has_external_doctype(source: &str) -> bool {
    document_type_span(source).is_some_and(|(start, end)| {
        let declaration = &source[start..end];
        contains_xml_keyword(declaration, "SYSTEM") || contains_xml_keyword(declaration, "PUBLIC")
    })
}

fn document_type_span(source: &str) -> Option<(usize, usize)> {
    let mut reader = quick_xml::Reader::from_str(source);
    loop {
        let start = usize::try_from(reader.buffer_position()).ok()?;
        match reader.read_event() {
            Ok(quick_xml::events::Event::DocType(_)) => {
                let end = usize::try_from(reader.buffer_position()).ok()?;
                return Some((start, end));
            }
            Ok(quick_xml::events::Event::Eof) | Err(_) => return None,
            Ok(_) => {}
        }
    }
}

fn contains_xml_keyword(source: &str, expected: &str) -> bool {
    source
        .split(|character: char| !character.is_ascii_alphabetic())
        .any(|word| word.eq_ignore_ascii_case(expected))
}

fn normalize_diagnostics(
    stderr: &str,
    source: &str,
    revision: Revision,
    file: Option<&str>,
    engine: &'static str,
    limit: usize,
) -> Vec<Diagnostic> {
    stderr
        .lines()
        .filter_map(|line| diagnostic_from_line(line, source, revision, file, engine))
        .take(limit)
        .collect()
}

fn diagnostic_from_line(
    line: &str,
    source: &str,
    revision: Revision,
    file: Option<&str>,
    engine: &'static str,
) -> Option<Diagnostic> {
    let mut parts = line.splitn(3, ':');
    let _origin = parts.next()?;
    let line_number = parts.next()?.trim().parse::<usize>().ok()?;
    let details = parts.next()?.trim();
    if details.is_empty() {
        return None;
    }
    let severity = if details.to_ascii_lowercase().contains("warning") {
        Severity::Warning
    } else {
        Severity::Error
    };
    let message = details
        .split_once(" : ")
        .map_or(details, |(_, message)| message)
        .trim()
        .to_owned();
    let span = approximate_line_span(source, line_number);
    Some(Diagnostic {
        severity,
        message,
        source: engine.to_owned(),
        rule: None,
        file: file.map(ToOwned::to_owned),
        span,
        revision,
    })
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

fn error_class(error: &ValidationError) -> &'static str {
    match error {
        ValidationError::Catalog(_) => "catalog",
        ValidationError::GrammarNotFound => "grammar_not_found",
        ValidationError::GrammarIo { .. } => "grammar_io",
        ValidationError::GrammarTooLarge { .. } => "grammar_too_large",
        ValidationError::TooManyGrammarResources { .. } => "too_many_grammar_resources",
        ValidationError::GrammarGraphTooLarge { .. } => "grammar_graph_too_large",
        ValidationError::DocumentTooLarge { .. } => "document_too_large",
        ValidationError::ExternalDocumentType => "external_document_type",
        ValidationError::Spawn(_) => "spawn",
        ValidationError::MissingStdin => "missing_stdin",
        ValidationError::MissingStdout => "missing_stdout",
        ValidationError::MissingStderr => "missing_stderr",
        ValidationError::Protocol(_) => "protocol",
        ValidationError::Write(_) => "write",
        ValidationError::Read(_) => "read",
        ValidationError::ReadTask(_) => "read_task",
        ValidationError::Wait(_) => "wait",
        ValidationError::Timeout(_) => "timeout",
        ValidationError::InvalidExecutable(_) => "invalid_executable",
        ValidationError::InvalidEngineAsset(_) => "invalid_engine_asset",
        ValidationError::UnsupportedGrammar => "unsupported_grammar",
        ValidationError::CompactGrammarResourcesUnsupported => {
            "compact_grammar_resources_unsupported"
        }
        ValidationError::Staging(_) => "staging",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_libxml_diagnostics_to_source_lines() {
        let source = "<article>\n  <title/>\n</article>";
        let diagnostics = normalize_diagnostics(
            "-:2: element title: validity error : Element article content does not follow",
            source,
            Revision(7),
            Some("article.xml"),
            "libxml2-dtd",
            10,
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].span.start.line, 2);
        assert!(diagnostics[0].span.approximate);
        assert_eq!(diagnostics[0].revision, Revision(7));
    }

    #[test]
    fn masks_external_or_internal_doctype_without_moving_lines() {
        let source = "<?xml version=\"1.0\"?>\n<!DOCTYPE article PUBLIC \"public>id\" \"schema.dtd\" [\n<!ENTITY local \"value>\">\n]>\n<article/>";
        let masked = mask_document_type(source);
        assert!(!masked.contains("DOCTYPE"));
        assert!(masked.ends_with("<article/>"));
        assert_eq!(masked.len(), source.len());
        assert_eq!(masked.lines().count(), source.lines().count());
    }

    #[test]
    fn leaves_documents_without_doctype_borrowed() {
        assert!(matches!(mask_document_type("<article/>"), Cow::Borrowed(_)));
    }

    #[test]
    fn does_not_mask_doctype_text_inside_comments() {
        let source = "<!-- example: <!DOCTYPE article SYSTEM 'example.dtd'> --><article/>";
        assert!(matches!(mask_document_type(source), Cow::Borrowed(_)));
        assert!(!has_external_doctype(source));
    }
}
