import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

test("Tauri and Vite use the same loopback host", async () => {
  const config = JSON.parse(await readFile(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"));
  assert.equal(config.build.devUrl, "http://127.0.0.1:1420");
});

test("the desktop entrypoint provides startup feedback before loading the application", async () => {
  const index = await readFile(new URL("../index.html", import.meta.url), "utf8");
  assert.match(index, /<title>SciXML<\/title>/);
  assert.match(index, /branding\/scixml-logo\.png/);
  assert.match(index, /Carregando a aplicação/);
  assert.ok(index.indexOf("/src/bootstrap.ts") < index.indexOf("/src/main.ts"));
});
