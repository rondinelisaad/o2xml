mod audit;

use std::{
    collections::{BTreeMap, HashMap},
    env, fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
};

use serde::Serialize;
use tauri::State;
use tracing_subscriber::EnvFilter;
use xml_document::XmlDocument;
use xml_domain::{Diagnostic, OutlineEntry, Revision};
use xml_project::ProjectManifest;
use xml_xpath::{Libxml2XPathEngine, XPathEngine, XPathReport, XPathRequest};

#[derive(Default)]
struct AppState {
    next_id: AtomicU64,
    documents: Mutex<HashMap<u64, XmlDocument>>,
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
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProjectPayload {
    path: String,
    manifest: ProjectManifest,
    files: Vec<String>,
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
    let revision = snapshot.revision();
    let analysis = tauri::async_runtime::spawn_blocking(move || {
        let text = snapshot.text();
        let diagnostics = xml_syntax::validate_well_formed(&text, revision, file.as_deref());
        let outline = if diagnostics.is_empty() {
            xml_syntax::outline(&text)
        } else {
            Vec::new()
        };
        AnalysisPayload {
            revision,
            diagnostics,
            outline,
        }
    })
    .await
    .map_err(|error| format!("validation worker failed: {error}"))?;
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
        let helper = xpath_helper_path()?;
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

fn xpath_helper_path() -> Result<PathBuf, String> {
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
            format!("XPath helper is unavailable; build the workspace or set {ENVIRONMENT_KEY}")
        })
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
fn open_project(path: String) -> Result<ProjectPayload, String> {
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
    Ok(ProjectPayload {
        path: root.to_string_lossy().into_owned(),
        manifest,
        files,
    })
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
            evaluate_xpath,
            format_document,
            save_document,
            open_project,
        ])
        .run(tauri::generate_context!());

    if let Err(error) = application {
        tracing::error!(action = "application.run", result = "failure", error = %error);
    }
}
