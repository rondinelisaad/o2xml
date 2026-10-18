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
use xml_domain::{Revision, SourcePosition, SourceSpan};

const MAX_INPUT_BYTES: usize = 128 * 1024 * 1024;
const MAX_TOKENS: usize = 50_000;
const MAX_LCS_CELLS: usize = 4_000_000;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct XmlDiffOptions {
    pub ignore_whitespace_only_text: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XmlDiffReport {
    pub semantic_equal: bool,
    pub exact_equal: bool,
    pub hunks: Vec<XmlDiffHunk>,
    pub coarse: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XmlDiffHunk {
    pub kind: DiffKind,
    pub left: Option<SourceSpan>,
    pub right: Option<SourceSpan>,
    pub left_anchor: SourcePosition,
    pub right_anchor: SourcePosition,
    pub left_tokens: usize,
    pub right_tokens: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffKind {
    Added,
    Removed,
    Changed,
}

pub fn compare(
    left: &str,
    right: &str,
    options: XmlDiffOptions,
) -> Result<XmlDiffReport, XmlDiffError> {
    if left.len() > MAX_INPUT_BYTES || right.len() > MAX_INPUT_BYTES {
        return Err(XmlDiffError::InputTooLarge {
            limit: MAX_INPUT_BYTES,
        });
    }
    let left_tokens = tokenize(left, options)?;
    let right_tokens = tokenize(right, options)?;
    if left_tokens
        .iter()
        .map(|token| &token.canonical)
        .eq(right_tokens.iter().map(|token| &token.canonical))
    {
        return Ok(XmlDiffReport {
            semantic_equal: true,
            exact_equal: left == right,
            hunks: Vec::new(),
            coarse: false,
        });
    }
    let cells = left_tokens
        .len()
        .saturating_add(1)
        .saturating_mul(right_tokens.len().saturating_add(1));
    let (ranges, coarse) = if cells <= MAX_LCS_CELLS {
        (lcs_ranges(&left_tokens, &right_tokens), false)
    } else {
        (coarse_ranges(&left_tokens, &right_tokens), true)
    };
    let left_index = LineIndex::new(left);
    let right_index = LineIndex::new(right);
    let hunks = ranges
        .into_iter()
        .map(|range| {
            let left_count = range.left_end.saturating_sub(range.left_start);
            let right_count = range.right_end.saturating_sub(range.right_start);
            XmlDiffHunk {
                kind: match (left_count, right_count) {
                    (0, _) => DiffKind::Added,
                    (_, 0) => DiffKind::Removed,
                    _ => DiffKind::Changed,
                },
                left: token_span(
                    left,
                    &left_index,
                    &left_tokens,
                    range.left_start,
                    range.left_end,
                ),
                right: token_span(
                    right,
                    &right_index,
                    &right_tokens,
                    range.right_start,
                    range.right_end,
                ),
                left_anchor: token_anchor(left, &left_index, &left_tokens, range.left_start),
                right_anchor: token_anchor(right, &right_index, &right_tokens, range.right_start),
                left_tokens: left_count,
                right_tokens: right_count,
            }
        })
        .collect();
    Ok(XmlDiffReport {
        semantic_equal: false,
        exact_equal: false,
        hunks,
        coarse,
    })
}

/// Apply one semantic hunk from `right` to `left`, then reject the result unless
/// it remains a well-formed XML document. The caller still owns schema-level
/// validation and the editor transaction/undo boundary.
pub fn merge_hunk(
    left: &str,
    right: &str,
    options: XmlDiffOptions,
    hunk_index: usize,
) -> Result<String, XmlDiffError> {
    let report = compare(left, right, options)?;
    let hunk = report
        .hunks
        .get(hunk_index)
        .ok_or(XmlDiffError::UnknownHunk { index: hunk_index })?;
    let left_start = hunk
        .left
        .as_ref()
        .map_or(hunk.left_anchor.byte, |span| span.start.byte);
    let left_end = hunk
        .left
        .as_ref()
        .map_or(hunk.left_anchor.byte, |span| span.end.byte);
    let right_start = hunk
        .right
        .as_ref()
        .map_or(hunk.right_anchor.byte, |span| span.start.byte);
    let right_end = hunk
        .right
        .as_ref()
        .map_or(hunk.right_anchor.byte, |span| span.end.byte);
    let replacement = right
        .get(right_start..right_end)
        .ok_or(XmlDiffError::InvalidSpan)?;
    if left.get(left_start..left_end).is_none() {
        return Err(XmlDiffError::InvalidSpan);
    }
    let mut merged = String::with_capacity(
        left.len()
            .saturating_sub(left_end.saturating_sub(left_start))
            .saturating_add(replacement.len()),
    );
    merged.push_str(left.get(..left_start).ok_or(XmlDiffError::InvalidSpan)?);
    merged.push_str(replacement);
    merged.push_str(left.get(left_end..).ok_or(XmlDiffError::InvalidSpan)?);
    if let Some(diagnostic) = xml_syntax::validate_well_formed(&merged, Revision(0), None)
        .into_iter()
        .next()
    {
        return Err(XmlDiffError::InvalidMerge(diagnostic.message));
    }
    Ok(merged)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Token {
    canonical: String,
    start: usize,
    end: usize,
}

fn tokenize(source: &str, options: XmlDiffOptions) -> Result<Vec<Token>, XmlDiffError> {
    let mut reader = NsReader::from_str(source);
    reader.config_mut().trim_text(false);
    let mut tokens = Vec::new();
    loop {
        let start = usize::try_from(reader.buffer_position()).unwrap_or(usize::MAX);
        let (namespace, event) = reader
            .read_resolved_event()
            .map_err(|error| XmlDiffError::InvalidXml(error.to_string()))?;
        let namespace = OwnedNamespace::from(namespace);
        let end = usize::try_from(reader.buffer_position()).unwrap_or(source.len());
        match event {
            Event::Start(element) => push_token(
                &mut tokens,
                canonical_start(&reader, &namespace, &element)?,
                start,
                end,
            )?,
            Event::Empty(element) => {
                let name = expanded_name(&namespace, element.local_name().as_ref())?;
                push_token(
                    &mut tokens,
                    canonical_start(&reader, &namespace, &element)?,
                    start,
                    end,
                )?;
                push_token(&mut tokens, format!("end:{name}"), start, end)?;
            }
            Event::End(element) => push_token(
                &mut tokens,
                format!(
                    "end:{}",
                    expanded_name(&namespace, element.local_name().as_ref())?
                ),
                start,
                end,
            )?,
            Event::Text(text) => {
                let decoded = text
                    .xml_content(XmlVersion::Implicit1_0)
                    .map_err(|error| XmlDiffError::InvalidXml(error.to_string()))?;
                let decoded = unescape(&decoded)
                    .map_err(|error| XmlDiffError::InvalidXml(error.to_string()))?;
                if !(options.ignore_whitespace_only_text && decoded.trim().is_empty()) {
                    push_token(&mut tokens, format!("text:{decoded}"), start, end)?;
                }
            }
            Event::CData(text) => {
                let decoded = text
                    .decode()
                    .map_err(|error| XmlDiffError::InvalidXml(error.to_string()))?;
                push_token(&mut tokens, format!("text:{decoded}"), start, end)?;
            }
            Event::Comment(text) => push_token(
                &mut tokens,
                format!("comment:{}", String::from_utf8_lossy(text.as_ref())),
                start,
                end,
            )?,
            Event::PI(text) => push_token(
                &mut tokens,
                format!("pi:{}", String::from_utf8_lossy(text.as_ref())),
                start,
                end,
            )?,
            Event::DocType(text) => push_token(
                &mut tokens,
                format!("doctype:{}", String::from_utf8_lossy(text.as_ref())),
                start,
                end,
            )?,
            Event::GeneralRef(text) => push_token(
                &mut tokens,
                format!("ref:{}", String::from_utf8_lossy(text.as_ref())),
                start,
                end,
            )?,
            Event::Eof => break,
            Event::Decl(_) => {}
        }
    }
    Ok(tokens)
}

fn canonical_start(
    reader: &NsReader<&[u8]>,
    namespace: &OwnedNamespace,
    element: &BytesStart<'_>,
) -> Result<String, XmlDiffError> {
    let name = expanded_name(namespace, element.local_name().as_ref())?;
    let mut attributes = Vec::new();
    for attribute in element.attributes() {
        let attribute = attribute.map_err(|error| XmlDiffError::InvalidXml(error.to_string()))?;
        if attribute.key.as_ref() == b"xmlns" || attribute.key.as_ref().starts_with(b"xmlns:") {
            continue;
        }
        let (namespace, local) = reader.resolver().resolve_attribute(attribute.key);
        let namespace = OwnedNamespace::from(namespace);
        let name = expanded_name(&namespace, local.as_ref())?;
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
            .map_err(|error| XmlDiffError::InvalidXml(error.to_string()))?;
        attributes.push((name, value.into_owned()));
    }
    attributes.sort();
    let attributes = attributes
        .into_iter()
        .map(|(name, value)| format!("|{name}={value}"))
        .collect::<String>();
    Ok(format!("start:{name}{attributes}"))
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

fn expanded_name(namespace: &OwnedNamespace, local: &[u8]) -> Result<String, XmlDiffError> {
    let local = String::from_utf8_lossy(local);
    match namespace {
        OwnedNamespace::Bound(namespace) => {
            Ok(format!("{{{}}}{local}", String::from_utf8_lossy(namespace)))
        }
        OwnedNamespace::Unbound => Ok(local.into_owned()),
        OwnedNamespace::Unknown(prefix) => Err(XmlDiffError::UnknownPrefix(
            String::from_utf8_lossy(prefix).into_owned(),
        )),
    }
}

fn push_token(
    tokens: &mut Vec<Token>,
    canonical: String,
    start: usize,
    end: usize,
) -> Result<(), XmlDiffError> {
    if tokens.len() >= MAX_TOKENS {
        return Err(XmlDiffError::TooManyTokens { limit: MAX_TOKENS });
    }
    tokens.push(Token {
        canonical,
        start,
        end,
    });
    Ok(())
}

#[derive(Clone, Copy)]
struct DiffRange {
    left_start: usize,
    left_end: usize,
    right_start: usize,
    right_end: usize,
}

fn lcs_ranges(left: &[Token], right: &[Token]) -> Vec<DiffRange> {
    let columns = right.len() + 1;
    let mut matrix = vec![0_u32; (left.len() + 1) * columns];
    for left_index in (0..left.len()).rev() {
        for right_index in (0..right.len()).rev() {
            let index = left_index * columns + right_index;
            matrix[index] = if left[left_index].canonical == right[right_index].canonical {
                matrix[(left_index + 1) * columns + right_index + 1].saturating_add(1)
            } else {
                matrix[(left_index + 1) * columns + right_index]
                    .max(matrix[left_index * columns + right_index + 1])
            };
        }
    }
    let mut ranges = Vec::new();
    let mut left_index = 0;
    let mut right_index = 0;
    while left_index < left.len() || right_index < right.len() {
        if left_index < left.len()
            && right_index < right.len()
            && left[left_index].canonical == right[right_index].canonical
        {
            left_index += 1;
            right_index += 1;
            continue;
        }
        let left_start = left_index;
        let right_start = right_index;
        while left_index < left.len() || right_index < right.len() {
            if left_index < left.len()
                && right_index < right.len()
                && left[left_index].canonical == right[right_index].canonical
            {
                break;
            }
            let remove_score = if left_index < left.len() {
                matrix[(left_index + 1) * columns + right_index]
            } else {
                0
            };
            let add_score = if right_index < right.len() {
                matrix[left_index * columns + right_index + 1]
            } else {
                0
            };
            if left_index < left.len() && (right_index == right.len() || remove_score >= add_score)
            {
                left_index += 1;
            } else {
                right_index += 1;
            }
        }
        ranges.push(DiffRange {
            left_start,
            left_end: left_index,
            right_start,
            right_end: right_index,
        });
    }
    ranges
}

fn coarse_ranges(left: &[Token], right: &[Token]) -> Vec<DiffRange> {
    let prefix = left
        .iter()
        .zip(right)
        .take_while(|(left, right)| left.canonical == right.canonical)
        .count();
    let suffix = left[prefix..]
        .iter()
        .rev()
        .zip(right[prefix..].iter().rev())
        .take_while(|(left, right)| left.canonical == right.canonical)
        .count();
    vec![DiffRange {
        left_start: prefix,
        left_end: left.len().saturating_sub(suffix),
        right_start: prefix,
        right_end: right.len().saturating_sub(suffix),
    }]
}

fn token_span(
    source: &str,
    index: &LineIndex,
    tokens: &[Token],
    start: usize,
    end: usize,
) -> Option<SourceSpan> {
    if start == end {
        return None;
    }
    let first = tokens.get(start)?;
    let last = tokens.get(end.saturating_sub(1))?;
    Some(index.span(source, first.start, last.end))
}

fn token_anchor(source: &str, index: &LineIndex, tokens: &[Token], at: usize) -> SourcePosition {
    let byte = tokens
        .get(at)
        .map(|token| token.start)
        .or_else(|| tokens.last().map(|token| token.end))
        .unwrap_or_default();
    index.position(source, byte.min(source.len()))
}

#[derive(Debug, Error)]
pub enum XmlDiffError {
    #[error("XML diff input exceeds the {limit}-byte limit")]
    InputTooLarge { limit: usize },
    #[error("XML diff exceeds the {limit}-token limit")]
    TooManyTokens { limit: usize },
    #[error("invalid XML: {0}")]
    InvalidXml(String),
    #[error("unknown namespace prefix: {0}")]
    UnknownPrefix(String),
    #[error("XML diff hunk {index} does not exist")]
    UnknownHunk { index: usize },
    #[error("XML diff produced an invalid source span")]
    InvalidSpan,
    #[error("applying the XML diff hunk would create malformed XML: {0}")]
    InvalidMerge(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignores_attribute_order_and_namespace_prefix_spelling() -> Result<(), XmlDiffError> {
        let left = r#"<a:article xmlns:a="urn:jats" x="1" y="2"/>"#;
        let right = r#"<j:article xmlns:j="urn:jats" y="2" x="1"></j:article>"#;
        let report = compare(left, right, XmlDiffOptions::default())?;
        assert!(report.semantic_equal);
        assert!(!report.exact_equal);
        Ok(())
    }

    #[test]
    fn reports_navigable_structural_changes() -> Result<(), XmlDiffError> {
        let left = "<article><title>Old</title></article>";
        let right = "<article><title>New</title><body/></article>";
        let report = compare(left, right, XmlDiffOptions::default())?;
        assert!(!report.semantic_equal);
        assert_eq!(report.hunks.len(), 2);
        assert_eq!(report.hunks[0].kind, DiffKind::Changed);
        assert_eq!(report.hunks[1].kind, DiffKind::Added);
        assert_eq!(
            report.hunks[0].left.as_ref().map(|span| span.start.line),
            Some(1)
        );
        Ok(())
    }

    #[test]
    fn whitespace_policy_is_explicit() -> Result<(), XmlDiffError> {
        let compact = "<article><title>Example</title></article>";
        let formatted = "<article>\n  <title>Example</title>\n</article>";
        assert!(!compare(compact, formatted, XmlDiffOptions::default())?.semantic_equal);
        assert!(
            compare(
                compact,
                formatted,
                XmlDiffOptions {
                    ignore_whitespace_only_text: true,
                },
            )?
            .semantic_equal
        );
        Ok(())
    }

    #[test]
    fn applies_changed_and_added_hunks_only_when_result_is_well_formed() -> Result<(), XmlDiffError>
    {
        let left = "<article><title>Old</title></article>";
        let right = "<article><title>New</title><body/></article>";
        let changed = merge_hunk(left, right, XmlDiffOptions::default(), 0)?;
        assert_eq!(changed, "<article><title>New</title></article>");
        let added = merge_hunk(&changed, right, XmlDiffOptions::default(), 0)?;
        assert_eq!(added, right);
        Ok(())
    }

    #[test]
    fn rejects_unknown_hunks() {
        assert!(matches!(
            merge_hunk("<a/>", "<b/>", XmlDiffOptions::default(), 5),
            Err(XmlDiffError::UnknownHunk { index: 5 })
        ));
    }
}
