# Engine and dependency analysis

Research snapshot: 2026-10-17. Versions are selected at implementation time and
locked; this document records capability and licensing decisions, not an eternal
claim that a version is current.

## Answers to the twenty design questions

1. **GUI:** Tauri 2. It is cross-platform, keeps privileged logic in Rust, has a
   capability model, and permits a mature editor UI. It uses platform webviews,
   so rendering differences require cross-platform tests.
2. **Text editor:** CodeMirror 6 with `@codemirror/lang-xml`/Lezer. It provides
   viewport rendering, incremental parsing, line numbers, folding, matching,
   auto-close, indentation, commands, and schema-fed completion under MIT.
3. **Editable representation:** `ropey::Rope` plus revision, line index,
   encoding/newline metadata, and immutable snapshots. XML trees are derived.
4. **Parser:** `quick-xml` for Phase 1 streaming well-formedness; libxml2/JVM
   engines are validation adapters, not the editor model.
5. **Locations:** engine byte or line/column locations normalize to UTF-8 byte
   spans and one-based lines/Unicode columns; the UI bridge converts to UTF-16.
6. **DTD:** libxml2 with a custom policy resolver, local catalogs, `NONET`, and
   strict expansion limits in Phase 2.
7. **XSD 1.0:** libxml2 first, behind an adapter and conformance corpus. Xerces-J
   remains an alternative for difficult schemas.
8. **XSD 1.1:** possible, but not a default MVP feature. Xerces2-J publishes a
   separate XSD 1.1 distribution; Saxon schema-awareness requires Saxon-EE.
9. **RELAX NG:** isolated libxml2 is the RNG XML engine and reuses the catalog
   resource broker. A supervised Jing V20241231 adapter handles self-contained
   RNC. RNC `include`/`external` stays denied until a brokered resolver replaces
   Jing's ambient resource access.
10. **Schematron:** an isolated libxml2 baseline provides immediate XPath 1.0
    assertions behind the catalog broker. Conforming ISO processing compiles to
    XSLT/SVRL with SchXslt and Saxon; Quick Fix remains later.
11. **XPath:** libxml2 for 1.0; Saxon HE for non-schema-aware 3.1.
12. **XSLT:** libxslt for 1.0; Saxon HE for 3.0. Each scenario names its engine.
13. **XQuery:** Saxon HE in a supervised JVM process; XQuery is Phase 4 and can
    be disabled when Java is unavailable.
14. **Catalogs:** a native Catalog 1.1 resolver with exact, longest-prefix, base
    URI, preference, cycle, and `nextCatalog` semantics; adapters use its result.
15. **Completion:** compile DTD/XSD/RNG into a common content-model index and
    query it by expanded-name ancestor context; validation remains authoritative.
16. **JATS/SPS:** versioned composable framework manifests with catalogs,
    grammar, Schematron, inspector, docs, license, and checksums. SPS never
    collapses into “JATS valid”.
17. **XML security:** safe parser profile, sole resource resolver, no network by
    default, approved roots, expansion/depth/size/time/output limits, and process
    isolation for complex engines.
18. **Plugins:** internal Rust traits first; later WASI components or supervised
    processes with declared capabilities. No stable native Rust ABI promise.
19. **Non-Rust dependencies:** OS webview; Node only for frontend builds; Java
    runtime for Saxon/Jing/Xerces adapters; libxml2/libxslt for native validation
    and XSLT. End users should not need Node.
20. **Not realistic in MVP:** standards-complete DTD/XSD/RNG/Schematron,
    schema-aware completion, catalogs, XPath/XSLT/XQuery, SPS packs, Author Mode,
    Grid, semantic merge, plugins, remote services, and verified 500 MB support.

## Feature matrix

