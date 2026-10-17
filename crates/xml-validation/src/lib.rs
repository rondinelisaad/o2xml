use std::{
    collections::{HashSet, VecDeque},
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    process::Stdio,
    time::{Duration, Instant},
};

use quick_xml::{
    events::{BytesStart, Event},
    name::ResolveResult,
    reader::NsReader,
    XmlVersion,
};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
    time::timeout,
};
use tracing::{info, warn};
use xml_document::LineIndex;
use xml_domain::{Diagnostic, Revision, Severity, SourceSpan};
use xml_resource::{CatalogError, CatalogResolver, ResolvedResource};

const XML_SCHEMA_NAMESPACE: &[u8] = b"http://www.w3.org/2001/XMLSchema";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GrammarReference<'a> {
    Dtd {
        system_id: &'a str,
        public_id: Option<&'a str>,
    },
    Xsd {
        uri: &'a str,
    },
}

impl GrammarReference<'_> {
    fn engine_name(self) -> &'static str {
        match self {
            Self::Dtd { .. } => "libxml2-dtd",
            Self::Xsd { .. } => "libxml2-xsd",
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
    #[error("invalid XML Schema document {path}: {message}")]
    InvalidGrammarXml { path: PathBuf, message: String },
    #[error("xml:base is not supported in schema dependency preflight: {0}")]
    UnsupportedSchemaBase(PathBuf),
    #[error("document exceeds the {limit}-byte validation limit")]
    DocumentTooLarge { limit: usize },
    #[error("standalone adapter rejected a grammar with external dependencies")]
    ExternalGrammarDependency,
    #[error("standalone adapter rejected an XML document with an external DOCTYPE")]
    ExternalDocumentType,
    #[error("cannot start validation engine: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("validation engine stdin is unavailable")]
    MissingStdin,
    #[error("validation engine stderr is unavailable")]
    MissingStderr,
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
        if request.source.len() > self.max_document_bytes {
            return Err(ValidationError::DocumentTooLarge {
                limit: self.max_document_bytes,
            });
        }
        if has_external_doctype(request.source) {
            return Err(ValidationError::ExternalDocumentType);
        }

        let grammar = resolve_grammar(request.catalog, request.grammar)?;
        let approved_grammar_resources =
            self.preflight_grammar(request.catalog, &grammar, request.grammar)?;

        let mut command = Command::new(&self.executable);
        command
            .arg("--nonet")
            .arg("--noout")
            .env_clear()
            .env("XML_CATALOG_FILES", request.catalog.catalog_path())
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        match request.grammar {
            GrammarReference::Dtd { .. } => {
                command.arg("--dtdvalid").arg(&grammar.path);
            }
            GrammarReference::Xsd { .. } => {
                command.arg("--schema").arg(&grammar.path);
            }
        }
        command.arg("-");

        let mut child = command.spawn().map_err(ValidationError::Spawn)?;
        let mut stdin = child.stdin.take().ok_or(ValidationError::MissingStdin)?;
        timeout(self.deadline, stdin.write_all(request.source.as_bytes()))
            .await
            .map_err(|_| ValidationError::Timeout(self.deadline))?
            .map_err(ValidationError::Write)?;
        drop(stdin);

        let stderr = child.stderr.take().ok_or(ValidationError::MissingStderr)?;
        let max_output = self.max_stderr_bytes.saturating_add(1);
        let reader = tokio::spawn(async move {
            let mut bounded_stderr = stderr.take(max_output);
            let mut bytes = Vec::new();
            bounded_stderr
                .read_to_end(&mut bytes)
                .await
                .map_err(ValidationError::Read)?;
            Ok::<_, ValidationError>(bytes)
        });
        let status = match timeout(self.deadline, child.wait()).await {
            Ok(status) => status.map_err(ValidationError::Wait)?,
            Err(_) => {
                reader.abort();
                return Err(ValidationError::Timeout(self.deadline));
            }
        };
        let mut bytes = reader.await.map_err(ValidationError::ReadTask)??;

        let stderr_truncated =
            u64::try_from(bytes.len()).unwrap_or(u64::MAX) > self.max_stderr_bytes;
        if stderr_truncated {
            bytes.truncate(usize::try_from(self.max_stderr_bytes).unwrap_or(usize::MAX));
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
        if !status.success() && diagnostics.is_empty() {
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
            approved_grammar_resources,
        })
    }

    fn preflight_grammar(
        &self,
        catalog: &CatalogResolver,
        grammar: &ResolvedResource,
        reference: GrammarReference<'_>,
    ) -> Result<usize, ValidationError> {
        match reference {
            GrammarReference::Dtd { .. } => {
                let source = self.read_grammar(grammar)?;
                if dtd_has_external_dependency(&source) {
                    return Err(ValidationError::ExternalGrammarDependency);
                }
                Ok(1)
            }
            GrammarReference::Xsd { .. } => self.preflight_xsd_graph(catalog, grammar),
        }
    }

    fn preflight_xsd_graph(
        &self,
        catalog: &CatalogResolver,
        root: &ResolvedResource,
    ) -> Result<usize, ValidationError> {
        let mut pending = VecDeque::from([root.clone()]);
        let mut visited = HashSet::new();
        let mut total_bytes = 0_u64;

        while let Some(resource) = pending.pop_front() {
            if !visited.insert(resource.path.clone()) {
                continue;
            }
            if visited.len() > self.max_grammar_resources {
                return Err(ValidationError::TooManyGrammarResources {
                    limit: self.max_grammar_resources,
                });
            }
            let metadata = self.grammar_metadata(&resource)?;
            total_bytes = total_bytes.saturating_add(metadata.len());
            if total_bytes > self.max_grammar_graph_bytes {
                return Err(ValidationError::GrammarGraphTooLarge {
                    limit: self.max_grammar_graph_bytes,
                });
            }
            let source = self.read_grammar(&resource)?;
            if has_external_doctype(&source) {
                return Err(ValidationError::ExternalGrammarDependency);
            }
            for reference in xsd_dependency_references(&source, &resource.path)? {
                pending.push_back(catalog.resolve_reference(&resource, &reference)?);
            }
        }
        Ok(visited.len())
    }

    fn grammar_metadata(
        &self,
        grammar: &ResolvedResource,
    ) -> Result<std::fs::Metadata, ValidationError> {
        let metadata =
            std::fs::metadata(&grammar.path).map_err(|source| ValidationError::GrammarIo {
                path: grammar.path.clone(),
                source,
            })?;
        if metadata.len() > self.max_grammar_bytes {
            return Err(ValidationError::GrammarTooLarge {
                path: grammar.path.clone(),
                limit: self.max_grammar_bytes,
            });
        }
        Ok(metadata)
    }

    fn read_grammar(&self, grammar: &ResolvedResource) -> Result<String, ValidationError> {
        self.grammar_metadata(grammar)?;
        std::fs::read_to_string(&grammar.path).map_err(|source| ValidationError::GrammarIo {
            path: grammar.path.clone(),
            source,
        })
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
    };
    resolved.ok_or(ValidationError::GrammarNotFound)
}

