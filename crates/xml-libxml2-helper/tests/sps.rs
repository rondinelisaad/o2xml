use std::{collections::BTreeMap, error::Error, path::PathBuf};

use xml_domain::Revision;
use xml_resource::{CatalogResolver, ResourcePolicy};
use xml_schematron::parse_svrl;
use xml_transform::{LibxsltProcessEngine, LibxsltRequest};
use xml_xpath::{Libxml2XPathEngine, XPathEngine, XPathRequest, XPathValue};

fn sps_resources() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../app/src-tauri/resources/sps")
}

#[test]
fn bundled_sps_rules_produce_locatable_findings_offline() -> Result<(), Box<dyn Error>> {
    let resources = sps_resources();
    let policy = ResourcePolicy::offline([resources.clone()])?;
    let catalog = CatalogResolver::load(resources.join("catalog.xml"), policy)?;
    let executable = env!("CARGO_BIN_EXE_xml-libxml2-helper");
    let transform = LibxsltProcessEngine::new(executable)?;
    let xpath = Libxml2XPathEngine::new(executable)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;
    let source = r#"<article article-type="research-article" dtd-version="1.1" specific-use="sps-1.9">
  <front><journal-meta/><article-meta><title-group><article-title>Example</article-title></title-group></article-meta></front>
  <body><sec><p/></sec><sec><p>Text</p></sec></body>
  <back/>
</article>"#;
    let transformed = runtime.block_on(transform.transform(LibxsltRequest {
        source,
        revision: Revision(1),
        catalog: &catalog,
        stylesheet_uri: "urn:o2xml:framework:sps:1.9:compiled-schematron",
        parameters: None,
    }))?;
    let svrl = String::from_utf8(transformed.output)?;
    let report = parse_svrl(&svrl)?;
    assert!(!report.findings.is_empty());
    let section = report
        .findings
        .iter()
        .find(|finding| finding.location.as_deref() == Some("/article/body/sec[1]"))
        .ok_or_else(|| format!("expected an SPS finding for the first section: {report:?}"))?;
    let location = section.location.as_deref().ok_or("missing location")?;
    let namespaces = BTreeMap::new();
    let result = runtime.block_on(xpath.evaluate(XPathRequest {
        source,
        revision: Revision(1),
        expression: location,
        namespaces: &namespaces,
    }))?;
    let XPathValue::Nodes { items, .. } = result.value else {
        return Err("expected XPath nodes".into());
    };
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].path, location);
    assert_eq!(items[0].span.start.line, 3);
    Ok(())
}
