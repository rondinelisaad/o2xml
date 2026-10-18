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
use xml_domain::{Revision, SourceSpan};

const MAX_SOURCE_BYTES: usize = 32 * 1024 * 1024;
const MAX_ELEMENTS: usize = 100_000;
const MAX_TEXT_RUNS: usize = 200_000;
const MAX_DEPTH: usize = 1_024;
const MAX_EDIT_BYTES: usize = 256 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorDocument {
    pub root: AuthorElement,
    pub plain_text: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorElement {
    pub name: String,
    pub label: String,
    pub role: AuthorRole,
    pub span: SourceSpan,
    pub content: Vec<AuthorContent>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AuthorContent {
    Element { element: Box<AuthorElement> },
    Text { run: AuthorTextRun },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthorRole {
    Document,
    Section,
    Heading,
    Paragraph,
    List,
    ListItem,
    Caption,
    Figure,
    Table,
    Code,
    Emphasis,
    Link,
    Inline,
    Container,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorTextRun {
    pub id: u64,
    pub text: String,
    pub source_span: SourceSpan,
    pub author_start: usize,
    pub author_end: usize,
    pub editable: bool,
}

/// Projects the XML event stream into a bounded structural view while retaining
/// exact source spans. XML remains the only source of truth.
pub fn project(source: &str) -> Result<AuthorDocument, AuthorError> {
    if source.len() > MAX_SOURCE_BYTES {
        return Err(AuthorError::SourceTooLarge {
            limit: MAX_SOURCE_BYTES,
        });
    }
    if let Some(diagnostic) = xml_syntax::validate_well_formed(source, Revision(0), None).first() {
        return Err(AuthorError::InvalidXml(diagnostic.message.clone()));
    }

    let line_index = LineIndex::new(source);
    let mut reader = NsReader::from_str(source);
    reader.config_mut().trim_text(false);
    let mut stack = Vec::<PendingElement>::new();
    let mut root = None;
    let mut plain_text = String::new();
    let mut element_count = 0_usize;
    let mut run_count = 0_usize;

    loop {
        let start = usize::try_from(reader.buffer_position()).unwrap_or(source.len());
        let (namespace, event) = reader
            .read_resolved_event()
            .map_err(|error| AuthorError::InvalidXml(error.to_string()))?;
        let namespace = OwnedNamespace::from(namespace);
        let end = usize::try_from(reader.buffer_position()).unwrap_or(source.len());
        match event {
            Event::Start(element) => {
                if stack.len() >= MAX_DEPTH {
                    return Err(AuthorError::TooDeep { limit: MAX_DEPTH });
                }
                element_count = element_count.saturating_add(1);
                if element_count > MAX_ELEMENTS {
                    return Err(AuthorError::TooManyElements {
                        limit: MAX_ELEMENTS,
                    });
                }
                stack.push(PendingElement::new(
                    expanded_element_name(namespace, &element)?,
                    start,
                ));
            }
            Event::Empty(element) => {
                element_count = element_count.saturating_add(1);
                if element_count > MAX_ELEMENTS {
                    return Err(AuthorError::TooManyElements {
                        limit: MAX_ELEMENTS,
                    });
                }
                let pending =
                    PendingElement::new(expanded_element_name(namespace, &element)?, start);
                append_element(
                    &mut stack,
                    &mut root,
                    pending.finish(source, &line_index, end),
                )?;
            }
            Event::End(_) => {
                let pending = stack
                    .pop()
                    .ok_or_else(|| AuthorError::InvalidXml("unexpected closing tag".to_owned()))?;
                append_element(
                    &mut stack,
                    &mut root,
                    pending.finish(source, &line_index, end),
                )?;
            }
            Event::Text(text) => {
                let decoded = text
                    .xml_content(XmlVersion::Implicit1_0)
                    .map_err(|error| AuthorError::InvalidXml(error.to_string()))?;
                let decoded = unescape(&decoded)
                    .map_err(|error| AuthorError::InvalidXml(error.to_string()))?
                    .into_owned();
                append_text(
                    &mut stack,
                    &mut plain_text,
                    &line_index,
                    source,
                    &mut run_count,
                    decoded,
                    start,
                    end,
                    true,
                )?;
            }
            Event::CData(text) => {
                let decoded = text
                    .decode()
                    .map_err(|error| AuthorError::InvalidXml(error.to_string()))?
                    .into_owned();
                append_text(
                    &mut stack,
                    &mut plain_text,
                    &line_index,
                    source,
                    &mut run_count,
                    decoded,
                    start,
                    end,
                    false,
                )?;
            }
            Event::GeneralRef(reference) => {
                let name = String::from_utf8_lossy(reference.as_ref());
                let decoded = match name.as_ref() {
                    "amp" => "&".to_owned(),
                    "lt" => "<".to_owned(),
                    "gt" => ">".to_owned(),
                    "apos" => "'".to_owned(),
                    "quot" => "\"".to_owned(),
                    _ => format!("&{name};"),
                };
                append_text(
                    &mut stack,
                    &mut plain_text,
                    &line_index,
                    source,
                    &mut run_count,
                    decoded,
                    start,
                    end,
                    false,
                )?;
            }
            Event::Eof => break,
            Event::Decl(_) | Event::DocType(_) | Event::Comment(_) | Event::PI(_) => {}
        }
    }

    if !stack.is_empty() {
        return Err(AuthorError::InvalidXml("unclosed element".to_owned()));
    }
    Ok(AuthorDocument {
        root: root.ok_or(AuthorError::MissingRoot)?,
        plain_text,
    })
}

/// Replaces one complete editable text run. Markup is escaped, then the entire
/// document is reparsed before the transaction is returned to the caller.
pub fn edit_text_run(source: &str, run_id: u64, value: &str) -> Result<String, AuthorError> {
    if value.len() > MAX_EDIT_BYTES {
        return Err(AuthorError::EditTooLarge {
            limit: MAX_EDIT_BYTES,
        });
    }
    let document = project(source)?;
    let run = find_run(&document.root, run_id).ok_or(AuthorError::UnknownTextRun)?;
    if !run.editable {
        return Err(AuthorError::TextRunReadOnly);
    }
    let range = run.source_span.start.byte..run.source_span.end.byte;
    let escaped = quick_xml::escape::escape(value);
    let mut updated = String::with_capacity(
        source
            .len()
            .saturating_sub(range.len())
            .saturating_add(escaped.len()),
    );
    updated.push_str(&source[..range.start]);
    updated.push_str(&escaped);
    updated.push_str(&source[range.end..]);
    if let Some(diagnostic) = xml_syntax::validate_well_formed(&updated, Revision(0), None).first()
    {
        return Err(AuthorError::InvalidEdit(diagnostic.message.clone()));
    }
    Ok(updated)
}

#[must_use]
pub fn source_to_author(document: &AuthorDocument, source_byte: usize) -> Option<usize> {
    all_runs(&document.root)
        .into_iter()
        .find(|run| {
            source_byte >= run.source_span.start.byte && source_byte < run.source_span.end.byte
        })
        .map(|run| {
            let raw_len = run
                .source_span
                .end
                .byte
                .saturating_sub(run.source_span.start.byte);
            let text_len = run.author_end.saturating_sub(run.author_start);
            let relative = source_byte
                .saturating_sub(run.source_span.start.byte)
                .saturating_mul(text_len)
                .checked_div(raw_len)
                .unwrap_or(0);
            run.author_start.saturating_add(relative)
        })
}

#[must_use]
pub fn author_to_source(document: &AuthorDocument, author_byte: usize) -> Option<usize> {
    all_runs(&document.root)
        .into_iter()
        .find(|run| author_byte >= run.author_start && author_byte < run.author_end)
        .map(|run| {
            let text_len = run.author_end.saturating_sub(run.author_start);
            let raw_len = run
                .source_span
                .end
                .byte
                .saturating_sub(run.source_span.start.byte);
            let relative = author_byte
                .saturating_sub(run.author_start)
                .saturating_mul(raw_len)
                .checked_div(text_len)
                .unwrap_or(0);
            run.source_span.start.byte.saturating_add(relative)
        })
}

#[derive(Debug)]
struct PendingElement {
    name: String,
    start: usize,
    content: Vec<AuthorContent>,
}

impl PendingElement {
    fn new(name: String, start: usize) -> Self {
        Self {
            name,
            start,
            content: Vec::new(),
        }
    }

    fn finish(self, source: &str, index: &LineIndex, end: usize) -> AuthorElement {
        AuthorElement {
            label: local_name(&self.name).to_owned(),
            role: role_for(local_name(&self.name)),
            name: self.name,
            span: index.span(source, self.start, end),
            content: self.content,
        }
    }
}

fn append_element(
    stack: &mut [PendingElement],
    root: &mut Option<AuthorElement>,
    element: AuthorElement,
) -> Result<(), AuthorError> {
    if let Some(parent) = stack.last_mut() {
        parent.content.push(AuthorContent::Element {
            element: Box::new(element),
        });
    } else if root.replace(element).is_some() {
        return Err(AuthorError::MultipleRoots);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn append_text(
    stack: &mut [PendingElement],
    plain_text: &mut String,
    index: &LineIndex,
    source: &str,
    run_count: &mut usize,
    text: String,
    start: usize,
    end: usize,
    editable: bool,
) -> Result<(), AuthorError> {
    let Some(parent) = stack.last_mut() else {
        return Ok(());
    };
    let formatting_whitespace = text.trim().is_empty() && text.contains(['\r', '\n']);
    if text.is_empty() || formatting_whitespace {
        return Ok(());
    }
    *run_count = run_count.saturating_add(1);
    if *run_count > MAX_TEXT_RUNS {
        return Err(AuthorError::TooManyTextRuns {
            limit: MAX_TEXT_RUNS,
        });
    }
    let author_start = plain_text.len();
    plain_text.push_str(&text);
    let author_end = plain_text.len();
    parent.content.push(AuthorContent::Text {
        run: AuthorTextRun {
            id: u64::try_from(*run_count).unwrap_or(u64::MAX),
            text,
            source_span: index.span(source, start, end),
            author_start,
            author_end,
            editable,
        },
    });
    Ok(())
}

fn find_run(element: &AuthorElement, id: u64) -> Option<&AuthorTextRun> {
    element.content.iter().find_map(|content| match content {
        AuthorContent::Text { run } if run.id == id => Some(run),
        AuthorContent::Element { element } => find_run(element, id),
        AuthorContent::Text { .. } => None,
    })
}

fn all_runs(element: &AuthorElement) -> Vec<&AuthorTextRun> {
    let mut runs = Vec::new();
    collect_runs(element, &mut runs);
    runs
}

fn collect_runs<'a>(element: &'a AuthorElement, runs: &mut Vec<&'a AuthorTextRun>) {
    for content in &element.content {
        match content {
            AuthorContent::Text { run } => runs.push(run),
            AuthorContent::Element { element } => collect_runs(element, runs),
        }
    }
}

fn role_for(local: &str) -> AuthorRole {
    match local {
        "article" | "book" | "TEI" => AuthorRole::Document,
        "front" | "body" | "back" | "sec" | "section" | "div" => AuthorRole::Section,
        "article-title" | "title" | "subtitle" => AuthorRole::Heading,
        "p" | "para" => AuthorRole::Paragraph,
        "list" | "def-list" => AuthorRole::List,
        "list-item" | "item" => AuthorRole::ListItem,
        "caption" | "label" => AuthorRole::Caption,
        "fig" | "figure" | "graphic" => AuthorRole::Figure,
        "table" | "table-wrap" => AuthorRole::Table,
        "code" | "preformat" => AuthorRole::Code,
        "bold" | "italic" | "underline" | "em" | "strong" | "sup" | "sub" => AuthorRole::Emphasis,
        "ext-link" | "xref" | "link" => AuthorRole::Link,
        "named-content" | "styled-content" | "span" | "surname" | "given-names" | "prefix"
        | "suffix" | "email" | "uri" | "monospace" | "sc" | "strike" | "inline-formula" => {
            AuthorRole::Inline
        }
        _ => AuthorRole::Container,
    }
}

fn expanded_element_name(
    namespace: OwnedNamespace,
    element: &BytesStart<'_>,
) -> Result<String, AuthorError> {
    let local_name = element.local_name();
    let local = String::from_utf8_lossy(local_name.as_ref());
    match namespace {
        OwnedNamespace::Unbound => Ok(local.into_owned()),
        OwnedNamespace::Bound(namespace) => Ok(format!(
            "{{{}}}{local}",
            String::from_utf8_lossy(&namespace)
        )),
        OwnedNamespace::Unknown(prefix) => Err(AuthorError::UnknownPrefix(
            String::from_utf8_lossy(&prefix).into_owned(),
        )),
    }
}

#[derive(Clone, Debug)]
enum OwnedNamespace {
    Unbound,
    Bound(Vec<u8>),
    Unknown(Vec<u8>),
}

impl From<ResolveResult<'_>> for OwnedNamespace {
    fn from(value: ResolveResult<'_>) -> Self {
        match value {
            ResolveResult::Unbound => Self::Unbound,
            ResolveResult::Bound(namespace) => Self::Bound(namespace.as_ref().to_vec()),
            ResolveResult::Unknown(prefix) => Self::Unknown(prefix),
        }
    }
}

fn local_name(expanded: &str) -> &str {
    expanded.rsplit('}').next().unwrap_or(expanded)
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum AuthorError {
    #[error("invalid XML: {0}")]
    InvalidXml(String),
    #[error("undeclared namespace prefix: {0}")]
    UnknownPrefix(String),
    #[error("XML source exceeds the {limit}-byte Author Mode limit")]
    SourceTooLarge { limit: usize },
    #[error("XML exceeds the {limit}-element Author Mode limit")]
    TooManyElements { limit: usize },
    #[error("XML exceeds the {limit}-text-run Author Mode limit")]
    TooManyTextRuns { limit: usize },
    #[error("XML exceeds the {limit}-level Author Mode depth limit")]
    TooDeep { limit: usize },
    #[error("Author Mode requires one document element")]
    MissingRoot,
    #[error("Author Mode found multiple document elements")]
    MultipleRoots,
    #[error("Author Mode text run does not exist")]
    UnknownTextRun,
    #[error("CDATA and entity-reference runs are read-only in Author Mode")]
    TextRunReadOnly,
    #[error("Author Mode edit exceeds the {limit}-byte limit")]
    EditTooLarge { limit: usize },
    #[error("Author Mode edit would produce invalid XML: {0}")]
    InvalidEdit(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projects_mixed_jats_content_in_source_order() -> Result<(), AuthorError> {
        let source = "<article><front><article-title>Digital <italic>Preservation</italic></article-title></front><body><p>Hello <bold>world</bold>.</p></body></article>";
        let document = project(source)?;
        assert_eq!(document.root.role, AuthorRole::Document);
        assert_eq!(document.plain_text, "Digital PreservationHello world.");
        let runs = all_runs(&document.root);
        assert_eq!(
            runs.iter().map(|run| run.text.as_str()).collect::<Vec<_>>(),
            vec!["Digital ", "Preservation", "Hello ", "world", "."]
        );
        assert!(runs.iter().all(|run| run.editable));
        Ok(())
    }

    #[test]
    fn edits_one_text_run_and_escapes_markup() -> Result<(), AuthorError> {
        let source = "<article><p>Hello <bold>world</bold>.</p></article>";
        let updated = edit_text_run(source, 2, "XML & more")?;
        assert_eq!(
            updated,
            "<article><p>Hello <bold>XML &amp; more</bold>.</p></article>"
        );
        assert_eq!(project(&updated)?.plain_text, "Hello XML & more.");
        Ok(())
    }

    #[test]
    fn maps_author_and_source_offsets() -> Result<(), AuthorError> {
        let source = "<article><p>alpha</p><p>beta</p></article>";
        let document = project(source)?;
        let beta_source = source.find("beta").ok_or(AuthorError::UnknownTextRun)?;
        let beta_author =
            source_to_author(&document, beta_source).ok_or(AuthorError::UnknownTextRun)?;
        assert_eq!(&document.plain_text[beta_author..beta_author + 4], "beta");
        assert_eq!(author_to_source(&document, beta_author), Some(beta_source));
        Ok(())
    }

    #[test]
    fn keeps_cdata_read_only() -> Result<(), AuthorError> {
        let source = "<article><p><![CDATA[a < b]]></p></article>";
        let document = project(source)?;
        let run = all_runs(&document.root)
            .into_iter()
            .next()
            .ok_or(AuthorError::UnknownTextRun)?;
        assert!(!run.editable);
        assert_eq!(
            edit_text_run(source, run.id, "replacement"),
            Err(AuthorError::TextRunReadOnly)
        );
        Ok(())
    }
}