fn dtd_has_external_dependency(source: &str) -> bool {
    contains_xml_keyword(source, "SYSTEM") || contains_xml_keyword(source, "PUBLIC")
}

fn has_external_doctype(source: &str) -> bool {
    let upper = source.to_ascii_uppercase();
    upper.contains("<!DOCTYPE")
        && (contains_xml_keyword(source, "SYSTEM") || contains_xml_keyword(source, "PUBLIC"))
}

fn contains_xml_keyword(source: &str, expected: &str) -> bool {
    source
        .split(|character: char| !character.is_ascii_alphabetic())
        .any(|word| word.eq_ignore_ascii_case(expected))
}

fn xsd_dependency_references(source: &str, path: &Path) -> Result<Vec<String>, ValidationError> {
    let mut reader = NsReader::from_str(source);
    reader.config_mut().trim_text(true);
    let mut references = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(element) | Event::Empty(element)) => {
                if schema_attribute(&element, b"xml:base", path)?.is_some() {
                    return Err(ValidationError::UnsupportedSchemaBase(path.to_path_buf()));
                }
                let resolver = reader.resolver().clone();
                let is_schema = matches!(
                    resolver.resolve_element(element.name()).0,
                    ResolveResult::Bound(namespace) if namespace.as_ref() == XML_SCHEMA_NAMESPACE
                );
                if is_schema
                    && matches!(
                        element.local_name().as_ref(),
                        b"include" | b"import" | b"redefine" | b"override"
                    )
                {
                    if let Some(reference) = schema_attribute(&element, b"schemaLocation", path)? {
                        references.push(reference);
                    }
                }
            }
            Ok(Event::Eof) => return Ok(references),
            Err(error) => {
                return Err(ValidationError::InvalidGrammarXml {
                    path: path.to_path_buf(),
                    message: error.to_string(),
                });
            }
            _ => {}
        }
    }
}

