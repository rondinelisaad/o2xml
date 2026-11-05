use std::{error::Error, path::PathBuf};

use xml_domain::Revision;
use xml_packtools::PacktoolsProcessEngine;

const SPS_1_10: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<article xmlns:xlink="http://www.w3.org/1999/xlink"
         dtd-version="1.3" specific-use="sps-1.10"
         article-type="research-article" xml:lang="en">
  <front>
    <journal-meta><journal-title-group><journal-title>Test</journal-title></journal-title-group></journal-meta>
    <article-meta>
      <title-group><article-title>Packtools test</article-title></title-group>
      <author-notes><fn fn-type="coi-statement"><p>No conflicts.</p></fn></author-notes>
    </article-meta>
  </front>
  <body><p>Test.</p></body>
  <back><fn-group><fn fn-type="data-availability"><p>Data available.</p></fn></fn-group></back>
  <sub-article id="SA1" article-type="reviewer-report" xml:lang="en">
    <front-stub><title-group><article-title>Review</article-title></title-group></front-stub>
    <body><p>Review text.</p></body>
  </sub-article>
</article>"#;

#[test]
fn runs_packtools_4_18_3_and_accepts_current_sps_values() -> Result<(), Box<dyn Error>> {
    let executable_name = if cfg!(windows) {
        "scixml-packtools-validator.exe"
    } else {
        "scixml-packtools-validator"
    };
    let executable = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/packtools-sidecar")
        .join(executable_name);
    let engine = PacktoolsProcessEngine::new(executable)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;
    let report = runtime.block_on(engine.validate(
        SPS_1_10,
        Revision(1),
        Some("packtools-smoke.xml"),
        "pt_BR",
    ))?;

    assert_eq!(report.version, "4.18.3");
    assert_eq!(report.engine, "packtools-4.18.3");
    assert!(report.diagnostics.iter().all(|diagnostic| {
        !diagnostic.message.contains("Invalid value 'coi-statement'")
            && !diagnostic
                .message
                .contains("Invalid value 'data-availability'")
            && !diagnostic
                .message
                .contains("Invalid value 'reviewer-report'")
    }));
    assert!(report
        .diagnostics
        .iter()
        .all(|diagnostic| diagnostic.span.start.line > 0));
    Ok(())
}
