# ADR 0007: broker libxml2 resources across a process boundary

Status: accepted

## Context

`xmllint --nonet` prevents network access but still lets libxml2 open arbitrary
local paths. A separate recursive XSD preflight cannot cover DTD parameter
entities completely and leaves a check/use interval between inspection and the
engine opening a file. libxml2 2.9, still supplied by supported macOS toolchains,
only offers a process-global external entity loader.

## Decision

Run libxml2 in a single-request helper process. Its custom external entity loader
sends every URL/public identifier to `xml-validation`. The parent resolves it
through `CatalogResolver`, enforces the offline root and graph limits, reads the
resource once, and returns bytes plus a canonical base URI over framed standard
I/O. The helper constructs libxml2 inputs from those bytes. It never falls back
to libxml2's filesystem or network loader.

The helper is the only workspace crate allowed to contain FFI `unsafe`. It is
linked to system libxml2 2.9+ through `pkg-config`; the safe Rust core remains
independent of libxml2 ABI and process-global state.

## Consequences

- XSD imports/includes and DTD external parameter entities share one policy.
- Approved bytes are stable for the duration of a validation run and repeated
  requests use the broker cache.
- Deadlines, bounded diagnostics, resource counts and aggregate byte limits are
  enforced by the parent, which can terminate the helper.
- Packaging must provide a compatible libxml2 and build metadata on each target.
- The protocol adds one copy per grammar resource; benchmarks will determine
  whether a future descriptor/shared-memory transport is warranted.
