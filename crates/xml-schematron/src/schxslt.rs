use std::{
    collections::BTreeMap,
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
use xml_domain::{Diagnostic, Revision};

use crate::{parse_svrl, SvrlError};

const SCHEMATRON_NAMESPACE: &[u8] = b"http://purl.oclc.org/dsdl/schematron";
const XSLT_NAMESPACE: &[u8] = b"http://www.w3.org/1999/XSL/Transform";
const MAX_SOURCE_BYTES: usize = 128 * 1024 * 1024;
const MAX_SCHEMA_BYTES: usize = 16 * 1024 * 1024;
const MAX_RESULT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_STDERR_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Copy)]
pub struct SchXsltRequest<'a> {
    pub source: &'a str,
    pub schematron: &'a str,
    pub revision: Revision,
    pub file: Option<&'a str>,
    pub phase: Option<&'a str>,
}

#[derive(Clone, Debug)]
pub struct SchXsltReport {
    pub revision: Revision,
    pub diagnostics: Vec<Diagnostic>,
    pub svrl: String,
    pub stderr_truncated: bool,
}

#[derive(Clone, Debug)]
pub struct SchXsltProcessEngine {
    java_executable: PathBuf,
    saxon_jar: PathBuf,
    pipeline_stylesheet: PathBuf,
    deadline: Duration,
}

impl SchXsltProcessEngine {
    pub fn new(
        java_executable: impl AsRef<Path>,
        saxon_jar: impl AsRef<Path>,
        pipeline_stylesheet: impl AsRef<Path>,
    ) -> Result<Self, SchXsltError> {
        Ok(Self {
            java_executable: canonical_file(java_executable.as_ref())?,
            saxon_jar: canonical_file(saxon_jar.as_ref())?,
            pipeline_stylesheet: canonical_file(pipeline_stylesheet.as_ref())?,
            deadline: Duration::from_secs(20),
        })
    }

    #[must_use]
    pub fn with_deadline(mut self, deadline: Duration) -> Self {
        self.deadline = deadline.max(Duration::from_millis(1));
        self
    }

    pub async fn validate(
        &self,
        request: SchXsltRequest<'_>,
    ) -> Result<SchXsltReport, SchXsltError> {
        let started = Instant::now();
        let result = self.run(request).await;
        info!(
            audit = true,
            action = "schematron.validate",
            result = if result.is_ok() { "success" } else { "failure" },
            resource = "document",
            revision = request.revision.0,
            engine = "schxslt-saxon",
            duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        );
        result
    }

    async fn run(&self, request: SchXsltRequest<'_>) -> Result<SchXsltReport, SchXsltError> {
        if request.source.len() > MAX_SOURCE_BYTES {
            return Err(SchXsltError::SourceTooLarge);
        }
        if request.schematron.len() > MAX_SCHEMA_BYTES {
            return Err(SchXsltError::SchemaTooLarge);
        }
        preflight_schematron(request.schematron)?;
        let schema = stage_input(".sch", request.schematron.as_bytes())?;
        let compiled = stage_output(".xsl")?;
        let document = stage_input(".xml", request.source.as_bytes())?;
        let svrl_path = stage_output(".svrl")?;

        let mut compile_arguments = vec![
            format!("-s:{}", schema.display()),
            format!("-xsl:{}", self.pipeline_stylesheet.display()),
            format!("-o:{}", compiled.display()),
        ];
        if let Some(phase) = request.phase {
            if phase.is_empty() || phase.len() > 256 {
                return Err(SchXsltError::InvalidPhase);
            }
            compile_arguments.push(format!("phase={phase}"));
        }
        let compile_stderr = self.invoke(&compile_arguments).await?;
        ensure_bounded_file(&compiled)?;
        let validation_stderr = self
            .invoke(&[
                format!("-s:{}", document.display()),
                format!("-xsl:{}", compiled.display()),
                format!("-o:{}", svrl_path.display()),
            ])
            .await?;
        ensure_bounded_file(&svrl_path)?;
        let svrl = fs::read_to_string(&svrl_path).map_err(SchXsltError::Io)?;
        let parsed = parse_svrl(&svrl)?;
        let diagnostics = parsed.diagnostics(
            request.source,
            request.revision,
            request.file,
            &BTreeMap::new(),
        );
        Ok(SchXsltReport {
            revision: request.revision,
            diagnostics,
            svrl,
            stderr_truncated: compile_stderr || validation_stderr,
        })
    }

