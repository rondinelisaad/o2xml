import "./styles.css";

import {
  autocompletion,
  closeBrackets,
  closeBracketsKeymap,
  completionKeymap,
  type CompletionContext,
  type CompletionResult,
} from "@codemirror/autocomplete";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { foldGutter, foldKeymap, indentOnInput, syntaxTree } from "@codemirror/language";
import { xml } from "@codemirror/lang-xml";
import { searchKeymap } from "@codemirror/search";
import { Compartment, EditorState, type Extension } from "@codemirror/state";
import { oneDark } from "@codemirror/theme-one-dark";
import {
  EditorView,
  highlightActiveLine,
  highlightActiveLineGutter,
  keymap,
  lineNumbers,
} from "@codemirror/view";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";

import type {
  AnalysisPayload,
  AuthorContent,
  AuthorEditPayload,
  AuthorElement,
  AuthorPayload,
  AuthorTextRun,
  Diagnostic,
  DiffPayload,
  DocumentPayload,
  DtdCompletionPayload,
  ExpandedNamePayload,
  FrameworkStage,
  GridEditPayload,
  GridPayload,
  JatsInspection,
  MergePayload,
  OutlineEntry,
  ProjectPayload,
  SourceSpan,
  TransformPayload,
  XPathReport,
  XsdCompletionPayload,
  XQueryPayload,
} from "./contracts";

interface OpenDocument extends DocumentPayload {
  viewState: EditorState;
  syncPromise?: Promise<boolean>;
}

const app = document.querySelector<HTMLDivElement>("#app");
if (!app) throw new Error("application root is missing");

app.innerHTML = `
  <div class="shell">
    <header class="menubar">
      <strong class="brand">O2XML</strong>
      <button data-action="new">New</button>
      <button data-action="open">Open…</button>
      <button data-action="save">Save</button>
      <button data-action="project">Project…</button>
      <button data-action="format">Pretty Print</button>
      <button data-action="validate">Validate</button>
      <select id="transform-scenario" class="transform-select" aria-label="Transformation scenario" disabled>
        <option>No scenarios</option>
      </select>
      <button data-action="transform" disabled>Transform</button>
      <button data-action="diff">Compare…</button>
      <button data-action="author">Author Mode</button>
      <button data-action="xpath">XPath</button>
      <span class="spacer"></span>
      <button data-action="theme" aria-label="Toggle color theme">Theme</button>
    </header>
    <main class="workspace">
      <aside class="sidebar">
        <section>
          <h2>Project</h2>
          <div id="project-title" class="muted">No project open</div>
          <div id="project-files" class="tree"></div>
        </section>
        <section class="outline-section">
          <h2>Outline</h2>
          <div id="outline" class="tree"></div>
        </section>
        <section class="outline-section">
          <h2>JATS Inspector</h2>
          <div id="jats-inspector" class="tree"><div class="empty">Not a JATS article</div></div>
        </section>
        <section class="outline-section">
          <h2>Validation Framework</h2>
          <div id="framework-stages" class="tree"><div class="empty">No framework active</div></div>
        </section>
      </aside>
      <section class="editor-column">
        <div id="tabs" class="tabs"></div>
        <div id="editor" class="editor"></div>
        <div id="author-view" class="author-view" hidden>
          <div class="author-toolbar">
            <strong>Structured Author Mode</strong>
            <span>XML remains the source of truth · click an element label to inspect source</span>
          </div>
          <div id="author-document" class="author-document" aria-live="polite"></div>
        </div>
      </section>
    </main>
    <section class="bottom-panel">
      <div class="panel-tabs">
        <button class="active" data-panel="problems">Problems <span id="problem-count">0</span></button>
        <button data-panel="output">Output</button>
        <button data-panel="xpath">XPath</button>
        <button data-panel="xquery">XQuery</button>
        <button data-panel="grid">XML Grid</button>
        <span id="status" class="status">Ready</span>
      </div>
      <div id="problems" class="panel-content active problems" data-panel-content="problems" aria-live="polite"></div>
      <div id="output-panel" class="panel-content" data-panel-content="output"><div class="empty">No output</div></div>
      <div id="xpath-panel" class="panel-content xpath-panel" data-panel-content="xpath">
        <div class="xpath-controls">
          <input id="xpath-expression" type="text" value="//*" aria-label="XPath expression" spellcheck="false" />
          <button id="xpath-run" type="button">Run</button>
        </div>
        <div class="xpath-body">
          <label class="xpath-namespaces">Namespaces <span>one prefix=URI per line</span>
            <textarea id="xpath-namespaces" spellcheck="false" placeholder="j=http://jats.nlm.nih.gov"></textarea>
          </label>
          <div id="xpath-results" class="xpath-results" aria-live="polite"><div class="empty">Run an XPath expression</div></div>
        </div>
      </div>
      <div id="xquery-panel" class="panel-content xquery-panel" data-panel-content="xquery">
        <div class="xquery-controls">
          <textarea id="xquery-expression" spellcheck="false" aria-label="XQuery expression">for $title in //article-title return $title</textarea>
          <button id="xquery-run" type="button">Run</button>
        </div>
        <div id="xquery-results" class="xquery-results" aria-live="polite"><div class="empty">Run a self-contained XQuery 3.1 expression</div></div>
      </div>
      <div id="grid-panel" class="panel-content grid-panel" data-panel-content="grid">
        <div class="grid-controls">
          <input id="grid-row-element" type="text" aria-label="Repeated row element" placeholder="Auto-detect, or enter contrib" spellcheck="false" />
          <button id="grid-run" type="button">Build Grid</button>
          <span id="grid-summary" class="muted">Editable scalar text; click a row to navigate</span>
        </div>
        <div id="grid-results" class="grid-results" aria-live="polite"><div class="empty">Build a grid from repeated sibling elements</div></div>
      </div>
    </section>
  </div>
  <div id="diff-dialog" class="diff-dialog" hidden>
    <div class="diff-window" role="dialog" aria-modal="true" aria-labelledby="diff-title">
      <header class="diff-header">
        <strong id="diff-title">XML-aware comparison</strong>
        <span id="diff-summary" class="muted"></span>
        <label><input id="diff-ignore-whitespace" type="checkbox" /> Ignore formatting whitespace</label>
        <button id="diff-use-right" type="button">Use right as current</button>
        <button id="diff-close" type="button">Close</button>
      </header>
      <div id="diff-hunks" class="diff-hunks"></div>
      <div class="diff-panes">
        <section><h2 id="diff-left-title">Current</h2><div id="diff-left" class="diff-code"></div></section>
        <section><h2 id="diff-right-title">Selected</h2><div id="diff-right" class="diff-code"></div></section>
      </div>
    </div>
  </div>
`;

