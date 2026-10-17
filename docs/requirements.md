# Requirements baseline

## Product intent

O2XML is a cross-platform desktop IDE for editing, validating, transforming,
and analysing real XML documents. It is an independent implementation and must
not copy proprietary source, assets, wording, UI identity, or behavior protected
by another product's intellectual property.

The reusable Rust core must also support a future CLI, library, and local service.
Linux, macOS, and Windows are target platforms.

## Priority users

1. Scientific publishing teams using JATS and SPS.
2. XML developers using DTD, XSD, RELAX NG, Schematron, XPath, XSLT, catalogs,
   and XInclude.
3. Projects using DocBook or TEI frameworks.

JATS/SPS is a product vertical, not a syntax-highlighting preset. JATS grammar
validation and SPS business-rule validation remain separate and visible stages.

## Functional scope

### Phase 1 acceptance scope

- open and save UTF-8 XML files without silently changing source;
- multiple editor tabs, dirty state, and basic session data;
- line numbers, XML highlighting, folding, matching, auto-close, indentation;
- find and replace;
- well-formedness diagnostics with file, byte range, line, and column;
- configurable pretty print;
- structural outline synchronized with the editor;
- Problems and Output panels;
- basic project manifest and project explorer;
- light and dark themes;
- non-blocking validation with revision checks and debounce.

### Later phases

- DTD, XSD, RELAX NG, ISO Schematron, XML Catalogs, XPath, and schema-aware
  completion;
- JATS and SPS framework packages;
- XSLT and optional XQuery processors;
- XML-aware diff/merge, Author Mode, XML Grid, remote resources, and plugins.

## Non-functional requirements

- No legitimate external failure may be handled with `unwrap()` in production.
- Engine failures produce typed errors and normalized diagnostics.
- Editor interaction target is below 16 ms and completion target below 100 ms;
  measured results, not aspirations, are reported.
- Documents of 1, 10, 50, 100, and 500 MB receive benchmark coverage.
- A document uses one mutable rope plus cheap immutable revision snapshots;
  engines may materialize a contiguous buffer only at explicit boundaries.
- Network and arbitrary filesystem access are disabled for XML processors by
  default.
- Logs never include document bodies, credentials, entity replacement text, or
  unrestricted environment dumps.
- Dependencies require maintenance, license, security, and portability review.

## Security requirements (analysis phase, NSI.04 4.3)

- Authentication/authorization: the local MVP has no accounts. Any future remote
  service must define authentication, roles, and authorization independently.
- Data classification: XML content and project paths are **restricted** by
  default; schemas distributed by standards bodies are **public**; settings and
  diagnostics are **internal** unless exported by the user.
- Confidentiality: document contents remain local unless the user explicitly
  enables a remote resource or service.
- Integrity: saves are atomic where the platform permits; stale background
  results are discarded by document revision.
- Availability: parsing and validation have size, time, recursion, and expansion
  budgets and run off the UI thread.
- Audit interface: structured events record action, result, resource class,
  opaque document ID, revision, duration, and error class. They do not record
  document text, credentials, or full paths.
- Environment separation: tests use synthetic fixtures; no production corpus or
  credentials belongs in the repository.

## Explicit exclusions from the first MVP

DTD/XSD/RNG/Schematron validation, catalogs, XPath, XSLT, XQuery, schema-aware
completion, Author Mode, Grid, XML-aware diff, merge, remote access, WebDAV,
database integration, native/WASM plugins, and 500 MB performance claims are not
Phase 1 acceptance criteria.
