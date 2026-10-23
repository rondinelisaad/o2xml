# Dependency policy and initial inventory

Dependencies are pinned by lockfiles for applications and reviewed before each
phase gate. CI will add `cargo audit`, `cargo deny`, npm audit, and license checks.

## Rust dependencies

| Dependency | Purpose | License | Rationale / constraint |
|---|---|---|---|
| `base64` 0.23.1 | Encode approved local preview images as self-contained data URLs | MIT OR Apache-2.0 | Mature no-I/O codec; default SIMD/unsafe feature disabled, only bounded bytes already confined by the Rust filesystem broker are encoded |
| `tauri` 2 | Desktop lifecycle and typed IPC | MIT OR Apache-2.0 | Cross-platform Rust core; capabilities kept minimal |
| `tauri-plugin-dialog` 2 | Native open/save dialog | MIT OR Apache-2.0 | Avoid custom platform dialog code |
| `quick-xml` | Streaming XML events | MIT | Mature, fast Phase 1 well-formedness/outline/format foundation |
| `regex` 1 | EXSLT `regexp:test` compatibility for the bundled SPS Schematron rules | MIT OR Apache-2.0 | Mature Rust implementation; patterns come only from the bundled, reviewed Packtools stylesheets and execute inside the isolated helper |
| `reqwest` 0.13.5 + Rustls | Opt-in pinned HTTPS resource retrieval | MIT OR Apache-2.0 | Redirects/proxies disabled; per-hop DNS results are policy-checked and pinned; certificate and hostname verification remain enabled |
| `ropey` | Large editable UTF-8 text | MIT | Logarithmic edits/queries and cheap thread-safe snapshots |
| `serde`, `serde_json` | IPC/project data | MIT OR Apache-2.0 | Ubiquitous typed serialization; input limits still required |
| `sha2` 0.11.1 | SHA-256 plugin entrypoint integrity | MIT OR Apache-2.0 | Maintained RustCrypto implementation; integrity only, never password hashing |
| `toml` | Project manifest | MIT OR Apache-2.0 | Human-editable `project.toml` |
| `thiserror` | Library errors | MIT OR Apache-2.0 | Typed errors without exposing application context |
| `anyhow` | Binary composition errors | MIT OR Apache-2.0 | Context at executable boundary only |
| `tracing`, `tracing-subscriber` | Structured telemetry | MIT | Allowlisted fields; no document content or secrets |
| `tokio` 1.53.3 | Supervised engine processes, bounded I/O, deadlines | MIT | Already transitively used by Tauri; explicit minimal features for validation adapters |
| `criterion` (dev) | Benchmarks | MIT OR Apache-2.0 | Repeatable performance tracking |
| `url` 2.5.8 | URI parsing and relative-reference resolution | MIT OR Apache-2.0 | Active rust-url project; avoids ad hoc catalog URI normalization |
| `zeroize` 1.9 | Clear transient WebDAV passwords on drop | Apache-2.0 OR MIT | Small audited primitive; credentials are neither serialized nor logged |
| `pkg-config` 0.3 | Build-time discovery of system libxml2 | MIT OR Apache-2.0 | Small, established build helper; never shipped as application runtime code |
| `vcpkg` 0.2 | Build-time discovery of libxml2/libxslt on Windows MSVC | MIT OR Apache-2.0 | Uses the GitHub runner's pinned native dependency installation; never shipped as application runtime code |
| `tempfile` 3.27 | Private staging files for supervised JVM engines | MIT OR Apache-2.0 | Mature cross-platform implementation; named files default to owner-only permissions and are deleted on drop |
| `wasmtime` 48.0.6 LTS | Embedded WebAssembly Component Model plugin runtime | Apache-2.0 WITH LLVM-exception | Minimal Component Model/Cranelift/runtime features; no WASI host imports in the initial profile; bounded memory, fuel, instances, and deadline |
| `wat` 1.262 (dev) | Compile the Component Model integration fixture | Apache-2.0 WITH LLVM-exception | Test-only parser; not linked into production artifacts |