    async fn invoke(&self, arguments: &[String]) -> Result<bool, SchXsltError> {
        let mut child = Command::new(&self.java_executable)
            .env_clear()
            .arg("-Djava.awt.headless=true")
            .arg("-Djava.net.useSystemProxies=false")
            .arg("-jar")
            .arg(&self.saxon_jar)
            .arg("--allow-external-functions:false")
            .args(arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(SchXsltError::Spawn)?;
        let stderr = child.stderr.take().ok_or(SchXsltError::MissingStderr)?;
        let reader = tokio::spawn(capture_bounded_stderr(stderr));
        let status = match timeout(self.deadline, child.wait()).await {
            Ok(status) => status.map_err(SchXsltError::Wait)?,
            Err(_) => {
                reader.abort();
                let _ = child.kill().await;
                return Err(SchXsltError::Timeout(self.deadline));
            }
        };
        let (stderr, stderr_truncated) = reader.await.map_err(SchXsltError::Task)??;
        if !status.success() {
            let message = String::from_utf8_lossy(&stderr);
            return Err(SchXsltError::EngineFailed(
                message.chars().take(8 * 1024).collect(),
            ));
        }
        Ok(stderr_truncated)
    }
}

async fn capture_bounded_stderr(
    mut stderr: tokio::process::ChildStderr,
) -> Result<(Vec<u8>, bool), std::io::Error> {
    let limit = usize::try_from(MAX_STDERR_BYTES).unwrap_or(usize::MAX);
    let mut captured = Vec::new();
    let mut truncated = false;
    let mut buffer = [0_u8; 8 * 1024];
    loop {
        let read = stderr.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        let remaining = limit.saturating_sub(captured.len());
        let retained = read.min(remaining);
        captured.extend_from_slice(&buffer[..retained]);
        truncated |= retained < read;
    }
    Ok((captured, truncated))
}

fn canonical_file(path: &Path) -> Result<PathBuf, SchXsltError> {
    let path = fs::canonicalize(path).map_err(SchXsltError::Io)?;
    if !path.is_file() {
        return Err(SchXsltError::InvalidAsset(path));
    }
    Ok(path)
}

fn stage_input(suffix: &str, bytes: &[u8]) -> Result<TempPath, SchXsltError> {
    let mut file = Builder::new().prefix("o2xml-").suffix(suffix).tempfile()?;
    file.write_all(bytes)?;
    file.flush()?;
    Ok(file.into_temp_path())
}

fn stage_output(suffix: &str) -> Result<TempPath, SchXsltError> {
    Ok(Builder::new()
        .prefix("o2xml-")
        .suffix(suffix)
        .tempfile()?
        .into_temp_path())
}

fn ensure_bounded_file(path: &Path) -> Result<(), SchXsltError> {
    let metadata = fs::metadata(path).map_err(SchXsltError::Io)?;
    if metadata.len() > MAX_RESULT_BYTES {
        return Err(SchXsltError::OutputTooLarge);
    }
    Ok(())
}

fn preflight_schematron(source: &str) -> Result<(), SchXsltError> {
    let mut reader = NsReader::from_str(source);
    loop {
        let (namespace, event) = reader
            .read_resolved_event()
            .map_err(|error| SchXsltError::InvalidSchema(error.to_string()))?;
        match event {
            Event::Start(element) | Event::Empty(element) => {
                let local = element.local_name();
                if matches!(namespace, ResolveResult::Bound(value) if value.as_ref() == SCHEMATRON_NAMESPACE)
                    && matches!(local.as_ref(), b"include" | b"extends")
                {
                    return Err(SchXsltError::ExternalResourcesDenied);
                }
                if matches!(namespace, ResolveResult::Bound(value) if value.as_ref() == XSLT_NAMESPACE)
                    && matches!(local.as_ref(), b"include" | b"import" | b"result-document")
                {
                    return Err(SchXsltError::ExternalResourcesDenied);
                }
                for attribute in element.attributes() {
                    let attribute = attribute
                        .map_err(|error| SchXsltError::InvalidSchema(error.to_string()))?;
                    if matches!(attribute.key.as_ref(), b"test" | b"select" | b"value") {
                        let value = attribute
                            .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
                            .map_err(|error| SchXsltError::InvalidSchema(error.to_string()))?;
                        if contains_external_function(&value) {
                            return Err(SchXsltError::ExternalResourcesDenied);
                        }
                    }
                }
            }
            Event::DocType(_) => return Err(SchXsltError::ExternalResourcesDenied),
            Event::Eof => return Ok(()),
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
        "collection(",
        "uri-collection(",
        "unparsed-text(",
        "json-doc(",
        "transform(",
    ]
    .iter()
    .any(|function| compact.contains(function))
}

#[derive(Debug, Error)]
pub enum SchXsltError {
    #[error("SchXslt or Saxon asset is not a file: {0}")]
    InvalidAsset(PathBuf),
    #[error("Schematron source exceeds its limit")]
    SchemaTooLarge,
    #[error("XML source exceeds its limit")]
    SourceTooLarge,
    #[error("Schematron phase is invalid")]
    InvalidPhase,
    #[error("Schematron external resource access is denied")]
    ExternalResourcesDenied,
    #[error("invalid Schematron XML: {0}")]
    InvalidSchema(String),
    #[error("SchXslt output exceeds its limit")]
    OutputTooLarge,
    #[error("cannot access SchXslt data: {0}")]
    Io(#[from] std::io::Error),
    #[error("cannot start Saxon: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("Saxon stderr is unavailable")]
    MissingStderr,
    #[error("cannot wait for Saxon: {0}")]
    Wait(#[source] std::io::Error),
    #[error("Saxon task failed: {0}")]
    Task(#[source] tokio::task::JoinError),
    #[error("Saxon exceeded its {0:?} deadline")]
    Timeout(Duration),
    #[error("Saxon failed: {0}")]
    EngineFailed(String),
    #[error(transparent)]
    Svrl(#[from] SvrlError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn denies_external_schematron_resources_and_functions() {
        assert!(matches!(
            preflight_schematron(
                r#"<schema xmlns="http://purl.oclc.org/dsdl/schematron"><include href="rules.sch"/></schema>"#
            ),
            Err(SchXsltError::ExternalResourcesDenied)
        ));
        assert!(matches!(
            preflight_schematron(
                r#"<schema xmlns="http://purl.oclc.org/dsdl/schematron"><pattern><rule context="article"><assert test="doc('file:///etc/passwd')">x</assert></rule></pattern></schema>"#
            ),
            Err(SchXsltError::ExternalResourcesDenied)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn runs_the_two_step_pipeline_and_parses_svrl() -> Result<(), Box<dyn std::error::Error>> {
        let directory = tempfile::tempdir()?;
        let java = directory.path().join("java");
        let saxon = directory.path().join("saxon.jar");
        let pipeline = directory.path().join("pipeline-for-svrl.xsl");
        fs::write(
            &java,
            r##"#!/bin/sh
out=""
for argument in "$@"; do
  case "$argument" in
    -o:*) out=${argument#-o:} ;;
  esac
done
case "$out" in
  *.xsl) printf '%s' '<xsl:stylesheet xmlns:xsl="http://www.w3.org/1999/XSL/Transform" version="2.0"/>' > "$out" ;;
  *.svrl) printf '%s' '<svrl:schematron-output xmlns:svrl="http://purl.oclc.org/dsdl/svrl"><svrl:failed-assert id="required-title" location="/article" test="title"><svrl:text>title is required</svrl:text></svrl:failed-assert></svrl:schematron-output>' > "$out" ;;
  *) exit 2 ;;
esac
"##,
        )?;
        let mut permissions = fs::metadata(&java)?.permissions();
        permissions.set_mode(0o700);
        fs::set_permissions(&java, permissions)?;
        fs::write(&saxon, b"test")?;
        fs::write(&pipeline, b"test")?;

        let engine = SchXsltProcessEngine::new(&java, &saxon, &pipeline)?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()?;
        let report = runtime.block_on(engine.validate(SchXsltRequest {
            source: "<article/>",
            schematron: r#"<schema xmlns="http://purl.oclc.org/dsdl/schematron"/>"#,
            revision: Revision(7),
            file: Some("article.xml"),
            phase: None,
        }))?;

        assert_eq!(report.revision, Revision(7));
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(report.diagnostics[0].message, "title is required");
        assert_eq!(
            report.diagnostics[0].rule.as_deref(),
            Some("required-title")
        );
        assert!(!report.stderr_truncated);
        Ok(())
    }
}
