# Architecture

## Decision summary

The desktop shell uses Tauri 2 and a TypeScript UI with CodeMirror 6. Rust owns
documents, persistence, validation orchestration, resource policy, diagnostics,
projects, frameworks, and engine adapters. CodeMirror owns interactive viewport
rendering and low-latency editing behavior, but its parse tree is never the
authoritative XML data model.

```text
CodeMirror UI (TypeScript)
        |
        | typed commands/events, revisions, compact edits
        v
Tauri application services (Rust)
        |
        +-- document/session/project services
        +-- validation coordinator + cancellation/revision gate
        +-- framework registry (JATS/SPS/TEI/DocBook)
        +-- resource policy + XML Catalog resolver
        v
XML service traits (Rust)
        |
        +-- native crates (quick-xml, ropey)
        +-- libxml2/libxslt adapter (isolated FFI helper)
        +-- JVM process adapter (Saxon HE, Jing, Xerces-J 1.1)
        +-- future WASM plugin host
```

Tauri is justified because the web editing ecosystem already supplies the
incremental syntax tree, viewport rendering, folding, selections, commands, and
extension model expected of a professional editor. Rebuilding those in a Rust
immediate-mode GUI would dominate the roadmap and still not improve XML
correctness. Tauri keeps the privileged and standards-sensitive core in Rust,
uses the operating system webview, and provides capability-scoped IPC.

## Workspace boundaries

The initial crate plan deliberately uses cohesive boundaries rather than one
crate per acronym:

| Component | Responsibility |
|---|---|
| `xml-domain` | diagnostics, source spans, revisions, engine-neutral types |
| `xml-document` | rope-backed text, edits, line index, snapshot lifecycle |
| `xml-syntax` | well-formedness, outline, formatting, lightweight associations |
| `xml-project` | `project.toml`, workspace paths, scenarios, settings |
| `xml-resource` | trust policy, catalog resolution, and opt-in DNS-pinned HTTPS retrieval |
| `xml-validation` | engine traits, pipeline, cancellation, diagnostic normalization |
| `xml-xpath` | XPath query contract, supervised engine adapter, navigable results |
| `xml-schema` | bounded common content-model index, regular-particle compiler, and completion queries |
| `xml-dtd` | normalized DTD declarations and mapping into the common particle model |
| `xml-xsd` | normalized XSD 1.0 declarations and mapping into the common particle model |
| `xml-libxml2-helper` | isolated unsafe FFI for DTD validation/index extraction, XSD 1.0, and XPath 1.0 |
| `xml-framework` | versioned JATS/SPS/TEI/DocBook manifests and rule composition |
| `xml-jats` | bounded JATS recognition and navigable editorial structure |
| `xml-grid` | bounded projection and transactional scalar-text edits for repeated sibling structures |
| `xml-schematron` | bounded SVRL normalization and diagnostic mapping |
| `xml-transform` | XSLT/XQuery process and FFI adapters, scenarios |
| `xml-plugin-api` | stable capability-based host contracts; added after core traits settle |
| `xml-cli` | future CLI over application services |
| `app/src-tauri` | desktop composition root and audited IPC commands |

Standards receive a dedicated crate when they contain reusable behavior rather
than only forwarding to an engine. `xml-dtd` and `xml-xsd`, for example, map
engine-normalized declarations into the bounded regular-particle compiler owned
by `xml-schema`. Grammar resources are brokered through `xml-resource`, while
validation remains behind `xml-validation`. RNG and Schematron stay adapters
there until their dependency and release lifecycles justify extraction.

Remote connectors must consume `RemoteResourcePolicy`; they cannot reuse the
offline catalog path as an implicit network escape. URL origin approval and
post-DNS public-address approval are distinct steps so a transport can pin the
approved addresses and defend against DNS rebinding on each redirect.

## Editable document model

`XmlDocument` contains a `ropey::Rope`, monotonically increasing `Revision`,
encoding/newline metadata, dirty state, and optional path. UI changes arrive as
ordered byte/character edits tagged with the base revision. Applying a stale edit
is rejected.

Background work receives a cheap immutable rope clone. Results contain the same
revision and are published only if it still matches the live document. Engines
that require contiguous UTF-8 receive one bounded snapshot at the adapter edge.
This avoids multiple permanent copies while acknowledging that libxml2 and JVM
engines need contiguous input.

The CodeMirror/Lezer tree enables immediate highlighting, folding, matching, and
error recovery. Rust `quick-xml` is the authoritative Phase 1 well-formedness
parser. Later schema engines operate on immutable snapshots.

## Source locations

All public spans use UTF-8 byte offsets plus one-based line and Unicode-scalar
column. `LineIndex` stores line-start byte offsets and converts engine offsets in
`O(log n)`. The UI bridge additionally converts to CodeMirror UTF-16 offsets.
This explicit boundary avoids the common byte/character/UTF-16 ambiguity.

`quick-xml::Reader::buffer_position()` supplies event/error byte positions.
libxml2 structured errors supply line/column where available. JVM adapters return
line/column and, when absent, the closest resolvable node span. A diagnostic is
marked `approximate` when exact recovery is impossible.

## Validation pipeline

```text
edit -> 300 ms debounce -> snapshot(revision) -> worker queue
     -> well-formedness -> associated grammar -> Schematron/framework rules
     -> normalize + deduplicate -> revision gate -> Problems panel
```

Only the newest queued revision per document is useful. Cooperative Rust stages
observe a cancellation token; external processes receive a deadline and are
terminated on cancellation. Schema compilation is cached by canonical URI,
content digest, engine version, and resource-policy fingerprint.

