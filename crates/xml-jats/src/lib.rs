use quick_xml::{events::Event, name::ResolveResult, reader::NsReader};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use xml_document::LineIndex;
use xml_domain::SourceSpan;

const MAX_DEPTH: usize = 4_096;
const MAX_ELEMENTS: usize = 250_000;
const MAX_ENTRIES: usize = 50_000;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JatsInspection {
    pub is_jats: bool,
    pub entries: Vec<JatsEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JatsEntry {
    pub role: JatsRole,
    pub element_name: String,
    pub label: String,
    pub depth: usize,
    pub span: SourceSpan,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JatsRole {
    Article,
    Front,
    JournalMeta,
    ArticleMeta,
    TitleGroup,
    ArticleTitle,
    ContribGroup,
    Body,
    Section,
    Back,
    RefList,
    FloatsGroup,
}

impl JatsRole {
    const fn label(self) -> &'static str {
        match self {
            Self::Article => "Article",
            Self::Front => "Front",
            Self::JournalMeta => "Journal Meta",
            Self::ArticleMeta => "Article Meta",
            Self::TitleGroup => "Title Group",
            Self::ArticleTitle => "Article Title",
            Self::ContribGroup => "Contributors",
            Self::Body => "Body",
            Self::Section => "Section",
            Self::Back => "Back",
            Self::RefList => "References",
            Self::FloatsGroup => "Floats",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct OpenElement {
    entry_index: Option<usize>,
}

pub fn inspect(source: &str) -> Result<JatsInspection, JatsError> {
    let mut reader = NsReader::from_str(source);
    reader.config_mut().trim_text(false);
    let line_index = LineIndex::new(source);
    let mut entries = Vec::new();
    let mut open_elements: Vec<OpenElement> = Vec::new();
    let mut recognized_depth = 0_usize;
    let mut element_count = 0_usize;
    let mut root_count = 0_usize;
    let mut is_jats = false;

    loop {
        let before = position(&reader, 0);
        let (resolution, event) =
            reader
                .read_resolved_event()
                .map_err(|error| JatsError::Malformed {
                    byte: before,
                    message: error.to_string(),
                })?;
        match event {
            Event::Start(element) => {
                element_count = count_element(element_count)?;
                if open_elements.len() >= MAX_DEPTH {
                    return Err(JatsError::TooDeep { limit: MAX_DEPTH });
                }
                if open_elements.is_empty() {
                    root_count = root_count.saturating_add(1);
                    if root_count > 1 {
                        return Err(JatsError::MultipleRoots);
                    }
                    is_jats = resolution == ResolveResult::Unbound
                        && element.name().as_ref() == b"article";
                }
                let entry_index = if is_jats && resolution == ResolveResult::Unbound {
                    role_for(element.name().as_ref())
                        .map(|role| {
                            add_entry(
                                &mut entries,
                                role,
                                recognized_depth,
                                &line_index,
                                source,
                                before,
                                position(&reader, before),
                            )
                        })
                        .transpose()?
                } else {
                    None
                };
                if entry_index.is_some() {
                    recognized_depth = recognized_depth.saturating_add(1);
                }
                open_elements.push(OpenElement { entry_index });
            }
            Event::Empty(element) => {
                element_count = count_element(element_count)?;
                if open_elements.is_empty() {
                    root_count = root_count.saturating_add(1);
                    if root_count > 1 {
                        return Err(JatsError::MultipleRoots);
                    }
                    is_jats = resolution == ResolveResult::Unbound
                        && element.name().as_ref() == b"article";
                }
                if is_jats && resolution == ResolveResult::Unbound {
                    if let Some(role) = role_for(element.name().as_ref()) {
                        add_entry(
                            &mut entries,
                            role,
                            recognized_depth,
                            &line_index,
                            source,
                            before,
                            position(&reader, before),
                        )?;
                    }
                }
            }
            Event::End(_) => {
                let frame = open_elements.pop().ok_or(JatsError::Malformed {
                    byte: before,
                    message: "unexpected closing element".to_owned(),
                })?;
                if let Some(entry_index) = frame.entry_index {
                    recognized_depth = recognized_depth.saturating_sub(1);
                    if let Some(entry) = entries.get_mut(entry_index) {
                        entry.span.end = line_index.position(source, position(&reader, before));
                    }
                }
            }
            Event::Text(text) if open_elements.is_empty() => {
                if !String::from_utf8_lossy(text.as_ref()).trim().is_empty() {
                    return Err(JatsError::TextOutsideRoot { byte: before });
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }

    if !open_elements.is_empty() {
        return Err(JatsError::Malformed {
            byte: source.len(),
            message: "unclosed element".to_owned(),
        });
    }
    Ok(JatsInspection { is_jats, entries })
}

fn position(reader: &NsReader<&[u8]>, fallback: usize) -> usize {
    usize::try_from(reader.buffer_position()).unwrap_or(fallback)
}

fn count_element(count: usize) -> Result<usize, JatsError> {
    let count = count.saturating_add(1);
    if count > MAX_ELEMENTS {
        return Err(JatsError::TooManyElements {
            limit: MAX_ELEMENTS,
        });
    }
    Ok(count)
}

fn add_entry(
    entries: &mut Vec<JatsEntry>,
    role: JatsRole,
    depth: usize,
    index: &LineIndex,
    source: &str,
    start: usize,
    end: usize,
) -> Result<usize, JatsError> {
    if entries.len() >= MAX_ENTRIES {
        return Err(JatsError::TooManyEntries { limit: MAX_ENTRIES });
    }
    let entry_index = entries.len();
    entries.push(JatsEntry {
        role,
        element_name: element_name(role).to_owned(),
        label: role.label().to_owned(),
        depth,
        span: index.span(source, start, end),
    });
    Ok(entry_index)
}

const fn role_for(name: &[u8]) -> Option<JatsRole> {
    match name {
        b"article" => Some(JatsRole::Article),
        b"front" => Some(JatsRole::Front),
        b"journal-meta" => Some(JatsRole::JournalMeta),
        b"article-meta" => Some(JatsRole::ArticleMeta),
        b"title-group" => Some(JatsRole::TitleGroup),
        b"article-title" => Some(JatsRole::ArticleTitle),
        b"contrib-group" => Some(JatsRole::ContribGroup),
        b"body" => Some(JatsRole::Body),
        b"sec" => Some(JatsRole::Section),
        b"back" => Some(JatsRole::Back),
        b"ref-list" => Some(JatsRole::RefList),
        b"floats-group" => Some(JatsRole::FloatsGroup),
        _ => None,
    }
}

const fn element_name(role: JatsRole) -> &'static str {
    match role {
        JatsRole::Article => "article",
        JatsRole::Front => "front",
        JatsRole::JournalMeta => "journal-meta",
        JatsRole::ArticleMeta => "article-meta",
        JatsRole::TitleGroup => "title-group",
        JatsRole::ArticleTitle => "article-title",
        JatsRole::ContribGroup => "contrib-group",
        JatsRole::Body => "body",
        JatsRole::Section => "sec",
        JatsRole::Back => "back",
        JatsRole::RefList => "ref-list",
        JatsRole::FloatsGroup => "floats-group",
    }
}

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum JatsError {
    #[error("malformed XML at byte {byte}: {message}")]
    Malformed { byte: usize, message: String },
    #[error("XML contains multiple document elements")]
    MultipleRoots,
    #[error("character data outside the document element at byte {byte}")]
    TextOutsideRoot { byte: usize },
    #[error("XML nesting exceeds the {limit}-element limit")]
    TooDeep { limit: usize },
    #[error("XML contains more than {limit} elements")]
    TooManyElements { limit: usize },
    #[error("JATS inspection contains more than {limit} semantic entries")]
    TooManyEntries { limit: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_the_editorial_jats_structure_with_positions() -> Result<(), JatsError> {
        let source = "<article>\n  <front><journal-meta/><article-meta><title-group><article-title>Title</article-title></title-group><contrib-group/></article-meta></front>\n  <body><sec/></body>\n  <back><ref-list/></back>\n</article>";
        let inspection = inspect(source)?;

        assert!(inspection.is_jats);
        assert_eq!(inspection.entries[0].role, JatsRole::Article);
        assert_eq!(inspection.entries[1].role, JatsRole::Front);
        assert_eq!(inspection.entries[2].label, "Journal Meta");
        assert_eq!(inspection.entries[2].depth, 2);
        assert_eq!(inspection.entries[8].role, JatsRole::Section);
        assert_eq!(inspection.entries[8].span.start.line, 3);
        assert_eq!(inspection.entries[0].span.end.line, 5);
        Ok(())
    }

    #[test]
    fn does_not_claim_unrelated_or_namespaced_articles_are_jats() -> Result<(), JatsError> {
        assert!(!inspect("<book/>")?.is_jats);
        assert!(!inspect("<article xmlns=\"urn:example\"/>")?.is_jats);
        assert!(!inspect("<x:article xmlns:x=\"urn:example\"/>")?.is_jats);
        Ok(())
    }

    #[test]
    fn rejects_multiple_document_elements() {
        assert_eq!(
            inspect("<article/><article/>"),
            Err(JatsError::MultipleRoots)
        );
    }
}
