use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Stdio,
    time::{Duration, Instant},
};

use tempfile::{Builder, TempPath};
use tokio::{process::Command, time::timeout};
use tracing::info;
use xml_document::LineIndex;
use xml_domain::{Diagnostic, Severity};

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
        let metadata =
            fs::metadata(&grammar.path).map_err(|source| ValidationError::GrammarIo {
                path: grammar.path.clone(),
                source,
            })?;
        if metadata.len() > self.max_grammar_bytes {
            return Err(ValidationError::GrammarTooLarge {
                path: grammar.path,
                limit: self.max_grammar_bytes,
            });
        }
        let grammar_bytes =
            fs::read(&grammar.path).map_err(|source| ValidationError::GrammarIo {
                path: grammar.path,
                source,
            })?;
        if compact_grammar_has_resource_directive(&grammar_bytes)? {
            return Err(ValidationError::CompactGrammarResourcesUnsupported);
        }

        let grammar_path = stage_file(".rnc", &grammar_bytes)?;
        let document_path = stage_file(".xml", request.source.as_bytes())?;
        let mut command = Command::new(&self.java_executable);
        command
            .env_clear()
            .arg("-Djava.awt.headless=true")
            .arg("-Djava.net.useSystemProxies=false")
            .arg("-jar")
            .arg(&self.jing_jar)
            .arg("-c")
            .arg(grammar_path.as_os_str())
            .arg(document_path.as_os_str())
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
            approved_grammar_resources: 1,
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

fn stage_file(suffix: &str, bytes: &[u8]) -> Result<TempPath, ValidationError> {
    let mut file = Builder::new()
        .prefix("o2xml-")
        .suffix(suffix)
        .tempfile()
        .map_err(ValidationError::Staging)?;
    file.write_all(bytes).map_err(ValidationError::Staging)?;
    file.flush().map_err(ValidationError::Staging)?;
    Ok(file.into_temp_path())
}

fn compact_grammar_has_resource_directive(bytes: &[u8]) -> Result<bool, ValidationError> {
    let grammar = std::str::from_utf8(bytes)
        .map_err(|_| ValidationError::CompactGrammarResourcesUnsupported)?;
    let mut characters = grammar.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '#' => {
                for next in characters.by_ref() {
                    if next == '\n' || next == '\r' {
                        break;
                    }
                }
            }
            '\'' | '"' => skip_string(&mut characters, character)?,
            value if is_identifier_start(value) => {
                let mut identifier = String::from(value);
                while characters
                    .peek()
                    .is_some_and(|next| is_identifier_part(*next))
                {
                    if let Some(next) = characters.next() {
                        identifier.push(next);
                    }
                }
                if matches!(identifier.as_str(), "include" | "external") {
                    return Ok(true);
                }
            }
            _ => {}
        }
    }
    Ok(false)
}

fn skip_string<I>(
    characters: &mut std::iter::Peekable<I>,
    quote: char,
) -> Result<(), ValidationError>
where
    I: Iterator<Item = char>,
{
    let mut escaped = false;
    for character in characters.by_ref() {
        if escaped {
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == quote {
            return Ok(());
        }
    }
    Err(ValidationError::CompactGrammarResourcesUnsupported)
}

const fn is_identifier_start(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}

const fn is_identifier_part(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '.')
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
    fn detects_only_resource_directives_outside_comments_and_strings() -> Result<(), ValidationError>
    {
        assert!(!compact_grammar_has_resource_directive(
            b"# include 'ignored.rnc'\nstart = element article { text }"
        )?);
        assert!(!compact_grammar_has_resource_directive(
            b"start = element article { 'external' }"
        )?);
        assert!(compact_grammar_has_resource_directive(
            b"include 'other.rnc'"
        )?);
        assert!(compact_grammar_has_resource_directive(
            b"external 'other.rnc'"
        )?);
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
        let catalog = directory.path().join("catalog.xml");
        fs::write(
            &executable,
            "#!/bin/sh\nlast=\nfor arg do last=$arg; done\nwhile IFS= read -r line; do\n  case $line in\n    *'<front'*) printf '%s:1:10: error: element front not allowed\\n' \"$last\" >&2; exit 1;;\n  esac\ndone < \"$last\"\nexit 0\n",
        )?;
        let mut permissions = fs::metadata(&executable)?.permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&executable, permissions)?;
        fs::write(&jar, [])?;
        fs::write(&grammar, "start = element article { text }\n")?;
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
        assert_eq!(invalid.diagnostics.len(), 1);
        assert_eq!(invalid.diagnostics[0].source, "jing-rnc");
        Ok(())
    }
}
