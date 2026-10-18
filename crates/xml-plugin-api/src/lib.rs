use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::{Component, Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    process::Command,
    time::timeout,
};
use url::Url;

mod wasi;
pub use wasi::WasiComponentEngine;

pub const API_VERSION: u32 = 1;
const MAX_PLUGINS: usize = 256;
const MAX_CAPABILITIES: usize = 32;
const MAX_SCOPES: usize = 128;
const MAX_STRING_BYTES: usize = 4_096;
const MAX_ENTRYPOINT_BYTES: u64 = 256 * 1024 * 1024;
const MAX_REQUEST_BYTES: usize = 1024 * 1024;
const MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;
const MAX_STDERR_BYTES: usize = 64 * 1024;
const PROCESS_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub api_version: u32,
    pub license: String,
    pub runtime: PluginRuntime,
    pub entrypoint: String,
    pub sha256: String,
    #[serde(default)]
    pub capabilities: BTreeSet<Capability>,
    #[serde(default)]
    pub scopes: PluginScopes,
    #[serde(default)]
    pub contributions: BTreeSet<Contribution>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginRuntime {
    WasiComponent,
    SupervisedProcess,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    DocumentRead,
    DiagnosticsPublish,
    ProjectRead,
    TransformRun,
    FileRead,
    FileWrite,
    Network,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Contribution {
    Validator,
    Action,
    Formatter,
    Transformation,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PluginScopes {
    #[serde(default)]
    pub read_paths: Vec<String>,
    #[serde(default)]
    pub write_paths: Vec<String>,
    #[serde(default)]
    pub network_origins: Vec<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HostPolicy {
    pub capabilities: BTreeSet<Capability>,
    pub allow_supervised_processes: bool,
    pub read_roots: Vec<PathBuf>,
    pub write_roots: Vec<PathBuf>,
    pub network_origins: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizedPlugin {
    manifest: PluginManifest,
}

impl AuthorizedPlugin {
    #[must_use]
    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    /// Resolve and hash the entrypoint below a canonical package root.
    ///
    /// This verifies package integrity but deliberately does not execute code.
    pub fn verify_entrypoint(
        &self,
        package_root: &Path,
    ) -> Result<VerifiedEntrypoint, PluginError> {
        let package_root = package_root
            .canonicalize()
            .map_err(|error| PluginError::EntrypointIo(error.to_string()))?;
        if !package_root.is_dir() {
            return Err(PluginError::InvalidPackageRoot);
        }
        let entrypoint = package_root
            .join(&self.manifest.entrypoint)
            .canonicalize()
            .map_err(|error| PluginError::EntrypointIo(error.to_string()))?;
        if !entrypoint.starts_with(&package_root) || !entrypoint.is_file() {
            return Err(PluginError::EntrypointOutsidePackage);
        }
        let metadata = entrypoint
            .metadata()
            .map_err(|error| PluginError::EntrypointIo(error.to_string()))?;
        if metadata.len() > MAX_ENTRYPOINT_BYTES {
            return Err(PluginError::EntrypointTooLarge {
                limit: MAX_ENTRYPOINT_BYTES,
            });
        }
        let mut file = File::open(&entrypoint)
            .map_err(|error| PluginError::EntrypointIo(error.to_string()))?;
        let mut hasher = Sha256::new();
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read = file
                .read(&mut buffer)
                .map_err(|error| PluginError::EntrypointIo(error.to_string()))?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        let digest = encode_hex(&hasher.finalize());
        if !digest.eq_ignore_ascii_case(&self.manifest.sha256) {
            return Err(PluginError::ChecksumMismatch);
        }
        Ok(VerifiedEntrypoint {
            path: entrypoint,
            sha256: digest,
            bytes: metadata.len(),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedEntrypoint {
    pub path: PathBuf,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, Default)]
pub struct PluginRegistry {
    plugins: BTreeMap<String, AuthorizedPlugin>,
}

impl PluginRegistry {
    pub fn authorize_and_insert(
        &mut self,
        manifest: PluginManifest,
        policy: &HostPolicy,
    ) -> Result<&AuthorizedPlugin, PluginError> {
        authorize(&manifest, policy)?;
        if self.plugins.len() >= MAX_PLUGINS {
            return Err(PluginError::TooManyPlugins { limit: MAX_PLUGINS });
        }
        if self.plugins.contains_key(&manifest.id) {
            return Err(PluginError::DuplicateId(manifest.id));
        }
        let id = manifest.id.clone();
        self.plugins
            .insert(id.clone(), AuthorizedPlugin { manifest });
        self.plugins
            .get(&id)
            .ok_or(PluginError::RegistryFailure(id))
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<&AuthorizedPlugin> {
        self.plugins.get(id)
    }
}

pub fn authorize(manifest: &PluginManifest, policy: &HostPolicy) -> Result<(), PluginError> {
    validate_manifest(manifest)?;
    for capability in &manifest.capabilities {
        if !policy.capabilities.contains(capability) {
            return Err(PluginError::CapabilityDenied(*capability));
        }
    }
    if manifest.runtime == PluginRuntime::SupervisedProcess && !policy.allow_supervised_processes {
        return Err(PluginError::RuntimeDenied(manifest.runtime));
    }
    for path in &manifest.scopes.read_paths {
        authorize_path(path, &policy.read_roots, Capability::FileRead)?;
    }
    for path in &manifest.scopes.write_paths {
        authorize_path(path, &policy.write_roots, Capability::FileWrite)?;
    }
    for origin in &manifest.scopes.network_origins {
        if !policy.network_origins.contains(origin) {
            return Err(PluginError::NetworkOriginDenied(origin.clone()));
        }
    }
    Ok(())
}

fn validate_manifest(manifest: &PluginManifest) -> Result<(), PluginError> {
    validate_id(&manifest.id)?;
    for value in [&manifest.name, &manifest.version, &manifest.license] {
        validate_text(value)?;
    }
    if manifest.api_version != API_VERSION {
        return Err(PluginError::UnsupportedApiVersion {
            requested: manifest.api_version,
            supported: API_VERSION,
        });
    }
    validate_relative_path(&manifest.entrypoint)?;
    match manifest.runtime {
        PluginRuntime::WasiComponent if !manifest.entrypoint.ends_with(".wasm") => {
            return Err(PluginError::InvalidEntrypoint)
        }
        PluginRuntime::SupervisedProcess if manifest.entrypoint.ends_with(".wasm") => {
            return Err(PluginError::InvalidEntrypoint)
        }
        _ => {}
    }
    if manifest.sha256.len() != 64 || !manifest.sha256.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(PluginError::InvalidChecksum);
    }
    if manifest.capabilities.len() > MAX_CAPABILITIES {
        return Err(PluginError::TooManyCapabilities {
            limit: MAX_CAPABILITIES,
        });
    }
    validate_scopes(manifest)?;
    Ok(())
}

fn validate_scopes(manifest: &PluginManifest) -> Result<(), PluginError> {
    let scopes = &manifest.scopes;
    if scopes.read_paths.len() + scopes.write_paths.len() + scopes.network_origins.len()
        > MAX_SCOPES
    {
        return Err(PluginError::TooManyScopes { limit: MAX_SCOPES });
    }
    if !scopes.read_paths.is_empty() && !manifest.capabilities.contains(&Capability::FileRead) {
        return Err(PluginError::MissingCapability(Capability::FileRead));
    }
    if !scopes.write_paths.is_empty() && !manifest.capabilities.contains(&Capability::FileWrite) {
        return Err(PluginError::MissingCapability(Capability::FileWrite));
    }
    if !scopes.network_origins.is_empty() && !manifest.capabilities.contains(&Capability::Network) {
        return Err(PluginError::MissingCapability(Capability::Network));
    }
    for path in scopes.read_paths.iter().chain(&scopes.write_paths) {
        validate_relative_path(path)?;
    }
    for origin in &scopes.network_origins {
        validate_origin(origin)?;
    }
    Ok(())
}

fn validate_id(value: &str) -> Result<(), PluginError> {
    let valid = !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_'));
    if valid {
        Ok(())
    } else {
        Err(PluginError::InvalidId)
    }
}

fn validate_text(value: &str) -> Result<(), PluginError> {
    if value.trim().is_empty() || value.len() > MAX_STRING_BYTES || value.contains('\0') {
        Err(PluginError::InvalidText)
    } else {
        Ok(())
    }
}

fn validate_relative_path(value: &str) -> Result<(), PluginError> {
    let path = Path::new(value);
    let valid = !value.is_empty()
        && value.len() <= MAX_STRING_BYTES
        && !value.contains('\\')
        && !path.is_absolute()
        && path.components().all(|component| {
            matches!(component, Component::Normal(_))
                && !component.as_os_str().to_string_lossy().contains('\0')
        });
    if valid {
        Ok(())
    } else {
        Err(PluginError::InvalidRelativePath(value.to_owned()))
    }
}

fn validate_origin(value: &str) -> Result<(), PluginError> {
    let url = Url::parse(value).map_err(|_| PluginError::InvalidNetworkOrigin(value.to_owned()))?;
    let valid = url.scheme() == "https"
        && url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none()
        && url.path() == "/"
        && url.query().is_none()
        && url.fragment().is_none();
    if valid {
        Ok(())
    } else {
        Err(PluginError::InvalidNetworkOrigin(value.to_owned()))
    }
}

fn authorize_path(
    requested: &str,
    allowed_roots: &[PathBuf],
    capability: Capability,
) -> Result<(), PluginError> {
    let requested = Path::new(requested);
    if allowed_roots.iter().any(|root| requested.starts_with(root)) {
        Ok(())
    } else {
        Err(PluginError::PathScopeDenied {
            capability,
            path: requested.to_string_lossy().into_owned(),
        })
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut encoded = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PluginRequest {
    pub api_version: u32,
    pub request_id: u64,
    pub method: PluginMethod,
    pub params: Value,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginMethod {
    Validate,
    Format,
    Transform,
    RunAction,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PluginResponse {
    pub api_version: u32,
    pub request_id: u64,
    pub result: Option<Value>,
    pub error: Option<PluginResponseError>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PluginResponseError {
    pub code: String,
    pub message: String,
}

/// Executes process plugins only through an explicitly configured sandbox
/// launcher. The launcher must treat the final argument as the staged plugin
/// entrypoint and provide OS-level filesystem/network/process containment.
#[derive(Clone, Debug)]
pub struct SandboxedProcessEngine {
    launcher: PathBuf,
    arguments: Vec<String>,
    deadline: Duration,
}

impl SandboxedProcessEngine {
    pub fn new(launcher: impl AsRef<Path>, arguments: Vec<String>) -> Result<Self, PluginError> {
        let launcher = launcher
            .as_ref()
            .canonicalize()
            .map_err(|error| PluginError::Launcher(error.to_string()))?;
        if !launcher.is_file() || arguments.len() > 64 {
            return Err(PluginError::InvalidLauncher);
        }
        if arguments
            .iter()
            .any(|argument| argument.len() > MAX_STRING_BYTES || argument.contains('\0'))
        {
            return Err(PluginError::InvalidLauncherArgument);
        }
        Ok(Self {
            launcher,
            arguments,
            deadline: PROCESS_TIMEOUT,
        })
    }

    pub async fn invoke(
        &self,
        plugin: &AuthorizedPlugin,
        package_root: &Path,
        request: &PluginRequest,
    ) -> Result<PluginResponse, PluginError> {
        let result = self.invoke_inner(plugin, package_root, request).await;
        tracing::info!(
            action = "plugin.invoke",
            result = if result.is_ok() { "success" } else { "failure" },
            plugin_id = %plugin.manifest.id,
            request_id = request.request_id,
            method = ?request.method,
            runtime = "supervised-process",
        );
        result
    }

    async fn invoke_inner(
        &self,
        plugin: &AuthorizedPlugin,
        package_root: &Path,
        request: &PluginRequest,
    ) -> Result<PluginResponse, PluginError> {
        if plugin.manifest.runtime != PluginRuntime::SupervisedProcess {
            return Err(PluginError::RuntimeDenied(plugin.manifest.runtime));
        }
        if request.api_version != API_VERSION {
            return Err(PluginError::UnsupportedApiVersion {
                requested: request.api_version,
                supported: API_VERSION,
            });
        }
        let request_bytes = serde_json::to_vec(request)
            .map_err(|error| PluginError::Protocol(error.to_string()))?;
        if request_bytes.len() > MAX_REQUEST_BYTES {
            return Err(PluginError::RequestTooLarge {
                limit: MAX_REQUEST_BYTES,
            });
        }
        let verified = plugin.verify_entrypoint(package_root)?;
        let staging =
            tempfile::tempdir().map_err(|error| PluginError::Staging(error.to_string()))?;
        let staged_path = staging.path().join("plugin-entrypoint");
        copy_and_verify(&verified, &staged_path)?;

        let mut command = Command::new(&self.launcher);
        command
            .args(&self.arguments)
            .arg(&staged_path)
            .current_dir(staging.path())
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        let mut child = command
            .spawn()
            .map_err(|error| PluginError::Launch(error.to_string()))?;
        let mut stdin = child.stdin.take().ok_or(PluginError::MissingPipe)?;
        let stdout = child.stdout.take().ok_or(PluginError::MissingPipe)?;
        let stderr = child.stderr.take().ok_or(PluginError::MissingPipe)?;
        let writer = tokio::spawn(async move {
            stdin.write_all(&request_bytes).await?;
            stdin.write_all(b"\n").await?;
            stdin.shutdown().await
        });
        let output_reader = tokio::spawn(read_bounded(stdout, MAX_RESPONSE_BYTES));
        let error_reader = tokio::spawn(read_bounded(stderr, MAX_STDERR_BYTES));

        let status = match timeout(self.deadline, child.wait()).await {
            Ok(status) => status.map_err(|error| PluginError::Launch(error.to_string()))?,
            Err(_) => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                return Err(PluginError::Timeout);
            }
        };
        writer
            .await
            .map_err(|error| PluginError::Task(error.to_string()))?
            .map_err(|error| PluginError::Protocol(error.to_string()))?;
        let output = output_reader
            .await
            .map_err(|error| PluginError::Task(error.to_string()))??;
        let stderr = error_reader
            .await
            .map_err(|error| PluginError::Task(error.to_string()))??;
        if !status.success() {
            return Err(PluginError::ProcessFailure {
                code: status.code(),
                stderr: String::from_utf8_lossy(&stderr).into_owned(),
            });
        }
        let response: PluginResponse = serde_json::from_slice(&output)
            .map_err(|error| PluginError::Protocol(error.to_string()))?;
        validate_response(request, &response)?;
        Ok(response)
    }
}

async fn read_bounded<R>(reader: R, limit: usize) -> Result<Vec<u8>, PluginError>
where
    R: tokio::io::AsyncRead + Unpin,
{
    let take_limit = u64::try_from(limit.saturating_add(1)).unwrap_or(u64::MAX);
    let mut output = Vec::new();
    reader
        .take(take_limit)
        .read_to_end(&mut output)
        .await
        .map_err(|error| PluginError::Protocol(error.to_string()))?;
    if output.len() > limit {
        return Err(PluginError::OutputTooLarge { limit });
    }
    Ok(output)
}

fn copy_and_verify(
    verified: &VerifiedEntrypoint,
    destination_path: &Path,
) -> Result<(), PluginError> {
    let mut source =
        File::open(&verified.path).map_err(|error| PluginError::EntrypointIo(error.to_string()))?;
    let mut destination =
        File::create(destination_path).map_err(|error| PluginError::Staging(error.to_string()))?;
    let mut hasher = Sha256::new();
    let mut copied = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = source
            .read(&mut buffer)
            .map_err(|error| PluginError::EntrypointIo(error.to_string()))?;
        if read == 0 {
            break;
        }
        copied = copied.saturating_add(u64::try_from(read).unwrap_or(u64::MAX));
        if copied > MAX_ENTRYPOINT_BYTES {
            return Err(PluginError::EntrypointTooLarge {
                limit: MAX_ENTRYPOINT_BYTES,
            });
        }
        std::io::Write::write_all(&mut destination, &buffer[..read])
            .map_err(|error| PluginError::Staging(error.to_string()))?;
        hasher.update(&buffer[..read]);
    }
    let digest = encode_hex(&hasher.finalize());
    if copied != verified.bytes || digest != verified.sha256 {
        return Err(PluginError::ChecksumMismatch);
    }
    set_staged_permissions(destination_path)?;
    Ok(())
}

#[cfg(unix)]
fn set_staged_permissions(destination: &Path) -> Result<(), PluginError> {
    use std::os::unix::fs::PermissionsExt;

    std::fs::set_permissions(destination, std::fs::Permissions::from_mode(0o500))
        .map_err(|error| PluginError::Staging(error.to_string()))
}

#[cfg(not(unix))]
fn set_staged_permissions(_destination: &Path) -> Result<(), PluginError> {
    Ok(())
}

fn validate_response(
    request: &PluginRequest,
    response: &PluginResponse,
) -> Result<(), PluginError> {
    let exactly_one_outcome = response.result.is_some() != response.error.is_some();
    if response.api_version != API_VERSION
        || response.request_id != request.request_id
        || !exactly_one_outcome
    {
        return Err(PluginError::InvalidResponse);
    }
    if let Some(error) = &response.error {
        validate_text(&error.code)?;
        validate_text(&error.message)?;
    }
    Ok(())
}

#[derive(Debug, Error, Eq, PartialEq)]
pub enum PluginError {
    #[error("plugin identifier is invalid")]
    InvalidId,
    #[error("plugin text field is empty, oversized, or contains NUL")]
    InvalidText,
    #[error("plugin API version {requested} is unsupported; host supports {supported}")]
    UnsupportedApiVersion { requested: u32, supported: u32 },
    #[error("plugin entrypoint does not match its runtime")]
    InvalidEntrypoint,
    #[error("plugin package checksum must be a 64-character hexadecimal SHA-256")]
    InvalidChecksum,
    #[error("plugin package root must be an existing directory")]
    InvalidPackageRoot,
    #[error("plugin entrypoint resolves outside its package root")]
    EntrypointOutsidePackage,
    #[error("plugin entrypoint exceeds the {limit}-byte limit")]
    EntrypointTooLarge { limit: u64 },
    #[error("plugin entrypoint checksum does not match its manifest")]
    ChecksumMismatch,
    #[error("plugin entrypoint cannot be inspected: {0}")]
    EntrypointIo(String),
    #[error("plugin sandbox launcher is invalid")]
    InvalidLauncher,
    #[error("plugin sandbox launcher argument is invalid")]
    InvalidLauncherArgument,
    #[error("plugin sandbox launcher cannot be resolved: {0}")]
    Launcher(String),
    #[error("plugin staging failed: {0}")]
    Staging(String),
    #[error("plugin process cannot be launched: {0}")]
    Launch(String),
    #[error("plugin process pipes are unavailable")]
    MissingPipe,
    #[error("plugin request exceeds the {limit}-byte limit")]
    RequestTooLarge { limit: usize },
    #[error("plugin output exceeds the {limit}-byte limit")]
    OutputTooLarge { limit: usize },
    #[error("plugin process exceeded its deadline")]
    Timeout,
    #[error("plugin process failed with code {code:?}: {stderr}")]
    ProcessFailure { code: Option<i32>, stderr: String },
    #[error("plugin protocol failed: {0}")]
    Protocol(String),
    #[error("plugin runtime task failed: {0}")]
    Task(String),
    #[error("plugin returned a mismatched or ambiguous response")]
    InvalidResponse,
    #[error("plugin WebAssembly component failed: {0}")]
    Component(String),
    #[error("plugin relative path is invalid: {0}")]
    InvalidRelativePath(String),
    #[error("plugin network origin must be an HTTPS origin without credentials or a path: {0}")]
    InvalidNetworkOrigin(String),
    #[error("plugin requests more than {limit} capabilities")]
    TooManyCapabilities { limit: usize },
    #[error("plugin declares more than {limit} resource scopes")]
    TooManyScopes { limit: usize },
    #[error("plugin scope requires the {0:?} capability")]
    MissingCapability(Capability),
    #[error("plugin capability is denied: {0:?}")]
    CapabilityDenied(Capability),
    #[error("plugin runtime is denied: {0:?}")]
    RuntimeDenied(PluginRuntime),
    #[error("plugin {capability:?} scope is denied: {path}")]
    PathScopeDenied {
        capability: Capability,
        path: String,
    },
    #[error("plugin network origin is denied: {0}")]
    NetworkOriginDenied(String),
    #[error("plugin identifier is already registered: {0}")]
    DuplicateId(String),
    #[error("plugin registry contains more than {limit} entries")]
    TooManyPlugins { limit: usize },
    #[error("plugin registry failed to retain entry: {0}")]
    RegistryFailure(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> PluginManifest {
        PluginManifest {
            id: "org.example.validator".to_owned(),
            name: "Example validator".to_owned(),
            version: "1.0.0".to_owned(),
            api_version: API_VERSION,
            license: "MIT".to_owned(),
            runtime: PluginRuntime::WasiComponent,
            entrypoint: "bin/validator.wasm".to_owned(),
            sha256: "a".repeat(64),
            capabilities: BTreeSet::from([
                Capability::DocumentRead,
                Capability::DiagnosticsPublish,
                Capability::FileRead,
            ]),
            scopes: PluginScopes {
                read_paths: vec!["schemas/jats".to_owned()],
                ..PluginScopes::default()
            },
            contributions: BTreeSet::from([Contribution::Validator]),
        }
    }

    #[test]
    fn default_policy_denies_every_requested_capability() {
        assert!(matches!(
            authorize(&manifest(), &HostPolicy::default()),
            Err(PluginError::CapabilityDenied(_))
        ));
    }

    #[test]
    fn grants_only_explicit_capabilities_and_scopes() -> Result<(), PluginError> {
        let policy = HostPolicy {
            capabilities: manifest().capabilities,
            read_roots: vec![PathBuf::from("schemas")],
            ..HostPolicy::default()
        };
        let mut registry = PluginRegistry::default();
        let plugin = registry.authorize_and_insert(manifest(), &policy)?;
        assert_eq!(plugin.manifest().id, "org.example.validator");
        Ok(())
    }

    #[test]
    fn rejects_traversal_and_ungranted_origins() {
        let mut invalid = manifest();
        invalid.scopes.read_paths = vec!["../outside".to_owned()];
        assert!(matches!(
            validate_manifest(&invalid),
            Err(PluginError::InvalidRelativePath(_))
        ));

        let mut networked = manifest();
        networked.capabilities.insert(Capability::Network);
        networked.scopes.network_origins = vec!["https://api.example.org/".to_owned()];
        let policy = HostPolicy {
            capabilities: networked.capabilities.clone(),
            read_roots: vec![PathBuf::from("schemas")],
            ..HostPolicy::default()
        };
        assert!(matches!(
            authorize(&networked, &policy),
            Err(PluginError::NetworkOriginDenied(_))
        ));
    }

    #[test]
    fn rejects_native_or_mismatched_entrypoints() {
        let mut invalid = manifest();
        invalid.entrypoint = "bin/plugin.dylib".to_owned();
        assert!(matches!(
            validate_manifest(&invalid),
            Err(PluginError::InvalidEntrypoint)
        ));
    }

    #[test]
    fn verifies_sha256_before_returning_an_entrypoint() -> Result<(), Box<dyn std::error::Error>> {
        let package = tempfile::tempdir()?;
        let bin = package.path().join("bin");
        std::fs::create_dir(&bin)?;
        let entrypoint = bin.join("validator.wasm");
        std::fs::write(&entrypoint, b"plugin")?;
        let mut plugin_manifest = manifest();
        plugin_manifest.sha256 = encode_hex(&Sha256::digest(b"plugin"));
        let policy = HostPolicy {
            capabilities: plugin_manifest.capabilities.clone(),
            read_roots: vec![PathBuf::from("schemas")],
            ..HostPolicy::default()
        };
        let mut registry = PluginRegistry::default();
        let plugin = registry.authorize_and_insert(plugin_manifest, &policy)?;
        let verified = plugin.verify_entrypoint(package.path())?;
        assert_eq!(verified.bytes, 6);

        std::fs::write(entrypoint, b"tampered")?;
        assert!(matches!(
            plugin.verify_entrypoint(package.path()),
            Err(PluginError::ChecksumMismatch)
        ));
        Ok(())
    }

    #[test]
    fn invokes_an_ambient_capability_free_component() -> Result<(), Box<dyn std::error::Error>> {
        const COMPONENT: &str = r#"
            (component
              (core module $plugin
                (memory (export "memory") 1)
                (data (i32.const 1024)
                  "{\"apiVersion\":1,\"requestId\":7,\"result\":{\"ok\":true},\"error\":null}")
                (func (export "realloc")
                  (param i32 i32 i32 i32) (result i32)
                  i32.const 8192)
                (func (export "invoke") (param i32 i32) (result i32)
                  i32.const 512
                  i32.const 1024
                  i32.store
                  i32.const 512
                  i32.const 64
                  i32.store offset=4
                  i32.const 512))
              (core instance $plugin-instance (instantiate $plugin))
              (alias core export $plugin-instance "memory" (core memory $memory))
              (alias core export $plugin-instance "realloc" (core func $realloc))
              (alias core export $plugin-instance "invoke" (core func $invoke))
              (type $invoke-type (func (param "request" string) (result string)))
              (func $invoke-component (type $invoke-type)
                (canon lift (core func $invoke)
                  (memory $memory)
                  (realloc $realloc)
                  string-encoding=utf8))
              (export "invoke" (func $invoke-component)))
        "#;
        let bytes = wat::parse_str(COMPONENT)?;
        let package = tempfile::tempdir()?;
        let bin = package.path().join("bin");
        std::fs::create_dir(&bin)?;
        std::fs::write(bin.join("validator.wasm"), &bytes)?;
        let mut plugin_manifest = manifest();
        plugin_manifest.sha256 = encode_hex(&Sha256::digest(&bytes));
        let policy = HostPolicy {
            capabilities: plugin_manifest.capabilities.clone(),
            read_roots: vec![PathBuf::from("schemas")],
            ..HostPolicy::default()
        };
        let mut registry = PluginRegistry::default();
        let plugin = registry.authorize_and_insert(plugin_manifest, &policy)?;
        let request = PluginRequest {
            api_version: API_VERSION,
            request_id: 7,
            method: PluginMethod::Validate,
            params: serde_json::json!({"revision": 3}),
        };
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let response = runtime.block_on(WasiComponentEngine::new()?.invoke(
            plugin,
            package.path(),
            &request,
        ))?;
        assert_eq!(response.request_id, request.request_id);
        assert_eq!(response.result, Some(serde_json::json!({"ok": true})));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn supervises_json_rpc_through_an_explicit_launcher() -> Result<(), Box<dyn std::error::Error>>
    {
        let package = tempfile::tempdir()?;
        let bin = package.path().join("bin");
        std::fs::create_dir(&bin)?;
        let entrypoint = bin.join("plugin");
        std::fs::write(&entrypoint, b"opaque plugin bytes")?;
        let mut plugin_manifest = manifest();
        plugin_manifest.runtime = PluginRuntime::SupervisedProcess;
        plugin_manifest.entrypoint = "bin/plugin".to_owned();
        plugin_manifest.sha256 = encode_hex(&Sha256::digest(b"opaque plugin bytes"));
        let policy = HostPolicy {
            capabilities: plugin_manifest.capabilities.clone(),
            allow_supervised_processes: true,
            read_roots: vec![PathBuf::from("schemas")],
            ..HostPolicy::default()
        };
        let mut registry = PluginRegistry::default();
        let plugin = registry.authorize_and_insert(plugin_manifest, &policy)?;
        let response = r#"{"apiVersion":1,"requestId":7,"result":{"ok":true},"error":null}"#;
        let engine = SandboxedProcessEngine::new(
            "/bin/sh",
            vec![
                "-c".to_owned(),
                format!("read request; printf '%s' '{response}'"),
                "--".to_owned(),
            ],
        )?;
        let request = PluginRequest {
            api_version: API_VERSION,
            request_id: 7,
            method: PluginMethod::Validate,
            params: serde_json::json!({"revision": 3}),
        };
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let response = runtime.block_on(engine.invoke(plugin, package.path(), &request))?;
        assert_eq!(response.request_id, request.request_id);
        assert!(response.result.is_some());
        Ok(())
    }
}
