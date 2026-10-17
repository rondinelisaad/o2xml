use std::{error::Error, path::PathBuf};

use xml_domain::Revision;
use xml_resource::{CatalogError, CatalogResolver, ResourcePolicy};
use xml_validation::{
    GrammarReference, Libxml2ProcessEngine, ValidationEngine, ValidationError, ValidationRequest,
};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../xml-validation/tests/fixtures")
}

fn resolver() -> Result<CatalogResolver, CatalogError> {
    let fixtures = fixtures();
    let policy = ResourcePolicy::offline([fixtures.clone()])?;
    CatalogResolver::load(fixtures.join("catalog.xml"), policy)
}

fn runtime() -> Result<tokio::runtime::Runtime, std::io::Error> {
    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
}

#[test]
fn validates_dtd_xsd_and_brokered_dependencies() -> Result<(), Box<dyn Error>> {
    let engine = Libxml2ProcessEngine::new(env!("CARGO_BIN_EXE_xml-libxml2-helper"))?;
    let catalog = resolver()?;
    let runtime = runtime()?;

    let dtd_report = runtime.block_on(engine.validate(ValidationRequest {
        source: "<article>text</article>",
        revision: Revision(1),
        file: Some("article.xml"),
        catalog: &catalog,
        grammar: GrammarReference::Dtd {
            system_id: "https://example.org/composite.dtd",
            public_id: None,
        },
    }))?;
    let invalid_dtd = runtime.block_on(engine.validate(ValidationRequest {
        source: "<article><title/></article>",
        revision: Revision(2),
        file: Some("invalid.xml"),
        catalog: &catalog,
        grammar: GrammarReference::Dtd {
            system_id: "https://example.org/article.dtd",
            public_id: None,
        },
    }))?;
    let xsd_report = runtime.block_on(engine.validate(ValidationRequest {
        source: "<article/>",
        revision: Revision(3),
        file: Some("article.xml"),
        catalog: &catalog,
        grammar: GrammarReference::Xsd {
            uri: "https://example.org/article.xsd",
        },
    }))?;
    let composite_report = runtime.block_on(engine.validate(ValidationRequest {
        source: r#"<a:article xmlns:a="urn:article" xmlns:c="urn:common"><c:common>ok</c:common></a:article>"#,
        revision: Revision(4),
        file: Some("composite.xml"),
        catalog: &catalog,
        grammar: GrammarReference::Xsd {
            uri: "https://example.org/composite.xsd",
        },
    }))?;

    assert!(dtd_report.diagnostics.is_empty());
    assert_eq!(dtd_report.approved_grammar_resources, 2);
    assert!(!invalid_dtd.diagnostics.is_empty());
    assert!(xsd_report.diagnostics.is_empty());
    assert!(composite_report.diagnostics.is_empty());
    assert_eq!(composite_report.approved_grammar_resources, 2);
    Ok(())
}

#[test]
fn denies_dependency_outside_allowed_roots() -> Result<(), Box<dyn Error>> {
    let engine = Libxml2ProcessEngine::new(env!("CARGO_BIN_EXE_xml-libxml2-helper"))?;
    let catalog = resolver()?;
    let runtime = runtime()?;
    let result = runtime.block_on(engine.validate(ValidationRequest {
        source: "<article/>",
        revision: Revision(5),
        file: Some("article.xml"),
        catalog: &catalog,
        grammar: GrammarReference::Xsd {
            uri: "https://example.org/escape.xsd",
        },
    }));

    assert!(
        matches!(
            &result,
            Err(ValidationError::Catalog(CatalogError::PathDenied(_)))
        ),
        "unexpected validation result: {result:?}"
    );
    Ok(())
}