fn schema_attribute(
    element: &BytesStart<'_>,
    key: &[u8],
    path: &Path,
) -> Result<Option<String>, ValidationError> {
    for attribute in element.attributes() {
        let attribute = attribute.map_err(|error| ValidationError::InvalidGrammarXml {
            path: path.to_path_buf(),
            message: error.to_string(),
        })?;
        if attribute.key.as_ref() == key {
            return Ok(Some(
                attribute
                    .normalized_value(XmlVersion::Implicit1_0)
                    .map_err(|error| ValidationError::InvalidGrammarXml {
                        path: path.to_path_buf(),
                        message: error.to_string(),
                    })?
                    .into_owned(),
            ));
        }
    }
    Ok(None)
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
        ValidationError::InvalidGrammarXml { .. } => "invalid_grammar_xml",
        ValidationError::UnsupportedSchemaBase(_) => "unsupported_schema_base",
        ValidationError::DocumentTooLarge { .. } => "document_too_large",
        ValidationError::ExternalGrammarDependency => "external_grammar_dependency",
        ValidationError::ExternalDocumentType => "external_document_type",
        ValidationError::Spawn(_) => "spawn",
        ValidationError::MissingStdin => "missing_stdin",
        ValidationError::MissingStderr => "missing_stderr",
        ValidationError::Write(_) => "write",
        ValidationError::Read(_) => "read",
        ValidationError::ReadTask(_) => "read_task",
        ValidationError::Wait(_) => "wait",
        ValidationError::Timeout(_) => "timeout",
        ValidationError::InvalidExecutable(_) => "invalid_executable",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    use xml_resource::ResourcePolicy;

    fn fixtures() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
    }

    fn resolver() -> Result<CatalogResolver, CatalogError> {
        let fixtures = fixtures();
        let policy = ResourcePolicy::offline([fixtures.clone()])?;
        CatalogResolver::load(fixtures.join("catalog.xml"), policy)
    }

    fn runtime() -> Result<tokio::runtime::Runtime, std::io::Error> {
        tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
    }

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
    fn preflights_xsd_dependencies_and_rejects_external_dtd_entities() -> Result<(), ValidationError>
    {
        assert!(dtd_has_external_dependency(
            "<!ENTITY % remote SYSTEM \"file:///tmp/remote.dtd\">"
        ));
        let references = xsd_dependency_references(
            r#"<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema"><xs:include schemaLocation="other.xsd"/></xs:schema>"#,
            Path::new("schema.xsd"),
        )?;
        assert_eq!(references, ["other.xsd"]);
        Ok(())
    }

    #[test]
    fn validates_dtd_and_xsd_with_installed_xmllint() -> Result<(), Box<dyn Error>> {
        let executable = Path::new("/usr/bin/xmllint");
        if !executable.is_file() {
            return Ok(());
        }
        let engine = Libxml2ProcessEngine::new(executable)?;
        let catalog = resolver()?;
        let runtime = runtime()?;

        let dtd_report = runtime.block_on(engine.validate(ValidationRequest {
            source: "<article>text</article>",
            revision: Revision(1),
            file: Some("article.xml"),
            catalog: &catalog,
            grammar: GrammarReference::Dtd {
                system_id: "https://example.org/article.dtd",
                public_id: None,
            },
        }))?;
        let invalid_dtd = runtime.block_on(engine.validate(ValidationRequest {
            source: "<article><title/></article>",
            revision: Revision(2),
            file: Some("invalid.xml"),
            catalog: &catalog,
            grammar: GrammarReference::Dtd {
                system_id: "https://example.org/article.dtd",
                public_id: None,
            },
        }))?;
        let xsd_report = runtime.block_on(engine.validate(ValidationRequest {
            source: "<article/>",
            revision: Revision(3),
            file: Some("article.xml"),
            catalog: &catalog,
            grammar: GrammarReference::Xsd {
                uri: "https://example.org/article.xsd",
            },
        }))?;
        let composite_report = runtime.block_on(engine.validate(ValidationRequest {
            source: r#"<a:article xmlns:a="urn:article" xmlns:c="urn:common"><c:common>ok</c:common></a:article>"#,
            revision: Revision(4),
            file: Some("composite.xml"),
            catalog: &catalog,
            grammar: GrammarReference::Xsd {
                uri: "https://example.org/composite.xsd",
            },
        }))?;

        assert!(dtd_report.diagnostics.is_empty());
        assert!(!invalid_dtd.diagnostics.is_empty());
        assert!(xsd_report.diagnostics.is_empty());
        assert!(composite_report.diagnostics.is_empty());
        assert_eq!(composite_report.approved_grammar_resources, 2);
        Ok(())
    }

    #[test]
    fn rejects_xsd_dependency_outside_allowed_roots() -> Result<(), Box<dyn Error>> {
        let executable = Path::new("/usr/bin/xmllint");
        if !executable.is_file() {
            return Ok(());
        }
        let engine = Libxml2ProcessEngine::new(executable)?;
        let catalog = resolver()?;
        let runtime = runtime()?;
        let result = runtime.block_on(engine.validate(ValidationRequest {
            source: "<article/>",
            revision: Revision(5),
            file: Some("article.xml"),
            catalog: &catalog,
            grammar: GrammarReference::Xsd {
                uri: "https://example.org/escape.xsd",
            },
        }));

        assert!(
            matches!(
                &result,
                Err(ValidationError::Catalog(CatalogError::PathDenied(_)))
            ),
            "unexpected validation result: {result:?}"
        );
        Ok(())
    }
}
