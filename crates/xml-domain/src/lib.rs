use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Revision(pub u64);

impl Revision {
    #[must_use]
    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourcePosition {
    pub byte: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SourceSpan {
    pub start: SourcePosition,
    pub end: SourcePosition,
    #[serde(default)]
    pub approximate: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub source: String,
    pub rule: Option<String>,
    pub file: Option<String>,
    pub span: SourceSpan,
    pub revision: Revision,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OutlineEntry {
    pub name: String,
    pub depth: usize,
    pub span: SourceSpan,
}
