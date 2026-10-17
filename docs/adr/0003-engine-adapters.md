# ADR 0003: Multiple standards-engine adapters

- Status: accepted
- Date: 2026-10-17

## Decision

Do not implement mature XML standards from scratch. Define engine-neutral traits
and normalized diagnostics, then integrate quick-xml, libxml2/libxslt,
Jing/Xerces, and Saxon according to their strengths. FFI and JVM processes remain
behind replaceable adapters.

## Consequences

The application can offer more than one processor and remain honest about
version/conformance differences. Packaging and cross-platform testing become
harder, but standards correctness is materially more achievable.
