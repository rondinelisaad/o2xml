use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::Stdio,
    time::{Duration, Instant},
};

use quick_xml::{events::Event, name::ResolveResult, reader::NsReader, XmlVersion};
use tempfile::{Builder, TempPath};
use thiserror::Error;
use tokio::{io::AsyncReadExt, process::Command, time::timeout};
use tracing::info;
use xml_domain::Revision;

mod libxslt;

pub use libxslt::{LibxsltProcessEngine, LibxsltRequest};

const XSLT_NAMESPACE: &[u8] = b"http://www.w3.org/1999/XSL/Transform";
const MAX_SOURCE_BYTES: usize = 128 * 1024 * 1024;
const MAX_STYLESHEET_BYTES: usize = 16 * 1024 * 1024;
const MAX_OUTPUT_BYTES: u64 = 128 * 1024 * 1024;
const MAX_STDERR_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug)]
pub struct TransformRequest<'a> {
    pub source: &'a str,
    pub stylesheet: &'a str,
    pub revision: Revision,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransformReport {
    pub revision: Revision,
    pub output: Vec<u8>,
    pub stderr_truncated: bool,
}

#[derive(Clone, Debug)]
pub struct SaxonProcessEngine {
    java_executable: PathBuf,
    saxon_jar: PathBuf,
    deadline: Duration,
}

