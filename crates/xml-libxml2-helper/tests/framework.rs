use std::{error::Error, fs, path::PathBuf};

use xml_domain::Revision;
use xml_framework::{FrameworkRegistry, ValidationLanguage};
use xml_resource::{CatalogResolver, ResourcePolicy};
use xml_validation::{GrammarReference, Libxml2ProcessEngine, ValidationEngine, ValidationRequest};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testdata/project")
}

#[test]
fn validates_synthetic_sps_framework_as_separate_stages() -> Result<(), Box<dyn Error>> {
    let root = project_root().canonicalize()?;
    let policy = ResourcePolicy::offline([root.clone()])?;
    let catalog = CatalogResolver::load(root.join("catalog.xml"), policy)?;
    let registry = FrameworkRegistry::with_builtin_scientific_frameworks()?;
    let plan = registry.validation_plan("SPS-1.10")?;
    let source = fs::read_to_string(root.join("xml/article.xml"))?
        .lines()
        .filter(|line| !line.contains("<!DOCTYPE"))
        .collect::<Vec<_>>()
        .join("\n");
    let engine = Libxml2ProcessEngine::new(env!("CARGO_BIN_EXE_xml-libxml2-helper"))?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;
    let mut results = Vec::new();

    for stage in plan {
        let grammar = match stage.language {
            ValidationLanguage::Dtd => GrammarReference::Dtd {
                system_id: &stage.logical_uri,
                public_id: None,
            },
            ValidationLanguage::Xsd10 => GrammarReference::Xsd {
                uri: &stage.logical_uri,
            },
            ValidationLanguage::RelaxNg => GrammarReference::RelaxNg {
                uri: &stage.logical_uri,
            },
            ValidationLanguage::Schematron => GrammarReference::Schematron {
                uri: &stage.logical_uri,
            },
        };
        let report = runtime.block_on(engine.validate(ValidationRequest {
            source: &source,
            revision: Revision(1),
            file: Some("article.xml"),
            catalog: &catalog,
            grammar,
        }))?;
        results.push((stage.source_label, report.diagnostics));
    }

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].0, "JATS 1.3");
    assert_eq!(results[1].0, "SPS 1.10");
    assert!(results
        .iter()
        .all(|(_, diagnostics)| diagnostics.is_empty()));
    Ok(())
}
