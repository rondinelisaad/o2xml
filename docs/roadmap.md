# Roadmap and gates

Every phase follows: plan, implement, compile, test, fix, run, document, commit.
A phase does not advance while its acceptance gate is red.

## Phase 0 — investigation

Deliverables: requirements, architecture, engine/dependency analysis, security
model, roadmap, and ADRs.

Gate:

- [x] twenty technical questions answered;
- [x] feature/engine/license/risk matrix recorded;
- [x] non-Rust dependencies and licensing risks identified;
- [x] threat model and default resource policy defined;
- [x] architecture preserves CLI/library reuse.

## Phase 1 — executable MVP

Deliver a Tauri desktop app with tabs, open/save, CodeMirror XML editing, line
numbers, folding, matching, auto-close, indentation, find/replace, formatting,
outline, well-formedness diagnostics, Problems/Output panels, basic project
manifest, themes, and revision-gated background validation.

Gate: Rust and frontend builds pass; unit/integration tests pass; the packaged or
development app opens a fixture, reports and navigates an error, formats, saves,
and reopens without data loss. No external resource is fetched.

## Phase 2 — professional XML

Add DTD, XSD 1.0, OASIS XML Catalog 1.1, XPath 1.0/3.1 adapters,
schema-aware completion, engine caches, and background validation hardening.

Current progress:

- [x] Native offline catalog core: `system`, `public`, `uri`, `rewriteSystem`,
  `rewriteURI`, guarded `nextCatalog`, cycle/size/count limits, and allowed-root
  enforcement.
- [x] Wire catalog resolution into an engine-neutral validation contract and a
  catalog-gated libxml2 process adapter for standalone DTD/XSD grammars, with
  deadlines and bounded diagnostics.
- [x] Preflight recursive XSD dependencies through the catalog policy, with
  aggregate byte/file limits and denial tests for root escape.
- [x] Replace XSD preflight and the standalone DTD guard with a libxml2 helper
  that installs a custom resource loader, closing the remaining TOCTOU window.
- [x] Add an XPath 1.0 query adapter and result-location contract, including
  namespace bindings, scalar values, result limits, and navigable node lines.
- [x] Expose XPath queries and results through the Tauri application service and
  the desktop XPath panel.
- [x] Define the common schema content-model index that will drive DTD/XSD
  element and attribute completion.
- [x] Compile normalized DTD particle trees into bounded deterministic content
  models, including sequence, choice, cardinality, mixed content, and attributes.
- [x] Extract normalized DTD declarations and attribute metadata through the
  isolated libxml2 helper without introducing a second DTD parser.
- [x] Broker DTD extraction through the offline catalog policy with process,
  resource-count, byte, output, and deadline limits plus sanitized audit events.
- [x] Expose project-bound, context-sensitive DTD element, attribute, and enum
  completion through an audited Tauri service and asynchronous CodeMirror source.
- [x] Compile normalized XSD 1.0 declarations into the common content-model
  index, including namespace-expanded element names, local sequences/choices,
  bounded occurrences, attribute uses, enumerations, and value constraints.
- [ ] Extract normalized XSD 1.0 declarations through the isolated libxml2
  helper, resolving global elements, local particles, types, groups, and imports.

Gate: relevant standards/conformance fixtures pass, offline catalog scenarios are
demonstrated, and all resource-policy denial tests pass.

## Phase 3 — scientific publishing

Add RNG/RNC, ISO Schematron/SVRL, JATS inspector, and versioned SPS 1.9/1.10
framework manifests. Investigate SPS 1.11/JATS 1.4 as a compatible manifest,
without changing the committed baseline silently.

Gate: JATS grammar and SPS rules are reported separately; licensed resources are
installed lawfully; golden diagnostics pass on synthetic articles.

## Phase 4 — transformation

Add libxslt and Saxon HE adapters, transformation scenarios, safe HTML preview,
and XQuery if the Saxon process contract proves portable.

Gate: filesystem/network sandbox tests pass and outputs are reproducible.

## Phase 5 — advanced workflows

Add Author Mode, XML Grid, text/XML-aware diff, merge, opt-in remote resources,
WebDAV/database adapters, and a WASI/process plugin ecosystem.

Author Mode is intentionally last: schema-preserving visual editing, selection
mapping, mixed content, track changes, and undo semantics are each substantial.

## Cross-cutting performance track

Criterion benchmarks use generated 1/10/50/100 MB fixtures first. A 500 MB gate
is introduced after streaming/lazy paths exist. Report median, p95, peak resident
memory, machine profile, and whether the targets were met. Do not claim a 16 ms
interaction or 100 ms completion target from unit timings alone.
