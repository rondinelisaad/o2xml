import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

import { xml } from "@codemirror/lang-xml";
import { EditorState } from "@codemirror/state";

import { linkedClosingTagChange } from "../src/editor-behavior.ts";

function rename(source: string, from: number, to: number, insert: string): string {
  const start = EditorState.create({ doc: source, extensions: [xml()] });
  const transaction = start.update({ changes: { from, to, insert } });
  const linked = linkedClosingTagChange(start, transaction.state, transaction.changes);
  return linked ? transaction.state.update({ changes: linked }).state.doc.toString() : transaction.state.doc.toString();
}

test("renames the matching closing tag with the opening tag", () => {
  assert.equal(rename("<title>Example</title>", 1, 6, "article-title"), "<article-title>Example</article-title>");
});

test("renames the closing tag when text is appended at the opening-name boundary", () => {
  assert.equal(rename("<sec>Example</sec>", 4, 4, "tion"), "<section>Example</section>");
});

test("renames the closing tag across incremental edits", () => {
  const first = rename("<sec>Example</sec>", 4, 4, "t");
  assert.equal(rename(first, 5, 5, "ion"), "<section>Example</section>");
});

test("does not rewrite an unrelated closing-tag edit", () => {
  assert.equal(rename("<title>Example</title>", 16, 21, "heading"), "<title>Example</heading>");
});

test("preserves an already mismatched closing tag", () => {
  assert.equal(rename("<title>Example</heading>", 1, 6, "article-title"), "<article-title>Example</heading>");
});

test("does not let the application stylesheet override the dynamic editor zoom", async () => {
  const stylesheet = await readFile(new URL("../src/styles.css", import.meta.url), "utf8");
  const editorRule = stylesheet.match(/\.cm-editor\s*\{([^}]*)\}/u)?.[1] ?? "";
  assert.doesNotMatch(editorRule, /font-size\s*:/u);
});
