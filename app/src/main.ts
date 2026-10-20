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
import {
  defaultHighlightStyle,
  foldGutter,
  foldKeymap,
  indentOnInput,
  syntaxHighlighting,
  syntaxTree,
} from "@codemirror/language";
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
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open, save } from "@tauri-apps/plugin-dialog";

import {
  getLocale,
  initializeLocale,
  isLocale,
  persistLocale,
  t,
  type MessageKey,
} from "./i18n";

import type {
  AnalysisPayload,
  AuthorContent,
  AuthorEditPayload,
  AuthorElement,
  AuthorPayload,
  AuthorPreviewPayload,
  AuthorTextRun,
  Diagnostic,
  DiffPayload,
  DocumentPayload,
  ExternalChangePayload,
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

initializeLocale(window.localStorage);
document.documentElement.lang = getLocale();

app.innerHTML = `
  <div class="shell">
    <header class="menubar">
      <strong class="brand">O2XML</strong>
      <button data-action="new">${t("menu.new")}</button>
      <button data-action="open">${t("menu.open")}</button>
      <button data-action="save">${t("menu.save")}</button>
      <button data-action="project">${t("menu.project")}</button>
      <button data-action="format">${t("menu.prettyPrint")}</button>
      <button data-action="validate">${t("menu.validate")}</button>
      <select id="transform-scenario" class="transform-select" aria-label="${t("transform.scenario")}" disabled>
        <option>${t("transform.noScenarios")}</option>
      </select>
      <button data-action="transform" disabled>${t("menu.transform")}</button>
      <button data-action="diff">${t("menu.compare")}</button>
      <button data-action="author">${t("menu.authorMode")}</button>
      <button data-action="xpath">XPath</button>
      <span class="spacer"></span>
      <label class="language-control"><span>${t("language.label")}</span>
        <select id="language" aria-label="${t("language.label")}">
          <option value="pt-BR">${t("language.portuguese")}</option>
          <option value="en">${t("language.english")}</option>
        </select>
      </label>
      <button data-action="theme" aria-label="${t("menu.toggleTheme")}">${t("menu.theme")}</button>
    </header>
    <main class="workspace">
      <aside class="sidebar">
        <section>
          <h2>${t("sidebar.project")}</h2>
          <div id="project-title" class="muted">${t("sidebar.noProject")}</div>
          <div id="project-files" class="tree"></div>
        </section>
        <section class="outline-section">
          <h2>${t("sidebar.outline")}</h2>
          <div id="outline" class="tree"></div>
        </section>
        <section class="outline-section">
          <h2>${t("sidebar.jatsInspector")}</h2>
          <div id="jats-inspector" class="tree"><div class="empty">${t("sidebar.notJats")}</div></div>
        </section>
        <section class="outline-section">
          <h2>${t("sidebar.validationFramework")}</h2>
          <div id="framework-stages" class="tree"><div class="empty">${t("sidebar.noFramework")}</div></div>
        </section>
      </aside>
      <section class="editor-column">
        <div id="tabs" class="tabs"></div>
        <div id="editor" class="editor"></div>
        <div id="author-view" class="author-view" hidden>
          <div class="author-toolbar">
            <strong>${t("author.title")}</strong>
            <button id="author-preview-button" type="button">${t("author.scieloPreview")}</button>
            <button id="author-structured-button" type="button">${t("author.structuredEditing")}</button>
            <span id="author-engine">${t("author.description")}</span>
          </div>
          <div id="author-document" class="author-document" aria-live="polite"></div>
          <iframe id="author-preview" class="author-preview" title="${t("author.previewTitle")}" sandbox="" referrerpolicy="no-referrer" hidden></iframe>
        </div>
      </section>
    </main>
    <section class="bottom-panel">
      <div class="panel-tabs">
        <button class="active" data-panel="problems">${t("panel.problems")} <span id="problem-count">0</span></button>
        <button data-panel="output">${t("panel.output")}</button>
        <button data-panel="xpath">XPath</button>
        <button data-panel="xquery">XQuery</button>
        <button data-panel="grid">${t("panel.grid")}</button>
        <span id="status" class="status">${t("status.ready")}</span>
      </div>
      <div id="problems" class="panel-content active problems" data-panel-content="problems" aria-live="polite"></div>
      <div id="output-panel" class="panel-content" data-panel-content="output"><div class="empty">${t("output.none")}</div></div>
      <div id="xpath-panel" class="panel-content xpath-panel" data-panel-content="xpath">
        <div class="xpath-controls">
          <input id="xpath-expression" type="text" value="//*" aria-label="${t("xpath.expression")}" spellcheck="false" />
          <button id="xpath-run" type="button">${t("common.run")}</button>
        </div>
        <div class="xpath-body">
          <label class="xpath-namespaces">${t("xpath.namespaces")} <span>${t("xpath.namespaceHint")}</span>
            <textarea id="xpath-namespaces" spellcheck="false" placeholder="j=http://jats.nlm.nih.gov"></textarea>
          </label>
          <div id="xpath-results" class="xpath-results" aria-live="polite"><div class="empty">${t("xpath.initial")}</div></div>
        </div>
      </div>
      <div id="xquery-panel" class="panel-content xquery-panel" data-panel-content="xquery">
        <div class="xquery-controls">
          <textarea id="xquery-expression" spellcheck="false" aria-label="${t("xquery.expression")}">for $title in //article-title return $title</textarea>
          <button id="xquery-run" type="button">${t("common.run")}</button>
        </div>
        <div id="xquery-results" class="xquery-results" aria-live="polite"><div class="empty">${t("xquery.initial")}</div></div>
      </div>
      <div id="grid-panel" class="panel-content grid-panel" data-panel-content="grid">
        <div class="grid-controls">
          <input id="grid-row-element" type="text" aria-label="${t("grid.repeatedElement")}" placeholder="${t("grid.placeholder")}" spellcheck="false" />
          <button id="grid-run" type="button">${t("grid.build")}</button>
          <span id="grid-summary" class="muted">${t("grid.hint")}</span>
        </div>
        <div id="grid-results" class="grid-results" aria-live="polite"><div class="empty">${t("grid.initial")}</div></div>
      </div>
    </section>
  </div>
  <div id="diff-dialog" class="diff-dialog" hidden>
    <div class="diff-window" role="dialog" aria-modal="true" aria-labelledby="diff-title">
      <header class="diff-header">
        <strong id="diff-title">${t("diff.title")}</strong>
        <span id="diff-summary" class="muted"></span>
        <label><input id="diff-ignore-whitespace" type="checkbox" /> ${t("diff.ignoreWhitespace")}</label>
        <button id="diff-use-right" type="button">${t("diff.useRight")}</button>
        <button id="diff-close" type="button">${t("diff.close")}</button>
      </header>
      <div id="diff-hunks" class="diff-hunks"></div>
      <div class="diff-panes">
        <section><h2 id="diff-left-title">${t("diff.current")}</h2><div id="diff-left" class="diff-code"></div></section>
        <section><h2 id="diff-right-title">${t("diff.selected")}</h2><div id="diff-right" class="diff-code"></div></section>
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
const authorPreviewFrame = requiredElement<HTMLIFrameElement>("#author-preview");
const authorPreviewButton = requiredElement<HTMLButtonElement>("#author-preview-button");
const authorStructuredButton = requiredElement<HTMLButtonElement>("#author-structured-button");
const authorEngine = requiredElement<HTMLSpanElement>("#author-engine");
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
const languageSelect = requiredElement<HTMLSelectElement>("#language");
languageSelect.value = getLocale();

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
let activeAuthorPreview: AuthorPreviewPayload | null = null;
let externalCheckInProgress = false;

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
    syntaxHighlighting(defaultHighlightStyle),
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
    const tab = window.document.createElement("div");
    tab.className = item.id === activeId ? "tab active" : "tab";
    tab.title = item.path ?? t("document.unsaved");
    const activate = window.document.createElement("button");
    activate.className = "tab-label";
    activate.textContent = `${item.name}${item.dirty ? " •" : ""}`;
    activate.addEventListener("click", () => activateDocument(item.id));
    const close = window.document.createElement("button");
    close.className = "tab-close";
    close.type = "button";
    close.textContent = "×";
    close.setAttribute("aria-label", t("document.close", { name: item.name }));
    close.title = t("document.close", { name: item.name });
    close.addEventListener("click", () => void closeDocument(item.id));
    tab.append(activate, close);
    return tab;
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
  setStatus(t("validation.validating"));
  try {
    const result = await invoke<AnalysisPayload>("analyze_document", { id: document.id });
    if (document.id !== activeId || result.revision !== document.revision) return;
    renderProblems(result.diagnostics, Boolean(result.jatsInspection), result.frameworkStages.length > 0);
    renderOutline(result.outline);
    renderJatsInspection(result.jatsInspection);
    renderFrameworkStages(result.frameworkStages);
    setStatus(
      result.diagnostics.length === 0
        ? validationSuccessMessage(result)
        : t("validation.failed"),
      result.diagnostics.length > 0,
    );
  } catch (error) {
    setStatus(String(error), true);
  }
}

function validationSuccessMessage(result: AnalysisPayload): string {
  if (!result.jatsInspection) return t("validation.wellFormed");
  const stages = new Set(result.frameworkStages.map((stage) => stage.id));
  if (stages.has("sps-rules")) return t("validation.jatsSpsValid");
  if (stages.has("jats-grammar")) return t("validation.jatsValidSpsNotValidated");
  return t("validation.jatsNotValidated");
}

function renderProblems(
  diagnostics: Diagnostic[],
  isJats = false,
  frameworkValidated = false,
): void {
  problemCount.textContent = String(diagnostics.length);
  if (diagnostics.length === 0) {
    const empty = window.document.createElement("div");
    empty.className = "empty";
    empty.textContent = isJats
      ? frameworkValidated
        ? t("validation.noFrameworkProblems")
        : t("validation.noJatsValidation")
      : t("validation.noProblems");
    problemsHost.replaceChildren(empty);
    return;
  }
  const rows = diagnostics.map((diagnostic) => {
    const row = window.document.createElement("button");
    row.className = `problem ${diagnostic.severity}`;
    row.textContent = `${translatedSeverity(diagnostic.severity)}  [${translatedDiagnosticSource(diagnostic.source)}] ${diagnostic.message}  (${diagnostic.span.start.line}:${diagnostic.span.start.column})`;
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
    empty.textContent = t("sidebar.notJats");
    jatsInspectorHost.replaceChildren(empty);
    return;
  }
  const rows = inspection.entries.map((entry) => {
    const button = window.document.createElement("button");
    button.className = "tree-row";
    button.style.paddingInlineStart = `${12 + entry.depth * 16}px`;
    button.textContent = translatedJatsLabel(entry.elementName, entry.label);
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
    empty.textContent = t("sidebar.noFramework");
    frameworkStagesHost.replaceChildren(empty);
    return;
  }
  const rows = stages.map((stage) => {
    const row = window.document.createElement("div");
    row.className = `tree-row static ${stage.status}`;
    row.textContent = `${stage.label}: ${translatedFrameworkStatus(stage.status)}`;
    row.title = `${stage.engine} · ${t("framework.diagnostics", { count: stage.diagnostics.length })}`;
    return row;
  });
  frameworkStagesHost.replaceChildren(...rows);
}

function translatedSeverity(severity: Diagnostic["severity"]): string {
  if (severity === "error") return t("severity.error");
  if (severity === "warning") return t("severity.warning");
  return t("severity.info");
}

function translatedDiagnosticSource(source: string): string {
  return source === "well-formedness" ? t("source.wellFormedness") : source;
}

function translatedFrameworkStatus(status: string): string {
  const keys: Record<string, MessageKey> = {
    passed: "framework.status.passed",
    failed: "framework.status.failed",
    skipped: "framework.status.skipped",
    unavailable: "framework.status.unavailable",
  };
  const key = keys[status];
  return key ? t(key) : status;
}

function translatedJatsLabel(elementName: string, fallback: string): string {
  const keys: Record<string, MessageKey> = {
    article: "jats.article",
    front: "jats.front",
    "journal-meta": "jats.journal-meta",
    "article-meta": "jats.article-meta",
    "title-group": "jats.title-group",
    "article-title": "jats.article-title",
    "contrib-group": "jats.contrib-group",
    body: "jats.body",
    sec: "jats.sec",
    back: "jats.back",
    "ref-list": "jats.ref-list",
    "floats-group": "jats.floats-group",
  };
  const key = keys[elementName];
  return key ? t(key) : fallback;
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
  activeAuthorPreview = null;
  authorPreviewFrame.srcdoc = "";
  authorView.hidden = true;
  editorHost.hidden = false;
  authorButton.textContent = t("menu.authorMode");
}

function showSourceAt(byte: number): void {
  showSourceMode();
  goToByte(byte);
}

async function toggleAuthorMode(): Promise<void> {
  if (authorActive) {
    showSourceMode();
    setStatus(t("status.xmlSource"));
    return;
  }
  await runAuthorMode();
}

async function runAuthorMode(): Promise<void> {
  const document = activeDocument();
  if (!document) return;
  if (document.dirty && !(await synchronize(document))) return;
  setStatus(t("status.buildingAuthor"));
  authorButton.disabled = true;
  try {
    const [payload, previewResult] = await Promise.all([
      invoke<AuthorPayload>("inspect_author", { id: document.id }),
      invoke<AuthorPreviewPayload>("preview_author", { id: document.id })
        .then((preview) => ({ preview, error: null }))
        .catch((error: unknown) => ({ preview: null, error: String(error) })),
    ]);
    if (document.id !== activeId || payload.revision !== document.revision) return;
    activeAuthor = payload;
    activeAuthorPreview = previewResult.preview;
    authorActive = true;
    renderAuthor(payload);
    if (previewResult.preview?.revision === document.revision) {
      renderAuthorPreview(previewResult.preview);
      showAuthorPane("preview");
    } else {
      authorEngine.textContent = previewResult.error
        ? t("author.previewUnavailable", { reason: previewResult.error })
        : t("author.description");
      showAuthorPane("structured");
    }
    editorHost.hidden = true;
    authorView.hidden = false;
    authorButton.textContent = t("menu.xmlSource");
    setStatus(t("status.authorReady"));
  } catch (error) {
    showSourceMode();
    setStatus(String(error), true);
  } finally {
    authorButton.disabled = false;
  }
}

function renderAuthorPreview(payload: AuthorPreviewPayload): void {
  authorPreviewFrame.srcdoc = sanitizePreviewHtml(payload.html, payload.stylesheet);
  authorEngine.textContent = payload.engine;
}

function showAuthorPane(pane: "preview" | "structured"): void {
  const preview = pane === "preview" && activeAuthorPreview !== null;
  authorPreviewFrame.hidden = !preview;
  authorDocumentHost.hidden = preview;
  authorPreviewButton.disabled = activeAuthorPreview === null;
  authorPreviewButton.classList.toggle("active", preview);
  authorStructuredButton.classList.toggle("active", !preview);
  if (!preview && activeAuthorPreview) authorEngine.textContent = t("author.description");
  if (preview && activeAuthorPreview) authorEngine.textContent = activeAuthorPreview.engine;
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
  node.title = inline ? `${element.label} · ${t("xpath.resultLine", { line: element.span.start.line })}` : "";

  if (!inline) {
    const label = window.document.createElement("button");
    label.className = "author-element-label";
    label.type = "button";
    label.textContent = element.label;
    label.title = t("author.inspect", {
      element: element.label,
      line: element.span.start.line,
    });
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
    ? t("author.editable", { line: run.sourceSpan.start.line })
    : t("author.readOnly");
  if (!run.editable) return text;

  text.contentEditable = "plaintext-only";
  text.spellcheck = true;
  text.setAttribute("role", "textbox");
  text.setAttribute("aria-label", t("author.textAria", { line: run.sourceSpan.start.line }));
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
    setStatus(t("status.authorStale"), true);
    return;
  }
  textNode.contentEditable = "false";
  setStatus(t("status.validatingAuthorEdit"));
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
      setStatus(t("status.authorChanged"), true);
      return;
    }
    editor.dispatch({
      changes: { from: 0, to: editor.state.doc.length, insert: payload.text },
    });
    const synchronized = await synchronize(document);
    if (synchronized) {
      setStatus(
        payload.schemaValidated
          ? t("status.authorSchemaApplied")
          : t("status.authorWellFormedApplied"),
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
  setStatus(t("status.buildingGrid"));
  gridRun.disabled = true;
  try {
    const rowElement = gridRowElement.value.trim() || null;
    const payload = await invoke<GridPayload>("inspect_grid", {
      id: document.id,
      rowElement,
    });
    if (document.id !== activeId || payload.revision !== document.revision) return;
    renderGrid(payload);
    setStatus(t("status.gridReady"));
  } catch (error) {
    gridResults.replaceChildren(messageElement(String(error), "empty error-text"));
    gridSummary.textContent = t("grid.hint");
    setStatus(String(error), true);
  } finally {
    gridRun.disabled = false;
  }
}

function renderGrid(payload: GridPayload): void {
  const { grid } = payload;
  gridSummary.textContent = t("grid.summary", {
    count: grid.rows.length,
    element: grid.rowElement,
    limited: grid.truncated ? t("common.limited") : "",
  });
  const table = window.document.createElement("table");
  table.className = "xml-grid-table";
  const head = window.document.createElement("thead");
  const headerRow = window.document.createElement("tr");
  const lineHeader = window.document.createElement("th");
  lineHeader.textContent = t("grid.line");
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
    tableRow.title = t("grid.goToLine", { line: row.span.start.line });
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
        input.setAttribute("aria-label", t("grid.cellAria", {
          column,
          row: rowIndex + 1,
        }));
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
        cell.title = t("grid.readOnly");
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
    setStatus(t("status.gridStale"), true);
    return;
  }
  input.disabled = true;
  setStatus(t("status.validatingGridEdit"));
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
      setStatus(t("status.gridChanged"), true);
      return;
    }
    editor.dispatch({
      changes: { from: 0, to: editor.state.doc.length, insert: payload.text },
    });
    const synchronized = await synchronize(document);
    if (synchronized) {
      setStatus(
        payload.schemaValidated
          ? t("status.gridSchemaApplied")
          : t("status.gridWellFormedApplied"),
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
      throw new Error(t("namespace.invalidBinding", { line: index + 1 }));
    }
    const prefix = line.slice(0, separator).trim();
    const uri = line.slice(separator + 1).trim();
    if (!/^[A-Za-z_][A-Za-z0-9._-]*$/u.test(prefix)) {
      throw new Error(t("namespace.invalidPrefix", { line: index + 1 }));
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
    setStatus(t("status.enterXPath"), true);
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
  setStatus(t("status.evaluatingXPath"));
  xpathRun.disabled = true;
  try {
    const report = await invoke<XPathReport>("evaluate_xpath", {
      id: document.id,
      expression,
      namespaces,
    });
    if (document.id !== activeId || report.revision !== document.revision) return;
    renderXPath(report);
    setStatus(t("status.xpathComplete"));
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
    xpathResults.replaceChildren(messageElement(t("xpath.noMatches"), "empty"));
    return;
  }
  const summary = messageElement(
    t("xpath.results", {
      count: items.length,
      limited: truncated ? t("common.limited") : "",
    }),
    "xpath-summary",
  );
  const rows = items.map((item) => {
    const row = window.document.createElement("button");
    row.className = "xpath-result";
    row.textContent = `${item.path}  ·  ${t("xpath.resultLine", { line: item.span.start.line })}`;
    row.addEventListener("click", () => goToByte(item.span.start.byte));
    return row;
  });
  xpathResults.replaceChildren(summary, ...rows);
}

async function runXQuery(): Promise<void> {
  const document = activeDocument();
  const query = xqueryExpression.value.trim();
  if (!document || !query) {
    setStatus(t("status.enterXQuery"), true);
    return;
  }
  if (document.dirty && !(await synchronize(document))) return;
  selectBottomPanel("xquery");
  xqueryRun.disabled = true;
  setStatus(t("status.evaluatingXQuery"));
  try {
    const report = await invoke<XQueryPayload>("evaluate_xquery", {
      id: document.id,
      query,
    });
    if (document.id !== activeId || report.revision !== document.revision) return;
    const summary = messageElement(
      `${report.engine} · ${report.bytes} bytes${report.truncated ? t("xquery.previewLimited") : ""}`,
      "output-summary",
    );
    const output = window.document.createElement("pre");
    output.className = "output-preview";
    output.textContent = report.preview;
    xqueryResults.replaceChildren(summary, output);
    setStatus(t("status.xqueryComplete"));
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
  setStatus(t("status.comparing"));
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
    setStatus(
      result.report.semanticEqual
        ? t("status.semanticEqual")
        : t("status.xmlDifferences", { count: result.report.hunks.length }),
    );
  } catch (error) {
    setStatus(String(error), true);
  }
}

function renderDiff(document: OpenDocument, result: DiffPayload): void {
  diffLeftTitle.textContent = document.name;
  diffRightTitle.textContent = result.otherName;
  diffSummary.textContent = result.report.semanticEqual
    ? result.report.exactEqual ? t("diff.exactMatch") : t("diff.semanticMatch")
    : t("diff.hunks", {
      count: result.report.hunks.length,
      coarse: result.report.coarse ? t("diff.coarse") : "",
    });
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
    apply.textContent = t("diff.apply");
    apply.title = t("diff.applyTitle");
    apply.addEventListener("click", () => void applyDiffHunk(index));
    group.append(button, apply);
    return group;
  });
  diffHunks.replaceChildren(...buttons);
}

async function applyDiffHunk(index: number): Promise<void> {
  const document = activeDocument();
  if (!document || !activeDiff || !diffOtherPath || activeDiff.revision !== document.revision) {
    setStatus(t("status.comparisonStale"), true);
    return;
  }
  setStatus(t("status.validatingMerge"));
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
    setStatus(t("status.hunkApplied"));
  } catch (error) {
    setStatus(String(error), true);
  }
}

function renderDiffLines(host: HTMLDivElement, source: string, spans: SourceSpan[], side: string): void {
  const lines = source.split(/\r?\n/u);
  if (lines.length > 50_000) {
    const preview = window.document.createElement("pre");
    preview.textContent = `${source.slice(0, 2 * 1024 * 1024)}\n${t("diff.displayLimited")}`;
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
    setStatus(t("status.comparisonStale"), true);
    return;
  }
  editor.dispatch({
    changes: { from: 0, to: editor.state.doc.length, insert: activeDiff.otherText },
  });
  diffDialog.hidden = true;
  setStatus(t("status.rightApplied"));
}

async function createNew(): Promise<void> {
  try {
    addDocument(await invoke<DocumentPayload>("new_document"));
  } catch (error) {
    setStatus(String(error), true);
  }
}

async function openPaths(paths: string[]): Promise<void> {
  const supported = /\.(?:xml|xsd|xsl|xslt|rng|sch|dtd)$/iu;
  const uniquePaths = [...new Set(paths.filter((path) => supported.test(path)))];
  for (const path of uniquePaths) {
    const existing = documents.find((document) => document.path === path);
    if (existing) {
      activateDocument(existing.id);
      continue;
    }
    try {
      addDocument(await invoke<DocumentPayload>("open_document", { path }));
    } catch (error) {
      setStatus(`${path}: ${String(error)}`, true);
    }
  }
}

async function openFile(): Promise<void> {
  const selected = await open({ multiple: true, filters: [{ name: "XML", extensions: ["xml", "xsd", "xsl", "xslt", "rng", "sch", "dtd"] }] });
  if (!selected) return;
  await openPaths(Array.isArray(selected) ? selected : [selected]);
}

async function closeDocument(id: number): Promise<void> {
  const index = documents.findIndex((document) => document.id === id);
  if (index < 0) return;
  const document = documents[index];
  if (!document) return;
  if (document.dirty && !window.confirm(t("document.closeUnsaved", { name: document.name }))) return;
  try {
    await invoke("close_document", { id });
    documents.splice(index, 1);
    if (activeId === id) {
      const next = documents[Math.min(index, documents.length - 1)];
      activeId = null;
      if (next) activateDocument(next.id);
      else await createNew();
    } else {
      renderTabs();
    }
  } catch (error) {
    setStatus(String(error), true);
  }
}

async function checkExternalChange(): Promise<void> {
  const document = activeDocument();
  if (!document?.path || externalCheckInProgress) return;
  externalCheckInProgress = true;
  try {
    const result = await invoke<ExternalChangePayload>("check_external_change", { id: document.id });
    if (document.id !== activeId || result.status === "unchanged") return;
    if (result.status === "deleted") {
      window.alert(t("document.deletedExternally", { name: document.name }));
      await invoke("acknowledge_external_change", { id: document.id });
      return;
    }
    const reload = window.confirm(t(
      document.dirty ? "document.modifiedExternallyDirty" : "document.modifiedExternally",
      { name: document.name },
    ));
    if (!reload) {
      await invoke("acknowledge_external_change", { id: document.id });
      return;
    }
    const payload = await invoke<DocumentPayload>("reload_document", { id: document.id });
    Object.assign(document, payload, { viewState: createEditorState(payload.text) });
    suppressChanges = true;
    editor.setState(document.viewState);
    suppressChanges = false;
    renderTabs();
    await analyze(document);
    setStatus(t("status.reloaded"));
  } catch (error) {
    setStatus(String(error), true);
  } finally {
    externalCheckInProgress = false;
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
    setStatus(t("status.saved"));
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
    transformScenario.replaceChildren(new Option(t("transform.noScenarios"), ""));
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
  setStatus(t("status.transforming"));
  try {
    const result = await invoke<TransformPayload>("run_transformation", {
      id: document.id,
      scenario,
    });
    if (document.id !== activeId || result.revision !== document.revision) return;
    const summary = messageElement(
      `${result.scenario} · ${result.engine} · ${result.mediaType} · ${result.bytesWritten} bytes → ${result.outputPath}${result.previewTruncated ? t("transform.previewLimited") : ""}`,
      "output-summary",
    );
    renderTransformationOutput(summary, result);
    setStatus(t("status.transformationComplete"));
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
  textButton.textContent = t("transform.source");
  const previewButton = window.document.createElement("button");
  previewButton.type = "button";
  previewButton.textContent = t("transform.safePreview");
  const frame = window.document.createElement("iframe");
  frame.className = "html-preview";
  frame.title = t("transform.previewTitle");
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

function sanitizePreviewHtml(source: string, stylesheet = ""): string {
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
  if (elements.length > 50_000) throw new Error(t("transform.previewElementLimit"));
  for (const element of elements) {
    if (!element.isConnected) continue;
    const name = element.localName.toLowerCase();
    if (!allowedElements.has(name)) {
      if (droppedWithContent.has(name)) element.remove();
      else element.replaceWith(...element.childNodes);
      continue;
    }
    for (const attribute of [...element.attributes]) {
      const attributeName = attribute.name.toLowerCase();
      if (attributeName === "href" && name === "a" && attribute.value.startsWith("#")) {
        continue;
      }
      if (!allowedAttributes.has(attributeName)) {
        element.removeAttribute(attribute.name);
      }
    }
  }
  const policy = "default-src 'none'; base-uri 'none'; form-action 'none'; navigate-to 'none'; style-src 'unsafe-inline'";
  const fallback = "body{font:14px system-ui,sans-serif;margin:20px;color:#20242a}table{border-collapse:collapse}th,td{border:1px solid #bbb;padding:4px 6px}pre,code{font-family:ui-monospace,monospace;white-space:pre-wrap}";
  const style = `${stylesheet}\n${fallback}`.replace(/<\/style/giu, "<\\/style");
  return `<!doctype html><html><head><meta http-equiv="Content-Security-Policy" content="${policy}"><style>${style}</style></head><body>${parsed.body.innerHTML}</body></html>`;
}

async function formatActive(): Promise<void> {
  const document = activeDocument();
  if (!document) return;
  if (document.dirty && !(await synchronize(document))) return;
  try {
    const formatted = await invoke<string>("format_document", { id: document.id, indentSize: 2 });
    editor.dispatch({ changes: { from: 0, to: editor.state.doc.length, insert: formatted } });
    setStatus(t("status.formatted"));
  } catch (error) {
    setStatus(String(error), true);
  }
}

function toggleTheme(): void {
  dark = !dark;
  document.documentElement.dataset.theme = dark ? "dark" : "light";
  editor.dispatch({ effects: theme.reconfigure(dark ? oneDark : []) });
}

function changeLanguage(value: string): void {
  if (!isLocale(value) || value === getLocale()) return;
  if (documents.some((document) => document.dirty) && !window.confirm(t("language.reloadWarning"))) {
    languageSelect.value = getLocale();
    return;
  }
  persistLocale(value, window.localStorage);
  window.location.reload();
}

document.documentElement.dataset.theme = dark ? "dark" : "light";

authorPreviewButton.addEventListener("click", () => showAuthorPane("preview"));
authorStructuredButton.addEventListener("click", () => showAuthorPane("structured"));

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
languageSelect.addEventListener("change", () => changeLanguage(languageSelect.value));
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
  if (modifier && event.key.toLowerCase() === "w") {
    event.preventDefault();
    const document = activeDocument();
    if (document) void closeDocument(document.id);
  }
});

window.addEventListener("focus", () => void checkExternalChange());
window.setInterval(() => {
  if (document.hasFocus()) void checkExternalChange();
}, 2_000);

void getCurrentWebview().onDragDropEvent((event) => {
  const shell = requiredElement<HTMLElement>(".shell");
  if (event.payload.type === "enter" || event.payload.type === "over") {
    shell.classList.add("drag-active");
  } else {
    shell.classList.remove("drag-active");
  }
  if (event.payload.type === "drop") void openPaths(event.payload.paths);
}).catch((error: unknown) => {
  console.debug("native drag-and-drop is unavailable", error);
});

void createNew();
