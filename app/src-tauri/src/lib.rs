mod audit;

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    env, fs,
    io::Write,
    path::{Component, Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
    time::SystemTime,
};

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use quick_xml::{events::Event, Reader, XmlVersion};
use serde::{Deserialize, Serialize};
use tauri::{path::BaseDirectory, AppHandle, Manager, State};
use tracing_subscriber::EnvFilter;
use xml_author::AuthorDocument;
use xml_diff::{XmlDiffOptions, XmlDiffReport};
use xml_document::{LineIndex, XmlDocument};
use xml_domain::{Diagnostic, OutlineEntry, Revision, Severity};
use xml_dtd::{DtdIndexEngine, DtdIndexRequest, Libxml2DtdIndexEngine};
use xml_framework::{FrameworkRegistry, ValidationLanguage};
use xml_grid::XmlGrid;
use xml_jats::JatsInspection;
use xml_project::{ProjectManifest, TransformEngine, TransformMediaType};
use xml_resource::{CatalogResolver, ResourcePolicy};
use xml_schema::{AttributeUse, ExpandedName, SchemaIndex, ValueConstraint};
use xml_schematron::{parse_svrl, SchXsltProcessEngine, SchXsltRequest};
use xml_transform::{
    LibxsltProcessEngine, LibxsltRequest, SaxonProcessEngine, SaxonXQueryEngine, TransformRequest,
    XQueryRequest,
};
use xml_validation::{GrammarReference, Libxml2ProcessEngine, ValidationEngine, ValidationRequest};
use xml_xpath::{Libxml2XPathEngine, XPathEngine, XPathReport, XPathRequest, XPathValue};
use xml_xsd::{Libxml2XsdIndexEngine, XsdIndexEngine, XsdIndexRequest};

