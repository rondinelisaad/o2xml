# ADR 0002: Rope-backed revisioned documents

- Status: accepted
- Date: 2026-10-17

## Decision

The authoritative editable source is a `ropey::Rope` with a monotonic revision.
Trees, outline, diagnostics, and schema models are derived artifacts tagged with
that revision. Background tasks use cheap immutable clones; stale results are
dropped. Contiguous copies are created only at engine boundaries.

## Consequences

Large edits and line mapping remain efficient and validation cannot overwrite a
newer Problems panel. Some external engines still require a full UTF-8 copy, so
memory benchmarks must measure adapter boundaries honestly.
