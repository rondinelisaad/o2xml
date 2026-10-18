use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use url::Url;

pub const API_VERSION: u32 = 1;
const MAX_PLUGINS: usize = 256;
const MAX_CAPABILITIES: usize = 32;
const MAX_SCOPES: usize = 128;
const MAX_STRING_BYTES: usize = 4_096;

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
}
