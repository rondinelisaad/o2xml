# ADR 0004: Deny external resources by default

- Status: accepted
- Date: 2026-10-17

## Decision

All engines obtain external bytes through one policy-aware resolver. The default
profile denies network and arbitrary filesystem reads. Catalogs map identifiers
but do not grant access. Project/framework roots, limits, and origins are explicit
capabilities.

## Consequences

Legacy documents may require a visible trust-policy change before validation.
This friction is intentional: XXE, SSRF, local file disclosure, and transform
side effects cannot be safe as ambient capabilities.