const editorHost = requiredElement<HTMLDivElement>("#editor");
const tabsHost = requiredElement<HTMLDivElement>("#tabs");
const outlineHost = requiredElement<HTMLDivElement>("#outline");
const jatsInspectorHost = requiredElement<HTMLDivElement>("#jats-inspector");
const frameworkStagesHost = requiredElement<HTMLDivElement>("#framework-stages");
const problemsHost = requiredElement<HTMLDivElement>("#problems");
const statusHost = requiredElement<HTMLSpanElement>("#status");
const problemCount = requiredElement<HTMLSpanElement>("#problem-count");
const projectFiles = requiredElement<HTMLDivElement>("#project-files");
const projectTitle = requiredElement<HTMLDivElement>("#project-title");
const xpathExpression = requiredElement<HTMLInputElement>("#xpath-expression");
const xpathNamespaces = requiredElement<HTMLTextAreaElement>("#xpath-namespaces");
const xpathResults = requiredElement<HTMLDivElement>("#xpath-results");
const xpathRun = requiredElement<HTMLButtonElement>("#xpath-run");
const xqueryExpression = requiredElement<HTMLTextAreaElement>("#xquery-expression");
const xqueryResults = requiredElement<HTMLDivElement>("#xquery-results");
const xqueryRun = requiredElement<HTMLButtonElement>("#xquery-run");
const gridRowElement = requiredElement<HTMLInputElement>("#grid-row-element");
const gridResults = requiredElement<HTMLDivElement>("#grid-results");
const gridRun = requiredElement<HTMLButtonElement>("#grid-run");
const gridSummary = requiredElement<HTMLSpanElement>("#grid-summary");
const authorView = requiredElement<HTMLDivElement>("#author-view");
const authorDocumentHost = requiredElement<HTMLDivElement>("#author-document");
const authorButton = requiredElement<HTMLButtonElement>("[data-action='author']");
const transformScenario = requiredElement<HTMLSelectElement>("#transform-scenario");
const transformButton = requiredElement<HTMLButtonElement>("[data-action='transform']");
const outputPanel = requiredElement<HTMLDivElement>("#output-panel");
const diffDialog = requiredElement<HTMLDivElement>("#diff-dialog");
const diffSummary = requiredElement<HTMLSpanElement>("#diff-summary");
const diffHunks = requiredElement<HTMLDivElement>("#diff-hunks");
const diffLeft = requiredElement<HTMLDivElement>("#diff-left");
const diffRight = requiredElement<HTMLDivElement>("#diff-right");
const diffLeftTitle = requiredElement<HTMLHeadingElement>("#diff-left-title");
const diffRightTitle = requiredElement<HTMLHeadingElement>("#diff-right-title");
const diffIgnoreWhitespace = requiredElement<HTMLInputElement>("#diff-ignore-whitespace");
const diffUseRight = requiredElement<HTMLButtonElement>("#diff-use-right");

const theme = new Compartment();
let dark = window.matchMedia("(prefers-color-scheme: dark)").matches;
let documents: OpenDocument[] = [];
let activeId: number | null = null;
let analysisTimer: number | undefined;
let suppressChanges = false;
let diffOtherPath: string | null = null;
let activeDiff: DiffPayload | null = null;
let authorActive = false;
let activeAuthor: AuthorPayload | null = null;

const editor = new EditorView({
  parent: editorHost,
  state: createEditorState(""),
});

function requiredElement<T extends Element>(selector: string): T {
  const element = document.querySelector<T>(selector);
  if (!element) throw new Error(`required element not found: ${selector}`);
  return element;
}

function baseExtensions(): Extension[] {
  return [
    lineNumbers(),
    highlightActiveLineGutter(),
    highlightActiveLine(),
    history(),
    foldGutter(),
    indentOnInput(),
    closeBrackets(),
    autocompletion({ override: [schemaCompletionSource] }),
    xml(),
    keymap.of([
      ...closeBracketsKeymap,
      ...defaultKeymap,
      ...searchKeymap,
      ...historyKeymap,
      ...foldKeymap,
      ...completionKeymap,
      indentWithTab,
    ]),
    EditorView.lineWrapping,
    EditorView.updateListener.of((update) => {
      if (!update.docChanged || suppressChanges) return;
      const document = activeDocument();
      if (!document) return;
      document.viewState = update.state;
      document.dirty = true;
      renderTabs();
      scheduleAnalysis(document);
    }),
    theme.of(dark ? oneDark : []),
  ];
}

type XmlSyntaxNode = ReturnType<typeof syntaxTree>["topNode"];

function tagName(node: XmlSyntaxNode, state: EditorState): string {
  const tag = node.name === "Element" ? node.firstChild : node;
  const name = tag?.getChild("TagName");
  return name ? state.doc.sliceString(name.from, name.to) : "";
}

function nearestNode(node: XmlSyntaxNode, names: ReadonlySet<string>): XmlSyntaxNode | null {
  for (let current: XmlSyntaxNode | null = node; current; current = current.parent) {
    if (names.has(current.name)) return current;
  }
  return null;
}

function parentElement(node: XmlSyntaxNode | null): XmlSyntaxNode | null {
  for (let current = node?.parent ?? null; current; current = current.parent) {
    if (current.name === "Element") return current;
  }
  return null;
}

function directChildNames(element: XmlSyntaxNode, before: number, state: EditorState): string[] {
  const names: string[] = [];
  for (let child = element.firstChild; child; child = child.nextSibling) {
    if (child.from >= before) break;
    if (child.name === "Element" || child.name === "SelfClosingTag") {
      const name = tagName(child, state);
      if (name) names.push(name);
    }
  }
  return names;
}

function attributeNames(tag: XmlSyntaxNode, state: EditorState): string[] {
  return tag.getChildren("Attribute").flatMap((attribute) => {
    const name = attribute.getChild("AttributeName");
    return name ? [state.doc.sliceString(name.from, name.to)] : [];
  });
}

