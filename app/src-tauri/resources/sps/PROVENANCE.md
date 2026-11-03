# SPS validation resources

SciXML combines the legacy SPS 1.9/1.10 Schematron with the current value-list
rules from SciELO Packtools 4.18.3. Packtools 4.18.3 still distributes the
legacy Schematron files unchanged from 4.16.9, while its current Python
validation layer accepts additional SPS 1.10 values. SciXML applies a narrow,
fail-closed reconciliation for those documented differences before presenting
the Schematron diagnostics.

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
- Packtools 4.18.3 `article_type_rules.json`: `13773d7187842841865937875663da1ff00191d9d36da6c3f4f528212a71dc3f`
- Packtools 4.18.3 `author_notes_rules.json`: `a1edd33a701ce7125ee0502e8520a38ff82a414b3c9cb738c7c61bd3ec847b93`
- Packtools 4.18.3 `fn_rules.json`: `d8ed5c5b6eceb76ec2da4992870c653f9835dfccb0c30ec5b507a98572a54e46`

The `.xsl` files were generated from those sources with the ISO Schematron
compiler distributed by lxml. They are committed so installed SciXML builds can
validate offline without Python, Java, or a runtime download. SciXML executes
the generated XSLT in the isolated libxslt helper with network and file writes
disabled.

The 4.18.3 compatibility overlay currently reconciles the legacy false
positives for `author-notes/fn/@fn-type="coi-statement"`,
`fn-group/fn/@fn-type="data-availability"`, and
`sub-article/@article-type="reviewer-report"`. Other Schematron findings are
preserved. This does not embed Python or claim to reproduce every Packtools
Python validation; it aligns the existing offline SPS stage with the upstream
4.18.3 value lists without weakening its resource sandbox.

Packtools is distributed under the BSD 2-Clause license. Its license notice is
preserved in `licenses/PACKTOOLS-BSD-2-CLAUSE.txt`.
