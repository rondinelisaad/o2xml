# Security model

## Assets and trust boundaries

Protected assets include unpublished article XML, local paths, transformation
outputs, credentials referenced by the operating system, and application
integrity. Inputs are untrusted: opened XML, DTD/XSD/RNG/Schematron/XSLT/XQuery,
catalogs, project manifests, plugin packages, archives, and process output.

Trust boundaries exist between the webview and Rust IPC, Rust and native FFI,
Rust and JVM/helper processes, the resource resolver and filesystem/network, and
the project and installed framework store.

## Threats and controls

| Threat | Default control |
|---|---|
| XXE / local file disclosure | external entity loading off; policy-aware resolver is the sole loader |
| Billion Laughs / expansion bombs | no substitution in editor parse; depth, entity, bytes, time budgets in validating engines |
| SSRF through schemas/catalogs/XInclude | network denied; catalog resolution precedes access; explicit origin allowlist |
| Path traversal / symlink escape | canonicalize target and verify it lies in an approved root before opening |
| XSLT/XQuery filesystem or network access | engine-specific URI resolver denies ambient access; output restricted to scenario target |
| Malicious native parser input | helper-process isolation is preferred for libxml2 validation; pin and scan versions |
| Webview compromise | strict CSP, no remote content, narrow Tauri capabilities, typed IPC, payload limits |
| Stale diagnostics corrupting UX | every request/result carries a monotonic document revision |
| Resource exhaustion | input-size, nesting, runtime, output, diagnostic-count, and worker concurrency limits |
| Plugin compromise | WASI capabilities/fuel or supervised process; no public in-process native plugin ABI |
| Sensitive logging | structured allowlisted fields only; never source text, secrets, entity bodies, or full paths |

## Resource policy

The resolver evaluates a request against an immutable policy:

1. normalize the URI without dereferencing it;
2. resolve through configured local catalogs with cycle and entry limits;
3. require an allowed scheme (`file` only by default);
4. canonicalize filesystem paths and enforce approved roots;
5. deny network unless the user enabled a pinned origin for this project;
6. enforce byte/time limits and record provenance;
7. return bytes plus the canonical identity, never an unrestricted handle.

`XML_PARSE_NONET` is necessary but not sufficient. libxml2 receives a custom
resource loader so local-file access is also constrained. Entity substitution,
DTD loading, XInclude, and `XML_PARSE_HUGE` are never enabled merely to make a
document pass.

XPath 1.0 parses the in-memory document with a deny-all external resource loader.
This permits ordinary JATS `DOCTYPE` declarations without fetching their remote
subsets; external entities remain unavailable to XPath evaluation.

DTD completion extraction runs in the isolated libxml2 helper and uses the same
deny-by-default resource callback as validation. Raw FFI pointers never cross
the process boundary. Element, particle, attribute, and enumeration counts or
depths are bounded before serialization; the safe Rust compiler then applies
independent NFA/DFA state and transition limits. The parent process additionally
caps individual and aggregate grammar bytes, resource count, helper output, and
execution time. Audit events record only result, duration, engine, resource
count, element count, and a bounded error class—not paths or grammar identifiers.
The desktop completion command accepts element/attribute context only. It uses
the catalog registered by `open_project`, rejects document and catalog paths
outside that canonical project root, and never accepts a catalog path from the
webview completion request.

## Processor profiles

- `editor-safe`: well-formedness only, no external resource access.
- `offline-framework`: access only to project and installed framework roots
  through catalogs.
- `trusted-transform`: explicit input, stylesheet, output root, and processor;
  still no ambient network.
- `network-enabled`: future opt-in profile with origin allowlist, cache, size
  limits, provenance, and a visible UI indicator.

## Logging and audit

Local desktop events have no meaningful client IP, authenticated user, or server
session; those fields are omitted rather than fabricated. If a future service is
added, its audit adapter must add timestamp with timezone, opaque user/session
IDs, client IP, action, resource, result, and sanitized before/after data.

Current event fields are: timestamp (subscriber), action, result, resource type,
opaque resource ID, revision, engine, duration, and bounded error category.

## Secure SDLC analysis checklist (NSI.04 4.3)

- [x] Authentication and authorization scope specified: absent from local MVP;
  required as a separate design before any remote service.
- [x] Data classified: documents/paths restricted, settings internal, public
  framework resources public subject to their licenses.
- [x] Confidentiality, integrity, and availability controls specified above.
- [x] Log/monitoring interface and prohibited fields specified.
- [x] Environment separation specified: only synthetic fixtures in tests and no
  production credentials/corpora in source control.

## Verification plan

- malicious XML corpus for external entities, recursive entities, deep nesting,
  oversized tokens, catalog cycles, path traversal, and XInclude;
- integration tests proving every engine respects the same denied resource;
- the isolated libxml2 helper cannot open grammar paths directly: its external
  entity loader requests each DTD/XSD resource from the parent, which applies
  catalogs, allowed roots, per-resource/count/aggregate limits and sends the
  approved bytes over a framed pipe;
  this is defense in depth, not a substitute for the planned custom loader;
- fuzzing of parsers, edit application, catalog normalization, and IPC inputs;
- `cargo audit`, `cargo deny`, npm audit, license policy, and secret scan in CI;
- differential validation against published conformance suites where licensing
  permits redistribution.