function namespaceBindings(scope: XmlSyntaxNode, state: EditorState): Map<string, string> {
  const bindings = new Map<string, string>();
  const visited = new Set<number>();
  for (let current: XmlSyntaxNode | null = scope; current; current = current.parent) {
    const tag = current.name === "OpenTag" || current.name === "SelfClosingTag"
      ? current
      : current.name === "Element"
        ? current.firstChild
        : null;
    if (!tag || visited.has(tag.from)) continue;
    visited.add(tag.from);
    for (const attribute of tag.getChildren("Attribute")) {
      const nameNode = attribute.getChild("AttributeName");
      const valueNode = attribute.getChild("AttributeValue");
      if (!nameNode || !valueNode) continue;
      const name = state.doc.sliceString(nameNode.from, nameNode.to);
      const prefix = name === "xmlns" ? "" : name.startsWith("xmlns:") ? name.slice(6) : null;
      if (prefix === null || bindings.has(prefix)) continue;
      const quoted = state.doc.sliceString(valueNode.from, valueNode.to);
      bindings.set(prefix, quoted.replace(/^['"]|['"]$/g, ""));
    }
  }
  return bindings;
}

function expandedName(
  lexical: string,
  bindings: Map<string, string>,
  attribute: boolean,
): ExpandedNamePayload {
  const separator = lexical.indexOf(":");
  if (separator >= 0) {
    const prefix = lexical.slice(0, separator);
    return {
      namespaceUri: bindings.get(prefix) ?? null,
      localName: lexical.slice(separator + 1),
    };
  }
  return {
    namespaceUri: attribute ? null : bindings.get("") ?? null,
    localName: lexical,
  };
}

function lexicalName(
  name: ExpandedNamePayload,
  bindings: Map<string, string>,
  attribute: boolean,
): string {
  if (!name.namespaceUri) return name.localName;
  for (const [prefix, namespace] of bindings) {
    if (namespace === name.namespaceUri && (!attribute || prefix !== "")) {
      return prefix ? `${prefix}:${name.localName}` : name.localName;
    }
  }
  return name.localName;
}

async function requestSchemaCompletion(
  document: OpenDocument,
  parent: string,
  precedingChildren: string[],
  presentAttributes: string[],
  state: EditorState,
  scope: XmlSyntaxNode,
): Promise<DtdCompletionPayload | null> {
  if (!(await synchronize(document))) return null;
  try {
    const source = state.doc.toString();
    const hasXsdAssociation = source.includes("http://www.w3.org/2001/XMLSchema-instance")
      && /(?:schemaLocation|noNamespaceSchemaLocation)\s*=/.test(source);
    if (hasXsdAssociation) {
      const bindings = namespaceBindings(scope, state);
      const result = await invoke<XsdCompletionPayload>("complete_xsd", {
        id: document.id,
        context: {
          parent: expandedName(parent, bindings, false),
          precedingChildren: precedingChildren.map((name) => expandedName(name, bindings, false)),
          presentAttributes: presentAttributes.map((name) => expandedName(name, bindings, true)),
        },
      });
      if (document.id !== activeId || result.revision !== document.revision) return null;
      return {
        revision: result.revision,
        children: result.children.map((name) => lexicalName(name, bindings, false)),
        attributes: result.attributes.map((attribute) => ({
          ...attribute,
          name: lexicalName(attribute.name, bindings, true),
        })),
        canClose: result.canClose,
        acceptsAny: result.acceptsAny,
      };
    }
    const result = await invoke<DtdCompletionPayload>("complete_dtd", {
      id: document.id,
      parent,
      precedingChildren,
      presentAttributes,
    });
    return document.id === activeId && result.revision === document.revision ? result : null;
  } catch {
    return null;
  }
}

async function schemaCompletionSource(
  context: CompletionContext,
): Promise<CompletionResult | null> {
  const document = activeDocument();
  if (!document) return null;
  const tree = syntaxTree(context.state);
  const resolved = tree.resolveInner(Math.max(0, context.pos - 1), 1);
  const tag = nearestNode(resolved, new Set(["OpenTag", "SelfClosingTag"]));
  const word = context.matchBefore(/[:\-.\w\u00b7-\uffff]*/u);

  if (tag) {
    const name = tagName(tag, context.state);
    const nameNode = tag.getChild("TagName");
    const valueNode = nearestNode(resolved, new Set(["AttributeValue"]));
    if (name && valueNode) {
      const attribute = nearestNode(valueNode, new Set(["Attribute"]));
      const attributeNameNode = attribute?.getChild("AttributeName");
      const attributeName = attributeNameNode
        ? context.state.doc.sliceString(attributeNameNode.from, attributeNameNode.to)
        : "";
      const present = attributeNames(tag, context.state).filter((item) => item !== attributeName);
      const result = await requestSchemaCompletion(
        document, name, [], present, context.state, tag,
      );
      const specification = result?.attributes.find((item) => item.name === attributeName);
      if (!specification || specification.values.length === 0) return null;
      return {
        from: valueNode.from,
        to: valueNode.to,
        options: specification.values.map((value) => ({
          label: `"${value}"`,
          type: "constant",
        })),
      };
    }
    if (name && nameNode && nameNode.to < context.pos && word) {
      const result = await requestSchemaCompletion(
        document,
        name,
        [],
        attributeNames(tag, context.state),
        context.state,
        tag,
      );
      if (!result) return null;
      return {
        from: word.from,
        options: result.attributes.map((attribute) => ({
          label: attribute.name,
          type: "property",
          detail: attribute.required ? "required" : attribute.fixed ? "fixed" : undefined,
          apply: `${attribute.name}="${attribute.defaultValue ?? ""}"`,
        })),
      };
    }
  }

  const closing = context.matchBefore(/<\/[:\-.\w\u00b7-\uffff]*/u);
  if (closing) {
    const element = nearestNode(resolved, new Set(["Element"]));
    const name = element ? tagName(element, context.state) : "";
    return name
      ? {
          from: closing.from + 2,
          options: [{ label: `${name}>`, type: "type", boost: 2 }],
        }
      : null;
  }

  const opening = context.matchBefore(/<[:\-.\w\u00b7-\uffff]*/u);
  if (!opening || (!context.explicit && opening.from === opening.to)) return null;
  let element = nearestNode(resolved, new Set(["Element"]));
  if (element) {
    const openTag = element.firstChild;
    if (openTag && openTag.from <= context.pos && openTag.to >= context.pos) {
      element = parentElement(element);
    }
  }
  if (!element) return null;
  const name = tagName(element, context.state);
  if (!name) return null;
  const result = await requestSchemaCompletion(
    document,
    name,
    directChildNames(element, context.pos, context.state),
    [],
    context.state,
    element,
  );
  if (!result) return null;
  return {
    from: opening.from + 1,
    options: result.children.map((child) => ({ label: child, type: "type" })),
  };
}

function createEditorState(text: string): EditorState {
  return EditorState.create({ doc: text, extensions: baseExtensions() });
}

function activeDocument(): OpenDocument | undefined {
  return documents.find((document) => document.id === activeId);
}

function activateDocument(id: number): void {
  const current = activeDocument();
  if (current) current.viewState = editor.state;
  const next = documents.find((document) => document.id === id);
  if (!next) return;
  showSourceMode();
  activeId = id;
  suppressChanges = true;
  editor.setState(next.viewState);
  suppressChanges = false;
  renderTabs();
  void analyze(next);
}

function addDocument(payload: DocumentPayload): void {
  const document: OpenDocument = {
    ...payload,
    viewState: createEditorState(payload.text),
  };
  documents.push(document);
  activateDocument(document.id);
}

function renderTabs(): void {
  const nodes = documents.map((item) => {
    const button = window.document.createElement("button");
    button.className = item.id === activeId ? "active" : "";
    button.textContent = `${item.name}${item.dirty ? " •" : ""}`;
    button.title = item.path ?? "Unsaved document";
    button.addEventListener("click", () => activateDocument(item.id));
    return button;
  });
  tabsHost.replaceChildren(...nodes);
}

function setStatus(message: string, error = false): void {
  statusHost.textContent = message;
  statusHost.classList.toggle("error", error);
}

function scheduleAnalysis(document: OpenDocument): void {
  if (analysisTimer !== undefined) window.clearTimeout(analysisTimer);
  analysisTimer = window.setTimeout(() => void synchronizeAndAnalyze(document), 300);
}

async function synchronizeAndAnalyze(document: OpenDocument): Promise<void> {
  if (activeId !== document.id) return;
  if (await synchronize(document)) await analyze(document);
}

async function synchronize(document: OpenDocument): Promise<boolean> {
  if (document.syncPromise) return document.syncPromise;
  if (activeId !== document.id) return false;
  const text = editor.state.doc.toString();
  if (text === document.text) return true;

  const operation = (async (): Promise<boolean> => {
    try {
      const revision = await invoke<number>("update_document", {
        id: document.id,
        baseRevision: document.revision,
        text,
      });
      document.revision = revision;
      document.text = text;
      return true;
    } catch (error) {
      setStatus(String(error), true);
      return false;
    }
  })();
  document.syncPromise = operation;

  try {
    const succeeded = await operation;
    if (!succeeded) return false;
  } finally {
    document.syncPromise = undefined;
  }

  if (activeId === document.id && editor.state.doc.toString() !== document.text) {
    return synchronize(document);
  }
  return true;
}

async function analyze(document: OpenDocument): Promise<void> {
  setStatus("Validating…");
  try {
    const result = await invoke<AnalysisPayload>("analyze_document", { id: document.id });
    if (document.id !== activeId || result.revision !== document.revision) return;
    renderProblems(result.diagnostics);
    renderOutline(result.outline);
    renderJatsInspection(result.jatsInspection);
    renderFrameworkStages(result.frameworkStages);
    setStatus(result.diagnostics.length === 0 ? "Well-formed" : "Validation failed", result.diagnostics.length > 0);
  } catch (error) {
    setStatus(String(error), true);
  }
}

function renderProblems(diagnostics: Diagnostic[]): void {
  problemCount.textContent = String(diagnostics.length);
  if (diagnostics.length === 0) {
    const empty = window.document.createElement("div");
    empty.className = "empty";
    empty.textContent = "No well-formedness problems";
    problemsHost.replaceChildren(empty);
    return;
  }
  const rows = diagnostics.map((diagnostic) => {
    const row = window.document.createElement("button");
    row.className = `problem ${diagnostic.severity}`;
    row.textContent = `${diagnostic.severity.toUpperCase()}  [${diagnostic.source}] ${diagnostic.message}  (${diagnostic.span.start.line}:${diagnostic.span.start.column})`;
    row.addEventListener("click", () => goToByte(diagnostic.span.start.byte));
    return row;
  });
  problemsHost.replaceChildren(...rows);
}

function renderOutline(entries: OutlineEntry[]): void {
  const rows = entries.map((entry) => {
    const button = window.document.createElement("button");
    button.className = "tree-row";
    button.style.paddingInlineStart = `${12 + entry.depth * 16}px`;
    button.textContent = entry.name;
    button.addEventListener("click", () => goToByte(entry.span.start.byte));
    return button;
  });
  outlineHost.replaceChildren(...rows);
}

function renderJatsInspection(inspection: JatsInspection | null): void {
  if (!inspection) {
    const empty = window.document.createElement("div");
    empty.className = "empty";
    empty.textContent = "Not a JATS article";
    jatsInspectorHost.replaceChildren(empty);
    return;
  }
  const rows = inspection.entries.map((entry) => {
    const button = window.document.createElement("button");
    button.className = "tree-row";
    button.style.paddingInlineStart = `${12 + entry.depth * 16}px`;
    button.textContent = entry.label;
    button.title = entry.elementName;
    button.addEventListener("click", () => goToByte(entry.span.start.byte));
    return button;
  });
  jatsInspectorHost.replaceChildren(...rows);
}

function renderFrameworkStages(stages: FrameworkStage[]): void {
  if (stages.length === 0) {
    const empty = window.document.createElement("div");
    empty.className = "empty";
    empty.textContent = "No framework active";
    frameworkStagesHost.replaceChildren(empty);
    return;
  }
  const rows = stages.map((stage) => {
    const row = window.document.createElement("div");
    row.className = `tree-row static ${stage.status}`;
    row.textContent = `${stage.label}: ${stage.status}`;
    row.title = `${stage.engine} · ${stage.diagnostics.length} diagnostic(s)`;
    return row;
  });
  frameworkStagesHost.replaceChildren(...rows);
}

function byteToUtf16(text: string, targetByte: number): number {
  const encoder = new TextEncoder();
  let bytes = 0;
  let utf16 = 0;
  for (const character of text) {
    const width = encoder.encode(character).length;
    if (bytes + width > targetByte) break;
    bytes += width;
    utf16 += character.length;
  }
  return utf16;
}

function utf16ToByte(text: string, targetUtf16: number): number {
  return new TextEncoder().encode(text.slice(0, targetUtf16)).length;
}

function goToByte(byte: number): void {
  const text = editor.state.doc.toString();
  const position = Math.min(byteToUtf16(text, byte), editor.state.doc.length);
  editor.dispatch({ selection: { anchor: position }, scrollIntoView: true });
  editor.focus();
}

function showSourceMode(): void {
  authorActive = false;
  activeAuthor = null;
  authorView.hidden = true;
  editorHost.hidden = false;
  authorButton.textContent = "Author Mode";
}

function showSourceAt(byte: number): void {
  showSourceMode();
  goToByte(byte);
}

async function toggleAuthorMode(): Promise<void> {
  if (authorActive) {
    showSourceMode();
    setStatus("XML source mode");
    return;
  }
  await runAuthorMode();
}

async function runAuthorMode(): Promise<void> {
  const document = activeDocument();
  if (!document) return;
  if (document.dirty && !(await synchronize(document))) return;
  setStatus("Building structured Author Mode…");
  authorButton.disabled = true;
  try {
    const payload = await invoke<AuthorPayload>("inspect_author", { id: document.id });
    if (document.id !== activeId || payload.revision !== document.revision) return;
    activeAuthor = payload;
    authorActive = true;
    renderAuthor(payload);
    editorHost.hidden = true;
    authorView.hidden = false;
    authorButton.textContent = "XML Source";
    setStatus("Author Mode ready");
  } catch (error) {
    showSourceMode();
    setStatus(String(error), true);
  } finally {
    authorButton.disabled = false;
  }
}

function renderAuthor(payload: AuthorPayload): void {
  const sourceByte = utf16ToByte(
    editor.state.doc.toString(),
    editor.state.selection.main.head,
  );
  const root = renderAuthorElement(payload, payload.document.root, 0);
  authorDocumentHost.replaceChildren(root);
  const mapped = Array.from(authorDocumentHost.querySelectorAll<HTMLElement>("[data-source-start]"))
    .find((node) => {
      const start = Number(node.dataset.sourceStart);
      const end = Number(node.dataset.sourceEnd);
      return sourceByte >= start && sourceByte < end;
    });
  if (mapped) {
    mapped.classList.add("source-mapped");
    mapped.scrollIntoView({ block: "center" });
  }
}

function renderAuthorElement(
  payload: AuthorPayload,
  element: AuthorElement,
  depth: number,
): HTMLElement {
  const inline = element.role === "emphasis" || element.role === "link" || element.role === "inline";
  const node = window.document.createElement(inline ? "span" : "section");
  node.className = `author-element author-role-${element.role}`;
  node.dataset.depth = String(depth);
  node.title = inline ? `${element.label} · line ${element.span.start.line}` : "";

  if (!inline) {
    const label = window.document.createElement("button");
    label.className = "author-element-label";
    label.type = "button";
    label.textContent = element.label;
    label.title = `Inspect <${element.label}> at line ${element.span.start.line}`;
    label.addEventListener("click", () => showSourceAt(element.span.start.byte));
    node.append(label);
  }

  const body = window.document.createElement(inline ? "span" : "div");
  body.className = "author-element-content";
  for (const content of element.content) {
    body.append(renderAuthorContent(payload, content, depth + 1));
  }
  node.append(body);
  return node;
}

function renderAuthorContent(
  payload: AuthorPayload,
  content: AuthorContent,
  depth: number,
): Node {
  if (content.kind === "element") {
    return renderAuthorElement(payload, content.element, depth);
  }
  return renderAuthorText(payload, content.run);
}

function renderAuthorText(payload: AuthorPayload, run: AuthorTextRun): HTMLElement {
  const text = window.document.createElement("span");
  text.className = run.editable ? "author-text editable" : "author-text readonly";
  text.dataset.sourceStart = String(run.sourceSpan.start.byte);
  text.dataset.sourceEnd = String(run.sourceSpan.end.byte);
  text.textContent = run.text;
  text.title = run.editable
    ? `Editable XML text · line ${run.sourceSpan.start.line}`
    : "Read-only lexical content (CDATA or entity reference)";
  if (!run.editable) return text;

  text.contentEditable = "plaintext-only";
  text.spellcheck = true;
  text.setAttribute("role", "textbox");
  text.setAttribute("aria-label", `XML text on line ${run.sourceSpan.start.line}`);
  text.addEventListener("keydown", (event) => {
    if (event.key === "Enter") {
      event.preventDefault();
      text.blur();
    }
    if (event.key === "Escape") {
      event.preventDefault();
      text.textContent = run.text;
      text.blur();
    }
  });
  text.addEventListener("paste", (event) => {
    event.preventDefault();
    insertPlainText(event.clipboardData?.getData("text/plain") ?? "");
  });
  text.addEventListener("blur", () => {
    const value = text.textContent ?? "";
    if (value !== run.text) void editAuthorText(payload, run, value, text);
  });
  return text;
}

function insertPlainText(value: string): void {
  const selection = window.getSelection();
  if (!selection || selection.rangeCount === 0) return;
  const range = selection.getRangeAt(0);
  range.deleteContents();
  const node = window.document.createTextNode(value);
  range.insertNode(node);
  range.setStartAfter(node);
  range.collapse(true);
  selection.removeAllRanges();
  selection.addRange(range);
}

async function editAuthorText(
  authorPayload: AuthorPayload,
  run: AuthorTextRun,
  value: string,
  textNode: HTMLElement,
): Promise<void> {
  const document = activeDocument();
  if (
    !document
    || !authorActive
    || activeAuthor !== authorPayload
    || authorPayload.revision !== document.revision
    || editor.state.doc.toString() !== document.text
  ) {
    textNode.textContent = run.text;
    setStatus("Author Mode is stale; rebuild it", true);
    return;
  }
  textNode.contentEditable = "false";
  setStatus("Validating Author Mode edit…");
  try {
    const payload = await invoke<AuthorEditPayload>("edit_author_text", {
      id: document.id,
      runId: run.id,
      value,
    });
    if (
      document.id !== activeId
      || !authorActive
      || payload.revision !== document.revision
      || editor.state.doc.toString() !== document.text
    ) {
      showSourceMode();
      setStatus("Document changed while the Author Mode edit was being validated", true);
      return;
    }
    editor.dispatch({
      changes: { from: 0, to: editor.state.doc.length, insert: payload.text },
    });
    const synchronized = await synchronize(document);
    if (synchronized) {
      setStatus(
        payload.schemaValidated
          ? "Schema-validated Author edit applied; Undo is available"
          : "Well-formed Author edit applied; Undo is available",
      );
      await runAuthorMode();
    }
  } catch (error) {
    textNode.textContent = run.text;
    textNode.contentEditable = "plaintext-only";
    setStatus(String(error), true);
  }
}

function selectBottomPanel(name: "problems" | "output" | "xpath" | "xquery" | "grid"): void {
  document.querySelectorAll<HTMLButtonElement>("[data-panel]").forEach((button) => {
    button.classList.toggle("active", button.dataset.panel === name);
  });
  document.querySelectorAll<HTMLElement>("[data-panel-content]").forEach((panel) => {
    panel.classList.toggle("active", panel.dataset.panelContent === name);
  });
  if (name === "xpath") xpathExpression.focus();
  if (name === "xquery") xqueryExpression.focus();
  if (name === "grid") gridRowElement.focus();
}

async function runGrid(): Promise<void> {
  const document = activeDocument();
  if (!document) return;
  if (document.dirty && !(await synchronize(document))) return;
  selectBottomPanel("grid");
  setStatus("Building XML Grid…");
  gridRun.disabled = true;
  try {
    const rowElement = gridRowElement.value.trim() || null;
    const payload = await invoke<GridPayload>("inspect_grid", {
      id: document.id,
      rowElement,
    });
    if (document.id !== activeId || payload.revision !== document.revision) return;
    renderGrid(payload);
    setStatus("XML Grid ready");
  } catch (error) {
    gridResults.replaceChildren(messageElement(String(error), "empty error-text"));
    gridSummary.textContent = "Editable scalar text; click a row to navigate";
    setStatus(String(error), true);
  } finally {
    gridRun.disabled = false;
  }
}

function renderGrid(payload: GridPayload): void {
  const { grid } = payload;
  gridSummary.textContent = `${grid.rows.length} ${grid.rowElement} row${grid.rows.length === 1 ? "" : "s"}${grid.truncated ? " (limited)" : ""}`;
  const table = window.document.createElement("table");
  table.className = "xml-grid-table";
  const head = window.document.createElement("thead");
  const headerRow = window.document.createElement("tr");
  const lineHeader = window.document.createElement("th");
  lineHeader.textContent = "Line";
  headerRow.append(lineHeader);
  for (const column of grid.columns) {
    const cell = window.document.createElement("th");
    cell.textContent = column;
    headerRow.append(cell);
  }
  head.append(headerRow);
  const body = window.document.createElement("tbody");
  for (const [rowIndex, row] of grid.rows.entries()) {
    const tableRow = window.document.createElement("tr");
    tableRow.tabIndex = 0;
    tableRow.title = `Go to line ${row.span.start.line}`;
    const navigate = (): void => goToByte(row.span.start.byte);
    tableRow.addEventListener("click", navigate);
    tableRow.addEventListener("keydown", (event) => {
      if (event.key === "Enter" || event.key === " ") navigate();
    });
    const line = window.document.createElement("td");
    line.className = "grid-line";
    line.textContent = String(row.span.start.line);
    tableRow.append(line);
    for (const [columnIndex, gridCell] of row.cells.entries()) {
      const column = grid.columns[columnIndex];
      if (column === undefined) continue;
      const cell = window.document.createElement("td");
      if (gridCell.editable) {
        const input = window.document.createElement("input");
        input.className = "grid-cell-input";
        input.value = gridCell.value;
        input.setAttribute("aria-label", `${column} row ${rowIndex + 1}`);
        input.addEventListener("click", (event) => event.stopPropagation());
        input.addEventListener("keydown", (event) => {
          event.stopPropagation();
          if (event.key === "Enter") input.blur();
          if (event.key === "Escape") {
            input.value = gridCell.value;
            input.blur();
          }
        });
        input.addEventListener("blur", () => {
          if (input.value !== gridCell.value) {
            void editGridCell(payload, rowIndex, column, input);
          }
        });
        cell.append(input);
      } else {
        cell.textContent = gridCell.value;
        cell.title = "Read-only: attributes, repeated values, and mixed XML are not edited in Grid";
      }
      tableRow.append(cell);
    }
    body.append(tableRow);
  }
  table.append(head, body);
  gridResults.replaceChildren(table);
}

async function editGridCell(
  gridPayload: GridPayload,
  rowIndex: number,
  column: string,
  input: HTMLInputElement,
): Promise<void> {
  const document = activeDocument();
  if (
    !document
    || gridPayload.revision !== document.revision
    || editor.state.doc.toString() !== document.text
  ) {
    setStatus("XML Grid is stale; build it again", true);
    return;
  }
  input.disabled = true;
  setStatus("Validating XML Grid edit…");
  try {
    const payload = await invoke<GridEditPayload>("edit_grid_cell", {
      id: document.id,
      rowElement: gridPayload.grid.rowElement,
      rowIndex,
      column,
      value: input.value,
    });
    if (
      document.id !== activeId
      || payload.revision !== document.revision
      || editor.state.doc.toString() !== document.text
    ) {
      input.disabled = false;
      setStatus("Document changed while the Grid edit was being validated", true);
      return;
    }
    editor.dispatch({
      changes: { from: 0, to: editor.state.doc.length, insert: payload.text },
    });
    const synchronized = await synchronize(document);
    if (synchronized) {
      setStatus(
        payload.schemaValidated
          ? "Schema-validated Grid edit applied; Undo is available"
          : "Well-formed Grid edit applied; Undo is available",
      );
      await runGrid();
    }
  } catch (error) {
    input.disabled = false;
    setStatus(String(error), true);
  }
}

function parseNamespaces(source: string): Record<string, string> {
  const namespaces: Record<string, string> = {};
  for (const [index, rawLine] of source.split(/\r?\n/u).entries()) {
    const line = rawLine.trim();
    if (!line) continue;
    const separator = line.indexOf("=");
    if (separator <= 0 || separator === line.length - 1) {
      throw new Error(`Invalid namespace binding on line ${index + 1}`);
    }
    const prefix = line.slice(0, separator).trim();
    const uri = line.slice(separator + 1).trim();
    if (!/^[A-Za-z_][A-Za-z0-9._-]*$/u.test(prefix)) {
      throw new Error(`Invalid namespace prefix on line ${index + 1}`);
    }
    namespaces[prefix] = uri;
  }
  return namespaces;
}

async function runXPath(): Promise<void> {
  const document = activeDocument();
  if (!document) return;
  const expression = xpathExpression.value.trim();
  if (!expression) {
    setStatus("Enter an XPath expression", true);
    return;
  }
  let namespaces: Record<string, string>;
  try {
    namespaces = parseNamespaces(xpathNamespaces.value);
  } catch (error) {
    setStatus(String(error), true);
    return;
  }
  if (document.dirty && !(await synchronize(document))) return;
  selectBottomPanel("xpath");
  setStatus("Evaluating XPath…");
  xpathRun.disabled = true;
  try {
    const report = await invoke<XPathReport>("evaluate_xpath", {
      id: document.id,
      expression,
      namespaces,
    });
    if (document.id !== activeId || report.revision !== document.revision) return;
    renderXPath(report);
    setStatus("XPath complete");
  } catch (error) {
    xpathResults.replaceChildren(messageElement(String(error), "empty error-text"));
    setStatus(String(error), true);
  } finally {
    xpathRun.disabled = false;
  }
}

function messageElement(message: string, className: string): HTMLDivElement {
  const element = window.document.createElement("div");
  element.className = className;
  element.textContent = message;
  return element;
}

function renderXPath(report: XPathReport): void {
  if (report.value.kind !== "nodes") {
    xpathResults.replaceChildren(messageElement(String(report.value.value), "xpath-scalar"));
    return;
  }
  const { items, truncated } = report.value.value;
  if (items.length === 0) {
    xpathResults.replaceChildren(messageElement("No matching nodes", "empty"));
    return;
  }
  const summary = messageElement(
    `${items.length} result${items.length === 1 ? "" : "s"}${truncated ? " (limited)" : ""}`,
    "xpath-summary",
  );
  const rows = items.map((item) => {
    const row = window.document.createElement("button");
    row.className = "xpath-result";
    row.textContent = `${item.path}  ·  line ${item.span.start.line}`;
    row.addEventListener("click", () => goToByte(item.span.start.byte));
    return row;
  });
  xpathResults.replaceChildren(summary, ...rows);
}

async function runXQuery(): Promise<void> {
  const document = activeDocument();
  const query = xqueryExpression.value.trim();
  if (!document || !query) {
    setStatus("Enter an XQuery expression", true);
    return;
  }
  if (document.dirty && !(await synchronize(document))) return;
  selectBottomPanel("xquery");
  xqueryRun.disabled = true;
  setStatus("Evaluating XQuery…");
  try {
    const report = await invoke<XQueryPayload>("evaluate_xquery", {
      id: document.id,
      query,
    });
    if (document.id !== activeId || report.revision !== document.revision) return;
    const summary = messageElement(
      `${report.engine} · ${report.bytes} bytes${report.truncated ? " · preview limited" : ""}`,
      "output-summary",
    );
    const output = window.document.createElement("pre");
    output.className = "output-preview";
    output.textContent = report.preview;
    xqueryResults.replaceChildren(summary, output);
    setStatus("XQuery complete");
  } catch (error) {
    xqueryResults.replaceChildren(messageElement(String(error), "empty error-text"));
    setStatus(String(error), true);
  } finally {
    xqueryRun.disabled = false;
  }
}

async function openDiff(): Promise<void> {
  const document = activeDocument();
  if (!document) return;
  const selected = await open({
    multiple: false,
    filters: [{ name: "XML", extensions: ["xml", "xsd", "xsl", "xslt", "rng", "sch"] }],
  });
  if (!selected) return;
  diffOtherPath = selected;
  await compareWithSelectedFile();
}

async function compareWithSelectedFile(): Promise<void> {
  const document = activeDocument();
  if (!document || !diffOtherPath) return;
  if (document.dirty && !(await synchronize(document))) return;
  setStatus("Comparing XML…");
  try {
    const result = await invoke<DiffPayload>("compare_document", {
      id: document.id,
      otherPath: diffOtherPath,
      ignoreWhitespaceOnlyText: diffIgnoreWhitespace.checked,
    });
    if (document.id !== activeId || result.revision !== document.revision) return;
    activeDiff = result;
    renderDiff(document, result);
    diffDialog.hidden = false;
    setStatus(result.report.semanticEqual ? "XML is semantically equal" : `${result.report.hunks.length} XML difference(s)`);
  } catch (error) {
    setStatus(String(error), true);
  }
}

function renderDiff(document: OpenDocument, result: DiffPayload): void {
  diffLeftTitle.textContent = document.name;
  diffRightTitle.textContent = result.otherName;
  diffSummary.textContent = result.report.semanticEqual
    ? result.report.exactEqual ? "Exact match" : "Semantic match"
    : `${result.report.hunks.length} hunk${result.report.hunks.length === 1 ? "" : "s"}${result.report.coarse ? " · coarse" : ""}`;
  renderDiffLines(diffLeft, document.text, result.report.hunks.flatMap((hunk) => hunk.left ? [hunk.left] : []), "left");
  renderDiffLines(diffRight, result.otherText, result.report.hunks.flatMap((hunk) => hunk.right ? [hunk.right] : []), "right");
  const buttons = result.report.hunks.map((hunk, index) => {
    const group = window.document.createElement("div");
    group.className = "diff-hunk";
    const button = window.document.createElement("button");
    button.type = "button";
    button.textContent = `${index + 1}. ${hunk.kind} · ${hunk.leftTokens}→${hunk.rightTokens}`;
    button.addEventListener("click", () => {
      scrollDiffLine(diffLeft, hunk.left?.start.line ?? hunk.leftAnchor.line);
      scrollDiffLine(diffRight, hunk.right?.start.line ?? hunk.rightAnchor.line);
    });
    const apply = window.document.createElement("button");
    apply.type = "button";
    apply.className = "diff-apply";
    apply.textContent = "Apply";
    apply.title = "Apply this hunk from the right and re-parse the result";
    apply.addEventListener("click", () => void applyDiffHunk(index));
    group.append(button, apply);
    return group;
  });
  diffHunks.replaceChildren(...buttons);
}

async function applyDiffHunk(index: number): Promise<void> {
  const document = activeDocument();
  if (!document || !activeDiff || !diffOtherPath || activeDiff.revision !== document.revision) {
    setStatus("Comparison is stale; run it again", true);
    return;
  }
  setStatus("Validating XML merge…");
  try {
    const payload = await invoke<MergePayload>("merge_diff_hunk", {
      id: document.id,
      otherPath: diffOtherPath,
      ignoreWhitespaceOnlyText: diffIgnoreWhitespace.checked,
      hunkIndex: index,
    });
    if (document.id !== activeId || payload.revision !== document.revision) return;
    editor.dispatch({
      changes: { from: 0, to: editor.state.doc.length, insert: payload.text },
    });
    diffDialog.hidden = true;
    setStatus("Validated XML hunk applied; Undo is available");
  } catch (error) {
    setStatus(String(error), true);
  }
}

function renderDiffLines(host: HTMLDivElement, source: string, spans: SourceSpan[], side: string): void {
  const lines = source.split(/\r?\n/u);
  if (lines.length > 50_000) {
    const preview = window.document.createElement("pre");
    preview.textContent = `${source.slice(0, 2 * 1024 * 1024)}\n… display limited …`;
    host.replaceChildren(preview);
    return;
  }
  const changed = new Set<number>();
  for (const span of spans) {
    for (let line = span.start.line; line <= span.end.line; line += 1) changed.add(line);
  }
  host.replaceChildren(...lines.map((line, index) => {
    const number = index + 1;
    const row = window.document.createElement("div");
    row.id = `diff-${side}-${number}`;
    row.className = changed.has(number) ? "diff-line changed" : "diff-line";
    const gutter = window.document.createElement("span");
    gutter.textContent = String(number);
    const content = window.document.createElement("code");
    content.textContent = line || " ";
    row.append(gutter, content);
    return row;
  }));
}

function scrollDiffLine(host: HTMLDivElement, line: number | undefined): void {
  if (!line) return;
  const row = host.querySelector<HTMLElement>(`#${CSS.escape(`diff-${host === diffLeft ? "left" : "right"}-${line}`)}`);
  if (row) host.scrollTop = Math.max(0, row.offsetTop - host.clientHeight / 3);
}

function useRightDiffVersion(): void {
  const document = activeDocument();
  if (!document || !activeDiff || activeDiff.revision !== document.revision) {
    setStatus("Comparison is stale; run it again", true);
    return;
  }
  editor.dispatch({
    changes: { from: 0, to: editor.state.doc.length, insert: activeDiff.otherText },
  });
  diffDialog.hidden = true;
  setStatus("Right version applied; Undo is available");
}

async function createNew(): Promise<void> {
  try {
    addDocument(await invoke<DocumentPayload>("new_document"));
  } catch (error) {
    setStatus(String(error), true);
  }
}

async function openFile(): Promise<void> {
  const selected = await open({ multiple: false, filters: [{ name: "XML", extensions: ["xml", "xsd", "xsl", "xslt", "rng", "sch", "dtd"] }] });
  if (!selected) return;
  try {
    addDocument(await invoke<DocumentPayload>("open_document", { path: selected }));
  } catch (error) {
    setStatus(String(error), true);
  }
}

async function saveFile(): Promise<void> {
  const document = activeDocument();
  if (!document) return;
  if (document.dirty && !(await synchronize(document))) return;
  const target = document.path ?? await save({ defaultPath: document.name, filters: [{ name: "XML", extensions: ["xml"] }] });
  if (!target) return;
  try {
    const payload = await invoke<DocumentPayload>("save_document", { id: document.id, path: target });
    Object.assign(document, payload, { viewState: editor.state });
    renderTabs();
    setStatus("Saved");
  } catch (error) {
    setStatus(String(error), true);
  }
}

async function openProject(): Promise<void> {
  const selected = await open({ multiple: false, filters: [{ name: "O2XML project", extensions: ["toml"] }] });
  if (!selected) return;
  try {
    const project = await invoke<ProjectPayload>("open_project", { path: selected });
    projectTitle.textContent = project.manifest.project.name;
    projectTitle.title = project.path;
    renderTransformScenarios(project);
    projectFiles.replaceChildren(...project.files.map((file) => {
      const row = window.document.createElement("div");
      row.className = "tree-row static";
      row.textContent = file;
      return row;
    }));
  } catch (error) {
    setStatus(String(error), true);
  }
}

function renderTransformScenarios(project: ProjectPayload): void {
  const scenarios = project.manifest.transform.scenarios;
  if (scenarios.length === 0) {
    transformScenario.replaceChildren(new Option("No scenarios", ""));
    transformScenario.disabled = true;
    transformButton.disabled = true;
    return;
  }
  transformScenario.replaceChildren(...scenarios.map((scenario) => {
    const option = new Option(`${scenario.name} · ${scenario.engine}`, scenario.name);
    option.title = `${scenario.stylesheet} → ${scenario.output}`;
    return option;
  }));
  transformScenario.value = project.manifest.transform.default ?? scenarios[0]?.name ?? "";
  transformScenario.disabled = false;
  transformButton.disabled = false;
}

async function runTransformation(): Promise<void> {
  const document = activeDocument();
  const scenario = transformScenario.value;
  if (!document || !scenario) return;
  if (document.dirty && !(await synchronize(document))) return;
  selectBottomPanel("output");
  transformButton.disabled = true;
  setStatus("Transforming…");
  try {
    const result = await invoke<TransformPayload>("run_transformation", {
      id: document.id,
      scenario,
    });
    if (document.id !== activeId || result.revision !== document.revision) return;
    const summary = messageElement(
      `${result.scenario} · ${result.engine} · ${result.mediaType} · ${result.bytesWritten} bytes → ${result.outputPath}${result.previewTruncated ? " · preview limited" : ""}`,
      "output-summary",
    );
    renderTransformationOutput(summary, result);
    setStatus("Transformation complete");
  } catch (error) {
    outputPanel.replaceChildren(messageElement(String(error), "empty error-text"));
    setStatus(String(error), true);
  } finally {
    transformButton.disabled = transformScenario.disabled;
  }
}

function renderTransformationOutput(summary: HTMLDivElement, result: TransformPayload): void {
  const text = window.document.createElement("pre");
  text.className = "output-preview";
  text.textContent = result.preview;
  if (result.mediaType !== "html") {
    outputPanel.replaceChildren(summary, text);
    return;
  }

  const controls = window.document.createElement("div");
  controls.className = "output-view-controls";
  const textButton = window.document.createElement("button");
  textButton.type = "button";
  textButton.textContent = "Source";
  const previewButton = window.document.createElement("button");
  previewButton.type = "button";
  previewButton.textContent = "Safe preview";
  const frame = window.document.createElement("iframe");
  frame.className = "html-preview";
  frame.title = "Sandboxed transformation preview";
  frame.setAttribute("sandbox", "");
  frame.referrerPolicy = "no-referrer";
  frame.srcdoc = sanitizePreviewHtml(result.preview);

  const show = (preview: boolean): void => {
    text.hidden = preview;
    frame.hidden = !preview;
    textButton.classList.toggle("active", !preview);
    previewButton.classList.toggle("active", preview);
  };
  textButton.addEventListener("click", () => show(false));
  previewButton.addEventListener("click", () => show(true));
  controls.append(textButton, previewButton);
  outputPanel.replaceChildren(summary, controls, text, frame);
  show(true);
}

function sanitizePreviewHtml(source: string): string {
  const parsed = new DOMParser().parseFromString(source, "text/html");
  const allowedElements = new Set([
    "html", "head", "title", "body", "article", "section", "main", "header", "footer",
    "h1", "h2", "h3", "h4", "h5", "h6", "p", "div", "span", "strong", "em",
    "b", "i", "u", "sup", "sub", "blockquote", "pre", "code", "br", "hr", "ul",
    "ol", "li", "dl", "dt", "dd", "table", "caption", "thead", "tbody", "tfoot",
    "tr", "th", "td", "a",
  ]);
  const droppedWithContent = new Set([
    "script", "style", "link", "meta", "base", "iframe", "object", "embed", "applet",
    "form", "input", "button", "select", "textarea", "option", "svg", "math", "audio",
    "video", "source", "track", "img", "picture", "canvas",
  ]);
  const allowedAttributes = new Set([
    "class", "id", "title", "lang", "dir", "colspan", "rowspan", "scope",
  ]);
  const elements = [...parsed.querySelectorAll("*")];
  if (elements.length > 50_000) throw new Error("HTML preview exceeds the element limit");
  for (const element of elements) {
    if (!element.isConnected) continue;
    const name = element.localName.toLowerCase();
    if (!allowedElements.has(name)) {
      if (droppedWithContent.has(name)) element.remove();
      else element.replaceWith(...element.childNodes);
      continue;
    }
    for (const attribute of [...element.attributes]) {
      if (!allowedAttributes.has(attribute.name.toLowerCase())) {
        element.removeAttribute(attribute.name);
      }
    }
  }
  const policy = "default-src 'none'; base-uri 'none'; form-action 'none'; navigate-to 'none'; style-src 'unsafe-inline'";
  const style = "body{font:14px system-ui,sans-serif;margin:20px;color:#20242a}table{border-collapse:collapse}th,td{border:1px solid #bbb;padding:4px 6px}pre,code{font-family:ui-monospace,monospace;white-space:pre-wrap}";
  return `<!doctype html><html><head><meta http-equiv="Content-Security-Policy" content="${policy}"><style>${style}</style></head><body>${parsed.body.innerHTML}</body></html>`;
}

async function formatActive(): Promise<void> {
  const document = activeDocument();
  if (!document) return;
  if (document.dirty && !(await synchronize(document))) return;
  try {
    const formatted = await invoke<string>("format_document", { id: document.id, indentSize: 2 });
    editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: formatted } });
    setStatus("Formatted");
  } catch (error) {
    setStatus(String(error), true);
  }
}

