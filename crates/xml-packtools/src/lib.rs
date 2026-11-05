//! Supervised adapter for the Packtools 4.18.3 SPS 1.10 validator sidecar.

use std::{
    io,
    path::PathBuf,
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
use tracing::{info, warn};
use xml_document::LineIndex;
use xml_domain::{Diagnostic, Revision, Severity};

const PROTOCOL_VERSION: u8 = 1;
const PACKTOOLS_VERSION: &str = "4.18.3";
const ENGINE: &str = "packtools-4.18.3";
const MAX_DOCUMENT_BYTES: usize = 100 * 1024 * 1024;
const MAX_STDOUT_BYTES: usize = 16 * 1024 * 1024;
const MAX_STDERR_BYTES: usize = 256 * 1024;
const MAX_DIAGNOSTICS: usize = 5_000;
const DEFAULT_DEADLINE: Duration = Duration::from_secs(60);

#[derive(Debug, Error)]
pub enum PacktoolsError {
    #[error("Packtools validator executable is invalid: {0}")]
    InvalidExecutable(PathBuf),
    #[error("document exceeds the Packtools validation limit of {limit} bytes")]
    DocumentTooLarge { limit: usize },
    #[error("failed to start Packtools validator: {0}")]
    Spawn(#[source] io::Error),
    #[error("Packtools validator stdin is unavailable")]
    MissingStdin,
    #[error("Packtools validator stdout is unavailable")]
    MissingStdout,
    #[error("Packtools validator stderr is unavailable")]
    MissingStderr,
    #[error("failed to send XML to Packtools validator: {0}")]
    Write(#[source] io::Error),
    #[error("failed to read Packtools validator output: {0}")]
    Read(#[source] io::Error),
    #[error("Packtools validation exceeded {0:?}")]
    Timeout(Duration),
    #[error("Packtools validator protocol error: {0}")]
    Protocol(String),
    #[error("Packtools validator failed: {0}")]
    Engine(String),
}

#[derive(Clone, Debug)]
pub struct PacktoolsProcessEngine {
    executable: PathBuf,
    deadline: Duration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PacktoolsReport {
    pub engine: &'static str,
    pub version: String,
    pub revision: Revision,
    pub diagnostics: Vec<Diagnostic>,
    pub truncated: bool,
}

impl PacktoolsProcessEngine {
    pub fn new(executable: impl Into<PathBuf>) -> Result<Self, PacktoolsError> {
        let executable = executable.into();
        if !executable.is_file() {
            return Err(PacktoolsError::InvalidExecutable(executable));
        }
        Ok(Self {
            executable,
            deadline: DEFAULT_DEADLINE,
        })
    }

    #[must_use]
    pub fn with_deadline(mut self, deadline: Duration) -> Self {
        self.deadline = deadline;
        self
    }

    pub async fn validate(
        &self,
        source: &str,
        revision: Revision,
        file: Option<&str>,
        locale: &str,
    ) -> Result<PacktoolsReport, PacktoolsError> {
        let started = Instant::now();
        let result = self.run(source, revision, file, locale).await;
        let (audit_result, diagnostic_count, error_class) = match &result {
            Ok(report) => ("success", report.diagnostics.len(), None),
            Err(error) => ("failure", 0, Some(error_class(error))),
        };
        info!(
            audit = true,
            action = "job.packtools_validate",
            result = audit_result,
            resource = "article",
            engine = ENGINE,
            packtools_version = PACKTOOLS_VERSION,
            revision = revision.0,
            diagnostic_count,
            duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
            error_class,
        );
        result
    }

    async fn run(
        &self,
        source: &str,
        revision: Revision,
        file: Option<&str>,
        locale: &str,
    ) -> Result<PacktoolsReport, PacktoolsError> {
        if source.len() > MAX_DOCUMENT_BYTES {
            return Err(PacktoolsError::DocumentTooLarge {
                limit: MAX_DOCUMENT_BYTES,
            });
        }
        let request = HelperRequest {
            protocol: PROTOCOL_VERSION,
            sps_version: "1.10",
            locale: normalized_locale(locale),
            source,
        };
        let payload = serde_json::to_vec(&request)
            .map_err(|error| PacktoolsError::Protocol(error.to_string()))?;

        let mut command = Command::new(&self.executable);
        command
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        preserve_runtime_environment(&mut command);
        xml_process::hide_console(&mut command);

        let mut child = command.spawn().map_err(PacktoolsError::Spawn)?;
        let mut stdin = child.stdin.take().ok_or(PacktoolsError::MissingStdin)?;
        let stdout = child.stdout.take().ok_or(PacktoolsError::MissingStdout)?;
        let stderr = child.stderr.take().ok_or(PacktoolsError::MissingStderr)?;
        let stdout_task = tokio::spawn(read_bounded(stdout, MAX_STDOUT_BYTES));
        let stderr_task = tokio::spawn(read_bounded(stderr, MAX_STDERR_BYTES));

        timeout(self.deadline, async {
            stdin.write_all(&payload).await?;
            stdin.shutdown().await
        })
        .await
        .map_err(|_| PacktoolsError::Timeout(self.deadline))?
        .map_err(PacktoolsError::Write)?;
        drop(stdin);

        let status = match timeout(self.deadline, child.wait()).await {
            Ok(status) => status.map_err(PacktoolsError::Spawn)?,
            Err(_) => {
                stdout_task.abort();
                stderr_task.abort();
                let _ = child.kill().await;
                return Err(PacktoolsError::Timeout(self.deadline));
            }
        };
        let (stdout, stdout_truncated) = stdout_task
            .await
            .map_err(|error| PacktoolsError::Protocol(error.to_string()))?
            .map_err(PacktoolsError::Read)?;
        let (stderr, stderr_truncated) = stderr_task
            .await
            .map_err(|error| PacktoolsError::Protocol(error.to_string()))?
            .map_err(PacktoolsError::Read)?;
        if stdout_truncated {
            return Err(PacktoolsError::Protocol(
                "Packtools validator output exceeded 16 MiB".to_owned(),
            ));
        }
        if stderr_truncated {
            warn!(
                action = "validation.output_truncated",
                result = "limited",
                engine = ENGINE,
            );
        }

        let response: HelperResponse = serde_json::from_slice(&stdout).map_err(|error| {
            let stderr = String::from_utf8_lossy(&stderr);
            PacktoolsError::Protocol(format!(
                "invalid JSON response: {error}; stderr: {}",
                stderr.trim().chars().take(500).collect::<String>()
            ))
        })?;
        if response.protocol != PROTOCOL_VERSION {
            return Err(PacktoolsError::Protocol(format!(
                "expected protocol {PROTOCOL_VERSION}, found {}",
                response.protocol
            )));
        }
        if response.packtools_version != PACKTOOLS_VERSION {
            return Err(PacktoolsError::Protocol(format!(
                "expected Packtools {PACKTOOLS_VERSION}, found {}",
                response.packtools_version
            )));
        }
        if let Some(error) = response.error {
            return Err(PacktoolsError::Engine(error));
        }
        if !status.success() {
            return Err(PacktoolsError::Engine(format!(
                "process exited with status {status}"
            )));
        }

        let mut diagnostics = response
            .diagnostics
            .into_iter()
            .take(MAX_DIAGNOSTICS)
            .map(|diagnostic| normalize_diagnostic(diagnostic, source, revision, file))
            .collect::<Vec<_>>();
        for engine_error in response.engine_errors.into_iter().take(100) {
            diagnostics.push(fallback_diagnostic(
                source,
                revision,
                file,
                Severity::Warning,
                format!("Packtools could not execute one rule group: {engine_error}"),
                Some("packtools:engine".to_owned()),
            ));
        }
        if response.truncated {
            diagnostics.push(fallback_diagnostic(
                source,
                revision,
                file,
                Severity::Warning,
                "Packtools diagnostics were limited to 5,000 results".to_owned(),
                Some("packtools:limit".to_owned()),
            ));
        }
        Ok(PacktoolsReport {
            engine: ENGINE,
            version: response.packtools_version,
            revision,
            diagnostics,
            truncated: response.truncated,
        })
    }
}

#[derive(Serialize)]
struct HelperRequest<'a> {
    protocol: u8,
    sps_version: &'static str,
    locale: &'a str,
    source: &'a str,
}

#[derive(Deserialize)]
struct HelperResponse {
    protocol: u8,
    packtools_version: String,
    #[serde(default)]
    diagnostics: Vec<HelperDiagnostic>,
    #[serde(default)]
    engine_errors: Vec<String>,
    #[serde(default)]
    truncated: bool,
    error: Option<String>,
}

#[derive(Deserialize)]
struct HelperDiagnostic {
    severity: String,
    message: String,
    rule: Option<String>,
    line: usize,
    column: usize,
    #[serde(default)]
    approximate: bool,
}

fn normalize_diagnostic(
    diagnostic: HelperDiagnostic,
    source: &str,
    revision: Revision,
    file: Option<&str>,
) -> Diagnostic {
    let line = diagnostic.line.max(1);
    let column = diagnostic.column.max(1);
    let line_byte = line_start_byte(source, line);
    let byte = source[line_byte..]
        .char_indices()
        .nth(column.saturating_sub(1))
        .map_or(line_byte, |(offset, _)| line_byte.saturating_add(offset));
    let index = LineIndex::new(source);
    let mut span = index.span(source, byte, byte);
    span.approximate = diagnostic.approximate;
    Diagnostic {
        severity: if diagnostic.severity.eq_ignore_ascii_case("error") {
            Severity::Error
        } else {
            Severity::Warning
        },
        message: diagnostic.message,
        source: "SPS 1.10".to_owned(),
        rule: diagnostic.rule,
        file: file.map(ToOwned::to_owned),
        span,
        revision,
    }
}

fn fallback_diagnostic(
    source: &str,
    revision: Revision,
    file: Option<&str>,
    severity: Severity,
    message: String,
    rule: Option<String>,
) -> Diagnostic {
    let index = LineIndex::new(source);
    let mut span = index.span(source, 0, 0);
    span.approximate = true;
    Diagnostic {
        severity,
        message,
        source: "SPS 1.10".to_owned(),
        rule,
        file: file.map(ToOwned::to_owned),
        span,
        revision,
    }
}

fn line_start_byte(source: &str, requested_line: usize) -> usize {
    if requested_line <= 1 {
        return 0;
    }
    source
        .match_indices('\n')
        .nth(requested_line.saturating_sub(2))
        .map_or(source.len(), |(byte, _)| byte.saturating_add(1))
}

fn normalized_locale(locale: &str) -> &str {
    match locale {
        "en" | "es" | "pt_BR" => locale,
        _ => "pt_BR",
    }
}

fn preserve_runtime_environment(command: &mut Command) {
    for key in ["TMPDIR", "TEMP", "TMP", "SystemRoot", "WINDIR"] {
        if let Some(value) = std::env::var_os(key) {
            command.env(key, value);
        }
    }
}

async fn read_bounded<R>(mut reader: R, limit: usize) -> io::Result<(Vec<u8>, bool)>
where
    R: AsyncRead + Unpin,
{
    let mut retained = Vec::new();
    let mut buffer = [0_u8; 8 * 1024];
    let mut truncated = false;
    loop {
        let read = reader.read(&mut buffer).await?;
        if read == 0 {
            break;
        }
        let available = limit.saturating_sub(retained.len());
        let keep = available.min(read);
        retained.extend_from_slice(&buffer[..keep]);
        truncated |= keep < read;
    }
    Ok((retained, truncated))
}

fn error_class(error: &PacktoolsError) -> &'static str {
    match error {
        PacktoolsError::InvalidExecutable(_) => "invalid_executable",
        PacktoolsError::DocumentTooLarge { .. } => "document_too_large",
        PacktoolsError::Spawn(_) => "spawn",
        PacktoolsError::MissingStdin => "missing_stdin",
        PacktoolsError::MissingStdout => "missing_stdout",
        PacktoolsError::MissingStderr => "missing_stderr",
        PacktoolsError::Write(_) => "write",
        PacktoolsError::Read(_) => "read",
        PacktoolsError::Timeout(_) => "timeout",
        PacktoolsError::Protocol(_) => "protocol",
        PacktoolsError::Engine(_) => "engine",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_helper_line_to_clickable_span() {
        let source = "<article>\n  <front/>\n</article>";
        let diagnostic = normalize_diagnostic(
            HelperDiagnostic {
                severity: "error".to_owned(),
                message: "invalid front".to_owned(),
                rule: Some("packtools:front:test".to_owned()),
                line: 2,
                column: 3,
                approximate: false,
            },
            source,
            Revision(7),
            Some("article.xml"),
        );
        assert_eq!(diagnostic.span.start.line, 2);
        assert_eq!(diagnostic.span.start.column, 3);
        assert_eq!(diagnostic.revision, Revision(7));
        assert_eq!(diagnostic.source, "SPS 1.10");
    }

    #[test]
    fn defaults_unknown_locale_to_brazilian_portuguese() {
        assert_eq!(normalized_locale("fr"), "pt_BR");
        assert_eq!(normalized_locale("en"), "en");
    }
}