Incremental editing does not imply incremental standards validation. Phase 1
uses incremental UI parsing and debounced full well-formedness validation. Later
work may revalidate affected subtrees only where engine semantics make that
sound; full validation remains the correctness fallback.

## Engine plan

- **Parsing/well-formedness:** `quick-xml` streaming reader in Phase 1. It is
  fast, namespace-capable, and does not fetch external entities by itself.
- **DTD:** libxml2 adapter with custom resource loader, catalogs, `NONET`, and
  expansion limits. DTD diagnostics and normalized declaration extraction are
  isolated from the editor parser; only bounded owned data crosses the process
  boundary.
- **XSD 1.0:** libxml2 initially; conformance gaps are covered by a test corpus
  and the adapter can be swapped. The isolated helper exposes bounded global
  declarations and projects local sequence/choice particles and attribute uses
  from libxml2-resolved types. Same-document attribute groups and enum facets
  are expanded under bounds; model groups, `all`, wildcards, cross-document
  groups, and inherited content remain conservative. Xerces-J is optional.
- **XSD 1.1:** realistic through the separate Xerces2-J 2.12.2 XSD 1.1
  distribution or commercial Saxon-EE, not as a default native feature.
- **RELAX NG:** Jing process adapter for XML and compact syntax; libxml2 can be a
  faster optional XML-syntax engine after differential tests.
- **Schematron:** ISO pipeline using SchXslt/SVRL with a configured XSLT engine.
  Never rely on libxml2's limited Schematron implementation as the only engine.
- **XPath:** libxml2 for XPath 1.0; Saxon HE process adapter for non-schema-aware
  XPath 3.1 evaluation.
- **XSLT:** libxslt for secure XSLT 1.0 and Saxon HE for XSLT 3.0. Processor
  selection is explicit in each transformation scenario.
- **XQuery:** Saxon HE process adapter for non-schema-aware XQuery 3.1.
- **Catalogs:** native OASIS XML Catalog 1.1 resolver implements exact `system`,
  `public`, and `uri`, longest-prefix `rewriteURI`/`rewriteSystem`, and guarded
  `nextCatalog`; adapters receive only policy-approved resolved URIs.

## Schema-aware completion

Grammar adapters compile a common, deliberately lossy `SchemaIndex`. Expanded
names key element declarations; deterministic state machines describe valid next
children and close states; attribute declarations retain required/prohibited use,
enums, and default/fixed values. `Empty` and conservative `Any` models cover the
boundary cases. A shared regular-particle compiler handles sequences, choices,
and bounded or unbounded occurrences with depth/state/transition/occurrence
limits. Completion resolves the cursor's namespace and ancestor context, then
queries this bounded model under a strict latency budget.

DTD/XSD/RNG-specific semantics stay in their compilers. If a construct cannot be
represented safely, completion uses the conservative `Any` model; it never
invents a restriction. CodeMirror receives a contextual result list, while
validation remains authoritative.

## JATS and SPS frameworks

A framework is a signed/versioned manifest, not special cases in the parser:

```toml
id = "sps-1.10"
base = "jats-publishing-1.3"
catalog = "catalog.xml"
grammars = ["JATS-journalpublishing1-3.dtd"]
schematron = ["sps-1.10.sch"]
inspector = "jats"
```

Framework composition runs base JATS grammar validation and SPS rules as separate
stages and labels each diagnostic source. SPS 1.9 maps to JATS 1.1 and SPS 1.10
to JATS 1.3; manifests allow SPS 1.11/JATS 1.4 and later versions without core
changes. Resource licenses and checksums are part of installation metadata.

## Plugin model

`xml-plugin-api` defines API version 1 manifests, request/response envelopes,
contribution types, checksums, and explicit document/project/file/network
capabilities. Registration validates every manifest and applies a deny-by-default
host policy before producing an authorized plugin handle. Loading a manifest does
not execute its entrypoint. The runtime default is WASI Component Model plugins
with bounded memory/fuel and no ambient network/filesystem. Engines that cannot
run in WASM use supervised JSON-RPC processes with the same policy. Native
dynamic Rust plugins are rejected as a public ABI because Rust has no stable ABI
and in-process native code defeats isolation.

The process adapter never launches a plugin entrypoint directly. A trusted,
explicitly configured OS sandbox launcher receives a freshly staged entrypoint
whose bytes were hashed again during copying. The child has a cleared
environment, private working directory, bounded JSON stdin/stdout/stderr, a hard
deadline, response correlation, and kill-on-drop. The launcher remains
responsible for kernel-level filesystem, network, syscall, and child-process
containment. No launcher is configured by default.

The embedded adapter uses Wasmtime 48 LTS and the root Component Model export
`invoke: func(string) -> string`. The first profile links no host imports, so it
has no ambient filesystem, network, clock, random, environment, or subprocess
authority. Entrypoint bytes are capped and hashed again immediately before
compilation. Each invocation gets a fresh store with fuel, epoch deadline,
64 MiB linear-memory limit, and bounded instances/memories/tables; protocol
sizes and response correlation are checked by the same API v1 contract as the
process adapter. This is deliberately a capability-free Component Model subset,
not a claim of general WASI Preview 2/3 compatibility.

## Observability

Libraries emit `tracing` spans; binaries choose subscribers. Audit events use
stable action names (`document.open`, `document.save`, `validation.run`) and
record result, opaque resource ID, revision, engine, duration, and error class.
They never include source text, passwords, tokens, entity contents, full request
bodies, unrestricted environment variables, or full paths.

## CLI reuse

Tauri commands are thin adapters over application services. The future CLI calls
the same services and engine registry, so desktop behavior is not hidden in UI
callbacks.
