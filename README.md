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
  and SPS 1.10/JATS 1.3 as separate grammar and rules stages. External JATS/SPS
  assets remain user-supplied and are not redistributed by O2XML. A bounded
  JATS Inspector recognizes the editorial structure and provides navigable
  source positions in the desktop UI. RELAX NG XML validation runs through the
  isolated libxml2 helper and the same offline catalog broker used by DTD/XSD;
  self-contained compact syntax has an optional supervised Jing adapter, while
  RNC resource directives remain denied pending a fully brokered resolver. The
  Schematron core now normalizes bounded SVRL reports into navigable diagnostics;
  an isolated libxml2 baseline executes XPath 1.0 assertions. A supervised
  SchXslt/Saxon two-step adapter compiles self-contained ISO Schematron to XSLT
  and normalizes the resulting SVRL, with private staging, deadlines, bounded
  output, and external-resource expressions denied before execution. Project framework
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
- Phase 5: planned; Schematron/JATS/SPS validation is not presented as complete
  while licensed upstream packs and conformance corpora are not installed.

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
path until platform bundle assembly is automated.

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

A debug macOS bundle can be produced with:

```sh
cd app
npx tauri build --debug --bundles app
```

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
