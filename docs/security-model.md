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

The optional SchXslt/Saxon adapter currently accepts only self-contained
Schematron and DTD-free XML input. It rejects DTDs, Schematron includes/extends, XSLT imports/includes
and secondary outputs, and known XPath document/collection access functions
before starting Java. Source, schema, generated XSLT, and SVRL use private
temporary files; the process has a deadline and bounded stderr/result files.
The Saxon extension-function switch is defense in depth, not a URI sandbox.
Until an engine-level brokered URI resolver exists, the SchXslt pipeline file is
an explicitly trusted installed asset and schemas requiring external resources
remain unavailable rather than receiving ambient filesystem or network access.
The desktop accepts Java, Saxon, and SchXslt executable/asset paths only from
its trusted host environment, never from a webview command or document. All
three values are required together, and project Schematron is still resolved
through the offline catalog and project-root policy.

The Phase 4 Saxon transformation adapter applies the same DTD-free input and
trusted-host engine boundary. Its initial profile accepts only self-contained
stylesheets: imports/includes, packages, schema imports, source documents,
dynamic evaluation, secondary results, and known document/collection functions
are denied before execution. This syntactic preflight is deliberately not
described as a general URI sandbox; brokered resource support remains disabled.
The libxslt XSLT 1.0 path runs only in the existing helper process. Stylesheet
and parser resources are served through the offline catalog broker with count,
individual-size, aggregate-size, deadline, and output limits. libxslt security
preferences deny file/network reads during transformation and all file/network
writes or directory creation; includes are resolved during compilation only
through the parent broker. Integration tests cover a catalog-brokered JATS
`DOCTYPE` transformation and a denied `document()` network read.
Scenario stylesheet and output paths are bounded project-relative paths;
absolute paths, URLs, empty components, and `.`/`..` traversal are rejected when
the manifest is parsed, before any filesystem operation.
The desktop canonicalizes stylesheet and output-parent paths again at execution,
rejects project-root escapes and symbolic-link output targets, writes through an
owner-private temporary file in the target directory, and publishes only after
the engine succeeds and the document revision is still current. The UI receives
at most 1 MiB of output as inert text; it does not execute transformed markup.
The audit event contains action, result, document ID, revision, and engine only—
never XML, stylesheet/output content, environment values, or full paths.
HTML preview first passes through an element/attribute allowlist that removes
scripts, embedded content, forms, media, images, SVG/MathML, styles, event
handlers, and every URL-bearing attribute. The sanitized fragment is rendered
in an iframe without sandbox permissions or same-origin access, with no referrer
and an inner `default-src 'none'` CSP. The original result remains available only
as escaped `textContent`; transformed HTML is never inserted into the app DOM.
The Saxon HE XQuery profile is a distinct process adapter, not an alias of XSLT.
It stages DTD-free input and query text privately, disables updates, XInclude,
schema-location loading, and external functions, and rejects imports, document/
collection/text/JSON loading, dynamic function lookup, environment access,
vendor/file namespaces, and updating syntax after removing nested comments and
string literals. This conservative self-contained profile remains the only
enabled XQuery mode until a brokered Saxon URI resolver exists.
The desktop XQuery command returns at most 1 MiB as inert text and discards a
result if the document revision changed during evaluation. Its audit event does
not contain query text, source XML, result bytes, engine paths, or environment
configuration.

XML-aware diff parses both inputs without resolving external resources or
expanding DTD entities. Each side is capped at 128 MiB and 50,000 semantic
tokens; detailed LCS allocation is capped at four million cells, after which a
single prefix/suffix-bounded coarse hunk is returned instead of risking memory
exhaustion. Whitespace-only text is significant unless the caller explicitly
selects the formatting-insensitive option.
Desktop comparison is an explicit native-file selection capped at 32 MiB and is
performed off the UI thread. File paths and contents are excluded from the audit
event. Side-by-side rendering uses `textContent` only. The initial merge command
replaces the current editor buffer as one undoable operation; it never writes to
disk automatically. Per-hunk merge rereads the bounded selected file, recomputes
the semantic diff against the current revision, uses explicit insertion anchors,
and performs a full well-formedness parse of the reconstructed document before
returning it. Stale or malformed results are rejected; successful changes are a
single undoable editor transaction and are not saved automatically.

XML Grid parses only the active in-memory revision and does not resolve external
resources or expand general entities. Input is capped at 32 MiB, 100,000
elements, 1,024 levels, 2,000 displayed rows, 64 columns, six scalar-descendant
levels, and 16 KiB per cell. Extraction runs off the UI thread and stale results
are discarded. Displayed cells use `textContent` or an input value, never HTML.

