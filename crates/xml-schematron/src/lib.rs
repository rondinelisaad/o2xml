use std::collections::BTreeMap;

use quick_xml::{
    escape::unescape,
    events::{BytesStart, Event},
    name::ResolveResult,
    reader::NsReader,
    XmlVersion,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use xml_document::LineIndex;
use xml_domain::{Diagnostic, Revision, Severity, SourceSpan};

mod schxslt;

pub use schxslt::{SchXsltError, SchXsltProcessEngine, SchXsltReport, SchXsltRequest};

const SVRL_NAMESPACE: &[u8] = b"http://purl.oclc.org/dsdl/svrl";
const MAX_SVRL_BYTES: usize = 16 * 1024 * 1024;
const MAX_FINDINGS: usize = 10_000;
const MAX_MESSAGE_BYTES: usize = 64 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SvrlReport {
    pub findings: Vec<SvrlFinding>,
    pub truncated: bool,
}

impl SvrlReport {
    #[must_use]
    pub fn diagnostics(
        &self,
        source: &str,
        revision: Revision,
        file: Option<&str>,
        resolved_locations: &BTreeMap<String, SourceSpan>,
    ) -> Vec<Diagnostic> {
        let index = LineIndex::new(source);
        self.findings
            .iter()
            .map(|finding| {
                let span = finding
                    .location
                    .as_ref()
                    .and_then(|location| resolved_locations.get(location))
                    .copied()
                    .unwrap_or_else(|| {
                        let mut span = index.span(source, 0, 0);
                        span.approximate = true;
                        span
                    });
                Diagnostic {
                    severity: finding.severity,
                    message: finding.message.clone(),
                    source: "schematron".to_owned(),
                    rule: finding.id.clone().or_else(|| finding.test.clone()),
                    file: file.map(ToOwned::to_owned),
                    span,
                    revision,
                }
            })
            .collect()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SvrlFinding {
    pub kind: FindingKind,
    pub severity: Severity,
    pub message: String,
    pub location: Option<String>,
    pub test: Option<String>,
    pub id: Option<String>,
    pub role: Option<String>,
    pub flag: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    FailedAssert,
    SuccessfulReport,
}

pub fn parse_svrl(source: &str) -> Result<SvrlReport, SvrlError> {
    if source.len() > MAX_SVRL_BYTES {
        return Err(SvrlError::TooLarge {
            limit: MAX_SVRL_BYTES,
        });
    }
    let mut reader = NsReader::from_str(source);
    reader.config_mut().trim_text(false);
    let mut findings = Vec::new();
    let mut current: Option<SvrlFinding> = None;
    let mut message = String::new();
    let mut capture_depth = 0_usize;
    let mut saw_root = false;
    let mut truncated = false;

    loop {
        let (resolution, event) = reader
            .read_resolved_event()
            .map_err(|error| SvrlError::InvalidXml(error.to_string()))?;
        match event {
            Event::Start(element) => {
                let is_svrl = is_svrl_namespace(&resolution);
                let local_name = element.local_name();
                if !saw_root {
                    if !is_svrl || local_name.as_ref() != b"schematron-output" {
                        return Err(SvrlError::InvalidRoot);
                    }
                    saw_root = true;
                }
                if capture_depth > 0 {
                    capture_depth = capture_depth.saturating_add(1);
                } else if current.is_some()
                    && is_svrl
                    && matches!(local_name.as_ref(), b"text" | b"diagnostic-reference")
                {
                    capture_depth = 1;
                } else if is_svrl {
                    if let Some(kind) = finding_kind(local_name.as_ref()) {
                        if current.is_some() {
                            return Err(SvrlError::NestedFinding);
                        }
                        current = Some(read_finding(kind, &element)?);
                        message.clear();
                    }
                }
            }
            Event::Empty(element) => {
                if is_svrl_namespace(&resolution) {
                    if let Some(kind) = finding_kind(element.local_name().as_ref()) {
                        push_finding(&mut findings, read_finding(kind, &element)?, &mut truncated);
                    }
                }
            }
            Event::Text(text) if capture_depth > 0 => {
                let decoded = text
                    .xml_content(XmlVersion::Implicit1_0)
                    .map_err(|error| SvrlError::InvalidXml(error.to_string()))?;
                let decoded =
                    unescape(&decoded).map_err(|error| SvrlError::InvalidXml(error.to_string()))?;
                if message.len().saturating_add(decoded.len()) > MAX_MESSAGE_BYTES {
                    return Err(SvrlError::MessageTooLarge {
                        limit: MAX_MESSAGE_BYTES,
                    });
                }
                message.push_str(&decoded);
            }
            Event::CData(text) if capture_depth > 0 => {
                let decoded = text
                    .decode()
                    .map_err(|error| SvrlError::InvalidXml(error.to_string()))?;
                if message.len().saturating_add(decoded.len()) > MAX_MESSAGE_BYTES {
                    return Err(SvrlError::MessageTooLarge {
                        limit: MAX_MESSAGE_BYTES,
                    });
                }
                message.push_str(&decoded);
            }
            Event::End(element) => {
                if capture_depth > 0 {
                    capture_depth = capture_depth.saturating_sub(1);
                } else if is_svrl_namespace(&resolution)
                    && finding_kind(element.local_name().as_ref()).is_some()
                {
                    let mut finding = current.take().ok_or(SvrlError::UnexpectedFindingEnd)?;
                    finding.message = normalize_message(&message);
                    push_finding(&mut findings, finding, &mut truncated);
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    if !saw_root {
        return Err(SvrlError::InvalidRoot);
    }
    if current.is_some() {
        return Err(SvrlError::UnexpectedEof);
    }
    Ok(SvrlReport {
        findings,
        truncated,
    })
}

fn is_svrl_namespace(resolution: &ResolveResult<'_>) -> bool {
    matches!(resolution, ResolveResult::Bound(namespace) if namespace.as_ref() == SVRL_NAMESPACE)
}

const fn finding_kind(local_name: &[u8]) -> Option<FindingKind> {
    match local_name {
        b"failed-assert" => Some(FindingKind::FailedAssert),
        b"successful-report" => Some(FindingKind::SuccessfulReport),
        _ => None,
    }
}

fn read_finding(kind: FindingKind, element: &BytesStart<'_>) -> Result<SvrlFinding, SvrlError> {
    let mut finding = SvrlFinding {
        kind,
        severity: default_severity(kind),
        message: String::new(),
        location: None,
        test: None,
        id: None,
        role: None,
        flag: None,
    };
    for attribute in element.attributes() {
        let attribute = attribute.map_err(|error| SvrlError::InvalidXml(error.to_string()))?;
        let value = attribute
            .normalized_value(XmlVersion::Implicit1_0)
            .map_err(|error| SvrlError::InvalidXml(error.to_string()))?
            .into_owned();
        match attribute.key.as_ref() {
            b"location" => finding.location = bounded_attribute(value)?,
            b"test" => finding.test = bounded_attribute(value)?,
            b"id" => finding.id = bounded_attribute(value)?,
            b"role" => finding.role = bounded_attribute(value)?,
            b"flag" => finding.flag = bounded_attribute(value)?,
            _ => {}
        }
    }
    finding.severity =
        severity_from_metadata(kind, finding.role.as_deref().or(finding.flag.as_deref()));
    Ok(finding)
}

fn bounded_attribute(value: String) -> Result<Option<String>, SvrlError> {
    if value.len() > MAX_MESSAGE_BYTES {
        return Err(SvrlError::AttributeTooLarge {
            limit: MAX_MESSAGE_BYTES,
        });
    }
    Ok((!value.is_empty()).then_some(value))
}

const fn default_severity(kind: FindingKind) -> Severity {
    match kind {
        FindingKind::FailedAssert => Severity::Error,
        FindingKind::SuccessfulReport => Severity::Information,
    }
}

fn severity_from_metadata(kind: FindingKind, value: Option<&str>) -> Severity {
    let Some(value) = value else {
        return default_severity(kind);
    };
    match value.to_ascii_lowercase().as_str() {
        "fatal" | "error" => Severity::Error,
        "warn" | "warning" => Severity::Warning,
        "info" | "information" => Severity::Information,
        "hint" => Severity::Hint,
        _ => default_severity(kind),
    }
}

fn normalize_message(message: &str) -> String {
    message.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn push_finding(findings: &mut Vec<SvrlFinding>, finding: SvrlFinding, truncated: &mut bool) {
    if findings.len() < MAX_FINDINGS {
        findings.push(finding);
    } else {
        *truncated = true;
    }
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum SvrlError {
    #[error("SVRL exceeds the {limit}-byte limit")]
    TooLarge { limit: usize },
    #[error("SVRL message exceeds the {limit}-byte limit")]
    MessageTooLarge { limit: usize },
    #[error("SVRL attribute exceeds the {limit}-byte limit")]
    AttributeTooLarge { limit: usize },
    #[error("invalid SVRL XML: {0}")]
    InvalidXml(String),
    #[error("document is not an SVRL schematron-output report")]
    InvalidRoot,
    #[error("SVRL findings cannot be nested")]
    NestedFinding,
    #[error("SVRL finding ended without a matching start")]
    UnexpectedFindingEnd,
    #[error("SVRL ended inside a finding")]
    UnexpectedEof,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_asserts_reports_metadata_and_mixed_messages() -> Result<(), SvrlError> {
        let source = r#"<svrl:schematron-output xmlns:svrl="http://purl.oclc.org/dsdl/svrl">
          <svrl:failed-assert id="article-type" role="warning" test="@article-type" location="/article[1]">
            <svrl:text>article-type <em>is</em> required</svrl:text>
          </svrl:failed-assert>
          <svrl:successful-report flag="info" test="title" location="/article[1]/front[1]">
            <svrl:text>Title was supplied</svrl:text>
          </svrl:successful-report>
        </svrl:schematron-output>"#;
        let report = parse_svrl(source)?;

        assert_eq!(report.findings.len(), 2);
        assert_eq!(report.findings[0].kind, FindingKind::FailedAssert);
        assert_eq!(report.findings[0].severity, Severity::Warning);
        assert_eq!(report.findings[0].message, "article-type is required");
        assert_eq!(report.findings[0].id.as_deref(), Some("article-type"));
        assert_eq!(report.findings[1].kind, FindingKind::SuccessfulReport);
        assert_eq!(report.findings[1].severity, Severity::Information);
        Ok(())
    }

    #[test]
    fn converts_locations_to_navigable_diagnostics() -> Result<(), SvrlError> {
        let source = "<article>\n  <front/>\n</article>";
        let report = parse_svrl(
            r#"<schematron-output xmlns="http://purl.oclc.org/dsdl/svrl"><failed-assert test="x" location="/article/front"><text>failure</text></failed-assert></schematron-output>"#,
        )?;
        let index = LineIndex::new(source);
        let mut locations = BTreeMap::new();
        locations.insert("/article/front".to_owned(), index.span(source, 12, 20));
        let diagnostics = report.diagnostics(source, Revision(4), Some("article.xml"), &locations);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].span.start.line, 2);
        assert_eq!(diagnostics[0].rule.as_deref(), Some("x"));
        assert_eq!(diagnostics[0].source, "schematron");
        Ok(())
    }

    #[test]
    fn rejects_non_svrl_documents() {
        assert_eq!(parse_svrl("<report/>"), Err(SvrlError::InvalidRoot));
    }
}
