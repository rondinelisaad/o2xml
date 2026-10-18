mod audit;

use std::{
    collections::{BTreeMap, HashMap},
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
};

use serde::{Deserialize, Serialize};
use tauri::State;
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
use xml_schematron::{SchXsltProcessEngine, SchXsltRequest};
use xml_transform::{
    LibxsltProcessEngine, LibxsltRequest, SaxonProcessEngine, SaxonXQueryEngine, TransformRequest,
    XQueryRequest,
};
use xml_validation::{GrammarReference, Libxml2ProcessEngine, ValidationEngine, ValidationRequest};
use xml_xpath::{Libxml2XPathEngine, XPathEngine, XPathReport, XPathRequest};
use xml_xsd::{Libxml2XsdIndexEngine, XsdIndexEngine, XsdIndexRequest};

#[derive(Default)]
struct AppState {
    next_id: AtomicU64,
    documents: Mutex<HashMap<u64, XmlDocument>>,
    project: Mutex<Option<OpenedProject>>,
    dtd_indexes: Mutex<HashMap<(String, Option<String>), SchemaIndex>>,
    xsd_indexes: Mutex<HashMap<String, SchemaIndex>>,
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
async fn analyze_document(id: u64, state: State<'_, AppState>) -> Result<AnalysisPayload, String> {
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
    const MAX_SCHEMATRON_BYTES: u64 = 16 * 1024 * 1024;
    let metadata = fs::metadata(&resource.path)
        .map_err(|error| format!("cannot inspect Schematron rules: {error}"))?;
    if metadata.len() > MAX_SCHEMATRON_BYTES {
        return Err("Schematron rules exceed the 16 MiB safety limit".to_owned());
    }
    let schematron = fs::read_to_string(&resource.path)
        .map_err(|error| format!("cannot read Schematron rules as UTF-8: {error}"))?;
    engine
        .validate(SchXsltRequest {
            source,
            schematron: &schematron,
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
