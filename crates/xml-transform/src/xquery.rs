use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Stdio,
    time::{Duration, Instant},
};

use quick_xml::{events::Event, reader::NsReader};
use tempfile::{Builder, TempPath};
use thiserror::Error;
use tokio::{process::Command, time::timeout};
use tracing::info;
use xml_domain::Revision;

use crate::capture_bounded_stderr;

const MAX_SOURCE_BYTES: usize = 128 * 1024 * 1024;
const MAX_QUERY_BYTES: usize = 1024 * 1024;
const MAX_OUTPUT_BYTES: u64 = 128 * 1024 * 1024;

#[derive(Clone, Copy, Debug)]
pub struct XQueryRequest<'a> {
    pub source: &'a str,
    pub query: &'a str,
    pub revision: Revision,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XQueryReport {
    pub revision: Revision,
    pub output: Vec<u8>,
    pub stderr_truncated: bool,
}

#[derive(Clone, Debug)]
pub struct SaxonXQueryEngine {
    java_executable: PathBuf,
    saxon_jar: PathBuf,
    deadline: Duration,
}

impl SaxonXQueryEngine {
    pub fn new(
        java_executable: impl AsRef<Path>,
        saxon_jar: impl AsRef<Path>,
    ) -> Result<Self, XQueryError> {
        Ok(Self {
            java_executable: canonical_file(java_executable.as_ref())?,
            saxon_jar: canonical_file(saxon_jar.as_ref())?,
            deadline: Duration::from_secs(30),
        })
    }

    #[must_use]
    pub fn with_deadline(mut self, deadline: Duration) -> Self {
        self.deadline = deadline.max(Duration::from_millis(1));
        self
    }

    pub async fn evaluate(&self, request: XQueryRequest<'_>) -> Result<XQueryReport, XQueryError> {
        let started = Instant::now();
        let result = self.run(request).await;
        info!(
            audit = true,
            action = "xquery.evaluate",
            result = if result.is_ok() { "success" } else { "failure" },
            resource = "document",
            revision = request.revision.0,
            engine = "saxon-he-xquery",
            duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        );
        result
    }

    async fn run(&self, request: XQueryRequest<'_>) -> Result<XQueryReport, XQueryError> {
        if request.source.len() > MAX_SOURCE_BYTES {
            return Err(XQueryError::SourceTooLarge);
        }
        if request.query.is_empty() || request.query.len() > MAX_QUERY_BYTES {
            return Err(XQueryError::QueryTooLarge);
        }
        preflight_document(request.source)?;
        preflight_query(request.query)?;
        let source = stage_input(".xml", request.source.as_bytes())?;
        let query = stage_input(".xq", request.query.as_bytes())?;
        let output = stage_output(".out")?;
        let mut child = Command::new(&self.java_executable)
            .env_clear()
            .arg("-Djava.awt.headless=true")
            .arg("-Djava.net.useSystemProxies=false")
            .arg("-cp")
            .arg(&self.saxon_jar)
            .arg("net.sf.saxon.Query")
            .arg("--allow-external-functions:off")
            .arg("-update:off")
            .arg("-xi:off")
            .arg("-xsiloc:off")
            .arg(format!("-s:{}", source.display()))
            .arg(format!("-q:{}", query.display()))
            .arg(format!("-o:{}", output.display()))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(XQueryError::Spawn)?;
        let stderr = child.stderr.take().ok_or(XQueryError::MissingStderr)?;
        let reader = tokio::spawn(capture_bounded_stderr(stderr));
        let status = match timeout(self.deadline, child.wait()).await {
            Ok(status) => status.map_err(XQueryError::Wait)?,
            Err(_) => {
                reader.abort();
                let _ = child.kill().await;
                return Err(XQueryError::Timeout(self.deadline));
            }
        };
        let (stderr, stderr_truncated) = reader.await.map_err(XQueryError::Task)??;
        if !status.success() {
            return Err(XQueryError::EngineFailed(
                String::from_utf8_lossy(&stderr)
                    .chars()
                    .take(8 * 1024)
                    .collect(),
            ));
        }
        let metadata = fs::metadata(&output)?;
        if metadata.len() > MAX_OUTPUT_BYTES {
            return Err(XQueryError::OutputTooLarge);
        }
        Ok(XQueryReport {
            revision: request.revision,
            output: fs::read(&output)?,
            stderr_truncated,
        })
    }
}

fn canonical_file(path: &Path) -> Result<PathBuf, XQueryError> {
    let path = fs::canonicalize(path)?;
    if !path.is_file() {
        return Err(XQueryError::InvalidAsset(path));
    }
    Ok(path)
}

fn stage_input(suffix: &str, bytes: &[u8]) -> Result<TempPath, XQueryError> {
    let mut file = Builder::new().prefix("o2xml-").suffix(suffix).tempfile()?;
    file.write_all(bytes)?;
    file.flush()?;
    Ok(file.into_temp_path())
}

fn stage_output(suffix: &str) -> Result<TempPath, XQueryError> {
    Ok(Builder::new()
        .prefix("o2xml-")
        .suffix(suffix)
        .tempfile()?
        .into_temp_path())
}

fn preflight_document(source: &str) -> Result<(), XQueryError> {
    let mut reader = NsReader::from_str(source);
    loop {
        match reader
            .read_event()
            .map_err(|error| XQueryError::InvalidSource(error.to_string()))?
        {
            Event::DocType(_) => return Err(XQueryError::ExternalDocumentType),
            Event::Eof => return Ok(()),
            _ => {}
        }
    }
}

