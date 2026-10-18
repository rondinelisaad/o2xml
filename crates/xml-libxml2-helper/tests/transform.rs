use std::{error::Error, fs, path::PathBuf};

use xml_domain::Revision;
use xml_resource::{CatalogResolver, ResourcePolicy};
use xml_transform::{LibxsltProcessEngine, LibxsltRequest};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testdata/project")
}

#[test]
fn transforms_with_catalog_brokered_stylesheet_and_doctype() -> Result<(), Box<dyn Error>> {
    let root = project_root().canonicalize()?;
    let policy = ResourcePolicy::offline([root.clone()])?;
    let catalog = CatalogResolver::load(root.join("catalog.xml"), policy)?;
    let source = fs::read_to_string(root.join("xml/article.xml"))?;
    let engine = LibxsltProcessEngine::new(env!("CARGO_BIN_EXE_xml-libxml2-helper"))?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;
    let report = runtime.block_on(engine.transform(LibxsltRequest {
        source: &source,
        revision: Revision(11),
        catalog: &catalog,
        stylesheet_uri: "urn:o2xml:transform:jats-html",
    }))?;

    let output = String::from_utf8(report.output)?;
    assert_eq!(report.revision, Revision(11));
    assert!(output.contains("<h1>O2XML synthetic test article</h1>"));
    Ok(())
}

#[test]
fn denies_stylesheet_network_reads() -> Result<(), Box<dyn Error>> {
    let root = project_root().canonicalize()?;
    let policy = ResourcePolicy::offline([root.clone()])?;
    let catalog = CatalogResolver::load(root.join("catalog.xml"), policy)?;
    let engine = LibxsltProcessEngine::new(env!("CARGO_BIN_EXE_xml-libxml2-helper"))?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;
    let result = runtime.block_on(engine.transform(LibxsltRequest {
        source: "<article/>",
        revision: Revision(12),
        catalog: &catalog,
        stylesheet_uri: "urn:o2xml:transform:forbidden-network",
    }));

    assert!(result.is_err());
    Ok(())
}
