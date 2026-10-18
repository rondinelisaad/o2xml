use std::{
    collections::HashMap,
    fs,
    ops::Range,
    path::{Path, PathBuf},
    process::Stdio,
    time::{Duration, Instant},
};

use tempfile::TempDir;
use tokio::{process::Command, time::timeout};
use tracing::info;
use xml_document::LineIndex;
use xml_domain::{Diagnostic, Severity};
use xml_resource::{CatalogResolver, ResolvedResource};

use super::{
    error_class, has_external_doctype, read_bounded, GrammarReference, ValidationEngine,
    ValidationError, ValidationFuture, ValidationReport, ValidationRequest,
};

#[derive(Clone, Debug)]
pub struct JingProcessEngine {
    java_executable: PathBuf,
    jing_jar: PathBuf,
    deadline: Duration,
    max_document_bytes: usize,
    max_grammar_bytes: u64,
    max_grammar_graph_bytes: u64,
    max_grammar_resources: usize,
    max_stderr_bytes: u64,
    max_diagnostics: usize,
}

impl JingProcessEngine {
    pub fn new(
        java_executable: impl AsRef<Path>,
        jing_jar: impl AsRef<Path>,
    ) -> Result<Self, ValidationError> {
        let java_executable =
            canonical_file(java_executable.as_ref(), ValidationError::InvalidExecutable)?;
        let jing_jar = canonical_file(jing_jar.as_ref(), ValidationError::InvalidEngineAsset)?;
        Ok(Self {
            java_executable,
            jing_jar,
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
        let result = self.run(request).await;
        let (result_name, diagnostic_count, error_class) = match &result {
            Ok(report) if report.diagnostics.is_empty() => ("success", 0, None),
            Ok(report) => ("failure", report.diagnostics.len(), None),
            Err(error) => ("failure", 0, Some(error_class(error))),
        };
        info!(
            audit = true,
            action = "validation.run",
            result = result_name,
            resource = "document",
            revision = request.revision.0,
            engine = "jing-rnc",
            duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
            diagnostic_count,
            error_class,
        );
        result
    }

    async fn run(
        &self,
        request: ValidationRequest<'_>,
    ) -> Result<ValidationReport, ValidationError> {
        let GrammarReference::RelaxNgCompact { uri } = request.grammar else {
            return Err(ValidationError::UnsupportedGrammar);
        };
        if request.source.len() > self.max_document_bytes {
            return Err(ValidationError::DocumentTooLarge {
                limit: self.max_document_bytes,
            });
        }
        if has_external_doctype(request.source) {
            return Err(ValidationError::ExternalDocumentType);
        }
        let grammar = request
            .catalog
            .resolve_uri(uri)?
            .ok_or(ValidationError::GrammarNotFound)?;
        let staged = stage_compact_grammar_graph(
            request.catalog,
            grammar,
            request.source.as_bytes(),
            self.max_grammar_bytes,
            self.max_grammar_graph_bytes,
            self.max_grammar_resources,
        )?;
        let mut command = Command::new(&self.java_executable);
        command
            .env_clear()
            .current_dir(staged.directory.path())
            .arg("-Djava.awt.headless=true")
            .arg("-Djava.net.useSystemProxies=false")
            .arg("-jar")
            .arg(&self.jing_jar)
            .arg("-c")
            .arg(&staged.grammar_path)
            .arg(&staged.document_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        let mut child = command.spawn().map_err(ValidationError::Spawn)?;
        let stderr = child.stderr.take().ok_or(ValidationError::MissingStderr)?;
        let reader = tokio::spawn(read_bounded(stderr, self.max_stderr_bytes));
        let status = match timeout(self.deadline, child.wait()).await {
            Ok(status) => status.map_err(ValidationError::Wait)?,
            Err(_) => {
                reader.abort();
                let _ = child.kill().await;
                return Err(ValidationError::Timeout(self.deadline));
            }
        };
        let (stderr, stderr_truncated) = reader.await.map_err(ValidationError::ReadTask)??;
        let mut diagnostics = normalize_jing_diagnostics(
            &String::from_utf8_lossy(&stderr),
            request,
            self.max_diagnostics,
        );
        if !status.success() && diagnostics.is_empty() {
            let index = LineIndex::new(request.source);
            let mut span = index.span(request.source, 0, 0);
            span.approximate = true;
            diagnostics.push(Diagnostic {
                severity: Severity::Error,
                message: "Jing exited without a structured diagnostic".to_owned(),
                source: "jing-rnc".to_owned(),
                rule: None,
                file: request.file.map(ToOwned::to_owned),
                span,
                revision: request.revision,
            });
        }
        Ok(ValidationReport {
            engine: "jing-rnc",
            revision: request.revision,
            diagnostics,
            stderr_truncated,
            approved_grammar_resources: staged.resource_count,
        })
    }
}

impl ValidationEngine for JingProcessEngine {
    fn name(&self) -> &'static str {
        "jing-rnc"
    }

    fn validate<'a>(&'a self, request: ValidationRequest<'a>) -> ValidationFuture<'a> {
        Box::pin(async move { self.validate_inner(request).await })
    }
}

fn canonical_file(
    path: &Path,
    invalid: fn(PathBuf) -> ValidationError,
) -> Result<PathBuf, ValidationError> {
    let canonical = fs::canonicalize(path).map_err(ValidationError::Spawn)?;
    if !canonical.is_file() {
        return Err(invalid(canonical));
    }
    Ok(canonical)
}

struct StagedCompactGrammar {
    directory: TempDir,
    grammar_path: PathBuf,
    document_path: PathBuf,
    resource_count: usize,
}

struct CompactGraphStager<'a> {
    catalog: &'a CatalogResolver,
    directory: &'a Path,
    names: HashMap<PathBuf, String>,
    total_bytes: u64,
    max_resource_bytes: u64,
    max_total_bytes: u64,
    max_resources: usize,
}

fn stage_compact_grammar_graph(
    catalog: &CatalogResolver,
    grammar: ResolvedResource,
    document: &[u8],
    max_resource_bytes: u64,
    max_total_bytes: u64,
    max_resources: usize,
) -> Result<StagedCompactGrammar, ValidationError> {
    let directory = TempDir::new().map_err(ValidationError::Staging)?;
    let mut stager = CompactGraphStager {
        catalog,
        directory: directory.path(),
        names: HashMap::new(),
        total_bytes: 0,
        max_resource_bytes,
        max_total_bytes,
        max_resources,
    };
    let grammar_name = stager.stage(grammar)?;
    let resource_count = stager.names.len();
    let grammar_path = directory.path().join(grammar_name);
    let document_path = directory.path().join("document.xml");
    fs::write(&document_path, document).map_err(ValidationError::Staging)?;
    Ok(StagedCompactGrammar {
        directory,
        grammar_path,
        document_path,
        resource_count,
    })
}

impl CompactGraphStager<'_> {
    fn stage(&mut self, resource: ResolvedResource) -> Result<String, ValidationError> {
        if let Some(name) = self.names.get(&resource.path) {
            return Ok(name.clone());
        }
        if self.names.len() >= self.max_resources {
            return Err(ValidationError::TooManyGrammarResources {
                limit: self.max_resources,
            });
        }
        let name = format!("grammar-{}.rnc", self.names.len());
        self.names.insert(resource.path.clone(), name.clone());

        let metadata =
            fs::metadata(&resource.path).map_err(|source| ValidationError::GrammarIo {
                path: resource.path.clone(),
                source,
            })?;
        if metadata.len() > self.max_resource_bytes {
            return Err(ValidationError::GrammarTooLarge {
                path: resource.path,
                limit: self.max_resource_bytes,
            });
        }
        self.total_bytes = self.total_bytes.saturating_add(metadata.len());
        if self.total_bytes > self.max_total_bytes {
            return Err(ValidationError::GrammarGraphTooLarge {
                limit: self.max_total_bytes,
            });
        }
        let bytes = fs::read(&resource.path).map_err(|source| ValidationError::GrammarIo {
            path: resource.path.clone(),
            source,
        })?;
        if bytes
            .iter()
            .copied()
            .find(|byte| !byte.is_ascii_whitespace())
            == Some(b'<')
        {
            return Err(ValidationError::CompactGrammarResourcesUnsupported);
        }
        let directives = compact_resource_directives(&bytes)?;
        let mut replacements = Vec::with_capacity(directives.len());
        for directive in directives {
            let dependency = self
                .catalog
                .resolve_reference(&resource, &directive.target)?;
            let staged_name = self.stage(dependency)?;
            replacements.push((directive.literal, format!("\"{staged_name}\"")));
        }
        let rewritten = rewrite_directives(&bytes, replacements)?;
        fs::write(self.directory.join(&name), rewritten).map_err(ValidationError::Staging)?;
        Ok(name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CompactDirective {
    literal: Range<usize>,
    target: String,
}

fn compact_resource_directives(bytes: &[u8]) -> Result<Vec<CompactDirective>, ValidationError> {
    let source = std::str::from_utf8(bytes)
        .map_err(|_| ValidationError::CompactGrammarResourcesUnsupported)?;
    let bytes = source.as_bytes();
    let mut directives = Vec::new();
    let mut cursor = 0_usize;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'#' => skip_comment(bytes, &mut cursor),
            b'\'' | b'"' => skip_literal(bytes, &mut cursor)?,
            b'\\' => {
                cursor = cursor.saturating_add(1);
                skip_identifier(bytes, &mut cursor);
            }
            byte if is_identifier_start(byte) => {
                let start = cursor;
                skip_identifier(bytes, &mut cursor);
                if matches!(&source[start..cursor], "include" | "external") {
                    skip_trivia(bytes, &mut cursor)?;
                    let literal_start = cursor;
                    let target = parse_resource_literal(source, bytes, &mut cursor)?;
                    directives.push(CompactDirective {
                        literal: literal_start..cursor,
                        target,
                    });
                }
            }
            _ => cursor = cursor.saturating_add(1),
        }
    }
    Ok(directives)
}

fn skip_trivia(bytes: &[u8], cursor: &mut usize) -> Result<(), ValidationError> {
    loop {
        while bytes.get(*cursor).is_some_and(u8::is_ascii_whitespace) {
            *cursor = cursor.saturating_add(1);
        }
        if bytes.get(*cursor) == Some(&b'#') {
            skip_comment(bytes, cursor);
        } else {
            break;
        }
    }
    if *cursor >= bytes.len() {
        return Err(ValidationError::CompactGrammarResourcesUnsupported);
    }
    Ok(())
}

fn skip_comment(bytes: &[u8], cursor: &mut usize) {
    while let Some(byte) = bytes.get(*cursor) {
        *cursor = cursor.saturating_add(1);
        if matches!(byte, b'\n' | b'\r') {
            break;
        }
    }
}

fn skip_identifier(bytes: &[u8], cursor: &mut usize) {
    while bytes
        .get(*cursor)
        .is_some_and(|byte| is_identifier_part(*byte))
    {
        *cursor = cursor.saturating_add(1);
    }
}

fn skip_literal(bytes: &[u8], cursor: &mut usize) -> Result<(), ValidationError> {
    let quote = bytes[*cursor];
    let triple = bytes.get(*cursor..cursor.saturating_add(3)) == Some(&[quote, quote, quote]);
    *cursor = cursor.saturating_add(if triple { 3 } else { 1 });
    while *cursor < bytes.len() {
        if triple && bytes.get(*cursor..cursor.saturating_add(3)) == Some(&[quote, quote, quote]) {
            *cursor = cursor.saturating_add(3);
            return Ok(());
        }
        if !triple && bytes[*cursor] == b'\\' {
            *cursor = cursor.saturating_add(2).min(bytes.len());
        } else if !triple && bytes[*cursor] == quote {
            *cursor = cursor.saturating_add(1);
            return Ok(());
        } else {
            *cursor = cursor.saturating_add(1);
        }
    }
    Err(ValidationError::CompactGrammarResourcesUnsupported)
}

fn parse_resource_literal(
    source: &str,
    bytes: &[u8],
    cursor: &mut usize,
) -> Result<String, ValidationError> {
    let quote = *bytes
        .get(*cursor)
        .filter(|byte| matches!(byte, b'\'' | b'"'))
        .ok_or(ValidationError::CompactGrammarResourcesUnsupported)?;
    if bytes.get(*cursor..cursor.saturating_add(3)) == Some(&[quote, quote, quote]) {
        return Err(ValidationError::CompactGrammarResourcesUnsupported);
    }
    *cursor = cursor.saturating_add(1);
    let start = *cursor;
    while let Some(byte) = bytes.get(*cursor) {
        if *byte == b'\\' || matches!(byte, b'\n' | b'\r') {
            return Err(ValidationError::CompactGrammarResourcesUnsupported);
        }
        if *byte == quote {
            if *cursor == start {
                return Err(ValidationError::CompactGrammarResourcesUnsupported);
            }
            let target = source[start..*cursor].to_owned();
            *cursor = cursor.saturating_add(1);
            return Ok(target);
        }
        *cursor = cursor.saturating_add(1);
    }
    Err(ValidationError::CompactGrammarResourcesUnsupported)
}

fn rewrite_directives(
    bytes: &[u8],
    mut replacements: Vec<(Range<usize>, String)>,
) -> Result<Vec<u8>, ValidationError> {
    replacements.sort_by_key(|(range, _)| range.start);
    let mut result = bytes.to_vec();
    for (range, value) in replacements.into_iter().rev() {
        if range.start > range.end || range.end > result.len() {
            return Err(ValidationError::CompactGrammarResourcesUnsupported);
        }
        result.splice(range, value.bytes());
    }
    Ok(result)
}

const fn is_identifier_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || byte >= 0x80
}

