use std::{
    collections::{BTreeMap, HashMap},
    ops::Range,
};

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
const MAX_DEPTH: usize = 1_024;
const MAX_ROWS: usize = 2_000;
const MAX_COLUMNS: usize = 64;
const MAX_CELL_BYTES: usize = 16 * 1024;
const MAX_LEAF_DEPTH: usize = 6;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct XmlGrid {
    pub row_element: String,
    pub columns: Vec<String>,
    pub rows: Vec<GridRow>,
    pub truncated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridRow {
    pub span: SourceSpan,
    pub cells: Vec<GridCell>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GridCell {
    pub value: String,
    pub editable: bool,
}

#[derive(Clone, Debug)]
struct Element {
    name: String,
    parent: Option<usize>,
    children: Vec<usize>,
    attributes: Vec<(String, String)>,
    text: String,
    text_edit: Option<Range<usize>>,
    text_edit_ambiguous: bool,
    start: usize,
    end: usize,
}

/// Extract a bounded, read-only table from repeated sibling elements.
///
/// `row_element` accepts either the expanded form (`{uri}local`) or a local
/// name. When omitted, the repeated sibling group with the most rows is used.
pub fn extract(source: &str, row_element: Option<&str>) -> Result<XmlGrid, GridError> {
    Ok(project(source, row_element)?.grid)
}

/// Replace one unambiguous scalar text cell and reject the transaction unless
/// the complete resulting document remains well-formed.
pub fn edit(
    source: &str,
    row_element: Option<&str>,
    row_index: usize,
    column: &str,
    value: &str,
) -> Result<String, GridError> {
    if value.len() > MAX_CELL_BYTES {
        return Err(GridError::CellTooLarge {
            limit: MAX_CELL_BYTES,
        });
    }
    let projection = project(source, row_element)?;
    let column_index = projection
        .grid
        .columns
        .iter()
        .position(|candidate| candidate == column)
        .ok_or_else(|| GridError::UnknownColumn(column.to_owned()))?;
    let target = projection
        .targets
        .get(row_index)
        .and_then(|row| row.get(column_index))
        .and_then(Clone::clone)
        .ok_or(GridError::CellReadOnly)?;
    let escaped = quick_xml::escape::escape(value);
    let mut updated = String::with_capacity(
        source
            .len()
            .saturating_sub(target.len())
            .saturating_add(escaped.len()),
    );
    updated.push_str(&source[..target.start]);
    updated.push_str(&escaped);
    updated.push_str(&source[target.end..]);
    let diagnostics = xml_syntax::validate_well_formed(&updated, Revision(0), None);
    if let Some(diagnostic) = diagnostics.first() {
        return Err(GridError::InvalidEdit(diagnostic.message.clone()));
    }
    Ok(updated)
}

struct GridProjection {
    grid: XmlGrid,
    targets: Vec<Vec<Option<Range<usize>>>>,
}

fn project(source: &str, row_element: Option<&str>) -> Result<GridProjection, GridError> {
    if source.len() > MAX_SOURCE_BYTES {
        return Err(GridError::SourceTooLarge {
            limit: MAX_SOURCE_BYTES,
        });
    }
    let elements = parse(source)?;
    let groups = repeated_groups(&elements, row_element);
    let (_, row_indexes) = groups.into_iter().next().ok_or(GridError::NoRepeatedRows)?;
    let rows_truncated = row_indexes.len() > MAX_ROWS;
    let row_indexes = &row_indexes[..row_indexes.len().min(MAX_ROWS)];

    let mut flattened = Vec::with_capacity(row_indexes.len());
    let mut column_frequency = HashMap::<String, usize>::new();
    for &index in row_indexes {
        let mut cells = BTreeMap::<String, Vec<CellValue>>::new();
        flatten_element(&elements, index, "", 0, &mut cells)?;
        for key in cells.keys() {
            *column_frequency.entry(key.clone()).or_default() += 1;
        }
        flattened.push(cells);
    }
    let mut columns = column_frequency.into_iter().collect::<Vec<_>>();
    columns.sort_by(|(left_name, left_count), (right_name, right_count)| {
        right_count
            .cmp(left_count)
            .then_with(|| left_name.cmp(right_name))
    });
    let columns_truncated = columns.len() > MAX_COLUMNS;
    let columns = columns
        .into_iter()
        .take(MAX_COLUMNS)
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
    if columns.is_empty() {
        return Err(GridError::NoScalarColumns);
    }

    let line_index = LineIndex::new(source);
    let mut targets = Vec::with_capacity(row_indexes.len());
    let rows = row_indexes
        .iter()
        .zip(flattened)
        .map(|(&index, values)| {
            let mut row_targets = Vec::with_capacity(columns.len());
            let cells = columns
                .iter()
                .map(|column| {
                    let parts = values.get(column);
                    let target = parts.and_then(|parts| {
                        (parts.len() == 1).then(|| parts[0].edit.clone()).flatten()
                    });
                    row_targets.push(target.clone());
                    GridCell {
                        value: parts
                            .map(|parts| {
                                parts
                                    .iter()
                                    .map(|part| part.value.as_str())
                                    .collect::<Vec<_>>()
                                    .join(" | ")
                            })
                            .unwrap_or_default(),
                        editable: target.is_some(),
                    }
                })
                .collect();
            targets.push(row_targets);
            GridRow {
                span: line_index.span(source, elements[index].start, elements[index].end),
                cells,
            }
        })
        .collect();

    Ok(GridProjection {
        grid: XmlGrid {
            row_element: elements[row_indexes[0]].name.clone(),
            columns,
            rows,
            truncated: rows_truncated || columns_truncated,
        },
        targets,
    })
}

fn parse(source: &str) -> Result<Vec<Element>, GridError> {
    let mut reader = NsReader::from_str(source);
    reader.config_mut().trim_text(false);
    let mut elements = Vec::<Element>::new();
    let mut stack = Vec::<usize>::new();
    loop {
        let start = usize::try_from(reader.buffer_position()).unwrap_or(source.len());
        let (namespace, event) = reader
            .read_resolved_event()
            .map_err(|error| GridError::InvalidXml(error.to_string()))?;
        let namespace = OwnedNamespace::from(namespace);
        let end = usize::try_from(reader.buffer_position()).unwrap_or(source.len());
        match event {
            Event::Start(element) => {
                if stack.len() >= MAX_DEPTH {
                    return Err(GridError::TooDeep { limit: MAX_DEPTH });
                }
                let index = push_element(
                    &mut elements,
                    stack.last().copied(),
                    expanded_element_name(namespace, &element)?,
                    attributes(&reader, &element)?,
                    start,
                    end,
                )?;
                stack.push(index);
            }
            Event::Empty(element) => {
                push_element(
                    &mut elements,
                    stack.last().copied(),
                    expanded_element_name(namespace, &element)?,
                    attributes(&reader, &element)?,
                    start,
                    end,
                )?;
            }
            Event::End(_) => {
                let index = stack.pop().ok_or_else(|| {
                    GridError::InvalidXml("unexpected closing element".to_owned())
                })?;
                elements[index].end = end;
            }
            Event::Text(text) => {
                if let Some(&index) = stack.last() {
                    let decoded = text
                        .xml_content(XmlVersion::Implicit1_0)
                        .map_err(|error| GridError::InvalidXml(error.to_string()))?;
                    let decoded = unescape(&decoded)
                        .map_err(|error| GridError::InvalidXml(error.to_string()))?;
                    elements[index].text.push_str(&decoded);
                    if !decoded.trim().is_empty() || elements[index].text_edit.is_some() {
                        extend_text_edit(&mut elements[index], start, end);
                    }
                }
            }
            Event::CData(text) => {
                if let Some(&index) = stack.last() {
                    let decoded = text
                        .decode()
                        .map_err(|error| GridError::InvalidXml(error.to_string()))?;
                    elements[index].text.push_str(&decoded);
                    if !decoded.trim().is_empty() {
                        elements[index].text_edit = None;
                        elements[index].text_edit_ambiguous = true;
                    }
                }
            }
            Event::GeneralRef(reference) => {
                if let Some(&index) = stack.last() {
                    let value = format!("&{};", String::from_utf8_lossy(reference.as_ref()));
                    let decoded = unescape(&value).unwrap_or_else(|_| value.clone().into());
                    elements[index].text.push_str(&decoded);
                    extend_text_edit(&mut elements[index], start, end);
                }
            }
            Event::Eof => break,
            Event::Comment(_) | Event::PI(_) => {
                if let Some(&index) = stack.last() {
                    elements[index].text_edit = None;
                    elements[index].text_edit_ambiguous = true;
                }
            }
            Event::Decl(_) | Event::DocType(_) => {}
        }
    }
    if !stack.is_empty() {
        return Err(GridError::InvalidXml("unclosed element".to_owned()));
    }
    Ok(elements)
}

fn extend_text_edit(element: &mut Element, start: usize, end: usize) {
    if element.text_edit_ambiguous {
        return;
    }
    match &mut element.text_edit {
        Some(range) if range.end == start => range.end = end,
        Some(_) => {
            element.text_edit = None;
            element.text_edit_ambiguous = true;
        }
        None => element.text_edit = Some(start..end),
    }
}

fn push_element(
    elements: &mut Vec<Element>,
    parent: Option<usize>,
    name: String,
    attributes: Vec<(String, String)>,
    start: usize,
    end: usize,
) -> Result<usize, GridError> {
    if elements.len() >= MAX_ELEMENTS {
        return Err(GridError::TooManyElements {
            limit: MAX_ELEMENTS,
        });
    }
    let index = elements.len();
    elements.push(Element {
        name,
        parent,
        children: Vec::new(),
        attributes,
        text: String::new(),
        text_edit: None,
        text_edit_ambiguous: false,
        start,
        end,
    });
    if let Some(parent) = parent {
        elements[parent].children.push(index);
    }
    Ok(index)
}

fn expanded_element_name(
    namespace: OwnedNamespace,
    element: &BytesStart<'_>,
) -> Result<String, GridError> {
    expanded_name(namespace, element.local_name().as_ref())
}

fn attributes(
    reader: &NsReader<&[u8]>,
    element: &BytesStart<'_>,
) -> Result<Vec<(String, String)>, GridError> {
    let mut result = Vec::new();
    for attribute in element.attributes() {
        let attribute = attribute.map_err(|error| GridError::InvalidXml(error.to_string()))?;
        if attribute.key.as_ref() == b"xmlns" || attribute.key.as_ref().starts_with(b"xmlns:") {
            continue;
        }
        let (namespace, local) = reader.resolver().resolve_attribute(attribute.key);
        let name = expanded_name(OwnedNamespace::from(namespace), local.as_ref())?;
        let value = attribute
            .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
            .map_err(|error| GridError::InvalidXml(error.to_string()))?;
        result.push((format!("@{name}"), value.into_owned()));
    }
    Ok(result)
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

fn expanded_name(namespace: OwnedNamespace, local: &[u8]) -> Result<String, GridError> {
    let local = String::from_utf8_lossy(local);
    match namespace {
        OwnedNamespace::Unbound => Ok(local.into_owned()),
        OwnedNamespace::Bound(namespace) => Ok(format!(
            "{{{}}}{local}",
            String::from_utf8_lossy(&namespace)
        )),
        OwnedNamespace::Unknown(prefix) => Err(GridError::UnknownPrefix(
            String::from_utf8_lossy(&prefix).into_owned(),
        )),
    }
}

fn repeated_groups(
    elements: &[Element],
    requested: Option<&str>,
) -> Vec<((usize, String), Vec<usize>)> {
    let requested = requested.map(str::trim).filter(|value| !value.is_empty());
    let mut groups = HashMap::<(usize, String), Vec<usize>>::new();
    for (index, element) in elements.iter().enumerate() {
        let Some(parent) = element.parent else {
            continue;
        };
        let matches =
            requested.is_none_or(|name| element.name == name || local_name(&element.name) == name);
        if matches {
            groups
                .entry((parent, element.name.clone()))
                .or_default()
                .push(index);
        }
    }
    let mut groups = groups
        .into_iter()
        .filter(|(_, rows)| {
            rows.len() >= 2
                && (requested.is_some()
                    || rows
                        .iter()
                        .any(|&index| element_has_scalar(elements, index, 0)))
        })
        .collect::<Vec<_>>();
    groups.sort_by_key(|(_, rows)| std::cmp::Reverse(rows.len()));
    groups
}

fn element_has_scalar(elements: &[Element], index: usize, depth: usize) -> bool {
    if depth > MAX_LEAF_DEPTH {
        return false;
    }
    let element = &elements[index];
    !element.attributes.is_empty()
        || !element.text.trim().is_empty()
        || element
            .children
            .iter()
            .any(|&child| element_has_scalar(elements, child, depth.saturating_add(1)))
}

fn local_name(expanded: &str) -> &str {
    expanded.rsplit('}').next().unwrap_or(expanded)
}

fn flatten_element(
    elements: &[Element],
    index: usize,
    prefix: &str,
    depth: usize,
    cells: &mut BTreeMap<String, Vec<CellValue>>,
) -> Result<(), GridError> {
    if depth > MAX_LEAF_DEPTH {
        return Ok(());
    }
    let element = &elements[index];
    for (name, value) in &element.attributes {
        let column = join_path(prefix, name);
        push_value(cells, column, value.trim(), None)?;
    }
    let text = element.text.trim();
    if !text.is_empty() {
        let column = if prefix.is_empty() { "#text" } else { prefix };
        let edit = element
            .children
            .is_empty()
            .then(|| element.text_edit.clone())
            .flatten();
        push_value(cells, column.to_owned(), text, edit)?;
    }
    for &child in &element.children {
        let child_name = local_name(&elements[child].name);
        let child_prefix = join_path(prefix, child_name);
        flatten_element(
            elements,
            child,
            &child_prefix,
            depth.saturating_add(1),
            cells,
        )?;
    }
    Ok(())
}

fn join_path(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix}/{name}")
    }
}

fn push_value(
    cells: &mut BTreeMap<String, Vec<CellValue>>,
    column: String,
    value: &str,
    edit: Option<Range<usize>>,
) -> Result<(), GridError> {
    if value.len() > MAX_CELL_BYTES {
        return Err(GridError::CellTooLarge {
            limit: MAX_CELL_BYTES,
        });
    }
    cells.entry(column).or_default().push(CellValue {
        value: value.to_owned(),
        edit,
    });
    Ok(())
}

#[derive(Clone, Debug)]
struct CellValue {
    value: String,
    edit: Option<Range<usize>>,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum GridError {
    #[error("invalid XML: {0}")]
    InvalidXml(String),
    #[error("undeclared namespace prefix: {0}")]
    UnknownPrefix(String),
    #[error("XML source exceeds the {limit}-byte grid limit")]
    SourceTooLarge { limit: usize },
    #[error("XML exceeds the {limit}-element grid limit")]
    TooManyElements { limit: usize },
    #[error("XML exceeds the {limit}-level grid depth limit")]
    TooDeep { limit: usize },
    #[error("grid cell exceeds the {limit}-byte limit")]
    CellTooLarge { limit: usize },
    #[error("no repeated sibling elements match the requested row")]
    NoRepeatedRows,
    #[error("the repeated rows contain no scalar values")]
    NoScalarColumns,
    #[error("grid column does not exist: {0}")]
    UnknownColumn(String),
    #[error("grid cell is read-only because it is absent, repeated, an attribute, or mixed XML")]
    CellReadOnly,
    #[error("grid edit would produce invalid XML: {0}")]
    InvalidEdit(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_nested_jats_contributors_as_columns() -> Result<(), GridError> {
        let source = r#"<article><contrib-group>
          <contrib contrib-type="author"><name><surname>Silva</surname><given-names>João</given-names></name><role>Editor</role></contrib>
          <contrib contrib-type="author"><name><surname>Souza</surname><given-names>Maria</given-names></name><role>Author</role></contrib>
        </contrib-group></article>"#;
        let grid = extract(source, Some("contrib"))?;
        assert_eq!(grid.row_element, "contrib");
        assert_eq!(grid.rows.len(), 2);
        assert!(grid.columns.contains(&"@contrib-type".to_owned()));
        let surname = grid
            .columns
            .iter()
            .position(|column| column == "name/surname")
            .ok_or(GridError::NoScalarColumns)?;
        assert_eq!(
            grid.rows[0]
                .cells
                .get(surname)
                .map(|cell| cell.value.as_str()),
            Some("Silva")
        );
        assert!(grid.rows[0].cells[surname].editable);
        assert_eq!(grid.rows[1].span.start.line, 3);
        Ok(())
    }

    #[test]
    fn automatic_selection_uses_largest_repeated_sibling_group() -> Result<(), GridError> {
        let source = "<root><x><v>1</v></x><x><v>2</v></x><y/><y/><y/></root>";
        let grid = extract(source, None)?;
        assert_eq!(grid.row_element, "x");
        assert_eq!(grid.rows.len(), 2);
        Ok(())
    }

    #[test]
    fn preserves_namespace_identity() -> Result<(), GridError> {
        let source = r#"<r xmlns:a="urn:a"><a:item a:id="1"/><a:item a:id="2"/></r>"#;
        let grid = extract(source, Some("item"))?;
        assert_eq!(grid.row_element, "{urn:a}item");
        assert_eq!(grid.columns, vec!["@{urn:a}id"]);
        assert!(!grid.rows[0].cells[0].editable);
        Ok(())
    }

    #[test]
    fn edits_a_unique_text_cell_and_escapes_xml_markup() -> Result<(), GridError> {
        let source = "<root><item><name>A</name></item><item><name>B</name></item></root>";
        let updated = edit(source, Some("item"), 1, "name", "C & <D>")?;
        assert_eq!(
            updated,
            "<root><item><name>A</name></item><item><name>C &amp; &lt;D&gt;</name></item></root>"
        );
        let rebuilt = extract(&updated, Some("item"))?;
        assert_eq!(rebuilt.rows[1].cells[0].value, "C & <D>");
        assert!(rebuilt.rows[1].cells[0].editable);
        Ok(())
    }

    #[test]
    fn rejects_attribute_repeated_and_unknown_cells() -> Result<(), GridError> {
        let source =
            "<root><item id=\"1\"><v>A</v><v>B</v></item><item id=\"2\"><v>C</v></item></root>";
        assert_eq!(
            edit(source, Some("item"), 0, "@id", "3"),
            Err(GridError::CellReadOnly)
        );
        assert_eq!(
            edit(source, Some("item"), 0, "v", "D"),
            Err(GridError::CellReadOnly)
        );
        assert!(matches!(
            edit(source, Some("item"), 0, "missing", "D"),
            Err(GridError::UnknownColumn(_))
        ));
        Ok(())
    }
}
