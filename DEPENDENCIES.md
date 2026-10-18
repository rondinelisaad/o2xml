# Dependency policy and initial inventory

Dependencies are pinned by lockfiles for applications and reviewed before each
phase gate. CI will add `cargo audit`, `cargo deny`, npm audit, and license checks.

## Rust dependencies

| Dependency | Purpose | License | Rationale / constraint |
|---|---|---|---|
| `tauri` 2 | Desktop lifecycle and typed IPC | MIT OR Apache-2.0 | Cross-platform Rust core; capabilities kept minimal |
| `tauri-plugin-dialog` 2 | Native open/save dialog | MIT OR Apache-2.0 | Avoid custom platform dialog code |
| `quick-xml` | Streaming XML events | MIT | Mature, fast Phase 1 well-formedness/outline/format foundation |
| `ropey` | Large editable UTF-8 text | MIT | Logarithmic edits/queries and cheap thread-safe snapshots |
| `serde`, `serde_json` | IPC/project data | MIT OR Apache-2.0 | Ubiquitous typed serialization; input limits still required |
| `toml` | Project manifest | MIT OR Apache-2.0 | Human-editable `project.toml` |
| `thiserror` | Library errors | MIT OR Apache-2.0 | Typed errors without exposing application context |
| `anyhow` | Binary composition errors | MIT OR Apache-2.0 | Context at executable boundary only |
| `tracing`, `tracing-subscriber` | Structured telemetry | MIT | Allowlisted fields; no document content or secrets |
| `tokio` 1.53.3 | Supervised engine processes, bounded I/O, deadlines | MIT | Already transitively used by Tauri; explicit minimal features for validation adapters |
| `criterion` (dev) | Benchmarks | MIT OR Apache-2.0 | Repeatable performance tracking |
| `url` 2.5.8 | URI parsing and relative-reference resolution | MIT OR Apache-2.0 | Active rust-url project; avoids ad hoc catalog URI normalization |
| `pkg-config` 0.3 | Build-time discovery of system libxml2 | MIT OR Apache-2.0 | Small, established build helper; never shipped as application runtime code |
| `tempfile` 3.27 | Private staging files for supervised JVM engines | MIT OR Apache-2.0 | Mature cross-platform implementation; named files default to owner-only permissions and are deleted on drop |

`xml-transform` reuses the reviewed `quick-xml`, `tempfile`, `tokio`,
`thiserror`, and `tracing` set; it introduces no additional third-party crate.
Its Saxon adapter requires an explicit user-installed HE JAR.

`xml-diff` reuses `quick-xml`, `serde`, `thiserror`, and the existing document,
domain, and syntax crates. It adds no third-party dependency or native engine;
the syntax crate performs the full post-merge well-formedness gate.

`xml-grid` reuses `quick-xml`, `serde`, `thiserror`, and the existing document/
domain crates for a bounded read-only projection. It adds no third-party
dependency or native engine.

`xml-plugin-api` reuses `serde`, `serde_json`, `thiserror`, and `url` for its
versioned inert protocol, manifest validation, and HTTPS-origin checks. It does
not yet include or execute a WASI runtime.

## Phase 1 frontend

| Dependency | Purpose | License | Rationale / constraint |
|---|---|---|---|
| CodeMirror 6 packages | Editor/state/view/XML/search/autocomplete | MIT | Professional incremental editor primitives |
| `@tauri-apps/api` / dialog plugin | Narrow desktop bridge | MIT OR Apache-2.0 | No broad filesystem API exposed to webview |
| TypeScript | Static UI types | Apache-2.0 | IPC contracts and maintainability |
| Vite | Build/dev server | MIT | Build-time only; dev server never shipped |

## Future external engines

libxml2 2.9+ and libxslt 1.1+ (MIT) are now required to build the isolated
DTD/XSD/RELAX NG/Schematron/XSLT 1.0 helper. The Rust core does not link to
either C library: the helper is a separate process
and all grammar bytes are brokered through the native catalog policy. Jing
(3-clause BSD) is an optional, user-installed adapter for self-contained RNC;
the supported baseline is upstream V20241231 and its JAR is never downloaded
silently. SchXslt (MIT) plus user-installed Saxon HE (MPL-2.0) now provide the
optional, supervised ISO Schematron-to-SVRL pipeline for self-contained rule
sets. The reviewed SaxonJ baseline is 13.0 (released 2026-05-29); O2XML accepts
an explicit local JAR and trusted `pipeline-for-svrl.xsl` path rather than
downloading either asset. The Xerces2-J XSD 1.1 distribution (Apache-2.0)
remains a planned optional adapter. Engine binaries, notices,
transitive dependencies, supported versions, and platform packaging are reviewed
before inclusion. Saxon PE/EE is commercial and is never silently bundled or
treated as open source.

Schemas and framework resources are dependencies too. Their asset licenses and
checksums live in framework manifests; the project source license does not
override them.