const fn is_identifier_part(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':') || byte >= 0x80
}

fn normalize_jing_diagnostics(
    stderr: &str,
    request: ValidationRequest<'_>,
    limit: usize,
) -> Vec<Diagnostic> {
    let index = LineIndex::new(request.source);
    stderr
        .lines()
        .filter_map(|line| {
            let (location, severity, message) =
                if let Some((location, message)) = line.split_once(": error:") {
                    (location, Severity::Error, message.trim())
                } else if let Some((location, message)) = line.split_once(": warning:") {
                    (location, Severity::Warning, message.trim())
                } else {
                    return None;
                };
            let mut location_parts = location.rsplitn(3, ':');
            let column = location_parts.next()?.trim().parse::<usize>().ok()?;
            let line_number = location_parts.next()?.trim().parse::<usize>().ok()?;
            let _path = location_parts.next()?;
            let line_start = if line_number <= 1 {
                0
            } else {
                request
                    .source
                    .match_indices('\n')
                    .nth(line_number.saturating_sub(2))
                    .map_or(request.source.len(), |(byte, _)| byte.saturating_add(1))
            };
            let byte = request.source[line_start..]
                .char_indices()
                .nth(column.saturating_sub(1))
                .map_or(line_start, |(offset, _)| line_start.saturating_add(offset));
            let mut span = index.span(request.source, byte, byte);
            span.approximate = true;
            (!message.is_empty()).then(|| Diagnostic {
                severity,
                message: message.to_owned(),
                source: "jing-rnc".to_owned(),
                rule: None,
                file: request.file.map(ToOwned::to_owned),
                span,
                revision: request.revision,
            })
        })
        .take(limit)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use xml_domain::Revision;
    use xml_resource::{CatalogResolver, ResourcePolicy};

    #[test]
    fn extracts_only_resource_directives_outside_comments_and_strings(
    ) -> Result<(), ValidationError> {
        let directives = compact_resource_directives(
            b"# include 'ignored.rnc'\nstart = element article { 'external' }\ninclude # note\n 'other.rnc'\nexternal \"leaf.rnc\"",
        )?;
        assert_eq!(directives.len(), 2);
        assert_eq!(directives[0].target, "other.rnc");
        assert_eq!(directives[1].target, "leaf.rnc");
        assert!(compact_resource_directives(b"\\include = text")?.is_empty());
        assert!(compact_resource_directives(b"element ns:include { text }")?.is_empty());
        Ok(())
    }

    #[test]
    fn brokers_and_rewrites_a_bounded_compact_grammar_graph() -> Result<(), Box<dyn Error>> {
        let directory = tempfile::tempdir()?;
        let grammar = directory.path().join("article.rnc");
        let included = directory.path().join("included.rnc");
        let catalog = directory.path().join("catalog.xml");
        fs::write(
            &grammar,
            "include 'included.rnc'\nstart = article\narticle = element article { body }\n",
        )?;
        fs::write(&included, "body = element body { text }\n")?;
        fs::write(
            &catalog,
            "<catalog xmlns=\"urn:oasis:names:tc:entity:xmlns:xml:catalog\"><uri name=\"urn:test:rnc\" uri=\"article.rnc\"/></catalog>",
        )?;
        let policy = ResourcePolicy::offline([directory.path().to_path_buf()])?;
        let resolver = CatalogResolver::load(&catalog, policy)?;
        let root = resolver
            .resolve_uri("urn:test:rnc")?
            .ok_or(ValidationError::GrammarNotFound)?;
        let staged = stage_compact_grammar_graph(
            &resolver,
            root,
            b"<article><body>text</body></article>",
            1024,
            4096,
            8,
        )?;
        assert_eq!(staged.resource_count, 2);
        assert_eq!(
            fs::read_to_string(&staged.grammar_path)?,
            "include \"grammar-1.rnc\"\nstart = article\narticle = element article { body }\n"
        );
        assert_eq!(
            fs::read_to_string(staged.directory.path().join("grammar-1.rnc"))?,
            "body = element body { text }\n"
        );
        Ok(())
    }

    #[test]
    fn compact_graph_denies_dependencies_outside_allowed_roots() -> Result<(), Box<dyn Error>> {
        let directory = tempfile::tempdir()?;
        let allowed = directory.path().join("allowed");
        fs::create_dir(&allowed)?;
        fs::write(allowed.join("article.rnc"), "include '../outside.rnc'\n")?;
        fs::write(directory.path().join("outside.rnc"), "start = empty\n")?;
        let catalog = allowed.join("catalog.xml");
        fs::write(
            &catalog,
            "<catalog xmlns=\"urn:oasis:names:tc:entity:xmlns:xml:catalog\"><uri name=\"urn:test:rnc\" uri=\"article.rnc\"/></catalog>",
        )?;
        let policy = ResourcePolicy::offline([allowed])?;
        let resolver = CatalogResolver::load(&catalog, policy)?;
        let root = resolver
            .resolve_uri("urn:test:rnc")?
            .ok_or(ValidationError::GrammarNotFound)?;
        assert!(matches!(
            stage_compact_grammar_graph(&resolver, root, b"<article/>", 1024, 4096, 8),
            Err(ValidationError::Catalog(
                xml_resource::CatalogError::PathDenied(_)
            ))
        ));
        Ok(())
    }

    #[test]
    fn compact_graph_bounds_resources_and_terminates_cycles() -> Result<(), Box<dyn Error>> {
        let directory = tempfile::tempdir()?;
        let first = directory.path().join("first.rnc");
        let second = directory.path().join("second.rnc");
        let catalog = directory.path().join("catalog.xml");
        fs::write(&first, "include 'second.rnc'\nstart = empty\n")?;
        fs::write(&second, "include 'first.rnc'\n")?;
        fs::write(
            &catalog,
            "<catalog xmlns=\"urn:oasis:names:tc:entity:xmlns:xml:catalog\"><uri name=\"urn:cycle\" uri=\"first.rnc\"/></catalog>",
        )?;
        let policy = ResourcePolicy::offline([directory.path().to_path_buf()])?;
        let resolver = CatalogResolver::load(&catalog, policy)?;
        let root = resolver
            .resolve_uri("urn:cycle")?
            .ok_or(ValidationError::GrammarNotFound)?;
        let staged =
            stage_compact_grammar_graph(&resolver, root.clone(), b"<article/>", 1024, 4096, 2)?;
        assert_eq!(staged.resource_count, 2);
        assert!(matches!(
            stage_compact_grammar_graph(&resolver, root, b"<article/>", 1024, 4096, 1),
            Err(ValidationError::TooManyGrammarResources { limit: 1 })
        ));
        Ok(())
    }

    #[test]
    fn normalizes_jing_line_and_column_diagnostics() -> Result<(), Box<dyn Error>> {
        let source = "<article>\n  <front/>\n</article>";
        let directory = tempfile::tempdir()?;
        let catalog = directory.path().join("catalog.xml");
        fs::write(directory.path().join("dummy.rnc"), "start = empty\n")?;
        fs::write(
            &catalog,
            "<catalog xmlns=\"urn:oasis:names:tc:entity:xmlns:xml:catalog\"><uri name=\"urn:dummy\" uri=\"dummy.rnc\"/></catalog>",
        )?;
        let policy = ResourcePolicy::offline([directory.path().to_path_buf()])?;
        let resolver = CatalogResolver::load(catalog, policy)?;
        let request = ValidationRequest {
            source,
            revision: Revision(9),
            file: Some("article.xml"),
            catalog: &resolver,
            grammar: GrammarReference::RelaxNgCompact { uri: "urn:test" },
        };
        let diagnostics = normalize_jing_diagnostics(
            "C:\\Temp\\document.xml:2:3: error: element front not allowed",
            request,
            10,
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].span.start.line, 2);
        assert_eq!(diagnostics[0].span.start.column, 3);
        assert_eq!(diagnostics[0].message, "element front not allowed");
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn supervises_the_external_jing_contract() -> Result<(), Box<dyn Error>> {
        let directory = tempfile::tempdir()?;
        let executable = directory.path().join("fake-java");
        let jar = directory.path().join("jing.jar");
        let grammar = directory.path().join("article.rnc");
        let included = directory.path().join("included.rnc");
        let catalog = directory.path().join("catalog.xml");
        fs::write(
            &executable,
            "#!/bin/sh\nlast=\nfor arg do last=$arg; done\nwhile IFS= read -r line; do\n  case $line in\n    *'<front'*) printf '%s:1:10: error: element front not allowed\\n' \"$last\" >&2; exit 1;;\n  esac\ndone < \"$last\"\nexit 0\n",
        )?;
        let mut permissions = fs::metadata(&executable)?.permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&executable, permissions)?;
        fs::write(&jar, [])?;
        fs::write(
            &grammar,
            "include 'included.rnc'\nstart = element article { text }\n",
        )?;
        fs::write(&included, "unused = empty\n")?;
        fs::write(
            &catalog,
            "<catalog xmlns=\"urn:oasis:names:tc:entity:xmlns:xml:catalog\"><uri name=\"urn:test:rnc\" uri=\"article.rnc\"/></catalog>",
        )?;
        let policy = ResourcePolicy::offline([directory.path().to_path_buf()])?;
        let resolver = CatalogResolver::load(&catalog, policy)?;
        let engine = JingProcessEngine::new(&executable, &jar)?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()?;
        let valid = runtime.block_on(engine.validate(ValidationRequest {
            source: "<article>text</article>\n",
            revision: Revision(1),
            file: Some("valid.xml"),
            catalog: &resolver,
            grammar: GrammarReference::RelaxNgCompact {
                uri: "urn:test:rnc",
            },
        }))?;
        let invalid = runtime.block_on(engine.validate(ValidationRequest {
            source: "<article><front/></article>\n",
            revision: Revision(2),
            file: Some("invalid.xml"),
            catalog: &resolver,
            grammar: GrammarReference::RelaxNgCompact {
                uri: "urn:test:rnc",
            },
        }))?;

        assert!(valid.diagnostics.is_empty());
        assert_eq!(valid.approved_grammar_resources, 2);
        assert_eq!(invalid.diagnostics.len(), 1);
        assert_eq!(invalid.diagnostics[0].source, "jing-rnc");
        Ok(())
    }
}
