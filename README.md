# O2XML

O2XML is an independent, open-source XML workbench focused on standards-based
editing and scientific publishing workflows. Its first-class publishing targets
are JATS and the SciELO Publishing Schema (SPS).

The project is intentionally incremental. Phase 0 records the architecture,
engine choices, security model, and delivery roadmap. Phase 1 builds a usable
desktop MVP with a Rust core and a Tauri/CodeMirror interface.

## Status

- Phase 0: architecture and technical investigation documented.
- Phase 1: desktop MVP implemented and verified on macOS.
- Phase 2: complete; offline XML Catalog resolution, isolated libxml2 DTD/XSD
  validation, XPath 1.0 with a desktop query panel, and the common schema
  content-model index are implemented. Normalized DTD particles can already be
  compiled for completion and extracted through the isolated libxml2 helper;
  offline catalog brokering and context-sensitive desktop completion are wired.
  Normalized XSD 1.0 declarations now compile through the same bounded particle
  engine, and global declarations are extracted through the catalog-brokered
  libxml2 helper. Local sequence/choice particles, occurrences, references, and
  attribute uses are projected from libxml2-resolved types. Same-document
  attribute groups, model groups, exact `all`, constrained wildcards, inherited
  content, and cross-document groups are supported.
- Phase 3: in progress; the bounded framework registry models SPS 1.9/JATS 1.1
  and SPS 1.10/JATS 1.3 as separate grammar and rules stages. The public-domain
  official JATS 1.1 Publishing DTD is bundled for automatic offline validation
  of matching standalone documents; other JATS versions and SPS rule packs
  remain explicitly installed project resources. A bounded
  JATS Inspector recognizes the editorial structure and provides navigable
  source positions in the desktop UI. RELAX NG XML validation runs through the
  isolated libxml2 helper and the same offline catalog broker used by DTD/XSD;
  compact syntax has an optional supervised Jing adapter. RNC `include` and
  `external` references are recursively resolved through the offline catalog,
  bounded, rewritten into private staging, and denied on root escape. The
  Schematron core now normalizes bounded SVRL reports into navigable diagnostics;
  an isolated libxml2 baseline executes XPath 1.0 assertions. A supervised
  SchXslt/Saxon two-step adapter compiles ISO Schematron to XSLT and normalizes
  the resulting SVRL. Static Schematron and XSLT includes/imports are recursively
  resolved through the offline catalog and rewritten into bounded private staging;
  an embedded JAXP launcher forces Saxon, disables external functions/protocols,
  and installs fail-closed compile/runtime URI resolvers. Project framework
  selection now runs JATS grammar and SPS rules as separate visible stages using
  only assets explicitly installed through the project catalog. SPS 1.11/JATS
  1.4 is exposed only as `SPS-1.11-candidate`, never as a silent default.
- Phase 4: complete for the committed safe profiles; a reusable transformation core and supervised Saxon HE
  adapter execute self-contained XSLT with private staging, deadlines, bounded
  accepted output, and denied external/dynamic resource access. Project
  manifests now define typed transformation scenarios with explicit engines,
  media types, and safe project-relative stylesheet/output paths. The desktop
  can run the selected scenario, atomically publish its output below the project
  root, and show a bounded text preview with engine provenance.
  A separate supervised Saxon HE adapter now evaluates the DTD-free,
  self-contained XQuery 3.1 profile; external modules/resources, environment
  access, extensions, and updating expressions are denied. The desktop XQuery
  panel runs the current revision and presents a bounded inert-text result.
