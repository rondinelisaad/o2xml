use std::collections::BTreeMap;

use quick_xml::{
    events::{BytesStart, Event},
    name::{NamespaceResolver, ResolveResult},
    reader::NsReader,
    Reader, Writer,
};
use thiserror::Error;
use xml_document::LineIndex;
use xml_domain::{Diagnostic, OutlineEntry, Revision, Severity};

const MAX_DOCTYPE_IDENTIFIER_BYTES: usize = 8 * 1024;
const XSI_NAMESPACE: &[u8] = b"http://www.w3.org/2001/XMLSchema-instance";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentType {
    pub root_name: String,
    pub system_id: Option<String>,
    pub public_id: Option<String>,
}

#[derive(Debug, Eq, Error, PartialEq)]
pub enum DocumentTypeError {
    #[error("invalid DOCTYPE declaration")]
    Invalid,
    #[error("DOCTYPE identifier exceeds the {limit}-byte limit")]
    IdentifierTooLarge { limit: usize },
    #[error("cannot inspect malformed XML DOCTYPE: {0}")]
    Xml(String),
}

pub fn document_type(source: &str) -> Result<Option<DocumentType>, DocumentTypeError> {
    let mut reader = Reader::from_str(source);
    loop {
        match reader.read_event() {
            Ok(Event::DocType(value)) => {
                return parse_document_type(&String::from_utf8_lossy(value.as_ref())).map(Some);
            }
            Ok(Event::Eof) => return Ok(None),
            Ok(_) => {}
            Err(error) => return Err(DocumentTypeError::Xml(error.to_string())),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SchemaAssociation {
    pub no_namespace_schema_location: Option<String>,
    pub namespace_locations: BTreeMap<String, String>,
}

#[derive(Debug, Eq, Error, PartialEq)]
pub enum SchemaAssociationError {
    #[error("schema location exceeds the {limit}-byte limit")]
    LocationTooLarge { limit: usize },
    #[error("xsi:schemaLocation must contain namespace and location pairs")]
    InvalidPairs,
    #[error("cannot inspect XML schema association: {0}")]
    Xml(String),
}

pub fn schema_association(
    source: &str,
) -> Result<Option<SchemaAssociation>, SchemaAssociationError> {
    let mut reader = NsReader::from_str(source);
    loop {
        match reader.read_resolved_event() {
            Ok((_, Event::Start(element) | Event::Empty(element))) => {
                let mut association = SchemaAssociation::default();
                for attribute in element.attributes() {
                    let attribute = attribute
                        .map_err(|error| SchemaAssociationError::Xml(error.to_string()))?;
                    let (resolution, local_name) =
                        reader.resolver().resolve_attribute(attribute.key);
                    if !matches!(resolution, ResolveResult::Bound(namespace) if namespace.as_ref() == XSI_NAMESPACE)
                    {
                        continue;
                    }
                    let value = attribute
                        .normalized_value(quick_xml::XmlVersion::Implicit1_0)
                        .map_err(|error| SchemaAssociationError::Xml(error.to_string()))?;
                    if local_name.as_ref() == b"noNamespaceSchemaLocation" {
                        validate_schema_location(&value)?;
                        association.no_namespace_schema_location = Some(value.into_owned());
                    } else if local_name.as_ref() == b"schemaLocation" {
                        let tokens = value.split_ascii_whitespace().collect::<Vec<_>>();
                        if tokens.len() % 2 != 0 {
                            return Err(SchemaAssociationError::InvalidPairs);
                        }
                        let (pairs, _) = tokens.as_chunks::<2>();
                        for [namespace, location] in pairs {
                            validate_schema_location(namespace)?;
                            validate_schema_location(location)?;
                            association
                                .namespace_locations
                                .insert((*namespace).to_owned(), (*location).to_owned());
                        }
                    }
                }
                return Ok((association != SchemaAssociation::default()).then_some(association));
            }
            Ok((_, Event::Eof)) => return Ok(None),
            Ok(_) => {}
            Err(error) => return Err(SchemaAssociationError::Xml(error.to_string())),
        }
    }
}

fn validate_schema_location(value: &str) -> Result<(), SchemaAssociationError> {
    if value.is_empty() || value.len() > MAX_DOCTYPE_IDENTIFIER_BYTES {
        return Err(SchemaAssociationError::LocationTooLarge {
            limit: MAX_DOCTYPE_IDENTIFIER_BYTES,
        });
    }
    Ok(())
}

fn parse_document_type(value: &str) -> Result<DocumentType, DocumentTypeError> {
    let value = value.trim();
    let root_end = value
        .find(|character: char| character.is_ascii_whitespace() || character == '[')
        .unwrap_or(value.len());
    if root_end == 0 {
        return Err(DocumentTypeError::Invalid);
    }
    let root_name = value[..root_end].to_owned();
    validate_doctype_identifier(&root_name)?;
    let rest = value[root_end..].trim_start();
    let (public_id, system_id) = if let Some(rest) = strip_keyword(rest, "SYSTEM") {
        let (system_id, _) = quoted_identifier(rest)?;
        (None, Some(system_id))
    } else if let Some(rest) = strip_keyword(rest, "PUBLIC") {
        let (public_id, rest) = quoted_identifier(rest)?;
        let (system_id, _) = quoted_identifier(rest)?;
        (Some(public_id), Some(system_id))
    } else if rest.is_empty() || rest.starts_with('[') {
        (None, None)
    } else {
        return Err(DocumentTypeError::Invalid);
    };
    Ok(DocumentType {
        root_name,
        system_id,
        public_id,
    })
}

fn strip_keyword<'a>(value: &'a str, keyword: &str) -> Option<&'a str> {
    let (candidate, rest) = value.split_once(char::is_whitespace)?;
    candidate
        .eq_ignore_ascii_case(keyword)
        .then(|| rest.trim_start())
}

fn quoted_identifier(value: &str) -> Result<(String, &str), DocumentTypeError> {
    let quote = value.chars().next().ok_or(DocumentTypeError::Invalid)?;
    if quote != '\'' && quote != '"' {
        return Err(DocumentTypeError::Invalid);
    }
    let content = &value[quote.len_utf8()..];
    let end = content.find(quote).ok_or(DocumentTypeError::Invalid)?;
    let identifier = content[..end].to_owned();
    validate_doctype_identifier(&identifier)?;
    Ok((identifier, content[end + quote.len_utf8()..].trim_start()))
}

fn validate_doctype_identifier(value: &str) -> Result<(), DocumentTypeError> {
    if value.is_empty() {
        return Err(DocumentTypeError::Invalid);
    }
    if value.len() > MAX_DOCTYPE_IDENTIFIER_BYTES {
        return Err(DocumentTypeError::IdentifierTooLarge {
            limit: MAX_DOCTYPE_IDENTIFIER_BYTES,
        });
    }
    Ok(())
}

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
    let mut reader = NsReader::from_str(source);
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
        let event = match reader.read_event() {
            Ok(event) => event,
            Err(error) => {
                let byte = usize::try_from(reader.buffer_position()).unwrap_or(source.len());
                return vec![diagnostic(error.to_string(), byte.saturating_sub(1), byte)];
            }
        };
        let resolver = reader.resolver().clone();
        match event {
            Event::Start(start) => {
                let resolution = resolver.resolve_element(start.name()).0;
                if let Some(problem) =
                    namespace_problem(resolution, &start, &resolver, before, &diagnostic)
                {
                    return vec![problem];
                }
                if open_elements.is_empty() {
                    root_elements = root_elements.saturating_add(1);
                }
                open_elements.push((
                    String::from_utf8_lossy(start.name().as_ref()).into_owned(),
                    before,
                ));
            }
            Event::Empty(start) => {
                let resolution = resolver.resolve_element(start.name()).0;
                if let Some(problem) =
                    namespace_problem(resolution, &start, &resolver, before, &diagnostic)
                {
                    return vec![problem];
                }
                if open_elements.is_empty() {
                    root_elements = root_elements.saturating_add(1);
                }
            }
            Event::End(_) => {
                open_elements.pop();
            }
            Event::Text(text) if open_elements.is_empty() => {
                if !String::from_utf8_lossy(text.as_ref()).trim().is_empty() {
                    let end = usize::try_from(reader.buffer_position()).unwrap_or(before);
                    return vec![diagnostic(
                        "character data is not allowed outside the document element".to_owned(),
                        before,
                        end,
                    )];
                }
            }
            Event::CData(data) if open_elements.is_empty() => {
                if !data.as_ref().is_empty() {
                    let end = usize::try_from(reader.buffer_position()).unwrap_or(before);
                    return vec![diagnostic(
                        "CDATA is not allowed outside the document element".to_owned(),
                        before,
                        end,
                    )];
                }
            }
            Event::Eof => {
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
            _ => {}
        }
    }
}

