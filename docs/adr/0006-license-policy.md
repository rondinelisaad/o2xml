# ADR 0006: Dual-license code; isolate framework assets

- Status: accepted
- Date: 2026-10-17

## Decision

Project-authored source is prepared for `MIT OR Apache-2.0`. Every framework
resource retains an independently recorded license and checksum. Resources whose
redistribution is incompatible or unclear are installed separately rather than
committed or bundled.

## Consequences

Users can adopt the core under a conventional permissive license. An SPS
framework may initially require user-supplied resources because the upstream
repository reports CC BY-NC 4.0 for its non-normative content.
