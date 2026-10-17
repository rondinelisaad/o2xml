use std::{collections::BTreeMap, error::Error};

use xml_domain::Revision;
use xml_xpath::{Libxml2XPathEngine, XPathEngine, XPathRequest, XPathValue};

fn runtime() -> Result<tokio::runtime::Runtime, std::io::Error> {
    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
}

#[test]
fn evaluates_nodes_scalars_and_namespaces() -> Result<(), Box<dyn Error>> {
    let engine = Libxml2XPathEngine::new(env!("CARGO_BIN_EXE_xml-libxml2-helper"))?;
    let runtime = runtime()?;
    let source = r#"<article xmlns="urn:jats">
  <front><title>First</title></front>
  <body><title>Second</title></body>
</article>"#;
    let namespaces = BTreeMap::from([("j".to_owned(), "urn:jats".to_owned())]);

    let nodes = runtime.block_on(engine.evaluate(XPathRequest {
        source,
        revision: Revision(9),
        expression: "//j:title",
        namespaces: &namespaces,
    }))?;
    let count = runtime.block_on(engine.evaluate(XPathRequest {
        source,
        revision: Revision(9),
        expression: "count(//j:title)",
        namespaces: &namespaces,
    }))?;

    let XPathValue::Nodes { items, truncated } = nodes.value else {
        return Err("expected node results".into());
    };
    assert!(!truncated);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].span.start.line, 2);
    assert_eq!(items[1].span.start.line, 3);
    assert!(items[0].path.starts_with('/'));
    assert_eq!(count.value, XPathValue::Number("2".to_owned()));
    Ok(())
}

#[test]
fn evaluates_doctype_without_loading_its_external_subset() -> Result<(), Box<dyn Error>> {
    let engine = Libxml2XPathEngine::new(env!("CARGO_BIN_EXE_xml-libxml2-helper"))?;
    let runtime = runtime()?;
    let namespaces = BTreeMap::new();
    let report = runtime.block_on(engine.evaluate(XPathRequest {
        source: r#"<!DOCTYPE article SYSTEM "https://example.org/article.dtd"><article/>"#,
        revision: Revision(10),
        expression: "/article",
        namespaces: &namespaces,
    }))?;
    let XPathValue::Nodes { items, truncated } = report.value else {
        return Err("expected node results".into());
    };
    assert_eq!(items.len(), 1);
    assert!(!truncated);
    Ok(())
}
