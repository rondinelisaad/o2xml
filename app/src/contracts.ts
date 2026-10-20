export interface SourcePosition {
  byte: number;
  line: number;
  column: number;
}

export interface SourceSpan {
  start: SourcePosition;
  end: SourcePosition;
  approximate: boolean;
}

export interface Diagnostic {
  severity: "error" | "warning" | "information" | "hint";
  message: string;
  source: string;
  rule: string | null;
  file: string | null;
  span: SourceSpan;
  revision: number;
}

export interface OutlineEntry {
  name: string;
  depth: number;
  span: SourceSpan;
}

export interface DocumentPayload {
  id: number;
  name: string;
  path: string | null;
  text: string;
  revision: number;
  dirty: boolean;
}

export interface ExternalChangePayload {
  status: "unchanged" | "modified" | "deleted";
}

export interface AnalysisPayload {
  revision: number;
  diagnostics: Diagnostic[];
  outline: OutlineEntry[];
  jatsInspection: JatsInspection | null;
  frameworkStages: FrameworkStage[];
}

export interface FrameworkStage {
  id: string;
  label: string;
  engine: string;
  status: "passed" | "failed" | "unavailable";
  diagnostics: Diagnostic[];
}

export interface JatsEntry {
  role: string;
  elementName: string;
  label: string;
  depth: number;
  span: SourceSpan;
}

export interface JatsInspection {
  isJats: boolean;
  entries: JatsEntry[];
}

export interface ProjectManifest {
  project: { name: string };
  xml: { catalog: string | null };
  validation: { framework: string | null };
  transform: {
    default: string | null;
    scenarios: TransformScenario[];
  };
}

export interface TransformScenario {
  name: string;
  engine: "libxslt" | "saxon-he";
  stylesheet: string;
  output: string;
  media_type: "xml" | "html" | "text";
}

export interface TransformPayload {
  revision: number;
  scenario: string;
  engine: string;
  mediaType: "xml" | "html" | "text";
  outputPath: string;
  bytesWritten: number;
  preview: string;
  previewTruncated: boolean;
}

export interface XQueryPayload {
  revision: number;
  engine: string;
  bytes: number;
  preview: string;
  truncated: boolean;
}

export interface XmlDiffHunk {
  kind: "added" | "removed" | "changed";
  left: SourceSpan | null;
  right: SourceSpan | null;
  leftAnchor: SourcePosition;
  rightAnchor: SourcePosition;
  leftTokens: number;
  rightTokens: number;
}

export interface MergePayload {
  revision: number;
  text: string;
}

export interface XmlDiffReport {
  semanticEqual: boolean;
  exactEqual: boolean;
  hunks: XmlDiffHunk[];
  coarse: boolean;
}

export interface DiffPayload {
  revision: number;
  otherName: string;
  otherText: string;
  report: XmlDiffReport;
}

export interface XmlGridRow {
  span: SourceSpan;
  cells: XmlGridCell[];
}

export interface XmlGridCell {
  value: string;
  editable: boolean;
}

export interface XmlGrid {
  rowElement: string;
  columns: string[];
  rows: XmlGridRow[];
  truncated: boolean;
}

export interface GridPayload {
  revision: number;
  grid: XmlGrid;
}

export interface GridEditPayload {
  revision: number;
  text: string;
  schemaValidated: boolean;
}

export type AuthorRole =
  | "document"
  | "section"
  | "heading"
  | "paragraph"
  | "list"
  | "listItem"
  | "caption"
  | "figure"
  | "table"
  | "code"
  | "emphasis"
  | "link"
  | "inline"
  | "container";

export interface AuthorTextRun {
  id: number;
  text: string;
  sourceSpan: SourceSpan;
  authorStart: number;
  authorEnd: number;
  editable: boolean;
}

export type AuthorContent =
  | { kind: "element"; element: AuthorElement }
  | { kind: "text"; run: AuthorTextRun };

export interface AuthorElement {
  name: string;
  label: string;
  role: AuthorRole;
  span: SourceSpan;
  content: AuthorContent[];
}

export interface AuthorDocument {
  root: AuthorElement;
  plainText: string;
}

export interface AuthorPayload {
  revision: number;
  document: AuthorDocument;
}

export interface AuthorPreviewPayload {
  revision: number;
  engine: string;
  html: string;
  stylesheet: string;
}

export interface AuthorEditPayload {
  revision: number;
  text: string;
  schemaValidated: boolean;
}

export interface ProjectPayload {
  path: string;
  manifest: ProjectManifest;
  files: string[];
}

export interface XPathNode {
  path: string;
  span: SourceSpan;
}

export type XPathValue =
  | { kind: "nodes"; value: { items: XPathNode[]; truncated: boolean } }
  | { kind: "boolean"; value: boolean }
  | { kind: "number"; value: string }
  | { kind: "string"; value: string };

export interface XPathReport {
  revision: number;
  value: XPathValue;
}

export interface DtdAttributeCompletion {
  name: string;
  required: boolean;
  values: string[];
  defaultValue: string | null;
  fixed: boolean;
}

export interface DtdCompletionPayload {
  revision: number;
  children: string[];
  attributes: DtdAttributeCompletion[];
  canClose: boolean;
  acceptsAny: boolean;
}

export interface ExpandedNamePayload {
  namespaceUri: string | null;
  localName: string;
}

export interface XsdAttributeCompletion {
  name: ExpandedNamePayload;
  required: boolean;
  values: string[];
  defaultValue: string | null;
  fixed: boolean;
}

export interface XsdCompletionPayload {
  revision: number;
  children: ExpandedNamePayload[];
  attributes: XsdAttributeCompletion[];
  canClose: boolean;
  acceptsAny: boolean;
}