impl SaxonProcessEngine {
    pub fn new(
        java_executable: impl AsRef<Path>,
        saxon_jar: impl AsRef<Path>,
    ) -> Result<Self, TransformError> {
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

    pub async fn transform(
        &self,
        request: TransformRequest<'_>,
    ) -> Result<TransformReport, TransformError> {
        let started = Instant::now();
        let result = self.run(request).await;
        info!(
            audit = true,
            action = "transform.run",
            result = if result.is_ok() { "success" } else { "failure" },
            resource = "document",
            revision = request.revision.0,
            engine = "saxon-he",
            duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        );
        result
    }

    async fn run(&self, request: TransformRequest<'_>) -> Result<TransformReport, TransformError> {
        if request.source.len() > MAX_SOURCE_BYTES {
            return Err(TransformError::SourceTooLarge);
        }
        if request.stylesheet.len() > MAX_STYLESHEET_BYTES {
            return Err(TransformError::StylesheetTooLarge);
        }
        preflight_document(request.source)?;
        preflight_stylesheet(request.stylesheet)?;

        let source = stage_input(".xml", request.source.as_bytes())?;
        let stylesheet = stage_input(".xsl", request.stylesheet.as_bytes())?;
        let output = stage_output(".out")?;
        let mut child = Command::new(&self.java_executable)
            .env_clear()
            .arg("-Djava.awt.headless=true")
            .arg("-Djava.net.useSystemProxies=false")
            .arg("-jar")
            .arg(&self.saxon_jar)
            .arg("--allow-external-functions:false")
            .arg(format!("-s:{}", source.display()))
            .arg(format!("-xsl:{}", stylesheet.display()))
            .arg(format!("-o:{}", output.display()))
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(TransformError::Spawn)?;
        let stderr = child.stderr.take().ok_or(TransformError::MissingStderr)?;
        let reader = tokio::spawn(capture_bounded_stderr(stderr));
        let status = match timeout(self.deadline, child.wait()).await {
            Ok(status) => status.map_err(TransformError::Wait)?,
            Err(_) => {
                reader.abort();
                let _ = child.kill().await;
                return Err(TransformError::Timeout(self.deadline));
            }
        };
        let (stderr, stderr_truncated) = reader.await.map_err(TransformError::Task)??;
        if !status.success() {
            return Err(TransformError::EngineFailed(
                String::from_utf8_lossy(&stderr)
                    .chars()
                    .take(8 * 1024)
                    .collect(),
            ));
        }
        let metadata = fs::metadata(&output)?;
        if metadata.len() > MAX_OUTPUT_BYTES {
            return Err(TransformError::OutputTooLarge);
        }
        let output = fs::read(&output)?;
        Ok(TransformReport {
            revision: request.revision,
            output,
            stderr_truncated,
        })
    }
}

fn canonical_file(path: &Path) -> Result<PathBuf, TransformError> {
    let path = fs::canonicalize(path)?;
    if !path.is_file() {
        return Err(TransformError::InvalidAsset(path));
    }
    Ok(path)
}

fn stage_input(suffix: &str, bytes: &[u8]) -> Result<TempPath, TransformError> {
    let mut file = Builder::new().prefix("o2xml-").suffix(suffix).tempfile()?;
    file.write_all(bytes)?;
    file.flush()?;
    Ok(file.into_temp_path())
}

fn stage_output(suffix: &str) -> Result<TempPath, TransformError> {
    Ok(Builder::new()
        .prefix("o2xml-")
        .suffix(suffix)
        .tempfile()?
        .into_temp_path())
}

fn preflight_document(source: &str) -> Result<(), TransformError> {
    let mut reader = NsReader::from_str(source);
    loop {
        match reader
            .read_event()
            .map_err(|error| TransformError::InvalidSource(error.to_string()))?
        {
            Event::DocType(_) => return Err(TransformError::ExternalDocumentType),
            Event::Eof => return Ok(()),
            _ => {}
        }
    }
}

fn preflight_stylesheet(source: &str) -> Result<(), TransformError> {
    let mut reader = NsReader::from_str(source);
    let mut saw_root = false;
    loop {
        let (namespace, event) = reader
            .read_resolved_event()
            .map_err(|error| TransformError::InvalidStylesheet(error.to_string()))?;
        match event {
            Event::Start(element) | Event::Empty(element) => {
                let local = element.local_name();
                let is_xslt = matches!(namespace, ResolveResult::Bound(value) if value.as_ref() == XSLT_NAMESPACE);
                if !saw_root {
                    if !is_xslt || !matches!(local.as_ref(), b"stylesheet" | b"transform") {
                        return Err(TransformError::InvalidStylesheetRoot);
                    }
                    saw_root = true;
                }
                if is_xslt
                    && matches!(
                        local.as_ref(),
                        b"include"
                            | b"import"
                            | b"import-schema"
                            | b"use-package"
                            | b"source-document"
                            | b"result-document"
                            | b"evaluate"
                    )
                {
                    return Err(TransformError::ExternalResourcesDenied);
                }
                for attribute in element.attributes() {
                    let attribute = attribute
                        .map_err(|error| TransformError::InvalidStylesheet(error.to_string()))?;
                    let value = attribute
                        .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
                        .map_err(|error| TransformError::InvalidStylesheet(error.to_string()))?;
                    if contains_external_function(&value) {
                        return Err(TransformError::ExternalResourcesDenied);
                    }
                }
            }
            Event::DocType(_) => return Err(TransformError::ExternalDocumentType),
            Event::Eof if saw_root => return Ok(()),
            Event::Eof => return Err(TransformError::InvalidStylesheetRoot),
            _ => {}
        }
    }
}

fn contains_external_function(expression: &str) -> bool {
    let compact = expression
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    [
        "doc(",
        "document(",
        "doc-available(",
        "collection(",
        "uri-collection(",
        "unparsed-text(",
        "unparsed-text-available(",
        "json-doc(",
        "transform(",
    ]
    .iter()
    .any(|function| compact.contains(function))
}

async fn capture_bounded_stderr(
    mut stderr: tokio::process::ChildStderr,
) -> Result<(Vec<u8>, bool), std::io::Error> {
    let mut captured = Vec::new();
    let mut truncated = false;
    let mut buffer = [0_u8; 8 * 1024];
    loop {
        let read = stderr.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        let remaining = MAX_STDERR_BYTES.saturating_sub(captured.len());
        let retained = read.min(remaining);
        captured.extend_from_slice(&buffer[..retained]);
        truncated |= retained < read;
    }
    Ok((captured, truncated))
}

#[derive(Debug, Error)]
pub enum TransformError {
    #[error("Saxon asset is not a file: {0}")]
    InvalidAsset(PathBuf),
    #[error("XML source exceeds its limit")]
    SourceTooLarge,
    #[error("XSLT stylesheet exceeds its limit")]
    StylesheetTooLarge,
    #[error("invalid XML source: {0}")]
    InvalidSource(String),
    #[error("invalid XSLT stylesheet: {0}")]
    InvalidStylesheet(String),
    #[error("the XSLT document has no xsl:stylesheet or xsl:transform root")]
    InvalidStylesheetRoot,
    #[error("DOCTYPE is denied by the Saxon-safe profile")]
    ExternalDocumentType,
    #[error("external or dynamic XSLT resource access is denied")]
    ExternalResourcesDenied,
    #[error("transformation output exceeds its limit")]
    OutputTooLarge,
    #[error("cannot access transformation data: {0}")]
    Io(#[from] std::io::Error),
    #[error("cannot start Saxon: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("transformation helper stdin is unavailable")]
    MissingStdin,
    #[error("transformation helper stdout is unavailable")]
    MissingStdout,
    #[error("Saxon stderr is unavailable")]
    MissingStderr,
    #[error("cannot wait for Saxon: {0}")]
    Wait(#[source] std::io::Error),
    #[error("cannot write to transformation helper: {0}")]
    Write(#[source] std::io::Error),
    #[error("cannot read from transformation helper: {0}")]
    Read(#[source] std::io::Error),
    #[error("Saxon task failed: {0}")]
    Task(#[source] tokio::task::JoinError),
    #[error("Saxon exceeded its {0:?} deadline")]
    Timeout(Duration),
    #[error("Saxon failed: {0}")]
    EngineFailed(String),
    #[error("transformation helper protocol failed: {0}")]
    Protocol(String),
    #[error("stylesheet is not mapped by the XML catalog")]
    StylesheetNotFound,
    #[error("transformation resource exceeds its safety limit")]
    ResourceTooLarge,
    #[error("transformation resource count exceeds its safety limit")]
    TooManyResources,
    #[error("transformation resource graph exceeds its safety limit")]
    ResourceGraphTooLarge,
    #[error(transparent)]
    Catalog(#[from] xml_resource::CatalogError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn rejects_external_and_dynamic_resources() {
        assert!(matches!(
            preflight_stylesheet(
                r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:include href="remote.xsl"/></xsl:stylesheet>"#
            ),
            Err(TransformError::ExternalResourcesDenied)
        ));
        assert!(matches!(
            preflight_stylesheet(
                r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"><xsl:value-of select="doc('secret.xml')"/></xsl:stylesheet>"#
            ),
            Err(TransformError::ExternalResourcesDenied)
        ));
        assert!(matches!(
            preflight_document(r#"<!DOCTYPE article SYSTEM "article.dtd"><article/>"#),
            Err(TransformError::ExternalDocumentType)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn supervises_a_saxon_transformation() -> Result<(), Box<dyn std::error::Error>> {
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
printf '%s' '<html><body>result</body></html>' > "$out"
"##,
        )?;
        let mut permissions = fs::metadata(&java)?.permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&java, permissions)?;
        fs::write(&saxon, b"test")?;
        let engine = SaxonProcessEngine::new(&java, &saxon)?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()?;
        let report = runtime.block_on(engine.transform(TransformRequest {
            source: "<article/>",
            stylesheet: r#"<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="3.0"/>"#,
            revision: Revision(9),
        }))?;
        assert_eq!(report.revision, Revision(9));
        assert_eq!(report.output, b"<html><body>result</body></html>");
        assert!(!report.stderr_truncated);
        Ok(())
    }
}
