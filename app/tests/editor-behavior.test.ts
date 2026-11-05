import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

import { xml } from "@codemirror/lang-xml";
import { EditorState } from "@codemirror/state";

import { linkedClosingTagChange } from "../src/editor-behavior.ts";
import {
  attributeApplication,
  attributeTextContext,
  isAttributeCompletionPosition,
  otherAttributeNames,
} from "../src/editor-completion.ts";
import { detachedDocumentRequest, isOutsideWindowDrop } from "../src/windowing.ts";

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

test("accepts only bounded identifiers for a detached document window", () => {
  assert.deepEqual(
    detachedDocumentRequest("?detachedDocument=42&sourceWindow=main&readyEvent=scixml-document-42-token-ready"),
    { documentId: 42, sourceLabel: "main", readyEvent: "scixml-document-42-token-ready" },
  );
  assert.equal(detachedDocumentRequest("?detachedDocument=../../etc&sourceWindow=main&readyEvent=bad"), null);
});

test("detaches a tab only when the drag finishes outside the source window", () => {
  const bounds = { screenX: 100, screenY: 100, outerWidth: 1000, outerHeight: 700 };
  assert.equal(isOutsideWindowDrop({ screenX: 500, screenY: 400 }, bounds), false);
  assert.equal(isOutsideWindowDrop({ screenX: 1200, screenY: 400 }, bounds), true);
  assert.equal(isOutsideWindowDrop({ screenX: 0, screenY: 0 }, bounds), false);
});

test("places the cursor inside a newly completed attribute value", () => {
  assert.deepEqual(attributeApplication("fn-type", null), {
    insert: 'fn-type=""',
    valueFromOffset: 9,
    valueToOffset: 9,
  });
  assert.deepEqual(attributeApplication("article-type", "research-article"), {
    insert: 'article-type="research-article"',
    valueFromOffset: 14,
    valueToOffset: 30,
  });
});

test("keeps the edited attribute available while excluding the other existing attributes", () => {
  const attributes = [
    { name: "id", from: 9, to: 16 },
    { name: "fn-type", from: 17, to: 41 },
  ];
  assert.deepEqual(otherAttributeNames(attributes, { from: 17, to: 41 }), ["id"]);
  assert.deepEqual(otherAttributeNames(attributes, null), ["id", "fn-type"]);
});

test("automatically requests completion while typing an attribute name or value", () => {
  const attributeName = '<fn fn></fn>';
  assert.equal(isAttributeCompletionPosition(attributeName, attributeName.indexOf(">")), true);
  const attributeValue = '<fn fn-type="data"></fn>';
  assert.equal(isAttributeCompletionPosition(attributeValue, attributeValue.indexOf('"', 13)), true);
  assert.equal(isAttributeCompletionPosition("<fn>text", 8), false);
  assert.equal(isAttributeCompletionPosition("</fn", 4), false);
});

test("recovers attribute completion context from temporarily invalid XML", () => {
  const nameSource = '<body><fn f></fn></body>';
  assert.deepEqual(attributeTextContext(nameSource, nameSource.indexOf(">", 10)), {
    elementName: "fn",
    presentAttributes: [],
    editingName: { from: 10, to: 11 },
    value: null,
  });

  const valueSource = '<fn id="n1" fn-type="data"></fn>';
  const valuePosition = valueSource.indexOf("data") + 4;
  assert.deepEqual(attributeTextContext(valueSource, valuePosition), {
    elementName: "fn",
    presentAttributes: ["id"],
    editingName: null,
    value: { attributeName: "fn-type", from: valueSource.indexOf("data"), to: valuePosition },
  });
});
