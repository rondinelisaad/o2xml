# SPS validation resources

SciXML uses the legacy bundled Schematron for SPS 1.9. SPS 1.10 is validated by
the complete Python validation pipeline published in SciELO Packtools 4.18.3,
frozen into the target-native `scixml-packtools-validator` sidecar. JATS 1.3
DTD validation remains a separate stage and does not gate SPS execution.

The Schematron sources are:

- `packtools/catalogs/scielo-style-1.9.sch`
- `packtools/catalogs/scielo-style-1.10.sch`
- `packtools/catalogs/common.sch`

Current upstream: <https://github.com/scieloorg/packtools/tree/4.18.3>
Commit: `72e536e3a3a0869a61f5658d5a8d3317e36081fb`

SHA-256:

- SPS 1.9 Schematron: `4662c0ed276f52d1c4f5262968f74ababd78e7f5ceee338a38c7c082ea45bfd3`
- SPS 1.9 generated XSLT: `157be86ef3d8a91d9b9a7e381ea96f490972f1abba2de486442a490c2e7f6e69`
- SPS 1.10 Schematron: `1f8934d006b7105a936aefe4cf0c1322da802170bd4d3499b1577b14b0467686`
- SPS 1.10 generated XSLT: `382b2cfc34be652c34afabaa251a43380d7c907fc0d18615ec9174b8f3ce95f4`

The `.xsl` files were generated from those sources with the ISO Schematron
compiler distributed by lxml and remain committed for SPS 1.9. The full SPS
1.10 engine is built from the official 4.18.3 tag archive, whose SHA-256 is
`40cb614213948330035dfb872fc294b3826531e91fc53a8b5da3c201ce07d507`.
Its frozen runtime includes all validation modules, JSON rule files, and
Brazilian Portuguese message catalogs. It accepts current SPS 1.10 values such
as `coi-statement`, `data-availability`, and `reviewer-report` while applying
the remaining Packtools rule groups.

The Packtools sidecar has no resource-broker API and therefore receives only
the in-memory XML string, parsed with DTD/entity loading disabled. Python socket
resolution is disabled, the process environment is cleared, and the Rust
supervisor enforces input, time, output, and diagnostic limits.

Packtools is distributed under the BSD 2-Clause license. Its license notice is
preserved in `licenses/PACKTOOLS-BSD-2-CLAUSE.txt`.
