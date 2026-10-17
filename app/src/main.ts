import "./styles.css";

import { closeBrackets, closeBracketsKeymap, completionKeymap } from "@codemirror/autocomplete";
import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { foldGutter, foldKeymap, indentOnInput } from "@codemirror/language";
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
  Diagnostic,
  DocumentPayload,
  OutlineEntry,
  ProjectPayload,
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
      </aside>
      <section class="editor-column">
        <div id="tabs" class="tabs"></div>
        <div id="editor" class="editor"></div>
      </section>
    </main>
    <section class="bottom-panel">
      <div class="panel-tabs">
        <button class="active">Problems <span id="problem-count">0</span></button>
        <button>Output</button>
        <span id="status" class="status">Ready</span>
      </div>
      <div id="problems" class="problems" aria-live="polite"></div>
    </section>
  </div>
`;

const editorHost = requiredElement<HTMLDivElement>("#editor");
const tabsHost = requiredElement<HTMLDivElement>("#tabs");
const outlineHost = requiredElement<HTMLDivElement>("#outline");
const problemsHost = requiredElement<HTMLDivElement>("#problems");
const statusHost = requiredElement<HTMLSpanElement>("#status");
const problemCount = requiredElement<HTMLSpanElement>("#problem-count");
const projectFiles = requiredElement<HTMLDivElement>("#project-files");
const projectTitle = requiredElement<HTMLDivElement>("#project-title");

const theme = new Compartment();
let dark = window.matchMedia("(prefers-color-scheme: dark)").matches;
let documents: OpenDocument[] = [];
let activeId: number | null = null;
let analysisTimer: number | undefined;
let suppressChanges = false;

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
    row.textContent = `${diagnostic.severity.toUpperCase()}  ${diagnostic.message}  (${diagnostic.span.start.line}:${diagnostic.span.start.column})`;
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

function goToByte(byte: number): void {
  const text = editor.state.doc.toString();
  const position = Math.min(byteToUtf16(text, byte), editor.state.doc.length);
  editor.dispatch({ selection: { anchor: position }, scrollIntoView: true });
  editor.focus();
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
  if (action === "theme") toggleTheme();
});

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