| Feature | Engine / crate | Rust native? | License | Risk | Phase |
|---|---|---:|---|---|---:|
| Desktop shell | Tauri 2 | Core yes; OS webview | MIT OR Apache-2.0 | MEDIUM | 1 |
| Text editor | CodeMirror 6 + Lezer XML | No (TypeScript) | MIT | LOW | 1 |
| Editable text | ropey | Yes | MIT | LOW | 1 |
| Well-formedness | quick-xml | Yes | MIT | MEDIUM | 1 |
| Outline/format | quick-xml + project code | Yes | MIT / project | MEDIUM | 1 |
| Background work | tokio + cancellation token | Yes | MIT | LOW | 1 |
| Structured logging | tracing / tracing-subscriber | Yes | MIT | LOW | 1 |
| DTD | libxml2 adapter | No (C FFI/helper) | MIT | HIGH | 2 |
| XSD 1.0 | libxml2 adapter | No (C FFI/helper) | MIT | HIGH | 2 |
| XSD 1.1 | Xerces2-J special distribution | No (JVM) | Apache-2.0 | EXPERIMENTAL | 5 |
| RELAX NG XML | libxml2 helper | No | MIT | MEDIUM | 3 |
| RELAX NG compact | Jing V20241231 | No (JVM) | 3-clause BSD | MEDIUM | 3 |
| Schematron baseline | libxml2 helper | No | MIT | HIGH | 3 |
| ISO Schematron/SVRL | SchXslt + Saxon HE | No | MIT / MPL-2.0 | HIGH | 3–4 |
| XPath 1.0 | libxml2 | No | MIT | MEDIUM | 2 |
| XPath 3.1 | Saxon HE | No (JVM) | MPL-2.0 | MEDIUM | 2 |
| XSLT 1.0 | libxslt | No (C) | MIT | HIGH | 4 |
| XSLT 3.0 | Saxon HE | No (JVM) | MPL-2.0 | MEDIUM | 4 |
| XQuery 3.1 | Saxon HE | No (JVM) | MPL-2.0 | HIGH | 4 |
| XML Catalog 1.1 | project resolver | Yes | Apache-2.0 OR MIT | HIGH | 2 |
| Schema completion | common model + adapters | Mixed | mixed/project | HIGH | 2 |
| JATS framework | NLM schemas + project manifest | Mixed | verify per asset | MEDIUM | 3 |
| SPS framework | external SPS rules/assets | Mixed | CC BY-NC 4.0 reported by upstream | HIGH | 3 |
| Plugin host | WASI Component Model / process | Yes/mixed | mixed | EXPERIMENTAL | 5 |
| Benchmarks | Criterion | Yes | Apache-2.0 OR MIT | LOW | 1+ |

## Important findings

- The upstream SciELO Publishing Schema repository says its non-normative content
  is CC BY-NC 4.0. Bundling those assets in a permissively licensed distribution
  requires an explicit legal/redistribution decision. O2XML initially stores only
  framework metadata and installs user-supplied or separately distributable packs.
- SPS/JATS mappings are versioned policy. Current SciELO project evidence maps
  SPS 1.9 to JATS 1.1, SPS 1.10 to JATS 1.3, and discusses SPS 1.11/JATS 1.4.
- libxml2 documents its XML Schema structures as incomplete. XSD 1.0 support must
  therefore be adapter-based, tested, and never described as perfect by default.
- Saxon HE covers non-schema-aware XPath/XSLT/XQuery; schema-aware XSD processing
  is not an HE entitlement and must not be implied.
- `quick-xml` is a streaming parser, not a DTD/XSD engine or error-recovering
  incremental syntax tree. Lezer provides editor recovery; standards engines
  provide authoritative validation.

## Primary sources

- Tauri architecture and license: <https://tauri.app/concept/architecture/>
- CodeMirror XML package: <https://github.com/codemirror/lang-xml>
- quick-xml documentation: <https://docs.rs/quick-xml/latest/quick_xml/>
- Ropey documentation: <https://docs.rs/ropey/latest/ropey/struct.Rope.html>
- libxml2 API: <https://gnome.pages.gitlab.gnome.org/libxml2/html/files.html>
- Xerces-J XSD 1.1: <https://xerces.apache.org/xerces2-j/xml-schema.html>
- Saxon documentation: <https://www.saxonica.com/html/documentation12/documentation.html>
- Jing/Trang: <https://github.com/relaxng/jing-trang>
- OASIS XML Catalog: <https://www.oasis-open.org/committees/entity/specs/cs-entity-xml-catalogs-1.0.html>
- JATS versions: <https://jats.nlm.nih.gov/publishing/versions.html>
- SPS documentation source: <https://github.com/scieloorg/scielo_publishing_schema>