fn preflight_query(query: &str) -> Result<(), XQueryError> {
    let code = code_without_comments_or_strings(query)?;
    let compact = code
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    let denied = [
        "importmodule",
        "importschema",
        "modulenamespace",
        "declareupdating",
        "insertnode",
        "deletenode",
        "replacenode",
        "renamenode",
        "copy$",
        "modify",
        "doc(",
        "doc#",
        "doc-available(",
        "collection(",
        "collection#",
        "uri-collection(",
        "unparsed-text(",
        "unparsed-text-available(",
        "json-doc(",
        "transform(",
        "put(",
        "function-lookup(",
        "load-xquery-module(",
        "environment-variable(",
        "available-environment-variables(",
        "java:",
        "file:",
        "saxon:",
    ]
    .iter()
    .any(|pattern| compact.contains(pattern));
    if denied {
        return Err(XQueryError::ExternalOrUpdatingQueryDenied);
    }
    Ok(())
}

fn code_without_comments_or_strings(query: &str) -> Result<String, XQueryError> {
    let characters = query.chars().collect::<Vec<_>>();
    let mut result = String::with_capacity(query.len());
    let mut index = 0;
    let mut comment_depth = 0_usize;
    let mut quote = None;
    while index < characters.len() {
        let current = characters[index];
        let next = characters.get(index + 1).copied();
        if let Some(delimiter) = quote {
            if current == delimiter && next == Some(delimiter) {
                result.push(' ');
                result.push(' ');
                index += 2;
                continue;
            }
            if current == delimiter {
                quote = None;
            }
            result.push(' ');
            index += 1;
            continue;
        }
        if comment_depth > 0 {
            if current == '(' && next == Some(':') {
                comment_depth = comment_depth.saturating_add(1);
                result.push_str("  ");
                index += 2;
            } else if current == ':' && next == Some(')') {
                comment_depth = comment_depth.saturating_sub(1);
                result.push_str("  ");
                index += 2;
            } else {
                result.push(' ');
                index += 1;
            }
            continue;
        }
        if current == '(' && next == Some(':') {
            comment_depth = 1;
            result.push_str("  ");
            index += 2;
        } else if matches!(current, '\'' | '"') {
            quote = Some(current);
            result.push(' ');
            index += 1;
        } else {
            result.push(current);
            index += 1;
        }
    }
    if comment_depth != 0 || quote.is_some() {
        return Err(XQueryError::InvalidQueryLexicalStructure);
    }
    Ok(result)
}

#[derive(Debug, Error)]
pub enum XQueryError {
    #[error("Saxon asset is not a file: {0}")]
    InvalidAsset(PathBuf),
    #[error("XML source exceeds its limit")]
    SourceTooLarge,
    #[error("XQuery is empty or exceeds its limit")]
    QueryTooLarge,
    #[error("invalid XML source: {0}")]
    InvalidSource(String),
    #[error("DOCTYPE is denied by the Saxon-safe profile")]
    ExternalDocumentType,
    #[error("XQuery comment or string literal is not closed")]
    InvalidQueryLexicalStructure,
    #[error("external resource, module, environment, extension, or updating XQuery is denied")]
    ExternalOrUpdatingQueryDenied,
    #[error("XQuery output exceeds its limit")]
    OutputTooLarge,
    #[error("cannot access XQuery data: {0}")]
    Io(#[from] std::io::Error),
    #[error("cannot start Saxon XQuery: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("Saxon XQuery stderr is unavailable")]
    MissingStderr,
    #[error("cannot wait for Saxon XQuery: {0}")]
    Wait(#[source] std::io::Error),
    #[error("Saxon XQuery task failed: {0}")]
    Task(#[source] tokio::task::JoinError),
    #[error("Saxon XQuery exceeded its {0:?} deadline")]
    Timeout(Duration),
    #[error("Saxon XQuery failed: {0}")]
    EngineFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn rejects_external_access_even_with_comments_and_strings_present() {
        assert!(preflight_query("for $x in //title return string($x)").is_ok());
        assert!(preflight_query("(: doc('ignored.xml') :) 'doc(''literal'')'").is_ok());
        assert!(matches!(
            preflight_query("function-lookup(QName('x', 'doc'), 1)('secret.xml')"),
            Err(XQueryError::ExternalOrUpdatingQueryDenied)
        ));
        assert!(matches!(
            preflight_query("doc (: split :) ('secret.xml')"),
            Err(XQueryError::ExternalOrUpdatingQueryDenied)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn supervises_a_saxon_xquery() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let java = directory.path().join("java");
        let saxon = directory.path().join("saxon.jar");
        fs::write(
            &java,
            r##"#!/bin/sh
out=""
for argument in "$@"; do
  case "$argument" in
    -o:*) out=${argument#-o:} ;;
  esac
done
printf '%s' '<titles><title>Example</title></titles>' > "$out"
"##,
        )?;
        let mut permissions = fs::metadata(&java)?.permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&java, permissions)?;
        fs::write(&saxon, b"test")?;
        let engine = SaxonXQueryEngine::new(&java, &saxon)?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()?;
        let report = runtime.block_on(engine.evaluate(XQueryRequest {
            source: "<article><title>Example</title></article>",
            query: "<titles>{//title}</titles>",
            revision: Revision(13),
        }))?;
        assert_eq!(report.revision, Revision(13));
        assert_eq!(report.output, b"<titles><title>Example</title></titles>");
        Ok(())
    }
}
