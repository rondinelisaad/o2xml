use std::{env, error::Error, fs, path::PathBuf, time::Duration};

use url::Url;
use xml_domain::Revision;
use xml_resource::{CatalogResolver, ResourcePolicy};
use xml_validation::{GrammarReference, JingProcessEngine, ValidationEngine, ValidationRequest};

const UPSTREAM_VERSION: &str = "20241231";
const UPSTREAM_GRAMMAR: &str = "trang/test/compacttest.rnc";
const UPSTREAM_CORPUS: &str = "mod/rng-schema/test/compacttest.xml";

#[test]
#[ignore = "requires an explicit Jing V20241231 checkout, real JAR, and Java runtime"]
fn validates_the_official_compact_corpus_container_with_real_jing() -> Result<(), Box<dyn Error>> {
    let upstream = required_path("O2XML_JING_UPSTREAM_ROOT")?;
    let jing_jar = required_path("O2XML_JING_JAR")?;
    let java = required_path("O2XML_JAVA")?;
    let version = fs::read_to_string(upstream.join("version.xml"))?;
    if !version.contains(&format!(">{UPSTREAM_VERSION}<")) {
        return Err(format!("expected Jing upstream version {UPSTREAM_VERSION}").into());
    }

    let grammar = fs::canonicalize(upstream.join(UPSTREAM_GRAMMAR))?;
    let corpus_path = fs::canonicalize(upstream.join(UPSTREAM_CORPUS))?;
    let corpus = fs::read_to_string(&corpus_path)?;
    let grammar_uri = Url::from_file_path(&grammar)
        .map_err(|()| "cannot convert upstream grammar path to a file URI")?;
    let directory = tempfile::tempdir()?;
    let catalog_path = directory.path().join("catalog.xml");
    fs::write(
        &catalog_path,
        format!(
            "<catalog xmlns=\"urn:oasis:names:tc:entity:xmlns:xml:catalog\"><uri name=\"urn:o2xml:test:jing-compact-corpus\" uri=\"{grammar_uri}\"/></catalog>"
        ),
    )?;
    let policy = ResourcePolicy::offline([upstream.clone(), directory.path().to_path_buf()])?;
    let catalog = CatalogResolver::load(&catalog_path, policy)?;
    let engine = JingProcessEngine::new(java, jing_jar)?.with_deadline(Duration::from_secs(60));
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;
    let report = runtime.block_on(engine.validate(ValidationRequest {
        source: &corpus,
        revision: Revision(1),
        file: Some(UPSTREAM_CORPUS),
        catalog: &catalog,
        grammar: GrammarReference::RelaxNgCompact {
            uri: "urn:o2xml:test:jing-compact-corpus",
        },
    }))?;

    if !report.diagnostics.is_empty() {
        return Err(format!(
            "official compact-syntax corpus produced {} diagnostics; first: {}",
            report.diagnostics.len(),
            report
                .diagnostics
                .first()
                .map_or("unavailable", |diagnostic| diagnostic.message.as_str())
        )
        .into());
    }
    if report.approved_grammar_resources != 2 {
        return Err("unexpected upstream grammar dependency count".into());
    }
    Ok(())
}

fn required_path(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let value = env::var_os(name).ok_or_else(|| format!("{name} is required"))?;
    Ok(fs::canonicalize(PathBuf::from(value))?)
}
