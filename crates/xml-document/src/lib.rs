use std::{ops::Range, path::PathBuf};

use ropey::Rope;
use thiserror::Error;
use xml_domain::{Revision, SourcePosition, SourceSpan};

#[derive(Debug, Error)]
pub enum DocumentError {
    #[error("edit was based on revision {actual:?}, but current revision is {expected:?}")]
    StaleRevision {
        expected: Revision,
        actual: Revision,
    },
    #[error("character range {start}..{end} is outside a document of {len} characters")]
    InvalidRange {
        start: usize,
        end: usize,
        len: usize,
    },
}

#[derive(Clone, Debug)]
pub struct DocumentSnapshot {
    rope: Rope,
    revision: Revision,
}

impl DocumentSnapshot {
    #[must_use]
    pub fn revision(&self) -> Revision {
        self.revision
    }

    #[must_use]
    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    #[must_use]
    pub fn len_bytes(&self) -> usize {
        self.rope.len_bytes()
    }
}

#[derive(Debug)]
pub struct XmlDocument {
    text: Rope,
    path: Option<PathBuf>,
    revision: Revision,
    saved_revision: Revision,
}

impl XmlDocument {
    #[must_use]
    pub fn new(text: &str, path: Option<PathBuf>) -> Self {
        Self {
            text: Rope::from_str(text),
            path,
            revision: Revision(0),
            saved_revision: Revision(0),
        }
    }

    #[must_use]
    pub fn revision(&self) -> Revision {
        self.revision
    }

    #[must_use]
    pub fn path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.revision != self.saved_revision
    }

    #[must_use]
    pub fn snapshot(&self) -> DocumentSnapshot {
        DocumentSnapshot {
            rope: self.text.clone(),
            revision: self.revision,
        }
    }

    pub fn replace_all(
        &mut self,
        base_revision: Revision,
        text: &str,
    ) -> Result<Revision, DocumentError> {
        if base_revision != self.revision {
            return Err(DocumentError::StaleRevision {
                expected: self.revision,
                actual: base_revision,
            });
        }
        self.text = Rope::from_str(text);
        self.revision = self.revision.next();
        Ok(self.revision)
    }

    pub fn apply_edit(
        &mut self,
        base_revision: Revision,
        char_range: Range<usize>,
        replacement: &str,
    ) -> Result<Revision, DocumentError> {
        if base_revision != self.revision {
            return Err(DocumentError::StaleRevision {
                expected: self.revision,
                actual: base_revision,
            });
        }
        if char_range.start > char_range.end || char_range.end > self.text.len_chars() {
            return Err(DocumentError::InvalidRange {
                start: char_range.start,
                end: char_range.end,
                len: self.text.len_chars(),
            });
        }
        self.text.remove(char_range.clone());
        self.text.insert(char_range.start, replacement);
        self.revision = self.revision.next();
        Ok(self.revision)
    }

    pub fn mark_saved(&mut self, path: PathBuf) {
        self.path = Some(path);
        self.saved_revision = self.revision;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineIndex {
    line_starts: Vec<usize>,
    len_bytes: usize,
}

impl LineIndex {
    #[must_use]
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (byte, character) in source.char_indices() {
            if character == '\n' {
                line_starts.push(byte + 1);
            }
        }
        Self {
            line_starts,
            len_bytes: source.len(),
        }
    }

    #[must_use]
    pub fn position(&self, source: &str, byte: usize) -> SourcePosition {
        let mut safe_byte = byte.min(self.len_bytes).min(source.len());
        while safe_byte > 0 && !source.is_char_boundary(safe_byte) {
            safe_byte -= 1;
        }
        let line_index = self
            .line_starts
            .partition_point(|start| *start <= safe_byte)
            - 1;
        let line_start = self.line_starts[line_index];
        let column = source[line_start..safe_byte].chars().count() + 1;
        SourcePosition {
            byte: safe_byte,
            line: line_index + 1,
            column,
        }
    }

    #[must_use]
    pub fn span(&self, source: &str, start: usize, end: usize) -> SourceSpan {
        SourceSpan {
            start: self.position(source, start),
            end: self.position(source, end.max(start)),
            approximate: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_stale_edits() {
        let mut document = XmlDocument::new("<a/>", None);
        let revision = document
            .replace_all(Revision(0), "<b/>")
            .unwrap_or_default();
        assert_eq!(revision, Revision(1));
        assert!(matches!(
            document.replace_all(Revision(0), "<c/>"),
            Err(DocumentError::StaleRevision { .. })
        ));
    }

    #[test]
    fn maps_utf8_byte_positions_to_unicode_columns() {
        let source = "<á>\n  <b/>";
        let index = LineIndex::new(source);
        assert_eq!(index.position(source, 5).line, 2);
        assert_eq!(index.position(source, 7).column, 3);
    }
}
