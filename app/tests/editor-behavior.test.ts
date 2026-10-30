import assert from "node:assert/strict";
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

test("does not rewrite an unrelated closing-tag edit", () => {
  assert.equal(rename("<title>Example</title>", 16, 21, "heading"), "<title>Example</heading>");
});

test("preserves an already mismatched closing tag", () => {
  assert.equal(rename("<title>Example</heading>", 1, 6, "article-title"), "<article-title>Example</heading>");
});
