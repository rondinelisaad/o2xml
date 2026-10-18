import assert from "node:assert/strict";
import test from "node:test";

import {
  getLocale,
  initializeLocale,
  isLocale,
  persistLocale,
  t,
} from "../src/i18n.ts";

test("uses Brazilian Portuguese by default and interpolates parameters", () => {
  assert.equal(initializeLocale(null), "pt-BR");
  assert.equal(t("menu.open"), "Abrir…");
  assert.equal(t("grid.goToLine", { line: 42 }), "Ir para a linha 42");
});

test("restores and persists a supported locale", () => {
  assert.equal(initializeLocale({ getItem: () => "en" }), "en");
  assert.equal(t("menu.open"), "Open…");

  let saved = "";
  persistLocale("pt-BR", { setItem: (_key, value) => { saved = value; } });
  assert.equal(saved, "pt-BR");
  assert.equal(getLocale(), "pt-BR");
});

test("rejects unsupported stored values and tolerates unavailable storage", () => {
  assert.equal(isLocale("es"), false);
  assert.equal(initializeLocale({ getItem: () => "es" }), "pt-BR");
  assert.doesNotThrow(() => persistLocale("en", {
    setItem: () => { throw new Error("storage unavailable"); },
  }));
});