- Phase 5: in progress; the bounded XML-aware diff core compares expanded names
  and sorted attributes, preserves text significance by default, reports source
  spans, and falls back to a coarse bounded hunk for large comparison matrices.
  The desktop presents navigable side-by-side hunks and an explicit whole-file
  “use right” merge that remains undoable in the editor. Individual hunks can
  also be applied after the core rebuilds and reparses the resulting XML.
  XML Grid detects repeated sibling structures, flattens bounded attributes and
  scalar descendants into columns, and navigates each row back to the source.
  Unambiguous scalar text cells can now be edited transactionally: markup is
  escaped, the full document is reparsed, configured project framework stages
  must pass, stale revisions are rejected, and the editor receives one undoable
  source-of-truth change. Attributes, repeated values, and mixed XML stay
  read-only.
  The versioned plugin contract now validates checksummed WASI/process packages,
  explicit capabilities, project-relative filesystem scopes, HTTPS origins,
  contributions, deny-by-default host authorization, and streaming SHA-256
  verification of bounded entrypoints before they can become executable inputs.
  Supervised process plugins can run only through an explicitly configured OS
  sandbox launcher, with private verified staging, JSON envelopes, cleared
  environment, deadlines, bounded I/O, and content-free audit events.
  The embedded Wasmtime 48 LTS adapter executes the same correlated JSON
  protocol through a Component Model root export, with no host imports, fresh
  bounded stores, fuel, epoch deadlines, memory/table/instance limits, and a
  second checksum gate immediately before compilation.
  Remote resources remain opt-in: the Rustls connector enforces exact HTTPS
  origin allowlists, rejects private/local/reserved DNS results, pins each
  connection to the approved addresses, manually revalidates every redirect,
  disables proxies, and bounds DNS/connect/request time plus response size.
  A least-privilege WebDAV adapter adds bounded `PROPFIND`/GET and conditional
  PUT under one configured root. Passwords are redacted and zeroed on drop;
  destructive methods and unconditional overwrite are deliberately absent.
  A separate BaseX REST adapter retrieves one XML document with GET under a
  fixed database scope. It accepts only non-administrative, transient
  credentials and exposes no query, command, mutation, or redirect surface.
  Author Mode now projects bounded mixed XML into a source-mapped structural
  desktop view. Plain-text runs are edited transactionally, escaped, reparsed,
  checked against the active project framework, revision-gated, and applied as
  one undoable editor change. CDATA and entity-reference runs remain visible
  but read-only; structural editing and track changes are not yet claimed.
  SPS validation is not presented as complete while its rule pack and real
  external-engine conformance corpora are not installed. JATS and SPS results
  remain visibly separate.

## Build and run

Prerequisites are the stable Rust toolchain, Node.js/npm, libxml2 2.9+,
libxslt 1.1+, `pkg-config`, and the
[Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/).
Java, Saxon HE, SchXslt, and Jing are optional user-installed engines; O2XML
never downloads or silently bundles them.

```sh
cargo build -p xml-libxml2-helper
cd app
npm install
npm run tauri dev
```

The desktop locates the helper in the workspace build output or beside the app
executable. Packagers can set `O2XML_LIBXML2_HELPER` to an explicit executable
path. Release bundles stage the target-specific helper automatically.

The bundled helper provides catalog-brokered XSLT 1.0 through libxslt. To opt
into XSLT 3.0, set trusted host variables `O2XML_JAVA` and
`O2XML_SAXON_JAR` to absolute local file paths. Conforming ISO Schematron also
requires `O2XML_SCHXSLT_PIPELINE`. The framework panel identifies the selected engine.
When they are absent, O2XML labels and uses the isolated libxml2 XPath 1.0
baseline; a partial configuration makes the stage unavailable instead of
silently selecting a mixed setup.

Quality gates:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd app && npm run build
```

The external Jing conformance gate is reproducible without committing its JAR:
build upstream tag `V20241231` at its signed release commit, then set `O2XML_JING_UPSTREAM_ROOT`,
`O2XML_JING_JAR`, and `O2XML_JAVA` and run the ignored `jing_upstream` test.
See [docs/conformance.md](docs/conformance.md) for the exact contract and the
recorded result.

A debug macOS bundle can be produced with:

```sh
cd app
npx tauri build --debug --bundles app
```

The manual GitHub Actions release workflow builds draft installers for Linux,
Windows, macOS Apple Silicon, and macOS Intel. See
[docs/release.md](docs/release.md) for local packaging, CI operation, signing,
smoke tests, and publication requirements.

## Repository layout

```text
app/                 Tauri desktop shell and TypeScript UI
crates/              Reusable Rust libraries
docs/                Requirements, architecture, roadmap, ADRs
testdata/             Safe, synthetic XML fixtures
tests/                Cross-crate and end-to-end tests
```

## Principles

- XML text is the source of truth.
- UI, application services, and XML engines are separate layers.
- External resources are denied unless a policy explicitly allows them.
- Mature standards engines are integrated instead of reimplemented.
- JATS validity and SPS conformance are distinct validation stages.

See [docs/architecture.md](docs/architecture.md) and
[docs/roadmap.md](docs/roadmap.md) for the current design.

## License

Source code is dual-licensed under [MIT](LICENSE-MIT) or
[Apache-2.0](LICENSE-APACHE). Third-party schemas, rule sets, and documentation
retain their own licenses and are not implicitly covered by the project license.
