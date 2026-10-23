# SPS validation resources

The SPS 1.9 and SPS 1.10 Schematron sources in this directory come from
SciELO Packtools 4.16.9:

- `packtools/catalogs/scielo-style-1.9.sch`
- `packtools/catalogs/scielo-style-1.10.sch`
- `packtools/catalogs/common.sch`

Upstream: <https://github.com/scieloorg/packtools/tree/4.16.9>
Commit: `a5455f82995d68bccc79365e3e0c5a023c70c7d4`

SHA-256:

- SPS 1.9 Schematron: `4662c0ed276f52d1c4f5262968f74ababd78e7f5ceee338a38c7c082ea45bfd3`
- SPS 1.9 generated XSLT: `157be86ef3d8a91d9b9a7e381ea96f490972f1abba2de486442a490c2e7f6e69`
- SPS 1.10 Schematron: `1f8934d006b7105a936aefe4cf0c1322da802170bd4d3499b1577b14b0467686`
- SPS 1.10 generated XSLT: `382b2cfc34be652c34afabaa251a43380d7c907fc0d18615ec9174b8f3ce95f4`

The `.xsl` files were generated from those sources with the ISO Schematron
compiler distributed by lxml. They are committed so installed O2XML builds can
validate offline without Python, Java, or a runtime download. O2XML executes
the generated XSLT in the isolated libxslt helper with network and file writes
disabled.

Packtools is distributed under the BSD 2-Clause license. Its license notice is
preserved in `licenses/PACKTOOLS-BSD-2-CLAUSE.txt`.
