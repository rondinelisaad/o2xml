# ADR 0005: WASI or process plugins, not native Rust ABI

- Status: proposed for Phase 5
- Date: 2026-10-17

## Decision

Stabilize internal Rust traits first. The public plugin boundary will prefer WASI
components with versioned WIT contracts and resource limits; incompatible engines
use supervised JSON-RPC processes. Do not promise ABI-stable native Rust dynamic
plugins.

## Consequences

Plugins pay serialization/isolation cost but gain portability, capability control,
crash containment, and an evolvable host contract.
