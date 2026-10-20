use std::{collections::BTreeMap, error::Error, fs, path::PathBuf};

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
        parameters: None,
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
        parameters: None,
    }));

    assert!(result.is_err());
    Ok(())
}

#[test]
fn passes_quoted_string_parameters_to_stylesheet() -> Result<(), Box<dyn Error>> {
    let root = project_root().canonicalize()?;
    let policy = ResourcePolicy::offline([root.clone()])?;
    let catalog = CatalogResolver::load(root.join("catalog.xml"), policy)?;
    let source = fs::read_to_string(root.join("xml/article.xml"))?;
    let engine = LibxsltProcessEngine::new(env!("CARGO_BIN_EXE_xml-libxml2-helper"))?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;
    let parameters = BTreeMap::from([("article_lang".to_owned(), "pt'BR".to_owned())]);
    let report = runtime.block_on(engine.transform(LibxsltRequest {
        source: &source,
        revision: Revision(13),
        catalog: &catalog,
        stylesheet_uri: "urn:o2xml:transform:jats-html",
        parameters: Some(&parameters),
    }))?;

    let output = String::from_utf8(report.output)?;
    assert!(output.contains("<p id=\"article-language\">pt'BR</p>"));
    Ok(())
}

#[test]
fn renders_bundled_packtools_scielo_preview_offline() -> Result<(), Box<dyn Error>> {
    let resources = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../app/src-tauri/resources/packtools-html/4.16.9")
        .canonicalize()?;
    let policy = ResourcePolicy::offline([resources.clone()])?;
    let catalog = CatalogResolver::load(resources.join("catalog.xml"), policy)?;
    let stylesheet = url::Url::from_file_path(resources.join("root-html-2.0.xslt"))
        .map_err(|()| "invalid Packtools stylesheet path")?;
    let source = r#"<article xmlns:xlink="http://www.w3.org/1999/xlink" article-type="research-article" xml:lang="pt" dtd-version="1.1"><front><journal-meta><journal-title-group><journal-title>Revista Teste</journal-title><abbrev-journal-title abbrev-type="publisher">Rev. Teste</abbrev-journal-title></journal-title-group><issn pub-type="epub">1234-5678</issn><publisher><publisher-name>SciELO</publisher-name></publisher></journal-meta><article-meta><article-id pub-id-type="doi">10.1234/test</article-id><title-group><article-title>Título de teste</article-title></title-group><contrib-group><contrib contrib-type="author"><name><surname>Silva</surname><given-names>Maria</given-names></name></contrib></contrib-group><pub-date publication-format="electronic" date-type="pub"><year>2026</year></pub-date><volume>1</volume><elocation-id>e1</elocation-id><abstract><title>Resumo</title><p>Conteúdo do resumo.</p></abstract></article-meta></front><body><sec><title>Introdução</title><p>Texto do artigo.</p></sec></body><back><ref-list><title>Referências</title></ref-list></back></article>"#;
    let parameters = BTreeMap::from([
        ("article_lang".to_owned(), "pt".to_owned()),
        ("is_translation".to_owned(), "False".to_owned()),
    ]);
    let engine = LibxsltProcessEngine::new(env!("CARGO_BIN_EXE_xml-libxml2-helper"))?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;
    let report = runtime.block_on(engine.transform(LibxsltRequest {
        source,
        revision: Revision(14),
        catalog: &catalog,
        stylesheet_uri: stylesheet.as_str(),
        parameters: Some(&parameters),
    }))?;

    let output = String::from_utf8(report.output)?;
    assert!(output.contains("Título de teste"));
    assert!(output.contains("Conteúdo do resumo."));
    assert!(output.contains("Texto do artigo."));
    assert!(output.contains("id=\"standalonearticle\""));
    Ok(())
}
