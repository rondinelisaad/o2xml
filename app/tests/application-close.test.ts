import assert from "node:assert/strict";
import test from "node:test";

import { approveApplicationClose } from "../src/application-close.ts";

interface TestDocument {
  name: string;
  dirty: boolean;
}

function closeOptions(
  documents: TestDocument[],
  decision: "save" | "discard" | "cancel",
  saved: string[],
  failOn?: string,
) {
  let promptCount = 0;
  return {
    options: {
      documents,
      isDirty: (document: TestDocument) => document.dirty,
      decide: async (dirtyCount: number) => {
        promptCount = dirtyCount;
        return decision;
      },
      save: async (document: TestDocument) => {
        saved.push(document.name);
        if (document.name === failOn) return false;
        document.dirty = false;
        return true;
      },
    },
    promptCount: () => promptCount,
  };
}

test("closes without prompting when no document was changed", async () => {
  const saved: string[] = [];
  const scenario = closeOptions([{ name: "article.xml", dirty: false }], "save", saved);

  assert.equal(await approveApplicationClose(scenario.options), true);
  assert.equal(scenario.promptCount(), 0);
  assert.deepEqual(saved, []);
});

test("saves every changed document before closing", async () => {
  const documents = [
    { name: "article.xml", dirty: true },
    { name: "schema.xsd", dirty: false },
    { name: "translation.xml", dirty: true },
  ];
  const saved: string[] = [];
  const scenario = closeOptions(documents, "save", saved);

  assert.equal(await approveApplicationClose(scenario.options), true);
  assert.equal(scenario.promptCount(), 2);
  assert.deepEqual(saved, ["article.xml", "translation.xml"]);
});

test("keeps the application open when the user cancels", async () => {
  const saved: string[] = [];
  const scenario = closeOptions([{ name: "article.xml", dirty: true }], "cancel", saved);

  assert.equal(await approveApplicationClose(scenario.options), false);
  assert.deepEqual(saved, []);
});

test("closes without saving when the user discards the changes", async () => {
  const saved: string[] = [];
  const scenario = closeOptions([{ name: "article.xml", dirty: true }], "discard", saved);

  assert.equal(await approveApplicationClose(scenario.options), true);
  assert.deepEqual(saved, []);
});

test("aborts closing after a save is cancelled or fails", async () => {
  const documents = [
    { name: "article.xml", dirty: true },
    { name: "translation.xml", dirty: true },
    { name: "supplement.xml", dirty: true },
  ];
  const saved: string[] = [];
  const scenario = closeOptions(documents, "save", saved, "translation.xml");

  assert.equal(await approveApplicationClose(scenario.options), false);
  assert.deepEqual(saved, ["article.xml", "translation.xml"]);
});
