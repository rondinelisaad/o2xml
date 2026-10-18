use std::{fs, path::Path, time::Duration};

use sha2::Digest;
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store, StoreLimits, StoreLimitsBuilder,
};

use crate::{
    validate_response, AuthorizedPlugin, PluginError, PluginRequest, PluginResponse, PluginRuntime,
    API_VERSION, MAX_REQUEST_BYTES, MAX_RESPONSE_BYTES,
};

const MAX_COMPONENT_BYTES: u64 = 32 * 1024 * 1024;
const MAX_LINEAR_MEMORY_BYTES: usize = 64 * 1024 * 1024;
const INITIAL_FUEL: u64 = 10_000_000;
const COMPONENT_TIMEOUT: Duration = Duration::from_secs(5);

/// Embedded Component Model runtime with no host imports. Components receive a
/// single JSON request and must return a single JSON response from the root
/// export `invoke: func(string) -> string`.
#[derive(Clone)]
pub struct WasiComponentEngine {
    engine: Engine,
}

impl WasiComponentEngine {
    pub fn new() -> Result<Self, PluginError> {
        let mut config = Config::new();
        config
            .wasm_component_model(true)
            .consume_fuel(true)
            .epoch_interruption(true);
        let engine = Engine::new(&config).map_err(component_error)?;
        Ok(Self { engine })
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
            plugin_id = %plugin.manifest().id,
            request_id = request.request_id,
            method = ?request.method,
            runtime = "wasmtime-component",
        );
        result
    }

    async fn invoke_inner(
        &self,
        plugin: &AuthorizedPlugin,
        package_root: &Path,
        request: &PluginRequest,
    ) -> Result<PluginResponse, PluginError> {
        if plugin.manifest().runtime != PluginRuntime::WasiComponent {
            return Err(PluginError::RuntimeDenied(plugin.manifest().runtime));
        }
        if request.api_version != API_VERSION {
            return Err(PluginError::UnsupportedApiVersion {
                requested: request.api_version,
                supported: API_VERSION,
            });
        }
        let request_json = serde_json::to_string(request)
            .map_err(|error| PluginError::Protocol(error.to_string()))?;
        if request_json.len() > MAX_REQUEST_BYTES {
            return Err(PluginError::RequestTooLarge {
                limit: MAX_REQUEST_BYTES,
            });
        }
        let verified = plugin.verify_entrypoint(package_root)?;
        if verified.bytes > MAX_COMPONENT_BYTES {
            return Err(PluginError::EntrypointTooLarge {
                limit: MAX_COMPONENT_BYTES,
            });
        }
        let bytes = fs::read(&verified.path)
            .map_err(|error| PluginError::EntrypointIo(error.to_string()))?;
        let digest = crate::encode_hex(&sha2::Sha256::digest(&bytes));
        if digest != verified.sha256 {
            return Err(PluginError::ChecksumMismatch);
        }

        let engine = self.engine.clone();
        let interrupt_engine = self.engine.clone();
        let task =
            tokio::task::spawn_blocking(move || run_component(&engine, &bytes, request_json));
        match tokio::time::timeout(COMPONENT_TIMEOUT, task).await {
            Ok(result) => result.map_err(|error| PluginError::Task(error.to_string()))?,
            Err(_) => {
                interrupt_engine.increment_epoch();
                Err(PluginError::Timeout)
            }
        }
    }
}

fn run_component(
    engine: &Engine,
    bytes: &[u8],
    request_json: String,
) -> Result<PluginResponse, PluginError> {
    let request: PluginRequest = serde_json::from_str(&request_json)
        .map_err(|error| PluginError::Protocol(error.to_string()))?;
    let component = Component::new(engine, bytes).map_err(component_error)?;
    let linker = Linker::<StoreLimits>::new(engine);
    let limits = StoreLimitsBuilder::new()
        .memory_size(MAX_LINEAR_MEMORY_BYTES)
        .instances(1)
        .memories(2)
        .tables(4)
        .build();
    let mut store = Store::new(engine, limits);
    store.limiter(|limits| limits);
    store.set_fuel(INITIAL_FUEL).map_err(component_error)?;
    store.set_epoch_deadline(1);
    let instance = linker
        .instantiate(&mut store, &component)
        .map_err(component_error)?;
    let function = instance
        .get_typed_func::<(String,), (String,)>(&mut store, "invoke")
        .map_err(component_error)?;
    let (response_json,) = function
        .call(&mut store, (request_json,))
        .map_err(component_error)?;
    if response_json.len() > MAX_RESPONSE_BYTES {
        return Err(PluginError::OutputTooLarge {
            limit: MAX_RESPONSE_BYTES,
        });
    }
    let response = serde_json::from_str(&response_json)
        .map_err(|error| PluginError::Protocol(error.to_string()))?;
    validate_response(&request, &response)?;
    Ok(response)
}

fn component_error(error: impl std::fmt::Display) -> PluginError {
    PluginError::Component(error.to_string())
}
