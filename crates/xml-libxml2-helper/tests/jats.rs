use std::{error::Error, path::PathBuf};

use xml_domain::Revision;
use xml_resource::{CatalogResolver, ResourcePolicy};
use xml_validation::{GrammarReference, Libxml2ProcessEngine, ValidationEngine, ValidationRequest};

const JATS_11_PUBLIC: &str = "-//NLM//DTD JATS (Z39.96) Journal Publishing DTD v1.1 20151215//EN";
const JATS_11_SYSTEM: &str = "https://jats.nlm.nih.gov/publishing/1.1/JATS-journalpublishing1.dtd";
const JATS_13_PUBLIC: &str = "-//NLM//DTD JATS (Z39.96) Journal Publishing DTD v1.3 20210610//EN";
const JATS_13_SYSTEM: &str =
    "https://jats.nlm.nih.gov/publishing/1.3/JATS-journalpublishing1-3.dtd";

fn jats_resources(version: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../app/src-tauri/resources/jats")
        .join(version)
}

#[test]
fn bundled_jats_rejects_invalid_fn_type_offline() -> Result<(), Box<dyn Error>> {
    let resources = jats_resources("1.1");
    let policy = ResourcePolicy::offline([resources.clone()])?;
    let catalog = CatalogResolver::load(resources.join("catalog-jats-v1-1-no-base.xml"), policy)?;
    let engine = Libxml2ProcessEngine::new(env!("CARGO_BIN_EXE_xml-libxml2-helper"))?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;
    let source = format!(
        "<!DOCTYPE article PUBLIC \"{JATS_11_PUBLIC}\" \"{JATS_11_SYSTEM}\">\n\
         <article dtd-version=\"1.1\"><front/><body><fn fn-type=\"xxxxxx\"/></body></article>"
    );
    let report = runtime.block_on(engine.validate(ValidationRequest {
        source: &source,
        revision: Revision(1),
        file: Some("invalid-jats.xml"),
        catalog: &catalog,
        grammar: GrammarReference::Dtd {
            system_id: JATS_11_SYSTEM,
            public_id: Some(JATS_11_PUBLIC),
        },
    }))?;

    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("fn-type")),
        "expected an fn-type diagnostic, got: {:?}",
        report.diagnostics
    );
    assert!(
        report.diagnostics.iter().all(|diagnostic| !diagnostic
            .message
            .contains("without a structured diagnostic")),
        "expected structured DTD diagnostics, got: {:?}",
        report.diagnostics
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.span.start.line == 2),
        "expected the invalid fn-type location on line 2, got: {:?}",
        report.diagnostics
    );
    Ok(())
}

#[test]
fn bundled_jats_reports_multiple_dtd_errors_with_locations() -> Result<(), Box<dyn Error>> {
    let resources = jats_resources("1.1");
    let policy = ResourcePolicy::offline([resources.clone()])?;
    let catalog = CatalogResolver::load(resources.join("catalog-jats-v1-1-no-base.xml"), policy)?;
    let engine = Libxml2ProcessEngine::new(env!("CARGO_BIN_EXE_xml-libxml2-helper"))?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;
    let source = format!(
        "<!DOCTYPE article PUBLIC \"{JATS_11_PUBLIC}\" \"{JATS_11_SYSTEM}\">\n\
         <article dtd-version=\"1.1\">\n\
         <front/>\n\
         <body><fn fn-type=\"first-invalid\"/></body>\n\
         <back><fn-group><fn fn-type=\"second-invalid\"/></fn-group></back>\n\
         </article>"
    );
    let report = runtime.block_on(engine.validate(ValidationRequest {
        source: &source,
        revision: Revision(1),
        file: Some("multiple-invalid-jats.xml"),
        catalog: &catalog,
        grammar: GrammarReference::Dtd {
            system_id: JATS_11_SYSTEM,
            public_id: Some(JATS_11_PUBLIC),
        },
    }))?;

    let invalid_values = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.message.contains("fn-type"))
        .collect::<Vec<_>>();
    assert!(
        invalid_values.len() >= 2,
        "expected both invalid fn-type diagnostics, got: {:?}",
        report.diagnostics
    );
    assert!(
        invalid_values
            .iter()
            .all(|diagnostic| diagnostic.span.start.line > 1),
        "expected useful source locations, got: {:?}",
        invalid_values
    );
    Ok(())
}

#[test]
fn bundled_jats_13_validates_offline() -> Result<(), Box<dyn Error>> {
    let resources = jats_resources("1.3");
    let policy = ResourcePolicy::offline([resources.clone()])?;
    let catalog = CatalogResolver::load(resources.join("catalog-jats-v1-3-no-base.xml"), policy)?;
    let engine = Libxml2ProcessEngine::new(env!("CARGO_BIN_EXE_xml-libxml2-helper"))?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;
    let source = format!(
        "<!DOCTYPE article PUBLIC \"{JATS_13_PUBLIC}\" \"{JATS_13_SYSTEM}\">\n\
         <article article-type=\"research-article\" dtd-version=\"1.3\" specific-use=\"sps-1.10\" xml:lang=\"en\">\
         <front><journal-meta><journal-id journal-id-type=\"publisher-id\">example</journal-id><issn pub-type=\"epub\">1234-5678</issn></journal-meta>\
         <article-meta><title-group><article-title>Example</article-title></title-group></article-meta></front>\
         <body><p>Text</p></body><back/></article>"
    );
    let report = runtime.block_on(engine.validate(ValidationRequest {
        source: &source,
        revision: Revision(1),
        file: Some("jats-1.3.xml"),
        catalog: &catalog,
        grammar: GrammarReference::Dtd {
            system_id: JATS_13_SYSTEM,
            public_id: Some(JATS_13_PUBLIC),
        },
    }))?;

    assert!(
        report.diagnostics.is_empty(),
        "expected valid JATS 1.3, got: {:?}",
        report.diagnostics
    );
    Ok(())
}
