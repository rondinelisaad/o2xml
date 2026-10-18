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

export interface AnalysisPayload {
  revision: number;
  diagnostics: Diagnostic[];
  outline: OutlineEntry[];
  jatsInspection: JatsInspection | null;
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
  transform: { default: string | null };
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