#[derive(Default)]
struct AppState {
    next_id: AtomicU64,
    documents: Mutex<HashMap<u64, XmlDocument>>,
    file_stamps: Mutex<HashMap<u64, FileStamp>>,
    project: Mutex<Option<OpenedProject>>,
    dtd_indexes: Mutex<HashMap<(String, Option<String>), SchemaIndex>>,
    xsd_indexes: Mutex<HashMap<String, SchemaIndex>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum FileStamp {
    Missing,
    Present {
        len: u64,
        modified: Option<SystemTime>,
    },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExternalChangePayload {
    status: ExternalChangeStatus,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum ExternalChangeStatus {
    Unchanged,
    Modified,
    Deleted,
}

#[derive(Clone)]
struct OpenedProject {
    root: PathBuf,
    manifest: ProjectManifest,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DocumentPayload {
    id: u64,
    name: String,
    path: Option<String>,
    text: String,
    revision: Revision,
    dirty: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AnalysisPayload {
    revision: Revision,
    diagnostics: Vec<Diagnostic>,
    outline: Vec<OutlineEntry>,
    jats_inspection: Option<JatsInspection>,
    framework_stages: Vec<FrameworkStagePayload>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FrameworkStagePayload {
    id: String,
    label: String,
    engine: &'static str,
    status: FrameworkStageStatus,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum FrameworkStageStatus {
    Passed,
    Failed,
    Unavailable,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectPayload {
    path: String,
    manifest: ProjectManifest,
    files: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TransformPayload {
    revision: Revision,
    scenario: String,
    engine: &'static str,
    media_type: TransformMediaType,
    output_path: String,
    bytes_written: usize,
    preview: String,
    preview_truncated: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct XQueryPayload {
    revision: Revision,
    engine: &'static str,
    bytes: usize,
    preview: String,
    truncated: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DiffPayload {
    revision: Revision,
    other_name: String,
    other_text: String,
    report: XmlDiffReport,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MergePayload {
    revision: Revision,
    text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GridPayload {
    revision: Revision,
    grid: XmlGrid,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GridEditPayload {
    revision: Revision,
    text: String,
    schema_validated: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthorPayload {
    revision: Revision,
    document: AuthorDocument,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthorPreviewPayload {
    revision: Revision,
    engine: &'static str,
    html: String,
    stylesheet: String,
    assets: Vec<AuthorPreviewAsset>,
    assets_skipped: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthorPreviewAsset {
    source: String,
    data_uri: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthorEditPayload {
    revision: Revision,
    text: String,
    schema_validated: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DtdCompletionPayload {
    revision: Revision,
    children: Vec<String>,
    attributes: Vec<DtdAttributePayload>,
    can_close: bool,
    accepts_any: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DtdAttributePayload {
    name: String,
    required: bool,
    values: Vec<String>,
    default_value: Option<String>,
    fixed: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct XsdCompletionPayload {
    revision: Revision,
    children: Vec<ExpandedName>,
    attributes: Vec<XsdAttributePayload>,
    can_close: bool,
    accepts_any: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct XsdAttributePayload {
    name: ExpandedName,
    required: bool,
    values: Vec<String>,
    default_value: Option<String>,
    fixed: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct XsdCompletionContext {
    parent: ExpandedName,
    preceding_children: Vec<ExpandedName>,
    present_attributes: Vec<ExpandedName>,
}

fn display_name(path: Option<&Path>) -> String {
    path.and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .unwrap_or("untitled.xml")
        .to_owned()
}

fn normalized_path(path: &Path) -> Result<PathBuf, String> {
    path.canonicalize()
        .map_err(|error| format!("cannot resolve selected path: {error}"))
}

fn file_stamp(path: &Path) -> FileStamp {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => FileStamp::Present {
            len: metadata.len(),
            modified: metadata.modified().ok(),
        },
        _ => FileStamp::Missing,
    }
}

fn document_payload(id: u64, document: &XmlDocument) -> DocumentPayload {
    DocumentPayload {
        id,
        name: display_name(document.path().map(PathBuf::as_path)),
        path: document
            .path()
            .map(|path| path.to_string_lossy().into_owned()),
        text: document.snapshot().text(),
        revision: document.revision(),
        dirty: document.is_dirty(),
    }
}

#[tauri::command]
fn new_document(state: State<'_, AppState>) -> Result<DocumentPayload, String> {
    let id = state.next_id.fetch_add(1, Ordering::Relaxed) + 1;
    let document = XmlDocument::new("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n", None);
    let payload = document_payload(id, &document);
    state
        .documents
        .lock()
        .map_err(|_| "document store is unavailable".to_owned())?
        .insert(id, document);
    audit::event(
        "document.new",
        "success",
        "document",
        id,
        Some(Revision(0)),
        None,
    );
    Ok(payload)
}

#[tauri::command]
fn open_document(path: String, state: State<'_, AppState>) -> Result<DocumentPayload, String> {
    let canonical = normalized_path(Path::new(&path))?;
    let metadata =
        fs::metadata(&canonical).map_err(|error| format!("cannot inspect file: {error}"))?;
    if !metadata.is_file() {
        return Err("selected path is not a file".to_owned());
    }
    const MAX_MVP_BYTES: u64 = 128 * 1024 * 1024;
    if metadata.len() > MAX_MVP_BYTES {
        return Err("file exceeds the Phase 1 safety limit of 128 MiB".to_owned());
    }
    let text = fs::read_to_string(&canonical)
        .map_err(|error| format!("cannot read UTF-8 XML file: {error}"))?;
    let id = state.next_id.fetch_add(1, Ordering::Relaxed) + 1;
    let document = XmlDocument::new(&text, Some(canonical));
    let payload = document_payload(id, &document);
    state
        .documents
        .lock()
        .map_err(|_| "document store is unavailable".to_owned())?
        .insert(id, document);
    state
        .file_stamps
        .lock()
        .map_err(|_| "file state store is unavailable".to_owned())?
        .insert(
            id,
            file_stamp(Path::new(payload.path.as_deref().unwrap_or_default())),
        );
    audit::event(
        "document.open",
        "success",
        "document",
        id,
        Some(Revision(0)),
        None,
    );
    Ok(payload)
}

#[tauri::command]
fn close_document(id: u64, state: State<'_, AppState>) -> Result<(), String> {
    let removed = state
        .documents
        .lock()
        .map_err(|_| "document store is unavailable".to_owned())?
        .remove(&id)
        .is_some();
    state
        .file_stamps
        .lock()
        .map_err(|_| "file state store is unavailable".to_owned())?
        .remove(&id);
    if !removed {
        return Err("document is not open".to_owned());
    }
    audit::event("document.close", "success", "document", id, None, None);
    Ok(())
}

#[tauri::command]
fn check_external_change(
    id: u64,
    state: State<'_, AppState>,
) -> Result<ExternalChangePayload, String> {
    let path = state
        .documents
        .lock()
        .map_err(|_| "document store is unavailable".to_owned())?
        .get(&id)
        .ok_or_else(|| "document is not open".to_owned())?
        .path()
        .cloned();
    let Some(path) = path else {
        return Ok(ExternalChangePayload {
            status: ExternalChangeStatus::Unchanged,
        });
    };
    let current = file_stamp(&path);
    let expected = state
        .file_stamps
        .lock()
        .map_err(|_| "file state store is unavailable".to_owned())?
        .get(&id)
        .cloned()
        .unwrap_or_else(|| current.clone());
    let status = if current == expected {
        ExternalChangeStatus::Unchanged
    } else if matches!(current, FileStamp::Missing) {
        ExternalChangeStatus::Deleted
    } else {
        ExternalChangeStatus::Modified
    };
    Ok(ExternalChangePayload { status })
}

#[tauri::command]
fn acknowledge_external_change(id: u64, state: State<'_, AppState>) -> Result<(), String> {
    let path = state
        .documents
        .lock()
        .map_err(|_| "document store is unavailable".to_owned())?
        .get(&id)
        .ok_or_else(|| "document is not open".to_owned())?
        .path()
        .cloned();
    if let Some(path) = path {
        state
            .file_stamps
            .lock()
            .map_err(|_| "file state store is unavailable".to_owned())?
            .insert(id, file_stamp(&path));
    }
    Ok(())
}

#[tauri::command]
fn reload_document(id: u64, state: State<'_, AppState>) -> Result<DocumentPayload, String> {
    const MAX_MVP_BYTES: u64 = 128 * 1024 * 1024;
    let mut documents = state
        .documents
        .lock()
        .map_err(|_| "document store is unavailable".to_owned())?;
    let document = documents
        .get_mut(&id)
        .ok_or_else(|| "document is not open".to_owned())?;
    let path = document
        .path()
        .cloned()
        .ok_or_else(|| "document has no file path".to_owned())?;
    let metadata = fs::metadata(&path).map_err(|error| format!("cannot inspect file: {error}"))?;
    if !metadata.is_file() || metadata.len() > MAX_MVP_BYTES {
        return Err("file is unavailable or exceeds the 128 MiB safety limit".to_owned());
    }
    let text = fs::read_to_string(&path)
        .map_err(|error| format!("cannot read UTF-8 XML file: {error}"))?;
    document
        .replace_all(document.revision(), &text)
        .map_err(|error| error.to_string())?;
    document.mark_saved(path.clone());
    let payload = document_payload(id, document);
    drop(documents);
    state
        .file_stamps
        .lock()
        .map_err(|_| "file state store is unavailable".to_owned())?
        .insert(id, file_stamp(&path));
    audit::event(
        "document.reload",
        "success",
        "document",
        id,
        Some(payload.revision),
        None,
    );
    Ok(payload)
}

#[tauri::command]
fn update_document(
    id: u64,
    base_revision: Revision,
    text: String,
    state: State<'_, AppState>,
) -> Result<Revision, String> {
    const MAX_MVP_BYTES: usize = 128 * 1024 * 1024;
    if text.len() > MAX_MVP_BYTES {
        return Err("document exceeds the Phase 1 safety limit of 128 MiB".to_owned());
    }
    let mut documents = state
        .documents
        .lock()
        .map_err(|_| "document store is unavailable".to_owned())?;
    let document = documents
        .get_mut(&id)
        .ok_or_else(|| "document is not open".to_owned())?;
    document
        .replace_all(base_revision, &text)
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn analyze_document(
    id: u64,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<AnalysisPayload, String> {
    let (snapshot, file) = {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        let document = documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?;
        (
            document.snapshot(),
            document
                .path()
                .map(|path| path.to_string_lossy().into_owned()),
        )
    };
    let project = state
        .project
        .lock()
        .map_err(|_| "project store is unavailable".to_owned())?
        .clone();
    let revision = snapshot.revision();
    let worker_file = file.clone();
    let mut analysis = tauri::async_runtime::spawn_blocking(move || {
        let text = snapshot.text();
        let diagnostics = xml_syntax::validate_well_formed(&text, revision, worker_file.as_deref());
        let outline = if diagnostics.is_empty() {
            xml_syntax::outline(&text)
        } else {
            Vec::new()
        };
        let jats_inspection = if diagnostics.is_empty() {
            xml_jats::inspect(&text)
                .ok()
                .filter(|inspection| inspection.is_jats)
        } else {
            None
        };
        AnalysisPayload {
            revision,
            diagnostics,
            outline,
            jats_inspection,
            framework_stages: Vec::new(),
        }
    })
    .await
    .map_err(|error| format!("validation worker failed: {error}"))?;
    if analysis.diagnostics.is_empty() {
        if let Some(project) = project.filter(|project| {
            file.as_deref()
                .map(Path::new)
                .is_none_or(|path| path.starts_with(&project.root))
        }) {
            let text = {
                let documents = state
                    .documents
                    .lock()
                    .map_err(|_| "document store is unavailable".to_owned())?;
                let document = documents
                    .get(&id)
                    .ok_or_else(|| "document is not open".to_owned())?;
                if document.revision() != revision {
                    return Ok(analysis);
                }
                document.snapshot().text()
            };
            analysis.framework_stages =
                validate_project_framework(&project, &text, revision, file.as_deref()).await?;
            analysis.diagnostics.extend(
                analysis
                    .framework_stages
                    .iter()
                    .flat_map(|stage| stage.diagnostics.iter().cloned()),
            );
        } else if analysis.jats_inspection.is_some() {
            let text = {
                let documents = state
                    .documents
                    .lock()
                    .map_err(|_| "document store is unavailable".to_owned())?;
                let document = documents
                    .get(&id)
                    .ok_or_else(|| "document is not open".to_owned())?;
                if document.revision() != revision {
                    return Ok(analysis);
                }
                document.snapshot().text()
            };
            analysis.framework_stages =
                validate_builtin_jats(&app, &text, revision, file.as_deref()).await?;
            analysis.diagnostics.extend(
                analysis
                    .framework_stages
                    .iter()
                    .flat_map(|stage| stage.diagnostics.iter().cloned()),
            );
        }
    }
    let result = if analysis.diagnostics.is_empty() {
        "success"
    } else {
        "failure"
    };
    audit::event(
        "validation.run",
        result,
        "document",
        id,
        Some(revision),
        Some("quick-xml"),
    );
    Ok(analysis)
}

async fn validate_builtin_jats(
    app: &AppHandle,
    source: &str,
    revision: Revision,
    file: Option<&str>,
) -> Result<Vec<FrameworkStagePayload>, String> {
    let Some(document_type) =
        xml_syntax::document_type(source).map_err(|error| error.to_string())?
    else {
        return Ok(Vec::new());
    };
    let Some(bundled) = bundled_jats_dtd(
        document_type.public_id.as_deref(),
        document_type.system_id.as_deref(),
    ) else {
        return Ok(Vec::new());
    };
    let catalog_path = bundled_resource(app, bundled.catalog_resource)?;
    let root = catalog_path
        .parent()
        .ok_or_else(|| "built-in JATS catalog has no parent directory".to_owned())?;
    let policy =
        ResourcePolicy::offline([root.to_path_buf()]).map_err(|error| error.to_string())?;
    let catalog =
        CatalogResolver::load(&catalog_path, policy).map_err(|error| error.to_string())?;
    let engine =
        Libxml2ProcessEngine::new(libxml2_helper_path()?).map_err(|error| error.to_string())?;
    let report = engine
        .validate(ValidationRequest {
            source,
            revision,
            file,
            catalog: &catalog,
            grammar: GrammarReference::Dtd {
                system_id: bundled.canonical_system_id,
                public_id: Some(bundled.public_id),
            },
        })
        .await
        .map_err(|error| error.to_string())?;
    let status = if report.diagnostics.is_empty() {
        FrameworkStageStatus::Passed
    } else {
        FrameworkStageStatus::Failed
    };
    let mut stages = vec![FrameworkStagePayload {
        id: "jats-grammar".to_owned(),
        label: bundled.label.to_owned(),
        engine: "libxml2",
        status,
        diagnostics: report.diagnostics,
    }];
    if let Some(sps_version) = detected_sps_version(source)? {
        stages.push(validate_builtin_sps(app, source, revision, file, sps_version).await?);
    }
    Ok(stages)
}

fn detected_sps_version(source: &str) -> Result<Option<&'static str>, String> {
    let mut reader = Reader::from_str(source);
    reader.config_mut().trim_text(false);
    loop {
        match reader.read_event().map_err(|error| error.to_string())? {
            Event::Start(element) | Event::Empty(element)
                if element.name().as_ref() == b"article" =>
            {
                for attribute in element.attributes() {
                    let attribute = attribute.map_err(|error| error.to_string())?;
                    if attribute.key.as_ref() != b"specific-use" {
                        continue;
                    }
                    let value = attribute
                        .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
                        .map_err(|error| error.to_string())?;
                    return Ok(match value.trim().to_ascii_lowercase().as_str() {
                        "sps-1.9" => Some("1.9"),
                        "sps-1.10" => Some("1.10"),
                        _ => None,
                    });
                }
                return Ok(None);
            }
            Event::Eof => return Ok(None),
            _ => {}
        }
    }
}

async fn validate_builtin_sps(
    app: &AppHandle,
    source: &str,
    revision: Revision,
    file: Option<&str>,
    version: &'static str,
) -> Result<FrameworkStagePayload, String> {
    let catalog_path = bundled_resource(app, "resources/sps/catalog.xml")?;
    let root = catalog_path
        .parent()
        .ok_or_else(|| "built-in SPS catalog has no parent directory".to_owned())?;
    let policy =
        ResourcePolicy::offline([root.to_path_buf()]).map_err(|error| error.to_string())?;
    let catalog =
        CatalogResolver::load(&catalog_path, policy).map_err(|error| error.to_string())?;
    let stylesheet_uri = format!("urn:o2xml:framework:sps:{version}:compiled-schematron");
    let engine =
        LibxsltProcessEngine::new(libxml2_helper_path()?).map_err(|error| error.to_string())?;
    let transformed = engine
        .transform(LibxsltRequest {
            source,
            revision,
            catalog: &catalog,
            stylesheet_uri: &stylesheet_uri,
            parameters: None,
        })
        .await
        .map_err(|error| error.to_string())?;
    let svrl = String::from_utf8(transformed.output)
        .map_err(|_| "SPS validator produced non-UTF-8 SVRL".to_owned())?;
    let report = parse_svrl(&svrl).map_err(|error| error.to_string())?;
    let locations = match resolve_svrl_locations(source, revision, &report).await {
        Ok(locations) => locations,
        Err(error) => {
            tracing::warn!(
                action = "validation.sps_location_resolution",
                result = "failure",
                error = %error,
            );
            BTreeMap::new()
        }
    };
    let mut diagnostics = report.diagnostics(source, revision, file, &locations);
    let label = format!("SPS {version}");
    for diagnostic in &mut diagnostics {
        diagnostic.source.clone_from(&label);
    }
    let status = if diagnostics.is_empty() {
        FrameworkStageStatus::Passed
    } else {
        FrameworkStageStatus::Failed
    };
    Ok(FrameworkStagePayload {
        id: "sps-rules".to_owned(),
        label,
        engine: "packtools-schematron-libxslt",
        status,
        diagnostics,
    })
}

async fn resolve_svrl_locations(
    source: &str,
    revision: Revision,
    report: &xml_schematron::SvrlReport,
) -> Result<BTreeMap<String, xml_domain::SourceSpan>, String> {
    const MAX_LOCATION_EXPRESSION_BYTES: usize = 60 * 1024;
    let mut expression = String::new();
    for location in report
        .findings
        .iter()
        .filter_map(|finding| finding.location.as_deref())
        .collect::<BTreeSet<_>>()
    {
        let separator_bytes = usize::from(!expression.is_empty()).saturating_mul(3);
        if expression
            .len()
            .saturating_add(separator_bytes)
            .saturating_add(location.len())
            > MAX_LOCATION_EXPRESSION_BYTES
        {
            break;
        }
        if !expression.is_empty() {
            expression.push_str(" | ");
        }
        expression.push_str(location);
    }
    if expression.is_empty() {
        return Ok(BTreeMap::new());
    }
    let engine =
        Libxml2XPathEngine::new(libxml2_helper_path()?).map_err(|error| error.to_string())?;
    let namespaces = BTreeMap::new();
    let xpath = engine
        .evaluate(XPathRequest {
            source,
            revision,
            expression: &expression,
            namespaces: &namespaces,
        })
        .await
        .map_err(|error| error.to_string())?;
    let XPathValue::Nodes { items, .. } = xpath.value else {
        return Ok(BTreeMap::new());
    };
    Ok(items
        .into_iter()
        .map(|node| (node.path, node.span))
        .collect())
}

#[derive(Clone, Copy)]
struct BundledJatsDtd {
    label: &'static str,
    public_id: &'static str,
    canonical_system_id: &'static str,
    system_ids: &'static [&'static str],
    catalog_resource: &'static str,
}

const BUNDLED_JATS_DTDS: [BundledJatsDtd; 2] = [
    BundledJatsDtd {
        label: "JATS 1.1",
        public_id: "-//NLM//DTD JATS (Z39.96) Journal Publishing DTD v1.1 20151215//EN",
        canonical_system_id: "https://jats.nlm.nih.gov/publishing/1.1/JATS-journalpublishing1.dtd",
        system_ids: &[
            "http://jats.nlm.nih.gov/publishing/1.1/JATS-journalpublishing1.dtd",
            "https://jats.nlm.nih.gov/publishing/1.1/JATS-journalpublishing1.dtd",
        ],
        catalog_resource: "resources/jats/1.1/catalog-jats-v1-1-no-base.xml",
    },
    BundledJatsDtd {
        label: "JATS 1.3",
        public_id: "-//NLM//DTD JATS (Z39.96) Journal Publishing DTD v1.3 20210610//EN",
        canonical_system_id:
            "https://jats.nlm.nih.gov/publishing/1.3/JATS-journalpublishing1-3.dtd",
        system_ids: &[
            "http://jats.nlm.nih.gov/publishing/1.3/JATS-journalpublishing1-3.dtd",
            "https://jats.nlm.nih.gov/publishing/1.3/JATS-journalpublishing1-3.dtd",
        ],
        catalog_resource: "resources/jats/1.3/catalog-jats-v1-3-no-base.xml",
    },
];

fn bundled_jats_dtd(public_id: Option<&str>, system_id: Option<&str>) -> Option<BundledJatsDtd> {
    BUNDLED_JATS_DTDS.iter().copied().find(|candidate| {
        public_id == Some(candidate.public_id)
            || system_id.is_some_and(|value| candidate.system_ids.contains(&value))
    })
}

fn bundled_resource(app: &AppHandle, relative: &str) -> Result<PathBuf, String> {
    let bundled = app
        .path()
        .resolve(relative, BaseDirectory::Resource)
        .map_err(|error| format!("cannot locate application resources: {error}"))?;
    if bundled.is_file() {
        return Ok(bundled);
    }
    let development = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative);
    if development.is_file() {
        return Ok(development);
    }
    Err(format!(
        "required application resource is missing: {relative}"
    ))
}

async fn validate_project_framework(
    project: &OpenedProject,
    source: &str,
    revision: Revision,
    file: Option<&str>,
) -> Result<Vec<FrameworkStagePayload>, String> {
    let Some(framework_id) = project.manifest.validation.framework.as_deref() else {
        return Ok(Vec::new());
    };
    let registry = FrameworkRegistry::with_builtin_scientific_frameworks()
        .map_err(|error| error.to_string())?;
    let plan = registry
        .validation_plan(framework_id)
        .map_err(|error| error.to_string())?;
    let catalog_setting = project
        .manifest
        .xml
        .catalog
        .as_deref()
        .ok_or_else(|| "framework validation requires an XML catalog".to_owned())?;
    let catalog_path = normalized_path(&project.root.join(catalog_setting))?;
    if !catalog_path.starts_with(&project.root) {
        return Err("project catalog is outside the project root".to_owned());
    }
    let policy =
        ResourcePolicy::offline([project.root.clone()]).map_err(|error| error.to_string())?;
    let catalog = CatalogResolver::load(catalog_path, policy).map_err(|error| error.to_string())?;
    let engine =
        Libxml2ProcessEngine::new(libxml2_helper_path()?).map_err(|error| error.to_string())?;
    let mut stages = Vec::with_capacity(plan.len());
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
        let result = if stage.language == ValidationLanguage::Schematron {
            validate_schematron_stage(
                &engine,
                &catalog,
                &stage.logical_uri,
                source,
                revision,
                file,
            )
            .await
        } else {
            engine
                .validate(ValidationRequest {
                    source,
                    revision,
                    file,
                    catalog: &catalog,
                    grammar,
                })
                .await
                .map(|report| (report.diagnostics, "libxml2"))
                .map_err(|error| error.to_string())
        };
        let (status, diagnostics, selected_engine) = match result {
            Ok((mut diagnostics, selected_engine)) => {
                for diagnostic in &mut diagnostics {
                    diagnostic.source.clone_from(&stage.source_label);
                }
                let status = if diagnostics.is_empty() {
                    FrameworkStageStatus::Passed
                } else {
                    FrameworkStageStatus::Failed
                };
                (status, diagnostics, selected_engine)
            }
            Err(error) => {
                let selected_engine = if stage.language == ValidationLanguage::Schematron {
                    "schematron"
                } else {
                    "libxml2"
                };
                (
                    FrameworkStageStatus::Unavailable,
                    vec![framework_stage_error(
                        source,
                        revision,
                        file,
                        &stage.source_label,
                        &error,
                    )],
                    selected_engine,
                )
            }
        };
        stages.push(FrameworkStagePayload {
            id: stage.id,
            label: stage.source_label,
            engine: selected_engine,
            status,
            diagnostics,
        });
    }
    Ok(stages)
}

async fn validate_schematron_stage(
    baseline: &Libxml2ProcessEngine,
    catalog: &CatalogResolver,
    logical_uri: &str,
    source: &str,
    revision: Revision,
    file: Option<&str>,
) -> Result<(Vec<Diagnostic>, &'static str), String> {
    let Some(engine) = schxslt_engine_from_environment()? else {
        return baseline
            .validate(ValidationRequest {
                source,
                revision,
                file,
                catalog,
                grammar: GrammarReference::Schematron { uri: logical_uri },
            })
            .await
            .map(|report| (report.diagnostics, "libxml2-schematron-baseline"))
            .map_err(|error| error.to_string());
    };
    let resource = catalog
        .resolve_uri(logical_uri)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "Schematron rules are not installed in the project catalog".to_owned())?;
    engine
        .validate(SchXsltRequest {
            source,
            schematron: &resource,
            catalog,
            revision,
            file,
            phase: None,
        })
        .await
        .map(|report| (report.diagnostics, "schxslt-saxon"))
        .map_err(|error| error.to_string())
}

fn schxslt_engine_from_environment() -> Result<Option<SchXsltProcessEngine>, String> {
    const JAVA: &str = "O2XML_JAVA";
    const SAXON: &str = "O2XML_SAXON_JAR";
    const SCHXSLT: &str = "O2XML_SCHXSLT_PIPELINE";
    let values = [env::var_os(JAVA), env::var_os(SAXON), env::var_os(SCHXSLT)];
    if values.iter().all(Option::is_none) {
        return Ok(None);
    }
    let [Some(java), Some(saxon), Some(schxslt)] = values else {
        return Err(format!(
            "configure {JAVA}, {SAXON}, and {SCHXSLT} together to enable ISO Schematron"
        ));
    };
    SchXsltProcessEngine::new(java, saxon, schxslt)
        .map(Some)
        .map_err(|error| format!("invalid ISO Schematron engine configuration: {error}"))
}

#[tauri::command]
async fn run_transformation(
    id: u64,
    scenario: String,
    state: State<'_, AppState>,
) -> Result<TransformPayload, String> {
    let result = run_transformation_inner(id, &scenario, &state).await;
    let (revision, engine) = result
        .as_ref()
        .map(|payload| (Some(payload.revision), Some(payload.engine)))
        .unwrap_or((None, None));
    audit::event(
        "transform.run",
        if result.is_ok() { "success" } else { "failure" },
        "document",
        id,
        revision,
        engine,
    );
    result
}

async fn run_transformation_inner(
    id: u64,
    scenario_name: &str,
    state: &State<'_, AppState>,
) -> Result<TransformPayload, String> {
    let snapshot = {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?
            .snapshot()
    };
    let project = state
        .project
        .lock()
        .map_err(|_| "project store is unavailable".to_owned())?
        .clone()
        .ok_or_else(|| "open a project before running a transformation".to_owned())?;
    let scenario = project
        .manifest
        .transform
        .scenarios
        .iter()
        .find(|candidate| candidate.name == scenario_name)
        .cloned()
        .ok_or_else(|| "transformation scenario is not defined by the project".to_owned())?;
    let stylesheet_path = normalized_path(&project.root.join(&scenario.stylesheet))?;
    if !stylesheet_path.starts_with(&project.root) || !stylesheet_path.is_file() {
        return Err("transformation stylesheet is outside the project root".to_owned());
    }
    const MAX_STYLESHEET_BYTES: u64 = 16 * 1024 * 1024;
    let metadata = fs::metadata(&stylesheet_path)
        .map_err(|error| format!("cannot inspect transformation stylesheet: {error}"))?;
    if metadata.len() > MAX_STYLESHEET_BYTES {
        return Err("transformation stylesheet exceeds the 16 MiB safety limit".to_owned());
    }
    let source = snapshot.text();
    let (engine_name, report) =
        match scenario.engine {
            TransformEngine::SaxonHe => {
                let stylesheet = fs::read_to_string(&stylesheet_path).map_err(|error| {
                    format!("cannot read transformation stylesheet as UTF-8: {error}")
                })?;
                let engine = saxon_transform_engine_from_environment()?;
                let report = engine
                    .transform(TransformRequest {
                        source: &source,
                        stylesheet: &stylesheet,
                        revision: snapshot.revision(),
                    })
                    .await
                    .map_err(|error| error.to_string())?;
                ("saxon-he", report)
            }
            TransformEngine::Libxslt => {
                let catalog_setting =
                    project.manifest.xml.catalog.as_deref().ok_or_else(|| {
                        "libxslt transformations require an XML catalog".to_owned()
                    })?;
                let catalog_path = normalized_path(&project.root.join(catalog_setting))?;
                if !catalog_path.starts_with(&project.root) {
                    return Err("project catalog is outside the project root".to_owned());
                }
                let policy = ResourcePolicy::offline([project.root.clone()])
                    .map_err(|error| error.to_string())?;
                let catalog = CatalogResolver::load(catalog_path, policy)
                    .map_err(|error| error.to_string())?;
                let stylesheet_uri = url::Url::from_file_path(&stylesheet_path)
                    .map_err(|()| "cannot convert stylesheet path to a file URI".to_owned())?;
                let engine = LibxsltProcessEngine::new(libxml2_helper_path()?)
                    .map_err(|error| error.to_string())?;
                let report = engine
                    .transform(LibxsltRequest {
                        source: &source,
                        revision: snapshot.revision(),
                        catalog: &catalog,
                        stylesheet_uri: stylesheet_uri.as_str(),
                        parameters: None,
                    })
                    .await
                    .map_err(|error| error.to_string())?;
                ("libxslt-process", report)
            }
        };

    {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        let current = documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?;
        if current.revision() != report.revision {
            return Err("document changed while the transformation was running".to_owned());
        }
    }

    let requested_output = project.root.join(&scenario.output);
    let output_parent = requested_output
        .parent()
        .ok_or_else(|| "transformation output has no parent directory".to_owned())?;
    let canonical_parent = normalized_path(output_parent)?;
    if !canonical_parent.starts_with(&project.root) {
        return Err("transformation output is outside the project root".to_owned());
    }
    let file_name = requested_output
        .file_name()
        .ok_or_else(|| "transformation output has no file name".to_owned())?;
    let target = canonical_parent.join(file_name);
    if fs::symlink_metadata(&target).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return Err("transformation output cannot replace a symbolic link".to_owned());
    }
    let mut temporary = tempfile::NamedTempFile::new_in(&canonical_parent)
        .map_err(|error| format!("cannot create temporary transformation output: {error}"))?;
    temporary
        .write_all(&report.output)
        .map_err(|error| format!("cannot write transformation output: {error}"))?;
    temporary
        .as_file_mut()
        .sync_all()
        .map_err(|error| format!("cannot flush transformation output: {error}"))?;
    temporary
        .persist(&target)
        .map_err(|error| format!("cannot publish transformation output: {}", error.error))?;

    const MAX_PREVIEW_BYTES: usize = 1024 * 1024;
    let preview_length = report.output.len().min(MAX_PREVIEW_BYTES);
    let preview = String::from_utf8_lossy(&report.output[..preview_length]).into_owned();
    Ok(TransformPayload {
        revision: report.revision,
        scenario: scenario.name,
        engine: engine_name,
        media_type: scenario.media_type,
        output_path: scenario.output,
        bytes_written: report.output.len(),
        preview,
        preview_truncated: report.output.len() > preview_length,
    })
}

fn saxon_transform_engine_from_environment() -> Result<SaxonProcessEngine, String> {
    const JAVA: &str = "O2XML_JAVA";
    const SAXON: &str = "O2XML_SAXON_JAR";
    let java =
        env::var_os(JAVA).ok_or_else(|| format!("set {JAVA} to enable Saxon transformations"))?;
    let saxon =
        env::var_os(SAXON).ok_or_else(|| format!("set {SAXON} to enable Saxon transformations"))?;
    SaxonProcessEngine::new(java, saxon)
        .map_err(|error| format!("invalid Saxon transformation configuration: {error}"))
}

#[tauri::command]
async fn evaluate_xquery(
    id: u64,
    query: String,
    state: State<'_, AppState>,
) -> Result<XQueryPayload, String> {
    let result = evaluate_xquery_inner(id, &query, &state).await;
    let revision = result.as_ref().ok().map(|payload| payload.revision);
    audit::event(
        "xquery.evaluate",
        if result.is_ok() { "success" } else { "failure" },
        "document",
        id,
        revision,
        Some("saxon-he-xquery"),
    );
    result
}

async fn evaluate_xquery_inner(
    id: u64,
    query: &str,
    state: &State<'_, AppState>,
) -> Result<XQueryPayload, String> {
    let snapshot = {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?
            .snapshot()
    };
    const JAVA: &str = "O2XML_JAVA";
    const SAXON: &str = "O2XML_SAXON_JAR";
    let java = env::var_os(JAVA).ok_or_else(|| format!("set {JAVA} to enable XQuery"))?;
    let saxon = env::var_os(SAXON).ok_or_else(|| format!("set {SAXON} to enable XQuery"))?;
    let engine = SaxonXQueryEngine::new(java, saxon)
        .map_err(|error| format!("invalid Saxon XQuery configuration: {error}"))?;
    let source = snapshot.text();
    let report = engine
        .evaluate(XQueryRequest {
            source: &source,
            query,
            revision: snapshot.revision(),
        })
        .await
        .map_err(|error| error.to_string())?;
    {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        let current = documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?;
        if current.revision() != report.revision {
            return Err("document changed while XQuery was running".to_owned());
        }
    }
    const MAX_PREVIEW_BYTES: usize = 1024 * 1024;
    let preview_length = report.output.len().min(MAX_PREVIEW_BYTES);
    Ok(XQueryPayload {
        revision: report.revision,
        engine: "saxon-he-xquery",
        bytes: report.output.len(),
        preview: String::from_utf8_lossy(&report.output[..preview_length]).into_owned(),
        truncated: report.output.len() > preview_length,
    })
}

#[tauri::command]
async fn compare_document(
    id: u64,
    other_path: String,
    ignore_whitespace_only_text: bool,
    state: State<'_, AppState>,
) -> Result<DiffPayload, String> {
    let result = compare_document_inner(id, &other_path, ignore_whitespace_only_text, &state).await;
    let revision = result.as_ref().ok().map(|payload| payload.revision);
    audit::event(
        "diff.compare",
        if result.is_ok() { "success" } else { "failure" },
        "document",
        id,
        revision,
        Some("xml-aware"),
    );
    result
}

async fn compare_document_inner(
    id: u64,
    other_path: &str,
    ignore_whitespace_only_text: bool,
    state: &State<'_, AppState>,
) -> Result<DiffPayload, String> {
    let snapshot = {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?
            .snapshot()
    };
    let other_path = normalized_path(Path::new(other_path))?;
    let metadata = fs::metadata(&other_path)
        .map_err(|error| format!("cannot inspect comparison file: {error}"))?;
    const MAX_DESKTOP_DIFF_BYTES: u64 = 32 * 1024 * 1024;
    if !metadata.is_file() || metadata.len() > MAX_DESKTOP_DIFF_BYTES {
        return Err("comparison file must be a regular UTF-8 file of at most 32 MiB".to_owned());
    }
    let other_text = fs::read_to_string(&other_path)
        .map_err(|error| format!("cannot read comparison file as UTF-8: {error}"))?;
    let left = snapshot.text();
    let right = other_text.clone();
    let report = tauri::async_runtime::spawn_blocking(move || {
        xml_diff::compare(
            &left,
            &right,
            XmlDiffOptions {
                ignore_whitespace_only_text,
            },
        )
    })
    .await
    .map_err(|error| format!("diff worker failed: {error}"))?
    .map_err(|error| error.to_string())?;
    {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        let current = documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?;
        if current.revision() != snapshot.revision() {
            return Err("document changed while comparison was running".to_owned());
        }
    }
    Ok(DiffPayload {
        revision: snapshot.revision(),
        other_name: display_name(Some(&other_path)),
        other_text,
        report,
    })
}

#[tauri::command]
async fn merge_diff_hunk(
    id: u64,
    other_path: String,
    ignore_whitespace_only_text: bool,
    hunk_index: usize,
    state: State<'_, AppState>,
) -> Result<MergePayload, String> {
    let result = merge_diff_hunk_inner(
        id,
        &other_path,
        ignore_whitespace_only_text,
        hunk_index,
        &state,
    )
    .await;
    let revision = result.as_ref().ok().map(|payload| payload.revision);
    audit::event(
        "diff.merge_hunk",
        if result.is_ok() { "success" } else { "failure" },
        "document",
        id,
        revision,
        Some("xml-aware"),
    );
    result
}

async fn merge_diff_hunk_inner(
    id: u64,
    other_path: &str,
    ignore_whitespace_only_text: bool,
    hunk_index: usize,
    state: &State<'_, AppState>,
) -> Result<MergePayload, String> {
    let snapshot = {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?
            .snapshot()
    };
    let other_path = normalized_path(Path::new(other_path))?;
    let metadata = fs::metadata(&other_path)
        .map_err(|error| format!("cannot inspect merge source: {error}"))?;
    const MAX_DESKTOP_MERGE_BYTES: u64 = 32 * 1024 * 1024;
    if !metadata.is_file() || metadata.len() > MAX_DESKTOP_MERGE_BYTES {
        return Err("merge source must be a regular UTF-8 file of at most 32 MiB".to_owned());
    }
    let right = fs::read_to_string(other_path)
        .map_err(|error| format!("cannot read merge source as UTF-8: {error}"))?;
    let left = snapshot.text();
    let text = tauri::async_runtime::spawn_blocking(move || {
        xml_diff::merge_hunk(
            &left,
            &right,
            XmlDiffOptions {
                ignore_whitespace_only_text,
            },
            hunk_index,
        )
    })
    .await
    .map_err(|error| format!("merge worker failed: {error}"))?
    .map_err(|error| error.to_string())?;
    {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        let current = documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?;
        if current.revision() != snapshot.revision() {
            return Err("document changed while the merge was being validated".to_owned());
        }
    }
    Ok(MergePayload {
        revision: snapshot.revision(),
        text,
    })
}

#[tauri::command]
async fn inspect_grid(
    id: u64,
    row_element: Option<String>,
    state: State<'_, AppState>,
) -> Result<GridPayload, String> {
    let result = inspect_grid_inner(id, row_element, &state).await;
    let revision = result.as_ref().ok().map(|payload| payload.revision);
    audit::event(
        "grid.inspect",
        if result.is_ok() { "success" } else { "failure" },
        "document",
        id,
        revision,
        Some("native-bounded"),
    );
    result
}

async fn inspect_grid_inner(
    id: u64,
    row_element: Option<String>,
    state: &State<'_, AppState>,
) -> Result<GridPayload, String> {
    let snapshot = {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?
            .snapshot()
    };
    let source = snapshot.text();
    let requested = row_element.filter(|value| !value.trim().is_empty());
    let grid = tauri::async_runtime::spawn_blocking(move || {
        xml_grid::extract(&source, requested.as_deref())
    })
    .await
    .map_err(|error| format!("grid worker failed: {error}"))?
    .map_err(|error| error.to_string())?;
    {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        let current = documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?;
        if current.revision() != snapshot.revision() {
            return Err("document changed while XML Grid was being built".to_owned());
        }
    }
    Ok(GridPayload {
        revision: snapshot.revision(),
        grid,
    })
}

#[tauri::command]
async fn edit_grid_cell(
    id: u64,
    row_element: Option<String>,
    row_index: usize,
    column: String,
    value: String,
    state: State<'_, AppState>,
) -> Result<GridEditPayload, String> {
    let result = edit_grid_cell_inner(id, row_element, row_index, column, value, &state).await;
    let (revision, engine) = result
        .as_ref()
        .map(|payload| {
            (
                Some(payload.revision),
                Some(if payload.schema_validated {
                    "project-framework"
                } else {
                    "well-formedness"
                }),
            )
        })
        .unwrap_or((None, Some("grid-transaction")));
    audit::event(
        "grid.edit",
        if result.is_ok() { "success" } else { "failure" },
        "document",
        id,
        revision,
        engine,
    );
    result
}

async fn edit_grid_cell_inner(
    id: u64,
    row_element: Option<String>,
    row_index: usize,
    column: String,
    value: String,
    state: &State<'_, AppState>,
) -> Result<GridEditPayload, String> {
    let (snapshot, file) = {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        let document = documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?;
        (
            document.snapshot(),
            document
                .path()
                .map(|path| path.to_string_lossy().into_owned()),
        )
    };
    let project = state
        .project
        .lock()
        .map_err(|_| "project store is unavailable".to_owned())?
        .clone();
    let source = snapshot.text();
    let requested = row_element.filter(|candidate| !candidate.trim().is_empty());
    let text = tauri::async_runtime::spawn_blocking(move || {
        xml_grid::edit(&source, requested.as_deref(), row_index, &column, &value)
    })
    .await
    .map_err(|error| format!("grid edit worker failed: {error}"))?
    .map_err(|error| error.to_string())?;

    let applicable_project = project.filter(|project| {
        file.as_deref()
            .map(Path::new)
            .is_none_or(|path| path.starts_with(&project.root))
            && project.manifest.validation.framework.is_some()
    });
    let schema_validated = applicable_project.is_some();
    if let Some(project) = applicable_project {
        let stages =
            validate_project_framework(&project, &text, snapshot.revision(), file.as_deref())
                .await?;
        if let Some(stage) = stages
            .iter()
            .find(|stage| !matches!(stage.status, FrameworkStageStatus::Passed))
        {
            let detail = stage
                .diagnostics
                .first()
                .map(|diagnostic| diagnostic.message.as_str())
                .unwrap_or("validation did not pass");
            return Err(format!("grid edit rejected by {}: {detail}", stage.label));
        }
    }

    {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        let current = documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?;
        if current.revision() != snapshot.revision() {
            return Err("document changed while the grid edit was being validated".to_owned());
        }
    }
    Ok(GridEditPayload {
        revision: snapshot.revision(),
        text,
        schema_validated,
    })
}

#[tauri::command]
async fn inspect_author(id: u64, state: State<'_, AppState>) -> Result<AuthorPayload, String> {
    let result = inspect_author_inner(id, &state).await;
    let revision = result.as_ref().ok().map(|payload| payload.revision);
    audit::event(
        "author.inspect",
        if result.is_ok() { "success" } else { "failure" },
        "document",
        id,
        revision,
        Some("native-bounded"),
    );
    result
}

async fn inspect_author_inner(
    id: u64,
    state: &State<'_, AppState>,
) -> Result<AuthorPayload, String> {
    let snapshot = {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?
            .snapshot()
    };
    let source = snapshot.text();
    let document = tauri::async_runtime::spawn_blocking(move || xml_author::project(&source))
        .await
        .map_err(|error| format!("Author Mode worker failed: {error}"))?
        .map_err(|error| error.to_string())?;
    {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        let current = documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?;
        if current.revision() != snapshot.revision() {
            return Err("document changed while Author Mode was being built".to_owned());
        }
    }
    Ok(AuthorPayload {
        revision: snapshot.revision(),
        document,
    })
}

#[tauri::command]
async fn preview_author(
    id: u64,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<AuthorPreviewPayload, String> {
    let result = preview_author_inner(id, &app, &state).await;
    let revision = result.as_ref().ok().map(|payload| payload.revision);
    audit::event(
        "author.preview",
        if result.is_ok() { "success" } else { "failure" },
        "document",
        id,
        revision,
        Some("packtools-html-4.16.9"),
    );
    result
}

async fn preview_author_inner(
    id: u64,
    app: &AppHandle,
    state: &State<'_, AppState>,
) -> Result<AuthorPreviewPayload, String> {
    const MAX_AUTHOR_PREVIEW_SOURCE_BYTES: usize = 32 * 1024 * 1024;
    const MAX_AUTHOR_PREVIEW_CSS_BYTES: u64 = 1024 * 1024;
    let (snapshot, document_path) = {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        let document = documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?;
        (document.snapshot(), document.path().cloned())
    };
    let source = snapshot.text();
    if source.len() > MAX_AUTHOR_PREVIEW_SOURCE_BYTES {
        return Err("SciELO preview source exceeds the 32 MiB safety limit".to_owned());
    }
    let language = jats_article_language(&source)?;
    let catalog_path = bundled_resource(app, "resources/packtools-html/4.16.9/catalog.xml")?;
    let resource_root = catalog_path
        .parent()
        .ok_or_else(|| "Packtools preview catalog has no parent directory".to_owned())?;
    let stylesheet_path = resource_root.join("root-html-2.0.xslt");
    let css_path = resource_root.join("static/scielo-article-standalone.css");
    let css_metadata = fs::metadata(&css_path)
        .map_err(|error| format!("cannot inspect Packtools preview stylesheet: {error}"))?;
    if css_metadata.len() > MAX_AUTHOR_PREVIEW_CSS_BYTES {
        return Err("Packtools preview stylesheet exceeds the 1 MiB safety limit".to_owned());
    }
    let stylesheet_uri = url::Url::from_file_path(&stylesheet_path)
        .map_err(|()| "cannot convert Packtools stylesheet path to a file URI".to_owned())?;
    let policy = ResourcePolicy::offline([resource_root.to_path_buf()])
        .map_err(|error| error.to_string())?;
    let catalog =
        CatalogResolver::load(&catalog_path, policy).map_err(|error| error.to_string())?;
    let parameters = BTreeMap::from([
        ("article_lang".to_owned(), language),
        ("is_translation".to_owned(), "False".to_owned()),
        ("styles_css_path".to_owned(), String::new()),
        ("print_styles_css_path".to_owned(), String::new()),
        ("js_path".to_owned(), String::new()),
        ("output_style".to_owned(), String::new()),
        ("math_js".to_owned(), String::new()),
    ]);
    let engine =
        LibxsltProcessEngine::new(libxml2_helper_path()?).map_err(|error| error.to_string())?;
    let report = engine
        .transform(LibxsltRequest {
            source: &source,
            revision: snapshot.revision(),
            catalog: &catalog,
            stylesheet_uri: stylesheet_uri.as_str(),
            parameters: Some(&parameters),
        })
        .await
        .map_err(|error| error.to_string())?;
    {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        let current = documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?;
        if current.revision() != report.revision {
            return Err("document changed while the SciELO preview was being built".to_owned());
        }
    }
    let html = String::from_utf8(report.output)
        .map_err(|_| "Packtools preview output is not UTF-8".to_owned())?;
    let stylesheet = fs::read_to_string(css_path)
        .map_err(|error| format!("cannot read Packtools preview stylesheet: {error}"))?;
    let (assets, assets_skipped) =
        collect_author_preview_assets(&source, document_path.as_deref())?;
    Ok(AuthorPreviewPayload {
        revision: report.revision,
        engine: "Packtools HTMLGenerator 4.16.9",
        html,
        stylesheet,
        assets,
        assets_skipped,
    })
}

fn jats_article_language(source: &str) -> Result<String, String> {
    let mut reader = Reader::from_str(source);
    loop {
        match reader.read_event() {
            Ok(Event::Start(element) | Event::Empty(element)) => {
                if element.local_name().as_ref() != b"article" {
                    return Err("SciELO preview requires a JATS <article> document".to_owned());
                }
                for attribute in element.attributes() {
                    let attribute = attribute
                        .map_err(|error| format!("cannot read JATS article attributes: {error}"))?;
                    if attribute.key.as_ref() == b"xml:lang" {
                        let value = attribute
                            .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
                            .map_err(|error| format!("cannot read JATS article language: {error}"))?
                            .into_owned();
                        if (2..=16).contains(&value.len())
                            && value.bytes().all(|byte| {
                                byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')
                            })
                        {
                            return Ok(value);
                        }
                        return Err("JATS article has an invalid xml:lang value".to_owned());
                    }
                }
                return Err("SciELO preview requires xml:lang on the JATS article".to_owned());
            }
            Ok(Event::Eof) => {
                return Err("SciELO preview requires a JATS <article> document".to_owned());
            }
            Ok(_) => {}
            Err(error) => return Err(format!("cannot inspect JATS article: {error}")),
        }
    }
}

fn collect_author_preview_assets(
    source: &str,
    document_path: Option<&Path>,
) -> Result<(Vec<AuthorPreviewAsset>, usize), String> {
    const MAX_ASSETS: usize = 64;
    const MAX_ASSET_BYTES: u64 = 8 * 1024 * 1024;
    const MAX_TOTAL_ASSET_BYTES: u64 = 32 * 1024 * 1024;
    const MAX_REFERENCE_BYTES: usize = 1024;

    let mut references = BTreeSet::new();
    let mut reader = Reader::from_str(source);
    loop {
        match reader.read_event() {
            Ok(Event::Start(element) | Event::Empty(element)) => {
                if !matches!(
                    element.local_name().as_ref(),
                    b"graphic" | b"inline-graphic"
                ) {
                    continue;
                }
                for attribute in element.attributes() {
                    let attribute = attribute.map_err(|error| {
                        format!("cannot read JATS preview asset attributes: {error}")
                    })?;
                    if attribute.key.local_name().as_ref() != b"href" {
                        continue;
                    }
                    let value = attribute
                        .decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
                        .map_err(|error| {
                            format!("cannot decode JATS preview asset reference: {error}")
                        })?
                        .into_owned();
                    if !value.is_empty() && value.len() <= MAX_REFERENCE_BYTES {
                        references.insert(packtools_image_reference(&value));
                    }
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => return Err(format!("cannot inspect JATS preview assets: {error}")),
        }
    }

    let mut skipped = references.len().saturating_sub(MAX_ASSETS);
    let Some(document_path) = document_path else {
        return Ok((
            Vec::new(),
            skipped.saturating_add(references.len().min(MAX_ASSETS)),
        ));
    };
    let Some(parent) = document_path.parent() else {
        return Ok((
            Vec::new(),
            skipped.saturating_add(references.len().min(MAX_ASSETS)),
        ));
    };
    let root = parent
        .canonicalize()
        .map_err(|error| format!("cannot resolve the XML directory for preview assets: {error}"))?;
    let mut assets = Vec::new();
    let mut total_bytes = 0_u64;
    for reference in references.into_iter().take(MAX_ASSETS) {
        let Some(path) = confined_preview_asset_path(&root, &reference) else {
            skipped = skipped.saturating_add(1);
            continue;
        };
        let Ok(metadata) = fs::metadata(&path) else {
            skipped = skipped.saturating_add(1);
            continue;
        };
        if !metadata.is_file()
            || metadata.len() > MAX_ASSET_BYTES
            || total_bytes.saturating_add(metadata.len()) > MAX_TOTAL_ASSET_BYTES
        {
            skipped = skipped.saturating_add(1);
            continue;
        }
        let Ok(bytes) = fs::read(&path) else {
            skipped = skipped.saturating_add(1);
            continue;
        };
        let actual_len = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
        if actual_len > MAX_ASSET_BYTES
            || total_bytes.saturating_add(actual_len) > MAX_TOTAL_ASSET_BYTES
        {
            skipped = skipped.saturating_add(1);
            continue;
        }
        let Some(media_type) = preview_image_media_type(&bytes) else {
            skipped = skipped.saturating_add(1);
            continue;
        };
        total_bytes = total_bytes.saturating_add(actual_len);
        assets.push(AuthorPreviewAsset {
            source: reference,
            data_uri: format!("data:{media_type};base64,{}", BASE64_STANDARD.encode(bytes)),
        });
    }
    Ok((assets, skipped))
}

fn packtools_image_reference(reference: &str) -> String {
    let path = Path::new(reference);
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
    {
        None => format!("{reference}.jpg"),
        Some(extension) if extension.contains("tif") => {
            path.with_extension("jpg").to_string_lossy().into_owned()
        }
        Some(_) => reference.to_owned(),
    }
}

fn confined_preview_asset_path(root: &Path, reference: &str) -> Option<PathBuf> {
    if reference.is_empty() || url::Url::parse(reference).is_ok() {
        return None;
    }
    let relative = Path::new(reference);
    if relative.is_absolute()
        || !relative
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return None;
    }
    let candidate = root.join(relative);
    if fs::symlink_metadata(&candidate).is_ok_and(|metadata| metadata.file_type().is_symlink()) {
        return None;
    }
    let canonical = candidate.canonicalize().ok()?;
    canonical.starts_with(root).then_some(canonical)
}

fn preview_image_media_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if bytes.starts_with(b"\xff\xd8\xff") {
        Some("image/jpeg")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("image/gif")
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else {
        None
    }
}

#[tauri::command]
async fn edit_author_text(
    id: u64,
    run_id: u64,
    value: String,
    state: State<'_, AppState>,
) -> Result<AuthorEditPayload, String> {
    let result = edit_author_text_inner(id, run_id, value, &state).await;
    let (revision, engine) = result
        .as_ref()
        .map(|payload| {
            (
                Some(payload.revision),
                Some(if payload.schema_validated {
                    "project-framework"
                } else {
                    "well-formedness"
                }),
            )
        })
        .unwrap_or((None, Some("author-transaction")));
    audit::event(
        "author.edit",
        if result.is_ok() { "success" } else { "failure" },
        "document",
        id,
        revision,
        engine,
    );
    result
}

async fn edit_author_text_inner(
    id: u64,
    run_id: u64,
    value: String,
    state: &State<'_, AppState>,
) -> Result<AuthorEditPayload, String> {
    let (snapshot, file) = {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        let document = documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?;
        (
            document.snapshot(),
            document
                .path()
                .map(|path| path.to_string_lossy().into_owned()),
        )
    };
    let project = state
        .project
        .lock()
        .map_err(|_| "project store is unavailable".to_owned())?
        .clone();
    let source = snapshot.text();
    let text = tauri::async_runtime::spawn_blocking(move || {
        xml_author::edit_text_run(&source, run_id, &value)
    })
    .await
    .map_err(|error| format!("Author Mode edit worker failed: {error}"))?
    .map_err(|error| error.to_string())?;

    let applicable_project = project.filter(|project| {
        file.as_deref()
            .map(Path::new)
            .is_none_or(|path| path.starts_with(&project.root))
            && project.manifest.validation.framework.is_some()
    });
    let schema_validated = applicable_project.is_some();
    if let Some(project) = applicable_project {
        let stages =
            validate_project_framework(&project, &text, snapshot.revision(), file.as_deref())
                .await?;
        if let Some(stage) = stages
            .iter()
            .find(|stage| !matches!(stage.status, FrameworkStageStatus::Passed))
        {
            let detail = stage
                .diagnostics
                .first()
                .map(|diagnostic| diagnostic.message.as_str())
                .unwrap_or("validation did not pass");
            return Err(format!(
                "Author Mode edit rejected by {}: {detail}",
                stage.label
            ));
        }
    }

    {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        let current = documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?;
        if current.revision() != snapshot.revision() {
            return Err(
                "document changed while the Author Mode edit was being validated".to_owned(),
            );
        }
    }
    Ok(AuthorEditPayload {
        revision: snapshot.revision(),
        text,
        schema_validated,
    })
}

fn framework_stage_error(
    source: &str,
    revision: Revision,
    file: Option<&str>,
    stage: &str,
    message: &str,
) -> Diagnostic {
    let index = LineIndex::new(source);
    let mut span = index.span(source, 0, 0);
    span.approximate = true;
    Diagnostic {
        severity: Severity::Error,
        message: format!("validation stage is unavailable: {message}"),
        source: stage.to_owned(),
        rule: None,
        file: file.map(ToOwned::to_owned),
        span,
        revision,
    }
}

#[tauri::command]
async fn evaluate_xpath(
    id: u64,
    expression: String,
    namespaces: BTreeMap<String, String>,
    state: State<'_, AppState>,
) -> Result<XPathReport, String> {
    let snapshot = {
        let documents = state.documents.lock().map_err(|_| {
            audit::event(
                "xpath.evaluate",
                "failure",
                "document",
                id,
                None,
                Some("libxml2"),
            );
            "document store is unavailable".to_owned()
        })?;
        documents
            .get(&id)
            .ok_or_else(|| {
                audit::event(
                    "xpath.evaluate",
                    "failure",
                    "document",
                    id,
                    None,
                    Some("libxml2"),
                );
                "document is not open".to_owned()
            })?
            .snapshot()
    };
    let revision = snapshot.revision();
    let result = async {
        let helper = libxml2_helper_path()?;
        let engine = Libxml2XPathEngine::new(helper).map_err(|error| error.to_string())?;
        let source = snapshot.text();
        engine
            .evaluate(XPathRequest {
                source: &source,
                revision,
                expression: &expression,
                namespaces: &namespaces,
            })
            .await
            .map_err(|error| error.to_string())
    }
    .await;
    audit::event(
        "xpath.evaluate",
        if result.is_ok() { "success" } else { "failure" },
        "document",
        id,
        Some(revision),
        Some("libxml2"),
    );
    result
}

fn libxml2_helper_path() -> Result<PathBuf, String> {
    const ENVIRONMENT_KEY: &str = "O2XML_LIBXML2_HELPER";
    let executable_name = if cfg!(windows) {
        "xml-libxml2-helper.exe"
    } else {
        "xml-libxml2-helper"
    };
    let mut candidates = Vec::new();
    if let Some(path) = env::var_os(ENVIRONMENT_KEY) {
        candidates.push(PathBuf::from(path));
    }
    if let Ok(current) = env::current_exe() {
        if let Some(directory) = current.parent() {
            candidates.push(directory.join(executable_name));
            candidates.push(directory.join("../Resources").join(executable_name));
        }
    }
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    candidates.push(workspace.join("target/debug").join(executable_name));
    candidates.push(workspace.join("target/release").join(executable_name));

    candidates
        .into_iter()
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| {
            format!("libxml2 helper is unavailable; build the workspace or set {ENVIRONMENT_KEY}")
        })
}

#[tauri::command]
async fn complete_dtd(
    id: u64,
    parent: String,
    preceding_children: Vec<String>,
    present_attributes: Vec<String>,
    state: State<'_, AppState>,
) -> Result<DtdCompletionPayload, String> {
    const MAX_CONTEXT_NAMES: usize = 4_096;
    const MAX_NAME_BYTES: usize = 8 * 1024;
    if preceding_children.len() > MAX_CONTEXT_NAMES
        || present_attributes.len() > MAX_CONTEXT_NAMES
        || std::iter::once(&parent)
            .chain(preceding_children.iter())
            .chain(present_attributes.iter())
            .any(|name| name.is_empty() || name.len() > MAX_NAME_BYTES)
    {
        return Err("DTD completion context exceeds its safety limits".to_owned());
    }
    let (snapshot, document_path) = {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        let document = documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?;
        (document.snapshot(), document.path().cloned())
    };
    let revision = snapshot.revision();
    let result = async {
        let project = state
            .project
            .lock()
            .map_err(|_| "project store is unavailable".to_owned())?
            .clone()
            .ok_or_else(|| "open a project before requesting DTD completion".to_owned())?;
        if document_path
            .as_ref()
            .is_some_and(|path| !path.starts_with(&project.root))
        {
            return Err("document is outside the active project".to_owned());
        }
        let catalog_setting = project
            .manifest
            .xml
            .catalog
            .as_deref()
            .ok_or_else(|| "active project has no XML catalog".to_owned())?;
        let catalog_path = normalized_path(&project.root.join(catalog_setting))?;
        if !catalog_path.starts_with(&project.root) {
            return Err("project catalog is outside the project root".to_owned());
        }
        let source = snapshot.text();
        let document_type = xml_syntax::document_type(&source)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "document has no DOCTYPE association".to_owned())?;
        let system_id = document_type
            .system_id
            .ok_or_else(|| "DOCTYPE has no external system identifier".to_owned())?;
        let cache_key = (system_id.clone(), document_type.public_id.clone());
        let cached = state
            .dtd_indexes
            .lock()
            .map_err(|_| "DTD index cache is unavailable".to_owned())?
            .get(&cache_key)
            .cloned();
        let index = if let Some(index) = cached {
            index
        } else {
            let policy = ResourcePolicy::offline([project.root.clone()])
                .map_err(|error| error.to_string())?;
            let catalog =
                CatalogResolver::load(&catalog_path, policy).map_err(|error| error.to_string())?;
            let engine = Libxml2DtdIndexEngine::new(libxml2_helper_path()?)
                .map_err(|error| error.to_string())?;
            let report = engine
                .compile(DtdIndexRequest {
                    catalog: &catalog,
                    system_id: &system_id,
                    public_id: document_type.public_id.as_deref(),
                })
                .await
                .map_err(|error| error.to_string())?;
            state
                .dtd_indexes
                .lock()
                .map_err(|_| "DTD index cache is unavailable".to_owned())?
                .insert(cache_key, report.index.clone());
            report.index
        };
        let parent = ExpandedName::new(None, &parent).map_err(|error| error.to_string())?;
        let children = preceding_children
            .iter()
            .map(|name| ExpandedName::new(None, name).map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        let present = present_attributes
            .iter()
            .map(|name| ExpandedName::new(None, name).map_err(|error| error.to_string()))
            .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
        let child_completion = index
            .complete_children(&parent, &children)
            .map_err(|error| error.to_string())?;
        let attributes = index
            .complete_attributes(&parent, &present)
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(|attribute| {
                let (default_value, fixed) = match attribute.value_constraint {
                    Some(ValueConstraint::Default(value)) => (Some(value), false),
                    Some(ValueConstraint::Fixed(value)) => (Some(value), true),
                    None => (None, false),
                };
                DtdAttributePayload {
                    name: attribute.name.local_name,
                    required: attribute.use_kind == AttributeUse::Required,
                    values: attribute.allowed_values,
                    default_value,
                    fixed,
                }
            })
            .collect();
        Ok(DtdCompletionPayload {
            revision,
            children: child_completion
                .children
                .into_iter()
                .map(|name| name.local_name)
                .collect(),
            attributes,
            can_close: child_completion.can_close,
            accepts_any: child_completion.accepts_any,
        })
    }
    .await;
    audit::event(
        "completion.dtd",
        if result.is_ok() { "success" } else { "failure" },
        "document",
        id,
        Some(revision),
        Some("libxml2"),
    );
    result
}

#[tauri::command]
async fn complete_xsd(
    id: u64,
    context: XsdCompletionContext,
    state: State<'_, AppState>,
) -> Result<XsdCompletionPayload, String> {
    const MAX_CONTEXT_NAMES: usize = 4_096;
    if context.preceding_children.len() > MAX_CONTEXT_NAMES
        || context.present_attributes.len() > MAX_CONTEXT_NAMES
    {
        return Err("XSD completion context exceeds its safety limits".to_owned());
    }
    let validate_name = |name: &ExpandedName| {
        ExpandedName::new(name.namespace_uri.as_deref(), &name.local_name)
            .map(|_| ())
            .map_err(|error| error.to_string())
    };
    validate_name(&context.parent)?;
    for name in context
        .preceding_children
        .iter()
        .chain(context.present_attributes.iter())
    {
        validate_name(name)?;
    }
    let (snapshot, document_path) = {
        let documents = state
            .documents
            .lock()
            .map_err(|_| "document store is unavailable".to_owned())?;
        let document = documents
            .get(&id)
            .ok_or_else(|| "document is not open".to_owned())?;
        (document.snapshot(), document.path().cloned())
    };
    let revision = snapshot.revision();
    let result = async {
        let project = state
            .project
            .lock()
            .map_err(|_| "project store is unavailable".to_owned())?
            .clone()
            .ok_or_else(|| "open a project before requesting XSD completion".to_owned())?;
        if document_path
            .as_ref()
            .is_some_and(|path| !path.starts_with(&project.root))
        {
            return Err("document is outside the active project".to_owned());
        }
        let catalog_setting = project
            .manifest
            .xml
            .catalog
            .as_deref()
            .ok_or_else(|| "active project has no XML catalog".to_owned())?;
        let catalog_path = normalized_path(&project.root.join(catalog_setting))?;
        if !catalog_path.starts_with(&project.root) {
            return Err("project catalog is outside the project root".to_owned());
        }
        let association = xml_syntax::schema_association(&snapshot.text())
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "document has no XSD association".to_owned())?;
        let schema_uri = match context.parent.namespace_uri.as_deref() {
            Some(namespace) => association.namespace_locations.get(namespace).cloned(),
            None => association.no_namespace_schema_location,
        }
        .ok_or_else(|| "document has no XSD association for the current namespace".to_owned())?;
        let cached = state
            .xsd_indexes
            .lock()
            .map_err(|_| "XSD index cache is unavailable".to_owned())?
            .get(&schema_uri)
            .cloned();
        let index = if let Some(index) = cached {
            index
        } else {
            let policy = ResourcePolicy::offline([project.root.clone()])
                .map_err(|error| error.to_string())?;
            let catalog =
                CatalogResolver::load(&catalog_path, policy).map_err(|error| error.to_string())?;
            let engine = Libxml2XsdIndexEngine::new(libxml2_helper_path()?)
                .map_err(|error| error.to_string())?;
            let report = engine
                .compile(XsdIndexRequest {
                    catalog: &catalog,
                    uri: &schema_uri,
                })
                .await
                .map_err(|error| error.to_string())?;
            state
                .xsd_indexes
                .lock()
                .map_err(|_| "XSD index cache is unavailable".to_owned())?
                .insert(schema_uri, report.index.clone());
            report.index
        };
        let present = context
            .present_attributes
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        let child_completion = index
            .complete_children(&context.parent, &context.preceding_children)
            .map_err(|error| error.to_string())?;
        let attributes = index
            .complete_attributes(&context.parent, &present)
            .map_err(|error| error.to_string())?
            .into_iter()
            .map(|attribute| {
                let (default_value, fixed) = match attribute.value_constraint {
                    Some(ValueConstraint::Default(value)) => (Some(value), false),
                    Some(ValueConstraint::Fixed(value)) => (Some(value), true),
                    None => (None, false),
                };
                XsdAttributePayload {
                    name: attribute.name,
                    required: attribute.use_kind == AttributeUse::Required,
                    values: attribute.allowed_values,
                    default_value,
                    fixed,
                }
            })
            .collect();
        Ok(XsdCompletionPayload {
            revision,
            children: child_completion.children,
            attributes,
            can_close: child_completion.can_close,
            accepts_any: child_completion.accepts_any,
        })
    }
    .await;
    audit::event(
        "completion.xsd",
        if result.is_ok() { "success" } else { "failure" },
        "document",
        id,
        Some(revision),
        Some("libxml2"),
    );
    result
}

#[tauri::command]
fn format_document(
    id: u64,
    indent_size: usize,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let documents = state
        .documents
        .lock()
        .map_err(|_| "document store is unavailable".to_owned())?;
    let document = documents
        .get(&id)
        .ok_or_else(|| "document is not open".to_owned())?;
    let revision = document.revision();
    let result = xml_syntax::pretty_print(&document.snapshot().text(), indent_size)
        .map_err(|error| error.to_string());
    audit::event(
        "document.format",
        if result.is_ok() { "success" } else { "failure" },
        "document",
        id,
        Some(revision),
        None,
    );
    result
}

#[tauri::command]
fn save_document(
    id: u64,
    path: String,
    state: State<'_, AppState>,
) -> Result<DocumentPayload, String> {
    let requested = PathBuf::from(path);
    let parent = requested
        .parent()
        .ok_or_else(|| "save path has no parent directory".to_owned())?;
    let canonical_parent = normalized_path(parent)?;
    let file_name = requested
        .file_name()
        .ok_or_else(|| "save path has no file name".to_owned())?;
    let target = canonical_parent.join(file_name);

    let mut documents = state
        .documents
        .lock()
        .map_err(|_| "document store is unavailable".to_owned())?;
    let document = documents
        .get_mut(&id)
        .ok_or_else(|| "document is not open".to_owned())?;
    let snapshot = document.snapshot();
    let temporary = target.with_extension("o2xml.tmp");
    fs::write(&temporary, snapshot.text())
        .map_err(|error| format!("cannot write file: {error}"))?;
    fs::rename(&temporary, &target)
        .map_err(|error| format!("cannot atomically replace file: {error}"))?;
    document.mark_saved(target);
    let payload = document_payload(id, document);
    let saved_path = document.path().cloned();
    drop(documents);
    if let Some(saved_path) = saved_path {
        state
            .file_stamps
            .lock()
            .map_err(|_| "file state store is unavailable".to_owned())?
            .insert(id, file_stamp(&saved_path));
    }
    audit::event(
        "document.save",
        "success",
        "document",
        id,
        Some(payload.revision),
        None,
    );
    Ok(payload)
}

#[tauri::command]
fn open_project(path: String, state: State<'_, AppState>) -> Result<ProjectPayload, String> {
    let manifest_path = normalized_path(Path::new(&path))?;
    if manifest_path.file_name().and_then(|name| name.to_str()) != Some("project.toml") {
        return Err("select a project.toml file".to_owned());
    }
    let source = fs::read_to_string(&manifest_path)
        .map_err(|error| format!("cannot read project manifest: {error}"))?;
    let manifest = xml_project::parse_manifest(&source).map_err(|error| error.to_string())?;
    let root = manifest_path
        .parent()
        .ok_or_else(|| "project manifest has no parent directory".to_owned())?;
    let mut files = Vec::new();
    collect_project_files(root, root, 0, &mut files)?;
    files.sort();
    let payload = ProjectPayload {
        path: root.to_string_lossy().into_owned(),
        manifest: manifest.clone(),
        files,
    };
    *state
        .project
        .lock()
        .map_err(|_| "project store is unavailable".to_owned())? = Some(OpenedProject {
        root: root.to_path_buf(),
        manifest,
    });
    state
        .dtd_indexes
        .lock()
        .map_err(|_| "DTD index cache is unavailable".to_owned())?
        .clear();
    state
        .xsd_indexes
        .lock()
        .map_err(|_| "XSD index cache is unavailable".to_owned())?
        .clear();
    audit::event("project.open", "success", "project", 0, None, None);
    Ok(payload)
}

fn collect_project_files(
    root: &Path,
    directory: &Path,
    depth: usize,
    files: &mut Vec<String>,
) -> Result<(), String> {
    const MAX_DEPTH: usize = 12;
    const MAX_FILES: usize = 10_000;
    if depth > MAX_DEPTH || files.len() >= MAX_FILES {
        return Ok(());
    }
    let entries =
        fs::read_dir(directory).map_err(|error| format!("cannot read project: {error}"))?;
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot read project entry: {error}"))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("cannot inspect project entry: {error}"))?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            collect_project_files(root, &entry.path(), depth + 1, files)?;
        } else if file_type.is_file() {
            if let Ok(relative) = entry.path().strip_prefix(root) {
                files.push(relative.to_string_lossy().into_owned());
            }
        }
        if files.len() >= MAX_FILES {
            break;
        }
    }
    Ok(())
}

pub fn run() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let _subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .try_init();

    let application = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            new_document,
            open_document,
            close_document,
            check_external_change,
            acknowledge_external_change,
            reload_document,
            update_document,
            analyze_document,
            run_transformation,
            evaluate_xpath,
            evaluate_xquery,
            compare_document,
            merge_diff_hunk,
            inspect_grid,
            edit_grid_cell,
            inspect_author,
            preview_author,
            edit_author_text,
            complete_dtd,
            complete_xsd,
            format_document,
            save_document,
            open_project,
        ])
        .run(tauri::generate_context!());

    if let Err(error) = application {
        tracing::error!(action = "application.run", result = "failure", error = %error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_bundled_jats_11_and_13_identifiers() {
        let jats_11 = bundled_jats_dtd(
            Some("-//NLM//DTD JATS (Z39.96) Journal Publishing DTD v1.1 20151215//EN"),
            Some("http://jats.nlm.nih.gov/publishing/1.1/JATS-journalpublishing1.dtd"),
        );
        let jats_13 = bundled_jats_dtd(
            Some("-//NLM//DTD JATS (Z39.96) Journal Publishing DTD v1.3 20210610//EN"),
            Some("https://jats.nlm.nih.gov/publishing/1.3/JATS-journalpublishing1-3.dtd"),
        );

        assert_eq!(jats_11.map(|jats| jats.label), Some("JATS 1.1"));
        assert_eq!(jats_13.map(|jats| jats.label), Some("JATS 1.3"));
        assert!(bundled_jats_dtd(None, Some("https://example.org/article.dtd")).is_none());
    }

    #[test]
    fn detects_sps_version_from_article_specific_use() -> Result<(), String> {
        assert_eq!(
            detected_sps_version(r#"<article specific-use="sps-1.9"/>"#)?,
            Some("1.9")
        );
        assert_eq!(
            detected_sps_version(r#"<article specific-use="SPS-1.10"/>"#)?,
            Some("1.10")
        );
        assert_eq!(
            detected_sps_version(r#"<article specific-use="other"/>"#)?,
            None
        );
        Ok(())
    }

    #[test]
    fn project_keeps_sps_stage_when_jats_stage_fails() -> Result<(), Box<dyn std::error::Error>> {
        let root = fs::canonicalize(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testdata/project"),
        )?;
        let manifest =
            xml_project::parse_manifest(&fs::read_to_string(root.join("project.toml"))?)?;
        let project = OpenedProject { root, manifest };
        let source = r#"<article article-type="unsupported" dtd-version="1.3" specific-use="sps-1.10"><front><article-meta><title-group><article-title>Example</article-title></title-group></article-meta></front><body/></article>"#;
        let stages = tauri::async_runtime::block_on(validate_project_framework(
            &project,
            source,
            Revision(1),
            Some("article.xml"),
        ))?;

        assert_eq!(stages.len(), 2);
        assert_eq!(stages[0].id, "jats-grammar");
        assert!(matches!(stages[0].status, FrameworkStageStatus::Failed));
        assert_eq!(stages[1].id, "sps-rules");
        Ok(())
    }

    #[test]
    fn author_preview_loads_only_declared_confined_images() -> Result<(), Box<dyn std::error::Error>>
    {
        let temporary = tempfile::tempdir()?;
        let article_directory = temporary.path().join("article");
        fs::create_dir(&article_directory)?;
        let document_path = article_directory.join("article.xml");
        fs::write(&document_path, b"placeholder")?;
        fs::write(
            article_directory.join("figure.png"),
            b"\x89PNG\r\n\x1a\npreview",
        )?;
        fs::write(
            temporary.path().join("outside.png"),
            b"\x89PNG\r\n\x1a\noutside",
        )?;
        let source = r#"<article xmlns:xlink="http://www.w3.org/1999/xlink"><graphic xlink:href="figure.png"/><graphic xlink:href="../outside.png"/><inline-graphic xlink:href="https://example.org/remote.png"/></article>"#;

        let (assets, skipped) = collect_author_preview_assets(source, Some(&document_path))?;

        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].source, "figure.png");
        assert!(assets[0].data_uri.starts_with("data:image/png;base64,"));
        assert_eq!(skipped, 2);
        Ok(())
    }

    #[test]
    fn author_preview_rejects_disguised_and_unknown_images(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let temporary = tempfile::tempdir()?;
        let document_path = temporary.path().join("article.xml");
        fs::write(&document_path, b"placeholder")?;
        fs::write(temporary.path().join("vector.svg"), b"<svg><script/></svg>")?;
        fs::write(temporary.path().join("fake.jpg"), b"not an image")?;
        let source = r#"<article xmlns:xlink="http://www.w3.org/1999/xlink"><graphic xlink:href="vector.svg"/><graphic xlink:href="fake.jpg"/></article>"#;

        let (assets, skipped) = collect_author_preview_assets(source, Some(&document_path))?;

        assert!(assets.is_empty());
        assert_eq!(skipped, 2);
        Ok(())
    }

    #[test]
    fn follows_packtools_raster_extension_rules() {
        assert_eq!(packtools_image_reference("figure"), "figure.jpg");
        assert_eq!(packtools_image_reference("figure.tif"), "figure.jpg");
        assert_eq!(packtools_image_reference("figure.TIFF"), "figure.jpg");
        assert_eq!(packtools_image_reference("figure.webp"), "figure.webp");
    }

    #[test]
    fn detects_only_browser_safe_raster_signatures() {
        assert_eq!(
            preview_image_media_type(b"\x89PNG\r\n\x1a\n"),
            Some("image/png")
        );
        assert_eq!(
            preview_image_media_type(b"\xff\xd8\xff\xe0"),
            Some("image/jpeg")
        );
        assert_eq!(preview_image_media_type(b"GIF89a"), Some("image/gif"));
        assert_eq!(
            preview_image_media_type(b"RIFF0000WEBP"),
            Some("image/webp")
        );
        assert_eq!(preview_image_media_type(b"<svg><script/></svg>"), None);
    }
}