`xml-transform` reuses the reviewed `quick-xml`, `tempfile`, `tokio`,
`thiserror`, and `tracing` set; it introduces no additional third-party crate.
Its Saxon adapter requires an explicit user-installed HE JAR.

`xml-diff` reuses `quick-xml`, `serde`, `thiserror`, and the existing document,
domain, and syntax crates. It adds no third-party dependency or native engine;
the syntax crate performs the full post-merge well-formedness gate.

`xml-grid` reuses `quick-xml`, `serde`, `thiserror`, and the existing document,
domain, and syntax crates for a bounded projection and full-document
well-formedness gate. It adds no third-party dependency or native engine.

`xml-author` reuses the same reviewed `quick-xml`, `serde`, `thiserror`,
document, domain, and syntax dependencies for its bounded projection, source
mapping, escaping, and full-document reparse gate. It adds no third-party
dependency or rendering engine.

`xml-resource` uses the async-only `reqwest` Rustls profile for explicit GET
operations. This adds Hyper, Rustls, the platform certificate verifier, and the
AWS-LC cryptographic provider, increasing build time and binary size. Compression,
cookies, proxies, automatic redirects, authentication helpers, and HTTP/3 are
not enabled by O2XML. TLS certificate and hostname checks are never bypassed.
Its WebDAV adapter also uses `zeroize` for transient Basic-auth passwords; no
credential store or configuration serialization is provided.
The BaseX REST reader reuses exactly this reviewed transport and dependency set.
It is an optional client integration, so BaseX itself is not linked or bundled;
upstream BaseX is BSD-3-Clause. The adapter implements only bounded GET document
retrieval and requires a separately provisioned read-only server account.

`xml-plugin-api` reuses `serde`, `serde_json`, `thiserror`, and `url` for its
versioned inert protocol, manifest validation, and HTTPS-origin checks. It uses
RustCrypto `sha2` 0.11.1 to stream and verify the declared SHA-256 of a bounded
entrypoint before returning a verified handle. SHA-256 is used only for file
integrity, not credentials. Wasmtime 48.0.6 LTS provides the embedded Component
Model runtime. The initial profile intentionally exposes no WASI imports or
other host functions: a component receives one bounded JSON string and returns
one bounded JSON string. `wat` is used only to build an executable test fixture.

The supervised plugin adapter also reuses the existing `tokio`, `tempfile`, and
`tracing` dependencies for bounded asynchronous pipes, private staging,
deadlines, child termination, and content-free audit events. It requires a
separately configured OS sandbox launcher and never executes an entrypoint
directly.

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
(3-clause BSD) is an optional, user-installed adapter for RNC; its recursive
resource graph is catalog-resolved, bounded, and rewritten into private staging
before execution. The supported baseline is upstream V20241231 and its JAR is
never downloaded silently. Its official compact-syntax corpus is covered by an
explicitly ignored, environment-gated test; see `docs/conformance.md`. SchXslt
(MIT) plus user-installed Saxon HE (MPL-2.0) now provide the
optional, supervised ISO Schematron-to-SVRL pipeline. Static rule-set dependencies
are catalog-resolved and rewritten into private staging before execution. A
project-owned Java source launcher uses only JAXP APIs, forces Saxon, and applies
the engine-level deny-all protocol and URI-resolver policy. The
reviewed SaxonJ baseline is 13.0 (released 2026-05-29); O2XML accepts
an explicit local JAR and trusted `pipeline-for-svrl.xsl` path rather than
downloading either asset. The Xerces2-J XSD 1.1 distribution (Apache-2.0)
remains a planned optional adapter. Engine binaries, notices,
transitive dependencies, supported versions, and platform packaging are reviewed
before inclusion. Saxon PE/EE is commercial and is never silently bundled or
treated as open source.