function toggleTheme(): void {
  dark = !dark;
  document.documentElement.dataset.theme = dark ? "dark" : "light";
  editor.dispatch({ effects: theme.reconfigure(dark ? oneDark : []) });
}

document.documentElement.dataset.theme = dark ? "dark" : "light";

requiredElement<HTMLElement>(".menubar").addEventListener("click", (event) => {
  const action = (event.target as HTMLElement).closest<HTMLButtonElement>("button")?.dataset.action;
  if (action === "new") void createNew();
  if (action === "open") void openFile();
  if (action === "save") void saveFile();
  if (action === "project") void openProject();
  if (action === "format") void formatActive();
  if (action === "validate") {
    const document = activeDocument();
    if (document) void synchronizeAndAnalyze(document);
  }
  if (action === "transform") void runTransformation();
  if (action === "diff") void openDiff();
  if (action === "author") void toggleAuthorMode();
  if (action === "xpath") selectBottomPanel("xpath");
  if (action === "theme") toggleTheme();
});

requiredElement<HTMLElement>(".panel-tabs").addEventListener("click", (event) => {
  const panel = (event.target as HTMLElement).closest<HTMLButtonElement>("[data-panel]")?.dataset.panel;
  if (panel === "problems" || panel === "output" || panel === "xpath" || panel === "xquery" || panel === "grid") selectBottomPanel(panel);
});

xpathRun.addEventListener("click", () => void runXPath());
xpathExpression.addEventListener("keydown", (event) => {
  if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) {
    event.preventDefault();
    void runXPath();
  }
});
xqueryRun.addEventListener("click", () => void runXQuery());
gridRun.addEventListener("click", () => void runGrid());
gridRowElement.addEventListener("keydown", (event) => {
  if (event.key === "Enter") void runGrid();
});
xqueryExpression.addEventListener("keydown", (event) => {
  if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) {
    event.preventDefault();
    void runXQuery();
  }
});
requiredElement<HTMLButtonElement>("#diff-close").addEventListener("click", () => {
  diffDialog.hidden = true;
});
diffUseRight.addEventListener("click", useRightDiffVersion);
diffIgnoreWhitespace.addEventListener("change", () => void compareWithSelectedFile());

window.addEventListener("keydown", (event) => {
  const modifier = event.metaKey || event.ctrlKey;
  if (modifier && event.key.toLowerCase() === "s") {
    event.preventDefault();
    void saveFile();
  }
  if (modifier && event.key.toLowerCase() === "o") {
    event.preventDefault();
    void openFile();
  }
});

void createNew();
