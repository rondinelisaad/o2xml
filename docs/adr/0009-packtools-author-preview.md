# ADR 0009: Packtools HTML preview in Author Mode

## Status

Accepted.

## Context

SPSValidator v0.3.0 renders article previews through the Packtools 4.16.9
`HTMLGenerator`. SPSValidator itself is GPL-3.0, while O2XML is distributed
under `MIT OR Apache-2.0`. Copying or linking the SPSValidator implementation
into O2XML would therefore change the licensing obligations of the desktop
application. Packtools is BSD-3-Clause and its HTML generator is the actual
publication-preview engine used by that SPSValidator release.

The preview consumes untrusted XML and produces HTML. It must not gain network,
filesystem-write, script, navigation, or arbitrary local-file capabilities.

## Decision

O2XML independently integrates the Packtools 4.16.9 HTML generator XSLT 1.0
and standalone CSS under the upstream BSD-3-Clause license. No SPSValidator
source is copied or linked. The assets and upstream license are bundled under
`resources/packtools-html/4.16.9`.

The existing isolated libxslt helper performs the transformation. Stylesheet
includes and `document()` reads are resolved through the Rust XML Catalog
broker under one offline, canonical resource root. Transform parameters are
bounded and quoted as strings by libxslt. File writes, directory creation,
network reads, and network writes remain forbidden.

The generated HTML is parsed through an element/attribute allowlist and shown
in a sandboxed iframe with a deny-all Content Security Policy. Scripts, forms,
images, embedded objects, external links, event handlers, and remote resources
are removed. Only same-document fragment links survive. The bundled CSS is
injected locally. The structured, schema-gated editing projection remains
available alongside this read-only publication preview.

## Consequences

- JATS articles can be previewed offline with the SciELO presentation used by
  SPSValidator v0.3.0, without requiring Python or a GPL runtime.
- The first integration is intentionally a safe static preview. Interactive
  Packtools JavaScript and article media are not executed or loaded.
- Updates require an explicit Packtools version review, asset update, license
  notice update, and golden/integration test run.
