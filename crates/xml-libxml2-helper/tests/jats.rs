use std::{error::Error, path::PathBuf};

use xml_domain::Revision;
use xml_resource::{CatalogResolver, ResourcePolicy};
use xml_validation::{GrammarReference, Libxml2ProcessEngine, ValidationEngine, ValidationRequest};

const JATS_11_PUBLIC: &str = "-//NLM//DTD JATS (Z39.96) Journal Publishing DTD v1.1 20151215//EN";
const JATS_11_SYSTEM: &str = "https://jats.nlm.nih.gov/publishing/1.1/JATS-journalpublishing1.dtd";

fn jats_resources() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../app/src-tauri/resources/jats/1.1")
}

#[test]
fn bundled_jats_rejects_invalid_fn_type_offline() -> Result<(), Box<dyn Error>> {
    let resources = jats_resources();
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
    Ok(())
}
