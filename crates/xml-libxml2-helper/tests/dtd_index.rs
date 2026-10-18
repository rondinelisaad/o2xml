use std::{
    collections::BTreeSet,
    error::Error,
    io::{BufRead, BufReader, Error as IoError, ErrorKind, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

use serde::Deserialize;
use serde_json::json;
use xml_dtd::{compile_dtd, DtdElementDeclaration};
use xml_schema::{AttributeUse, ExpandedName};

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum HelperMessage {
    Resource {
        url: String,
        public_id: Option<String>,
    },
    Result {
        valid: bool,
        dtd: Option<Vec<DtdElementDeclaration>>,
    },
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/completion.dtd")
}

fn name(local_name: &str) -> ExpandedName {
    ExpandedName {
        namespace_uri: None,
        local_name: local_name.to_owned(),
    }
}

#[test]
fn extracts_dtd_declarations_for_the_safe_compiler() -> Result<(), Box<dyn Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_xml-libxml2-helper"))
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| IoError::new(ErrorKind::BrokenPipe, "helper stdin unavailable"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| IoError::new(ErrorKind::BrokenPipe, "helper stdout unavailable"))?;
    let mut stdout = BufReader::new(stdout);
    serde_json::to_writer(
        &mut stdin,
        &json!({
            "mode": "dtd_index",
            "grammar_uri": "https://example.org/completion.dtd",
            "public_id": null,
            "document_bytes": 0
        }),
    )?;
    stdin.write_all(b"\n\n")?;
    stdin.flush()?;

    let declarations = loop {
        let mut line = String::new();
        if stdout.read_line(&mut line)? == 0 {
            return Err(IoError::new(ErrorKind::UnexpectedEof, "helper closed stdout").into());
        }
        match serde_json::from_str::<HelperMessage>(line.trim_end())? {
            HelperMessage::Resource { url, public_id } => {
                assert_eq!(url, "https://example.org/completion.dtd");
                assert_eq!(public_id, None);
                let bytes = std::fs::read(fixture())?;
                serde_json::to_writer(
                    &mut stdin,
                    &json!({
                        "allowed": true,
                        "uri": "file:///fixtures/completion.dtd",
                        "bytes": bytes.len()
                    }),
                )?;
                stdin.write_all(b"\n")?;
                stdin.write_all(&bytes)?;
                stdin.write_all(b"\n")?;
                stdin.flush()?;
            }
            HelperMessage::Result { valid, dtd } => {
                assert!(valid);
                break dtd.ok_or_else(|| {
                    IoError::new(ErrorKind::InvalidData, "helper omitted DTD declarations")
                })?;
            }
        }
    };
    drop(stdin);
    assert!(child.wait()?.success());

    let index = compile_dtd(declarations)?;
    let article = name("article");
    let after_front = index.complete_children(&article, &[name("front"), name("body")])?;
    assert_eq!(
        after_front.children,
        vec![name("back"), name("body"), name("floats-group")]
    );
    assert!(after_front.can_close);

    let attributes = index.complete_attributes(&article, &BTreeSet::new())?;
    assert_eq!(attributes.len(), 3);
    assert!(attributes.iter().any(|attribute| {
        attribute.name == name("article-type")
            && attribute.use_kind == AttributeUse::Required
            && attribute.allowed_values == ["research-article", "review-article"]
    }));
    assert!(attributes.iter().any(|attribute| {
        attribute.name == name("xml:lang") && attribute.use_kind == AttributeUse::Optional
    }));
    Ok(())
}
