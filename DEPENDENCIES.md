# Dependency policy and initial inventory

Dependencies are pinned by lockfiles for applications and reviewed before each
phase gate. CI will add `cargo audit`, `cargo deny`, npm audit, and license checks.

## Phase 1 Rust

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
| `tokio` | Tasks, timers, process supervision | MIT | Background validation and future adapters |
| `criterion` (dev) | Benchmarks | MIT OR Apache-2.0 | Repeatable performance tracking |

## Phase 1 frontend

| Dependency | Purpose | License | Rationale / constraint |
|---|---|---|---|
| CodeMirror 6 packages | Editor/state/view/XML/search/autocomplete | MIT | Professional incremental editor primitives |
| `@tauri-apps/api` / dialog plugin | Narrow desktop bridge | MIT OR Apache-2.0 | No broad filesystem API exposed to webview |
| TypeScript | Static UI types | Apache-2.0 | IPC contracts and maintainability |
| Vite | Build/dev server | MIT | Build-time only; dev server never shipped |

## Future external engines

libxml2/libxslt (MIT), Saxon HE (MPL-2.0), Jing/Trang (BSD-like), and the
Xerces2-J XSD 1.1 distribution (Apache-2.0) are optional adapters. Their binaries,
notices, transitive dependencies, supported versions, and platform packaging are
reviewed before inclusion. Saxon PE/EE is commercial and is never silently
bundled or treated as open source.

Schemas and framework resources are dependencies too. Their asset licenses and
checksums live in framework manifests; the project source license does not
override them.