Only one unambiguous scalar text event, including adjacent predefined entity
references, receives an edit target. Attributes, absent/repeated values, CDATA,
comments, processing instructions, child markup, and mixed XML remain read-only.
The backend resolves row and column identifiers again against its current
snapshot rather than trusting a client byte range, escapes replacement markup,
reparses the complete result, and runs every configured project framework stage.
A failed or unavailable stage rejects the edit. Revision is checked again after
validation; the frontend then applies one whole-document editor transaction so
Undo remains available. Audit events exclude row names, columns, values, source
XML, and validation content.

Plugin manifests are inert data. API version 1 permits only WASI components or
explicitly enabled supervised processes; native dynamic libraries are not a
runtime type. Package entrypoints and file scopes must be portable relative
paths, package digests are mandatory SHA-256 strings, and network scopes are
credential-free HTTPS origins. The default host policy grants no capability,
filesystem root, network origin, or process execution. A plugin receives an
authorized handle only after all requested capabilities and scopes are a subset
of host policy. Entrypoints are canonicalized below the package root, capped at
256 MiB, and streamed through RustCrypto SHA-256; mismatches are rejected. This
is an integrity hash, not password storage. Process execution additionally
requires an external containment layer.

The supervised-process profile is now available only through a trusted OS
sandbox launcher explicitly supplied by the host; direct entrypoint execution
is not implemented. It copies the verified bytes into a private staging
directory, hashes the copy again, clears the environment, bounds JSON request,
response, and stderr, enforces a five-second deadline, kills the child on drop,
and verifies API/request correlation and an unambiguous result/error response.
Audit events include only action, result, validated plugin id, request id,
method, and runtime—not parameters, source XML, output, paths, or stderr.

The embedded Component Model profile has no linked host imports. Wasmtime
compiles only a freshly re-read, checksum-matching component capped at 32 MiB.
Every invocation uses a new store capped at 64 MiB linear memory, one instance,
two memories, four tables, ten million fuel units, and a five-second epoch
deadline. Request and response envelopes retain their independent size limits
and response correlation gate. The timeout wakes running WebAssembly through an
engine epoch; compilation itself is bounded by component size but is not
preemptible, so future package installation should precompile approved modules.

Remote transport policy is separate from the offline catalog resolver and is
empty by default. It accepts only credential-free HTTPS URLs on an exact
host/port allowlist. A connector must resolve DNS, pass every candidate address
through the policy, pin the connection to the approved set, and repeat URL/DNS
approval after every redirect. Loopback, private, link-local, shared-address,
benchmark, documentation, multicast, unspecified, and reserved IPv4/IPv6 ranges
are rejected, including IPv4-mapped IPv6. Policy defaults cap redirects at three
and a resource at 16 MiB.

The opt-in HTTPS GET connector resolves each hop under a five-second deadline,
checks every returned address, pins Reqwest to that approved address set while
preserving TLS hostname verification, disables proxy discovery and automatic
redirects, and repeats URL/DNS approval for each explicit redirect. Request and
connect deadlines are bounded, `Content-Length` is checked when present, and
streamed bytes are independently capped. URL credentials/fragments, non-HTTPS
schemes, non-allowlisted redirect origins, non-success statuses, cookies, and
authentication are rejected or unsupported. WebDAV and database adapters remain
disabled unless separately configured.

WebDAV reuses the pinned connector and adds an independent root-path boundary
plus explicit read/write capability bits. It accepts only conservative relative
paths (no absolute path, traversal, percent escapes, query, fragment, backslash,
or NUL). Listing uses `Depth: 1`; DAV XML is capped by the transport, limited to
256 levels and 10,000 responses, and requires the `DAV:` namespace. Uploads are
capped at 16 MiB and require either a valid `If-Match` ETag or create-only
`If-None-Match: *`; no unconditional PUT, DELETE, MOVE, COPY, or recursive
operation exists. Basic-auth credentials are passed per operation, redacted from
Debug/audit data, never serialized, and the password buffer is zeroed on drop.

The BaseX adapter reuses the same DNS-pinned HTTPS transport but has a separate
read-only capability boundary. It constructs only `GET` requests for a
conservative resource path below one fixed database. Database names and paths
cannot contain traversal, query, fragment, or percent-escape syntax; redirects
are rejected rather than forwarding authentication. Credentials are supplied
per operation, redacted from Debug and audit data, and passwords are zeroed on
drop. Administrative usernames are rejected client-side, and deployment still
must provision a distinct BaseX account with only local `read` permission for
the intended database. No REST `query`, `command`, `run`, PUT, POST, or DELETE
operation is represented by the Rust API.

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