fn namespace_problem(
    element_resolution: ResolveResult<'_>,
    element: &BytesStart<'_>,
    resolver: &NamespaceResolver,
    byte: usize,
    diagnostic: &impl Fn(String, usize, usize) -> Diagnostic,
) -> Option<Diagnostic> {
    if let ResolveResult::Unknown(prefix) = element_resolution {
        return Some(diagnostic(
            format!(
                "undeclared namespace prefix '{}'",
                String::from_utf8_lossy(&prefix)
            ),
            byte,
            byte.saturating_add(element.len()),
        ));
    }

    for attribute in element.attributes() {
        let attribute = match attribute {
            Ok(attribute) => attribute,
            Err(error) => {
                return Some(diagnostic(
                    format!("invalid attribute: {error}"),
                    byte,
                    byte.saturating_add(element.len()),
                ));
            }
        };
        if let ResolveResult::Unknown(prefix) = resolver.resolve_attribute(attribute.key).0 {
            return Some(diagnostic(
                format!(
                    "undeclared namespace prefix '{}' on attribute",
                    String::from_utf8_lossy(&prefix)
                ),
                byte,
                byte.saturating_add(element.len()),
            ));
        }
    }
    None
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
    fn reads_system_and_public_document_type_associations() {
        assert_eq!(
            document_type(
                r#"<!DOCTYPE article SYSTEM "https://example.org/article.dtd"><article/>"#
            ),
            Ok(Some(DocumentType {
                root_name: "article".to_owned(),
                system_id: Some("https://example.org/article.dtd".to_owned()),
                public_id: None,
            }))
        );
        assert_eq!(
            document_type(
                r#"<!DOCTYPE article PUBLIC "-//NLM//DTD JATS 1.3//EN" "JATS.dtd"><article/>"#,
            ),
            Ok(Some(DocumentType {
                root_name: "article".to_owned(),
                system_id: Some("JATS.dtd".to_owned()),
                public_id: Some("-//NLM//DTD JATS 1.3//EN".to_owned()),
            }))
        );
    }

    #[test]
    fn reads_schema_locations_with_any_xsi_prefix() -> Result<(), SchemaAssociationError> {
        let association = schema_association(
            r#"<article xmlns:i="http://www.w3.org/2001/XMLSchema-instance"
                i:schemaLocation="urn:jats https://example.org/jats.xsd"
                i:noNamespaceSchemaLocation="https://example.org/local.xsd"/>"#,
        )?
        .ok_or(SchemaAssociationError::InvalidPairs)?;
        assert_eq!(
            association
                .namespace_locations
                .get("urn:jats")
                .map(String::as_str),
            Some("https://example.org/jats.xsd")
        );
        assert_eq!(
            association.no_namespace_schema_location.as_deref(),
            Some("https://example.org/local.xsd")
        );
        Ok(())
    }

    #[test]
    fn rejects_unpaired_schema_location() {
        assert_eq!(
            schema_association(
                r#"<article xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                    xsi:schemaLocation="urn:jats"/>"#,
            ),
            Err(SchemaAssociationError::InvalidPairs)
        );
    }

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

    #[test]
    fn rejects_undeclared_namespace_prefixes_and_duplicate_attributes() {
        assert!(!validate_well_formed("<p:article/>", Revision(0), None).is_empty());
        assert!(!validate_well_formed("<article p:id=\"1\"/>", Revision(0), None).is_empty());
        assert!(
            !validate_well_formed("<article id=\"1\" id=\"2\"/>", Revision(0), None).is_empty()
        );
        assert!(validate_well_formed(
            "<p:article xmlns:p=\"urn:test\" p:id=\"1\"/>",
            Revision(0),
            None
        )
        .is_empty());
    }
}
