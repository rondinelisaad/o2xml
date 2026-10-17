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
- Phases 2–5: planned; DTD/XSD/Catalog/Schematron/JATS/SPS validation is not
  presented as implemented yet.

## Build and run

Prerequisites are the stable Rust toolchain, Node.js/npm, and the
[Tauri platform prerequisites](https://v2.tauri.app/start/prerequisites/).

```sh
cd app
npm install
npm run tauri dev
```

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
