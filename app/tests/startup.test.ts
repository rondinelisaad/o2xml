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

test("the application starts with an empty workspace instead of creating an untitled XML", async () => {
  const source = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");
  assert.match(source, /async function initializeWorkspace\(\): Promise<void> \{\s*showEmptyWorkspace\(\);/);
  assert.match(source, /app\.dataset\.initialized = "true";/);
  assert.match(source, /void initializeWorkspace\(\);/);
  assert.doesNotMatch(source, /app\.dataset\.initialized = "true";\s*void createNew\(\);/);
});

test("the application installs a native close guard before the workspace is initialized", async () => {
  const source = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");
  const capability = JSON.parse(
    await readFile(new URL("../src-tauri/capabilities/default.json", import.meta.url), "utf8"),
  );
  assert.match(source, /onCloseRequested\(async \(event\) => \{\s*event\.preventDefault\(\);/);
  assert.ok(capability.permissions.includes("core:window:allow-destroy"));
  assert.ok(source.indexOf("void installApplicationCloseGuard()") < source.indexOf("void initializeWorkspace()"));
});

test("the main and detached windows can be resized for side-by-side comparison", async () => {
  const config = JSON.parse(await readFile(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"));
  const source = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");
  const stylesheet = await readFile(new URL("../src/styles.css", import.meta.url), "utf8");

  assert.ok(config.app.windows[0].minWidth <= 420);
  assert.match(source, /minWidth: 420,/);
  assert.match(stylesheet, /grid-template-columns: min\(var\(--sidebar-width\), 40vw\)/);
});

test("the Project sidebar panel uses the same accessible collapse control as the other panels", async () => {
  const source = await readFile(new URL("../src/main.ts", import.meta.url), "utf8");
  const stylesheet = await readFile(new URL("../src/styles.css", import.meta.url), "utf8");

  for (const panelId of ["project", "outline", "jats", "framework"]) {
    assert.match(source, new RegExp(`data-sidebar-panel="${panelId}"`, "u"));
  }
  assert.match(source, /data-sidebar-panel="project">\s*<h2><button[^>]+class="sidebar-panel-toggle"[^>]+aria-expanded="true"/u);
  assert.match(source, /toggle\.setAttribute\("aria-expanded", String\(!collapsed\)\)/u);
  assert.match(source, /scixml\.layout\.collapsedPanels/u);
  assert.match(stylesheet, /\.sidebar section\.collapsed \.sidebar-panel-chevron/u);
});
