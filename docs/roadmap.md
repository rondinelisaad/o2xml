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
- [x] Broker XSD index construction through the isolated libxml2 helper, with
  catalog-only resources, graph/output/deadline limits, audited execution, and
  conservative global element declarations from libxml2's compiled schema.
- [x] Enrich inline and resolved named XSD types with local sequences, choices,
  element references, bounded occurrences, attribute uses, and value constraints.
  Imported type nodes remain inside the already brokered libxml2 schema graph.
- [x] Expand same-document attribute groups and preserve enumeration facets,
  with cycle/depth/count limits and conservative fallback when a reference
  cannot be projected safely.
- [x] Expose project-bound XSD element, attribute, and enumeration completion
  through document schema associations, the offline catalog, a bounded cache,
  and an audited asynchronous desktop service.
- [x] Expand same-document model groups and preserve exact `xs:all` unordered,
  single-occurrence semantics with bounded state construction.
- [x] Expand same-document complex type extensions across bounded inheritance
  chains, combining base/derived content and inherited attribute declarations.
- [x] Resolve cross-document model groups and complex base types exclusively
  from the bounded resource graph already approved by the offline broker.
- [x] Preserve namespace-constrained wildcards in content automata, consuming
  only matching expanded names and suggesting only known eligible declarations.

Phase 2 implementation status: complete. Remaining standards expansion belongs
to the versioned engines and framework work in Phases 3 and 4.

Gate: relevant standards/conformance fixtures pass, offline catalog scenarios are
demonstrated, and all resource-policy denial tests pass.

## Phase 3 — scientific publishing

Add RNG/RNC, ISO Schematron/SVRL, JATS inspector, and versioned SPS 1.9/1.10
framework manifests. Investigate SPS 1.11/JATS 1.4 as a compatible manifest,
without changing the committed baseline silently.

Current progress:

- [x] Add a bounded, serializable framework registry with explicit grammar and
  Schematron rule stages for SPS 1.9/JATS 1.1 and SPS 1.10/JATS 1.3.
- [x] Mark JATS/SPS assets as externally required so licensed or third-party
  resources are never silently redistributed.
- [x] Add a bounded JATS structural inspector with editorial labels, source
  positions, and synchronized desktop navigation.
- [x] Add supervised RELAX NG XML validation through isolated libxml2 with the
  offline catalog broker, dependency limits, deadlines, and root-escape denial.
- [x] Add self-contained RELAX NG compact syntax through an optional,
  policy-controlled Jing adapter with private staging, deadlines, bounded
  diagnostics, and external resource directives denied by default.
- [ ] Add a brokered Jing resolver before permitting RNC `include`/`external`,
  and run the upstream conformance corpus with the real user-installed JAR.
- [x] Add bounded SVRL parsing for failed assertions and successful reports,
  severity metadata, rule identity, and navigable diagnostic normalization.
- [x] Add a catalog-brokered, process-isolated libxml2 Schematron baseline for
  immediate XPath 1.0 assertions with bounded structured diagnostics.
- [x] Add a supervised SchXslt/Saxon two-step pipeline for self-contained ISO
  Schematron: private staging, explicit trusted assets, deadlines, bounded
  stderr/output, external-resource preflight, and normalized SVRL diagnostics.
- [ ] Replace Schematron expression preflight with an engine-level brokered URI
  resolver before permitting includes, imports, or document/collection access.
- [x] Wire project framework selection and catalog-installed assets into the
  desktop validation service, preserving separate JATS grammar and SPS rule
  stages with passed/failed/unavailable state, labeled diagnostics, and visible
  engine provenance. A complete trusted host configuration selects SchXslt/Saxon;
  absence selects the explicitly identified libxml2 baseline.
- [x] Add synthetic golden fixtures and record SPS 1.11/JATS 1.4 as an explicit
  candidate manifest without changing the stable SPS 1.9/1.10 baselines.

Gate: JATS grammar and SPS rules are reported separately; licensed resources are
installed lawfully; golden diagnostics pass on synthetic articles.

## Phase 4 — transformation

Add libxslt and Saxon HE adapters, transformation scenarios, safe HTML preview,
and XQuery if the Saxon process contract proves portable.

Current progress:

- [x] Add a reusable Rust transformation core and supervised Saxon HE process
  adapter for self-contained XSLT, with private staging, DTD/external resource
  denial, deadlines, bounded stderr, and accepted-output limits.
- [ ] Add the isolated libxslt XSLT 1.0 adapter behind the catalog broker.
- [ ] Add project transformation scenarios and desktop execution/output panels.
- [ ] Add a safe HTML preview that never grants transformed content application
  privileges or ambient network access.
- [ ] Evaluate the Saxon XQuery process contract separately; do not infer safety
  from the XSLT adapter.

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
