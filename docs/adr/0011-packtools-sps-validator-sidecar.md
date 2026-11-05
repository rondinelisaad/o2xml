# ADR 0011: Freeze Packtools 4.18.3 as the SPS 1.10 validator

## Status

Accepted — 2026-11-05.

## Context

The original built-in SPS 1.10 stage executed Packtools' legacy Schematron and
reconciled only three changed value lists. It did not execute the extensive
Python validation groups added to Packtools 4.18.3, so the UI could claim SPS
1.10 approval without applying current metadata, contributor, date, link,
figure, table, formula, reference, peer-review, and accessibility rules.

Calling a system `python` or installing packages at application startup would
make validation dependent on the user's machine and network. Reimplementing the
rules in Rust would create an independent, diverging validator rather than use
the requested SciELO engine.

## Decision

Release CI installs the official Packtools tag `4.18.3` at commit
`72e536e3a3a0869a61f5658d5a8d3317e36081fb`, verifies the tag archive SHA-256,
and freezes its SPS validation entrypoint plus Python 3.12 into a target-native
PyInstaller sidecar. The Rust `xml-packtools` adapter sends only a bounded
in-memory XML request through stdin, verifies protocol and engine version,
normalizes diagnostics into source locations, enforces time/output/count limits,
and emits content-free audit events.

The sidecar runs only for documents declaring `specific-use="sps-1.10"`. JATS
1.3 DTD validation remains a separate stage and does not gate SPS validation.
SPS 1.9 continues to use the bundled legacy Schematron until an equivalent
versioned engine decision is made.

## Consequences

- SPS 1.10 results now come from the complete Packtools 4.18.3 Python validation
  pipeline rather than a three-rule compatibility overlay.
- Installers and the Windows portable package grow by approximately 25 MiB, but
  end users do not install Python or Packtools.
- Every release platform must build and smoke-test a native sidecar; Python is a
  build-time dependency.
- The helper denies DTD/entity/network processing and has no arbitrary path or
  URL input. Rules that require remote registries report Packtools' own
  “could not verify” result rather than making a request.
- Upgrading Packtools requires a new dependency/license/security review, pinned
  archive digest, conformance tests, and a new release.
