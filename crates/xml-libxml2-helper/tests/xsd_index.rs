use std::{error::Error, path::PathBuf};

use xml_resource::{CatalogError, CatalogResolver, ResourcePolicy};
use xml_schema::{ExpandedName, SchemaLanguage};
use xml_xsd::{Libxml2XsdIndexEngine, XsdIndexEngine, XsdIndexError, XsdIndexRequest};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../xml-validation/tests/fixtures")
}

fn runtime() -> Result<tokio::runtime::Runtime, std::io::Error> {
    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
}

#[test]
fn extracts_global_xsd_declarations_through_the_offline_broker() -> Result<(), Box<dyn Error>> {
    let root = fixtures();
    let policy = ResourcePolicy::offline([root.clone()])?;
    let catalog = CatalogResolver::load(root.join("catalog.xml"), policy)?;
    let engine = Libxml2XsdIndexEngine::new(env!("CARGO_BIN_EXE_xml-libxml2-helper"))?;
    let report = runtime()?.block_on(engine.compile(XsdIndexRequest {
        catalog: &catalog,
        uri: "https://example.org/composite.xsd",
    }))?;

    assert_eq!(report.index.language, SchemaLanguage::Xsd10);
    assert_eq!(report.approved_grammar_resources, 2);
    assert!(report
        .index
        .element(&ExpandedName::new(Some("urn:article"), "article")?)
        .is_some());
    Ok(())
}

#[test]
fn denies_an_xsd_outside_the_catalog_allowed_root() -> Result<(), Box<dyn Error>> {
    let root = fixtures();
    let policy = ResourcePolicy::offline([root.clone()])?;
    let catalog = CatalogResolver::load(root.join("catalog.xml"), policy)?;
    let engine = Libxml2XsdIndexEngine::new(env!("CARGO_BIN_EXE_xml-libxml2-helper"))?;
    let result = runtime()?.block_on(engine.compile(XsdIndexRequest {
        catalog: &catalog,
        uri: "https://example.org/escape.xsd",
    }));

    assert!(matches!(
        result,
        Err(XsdIndexError::Catalog(CatalogError::PathDenied(_)))
    ));
    Ok(())
}
