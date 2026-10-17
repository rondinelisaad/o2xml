use quick_xml::{events::Event, Reader, Writer};
use thiserror::Error;
use xml_document::LineIndex;
use xml_domain::{Diagnostic, OutlineEntry, Revision, Severity};

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("cannot format malformed XML at byte {byte}: {message}")]
    Malformed { byte: usize, message: String },
    #[error("cannot write formatted XML: {0}")]
    Write(String),
    #[error("formatted XML is not UTF-8: {0}")]
    Encoding(#[from] std::string::FromUtf8Error),
}

#[must_use]
pub fn validate_well_formed(
    source: &str,
    revision: Revision,
    file: Option<&str>,
) -> Vec<Diagnostic> {
    let mut reader = Reader::from_str(source);
    reader.config_mut().trim_text(false);
    let index = LineIndex::new(source);
    let mut open_elements: Vec<(String, usize)> = Vec::new();
    let mut root_elements = 0_usize;

    let diagnostic = |message: String, start: usize, end: usize| Diagnostic {
        severity: Severity::Error,
        message,
        source: "well-formedness".to_owned(),
        rule: None,
        file: file.map(ToOwned::to_owned),
        span: index.span(source, start, end),
        revision,
    };

    loop {
        let before = usize::try_from(reader.buffer_position()).unwrap_or_default();
        match reader.read_event() {
            Ok(Event::Start(start)) => {
                if open_elements.is_empty() {
                    root_elements = root_elements.saturating_add(1);
                }
                open_elements.push((
                    String::from_utf8_lossy(start.name().as_ref()).into_owned(),
                    before,
                ));
            }
            Ok(Event::Empty(_)) if open_elements.is_empty() => {
                root_elements = root_elements.saturating_add(1);
            }
            Ok(Event::End(_)) => {
                open_elements.pop();
            }
            Ok(Event::Text(text)) if open_elements.is_empty() => {
                if !String::from_utf8_lossy(text.as_ref()).trim().is_empty() {
                    let end = usize::try_from(reader.buffer_position()).unwrap_or(before);
                    return vec![diagnostic(
                        "character data is not allowed outside the document element".to_owned(),
                        before,
                        end,
                    )];
                }
            }
            Ok(Event::CData(data)) if open_elements.is_empty() => {
                if !data.as_ref().is_empty() {
                    let end = usize::try_from(reader.buffer_position()).unwrap_or(before);
                    return vec![diagnostic(
                        "CDATA is not allowed outside the document element".to_owned(),
                        before,
                        end,
                    )];
                }
            }
            Ok(Event::Eof) => {
                if let Some((name, start)) = open_elements.last() {
                    return vec![diagnostic(
                        format!("unclosed element <{name}>"),
                        *start,
                        source.len(),
                    )];
                }
                return match root_elements {
                    1 => Vec::new(),
                    0 => vec![diagnostic(
                        "document has no root element".to_owned(),
                        0,
                        source.len(),
                    )],
                    _ => vec![diagnostic(
                        "document has more than one root element".to_owned(),
                        0,
                        source.len(),
                    )],
                };
            }
            Ok(_) => {}
            Err(error) => {
                let byte = usize::try_from(reader.buffer_position()).unwrap_or(source.len());
                return vec![diagnostic(error.to_string(), byte.saturating_sub(1), byte)];
            }
        }
    }
}

#[must_use]
pub fn outline(source: &str) -> Vec<OutlineEntry> {
    let mut reader = Reader::from_str(source);
    reader.config_mut().trim_text(false);
    let index = LineIndex::new(source);
    let mut entries: Vec<OutlineEntry> = Vec::new();
    let mut open_entries: Vec<usize> = Vec::new();

    loop {
        let before = usize::try_from(reader.buffer_position()).unwrap_or_default();
        match reader.read_event() {
            Ok(Event::Start(start)) => {
                let after = usize::try_from(reader.buffer_position()).unwrap_or(before);
                entries.push(OutlineEntry {
                    name: String::from_utf8_lossy(start.name().as_ref()).into_owned(),
                    depth: open_entries.len(),
                    span: index.span(source, before, after),
                });
                open_entries.push(entries.len() - 1);
            }
            Ok(Event::Empty(start)) => {
                let after = usize::try_from(reader.buffer_position()).unwrap_or(before);
                entries.push(OutlineEntry {
                    name: String::from_utf8_lossy(start.name().as_ref()).into_owned(),
                    depth: open_entries.len(),
                    span: index.span(source, before, after),
                });
            }
            Ok(Event::End(_)) => {
                let after = usize::try_from(reader.buffer_position()).unwrap_or(before);
                if let Some(entry) = open_entries.pop().and_then(|entry| entries.get_mut(entry)) {
                    entry.span.end = index.position(source, after);
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            Ok(_) => {}
        }
    }
    entries
}

pub fn pretty_print(source: &str, indent_size: usize) -> Result<String, FormatError> {
    if let Some(problem) = validate_well_formed(source, Revision(0), None)
        .into_iter()
        .next()
    {
        return Err(FormatError::Malformed {
            byte: problem.span.start.byte,
            message: problem.message,
        });
    }
    let indent_size = indent_size.clamp(1, 8);
    let mut reader = Reader::from_str(source);
    reader.config_mut().trim_text(false);
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', indent_size);

    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            Ok(event) => writer
                .write_event(event.into_owned())
                .map_err(|error| FormatError::Write(error.to_string()))?,
            Err(error) => {
                return Err(FormatError::Malformed {
                    byte: usize::try_from(reader.buffer_position()).unwrap_or(source.len()),
                    message: error.to_string(),
                });
            }
        }
    }
    Ok(String::from_utf8(writer.into_inner())?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_mismatched_end_tag() {
        let diagnostics = validate_well_formed("<article><front></article>", Revision(4), None);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].revision, Revision(4));
        assert_eq!(diagnostics[0].severity, Severity::Error);
    }

    #[test]
    fn builds_flat_outline_with_depth() {
        let nodes = outline("<article><front/><body><p/></body></article>");
        let summary: Vec<_> = nodes
            .iter()
            .map(|node| (node.name.as_str(), node.depth))
            .collect();
        assert_eq!(
            summary,
            vec![("article", 0), ("front", 1), ("body", 1), ("p", 2),]
        );
    }

    #[test]
    fn formatter_rejects_malformed_input() {
        assert!(pretty_print("<a>", 2).is_err());
    }

    #[test]
    fn rejects_multiple_roots_and_text_outside_root() {
        assert!(!validate_well_formed("<a/><b/>", Revision(0), None).is_empty());
        assert!(!validate_well_formed("text<a/>", Revision(0), None).is_empty());
    }
}