The official vcpkg libxml2 port currently builds the generic Schematron module
out and does not expose it as a feature. Windows bundles still validate the
built-in SPS 1.9 and 1.10 profiles because their reviewed ISO Schematron is
precompiled to XSLT 1.0 and executed by the isolated libxslt helper. Arbitrary
project Schematron continues to use the optional SchXslt/Saxon adapter.

Schemas and framework resources are dependencies too. Their asset licenses and
checksums live in framework manifests; the project source license does not
override them.

### NISO JATS 1.1 Journal Publishing DTD

- **Source:** NLM/NCBI, `JATS-Publishing-1-1-MathML2-DTD.zip`.
- **Upstream archive SHA-256:**
  `cf86eba5b8baa56462fd4e276a620f62e5b9775bc69bf658c9f788696afed354`.
- **Purpose:** offline JATS 1.1 grammar validation, including modular entity and
  MathML resources.
- **License/status:** public domain. The unmodified upstream files and README are
  bundled under `app/src-tauri/resources/jats/1.1`.
- **Security:** resources are resolved only through the offline XML Catalog
  broker; the parser helper retains network denial and resource limits.

### NISO JATS 1.3 Journal Publishing DTD

- **Source:** NLM/NCBI, `JATS-Publishing-1-3-MathML2-DTD.zip`.
- **Upstream archive SHA-256:**
  `7f0a960cf33f0d37849c298f39c778f49f9d9d4f6d8226ea415aef7a845c7cc2`.
- **Purpose:** offline JATS 1.3 grammar validation for SPS 1.10-era documents,
  including modular entity and MathML 2 resources.
- **License/status:** public domain. The unmodified upstream files are bundled
  under `app/src-tauri/resources/jats/1.3`; archive-only macOS metadata is not
  distributed.
- **Security:** resources are resolved only through the offline XML Catalog
  broker; the parser helper retains network denial and resource limits.

### Packtools HTML Generator 4.16.9

- **Source:** `scieloorg/packtools`, tag `4.16.9`, commit
  `a5455f82995d68bccc79365e3e0c5a023c70c7d4`.
- **Purpose:** offline SciELO article preview in Author Mode using the same
  `HTMLGenerator` XSLT/CSS generation layer selected by SPSValidator v0.3.0.
- **License:** BSD-2-Clause. The upstream license is bundled at
  `app/src-tauri/resources/packtools-html/4.16.9/LICENSE` and its notice must
  accompany binary distributions.
- **Bundled subset:** `root-html-2.0.xslt`, its `v2.0` XSLT/config resources,
  and `scielo-article-standalone.css`. O2XML's catalog is an integration file;
  no SPSValidator GPL source is included.
- **Security:** libxslt runs in the isolated helper; local reads require approval
  from the offline catalog broker, writes and network remain denied, parameters
  are bounded/quoted, and generated HTML is allowlist-sanitized before entering
  a scriptless sandboxed iframe. Referenced local raster images are separately
  canonicalized below the XML directory, signature-checked, bounded, base64-
  encoded, and admitted only through an exact generated-source map.

### Packtools SPS Schematron 4.16.9

- **Source:** `scieloorg/packtools`, tag `4.16.9`, using
  `scielo-style-1.9.sch`, `scielo-style-1.10.sch`, and `common.sch`.
- **Purpose:** automatic, offline SPS 1.9 and SPS 1.10 validation when the JATS
  root declares the corresponding `specific-use` value.
- **License:** BSD-2-Clause. The notice is distributed at
  `app/src-tauri/resources/sps/licenses/PACKTOOLS-BSD-2-CLAUSE.txt`.
- **Generated assets:** ISO Schematron was compiled to committed XSLT 1.0 at
  build preparation time; installed validation needs neither Python nor Java.
- **Security:** the XSLT is served only by the offline catalog broker and runs
  in the isolated helper with filesystem writes and network access denied.
