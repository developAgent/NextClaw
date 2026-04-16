use crate::skills::manifest::SkillManifest;
use crate::skills::permissions::{Permission, PermissionSet};
use crate::skills::sandbox::SandboxConfig;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, info};
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::pipe::MemoryOutputPipe;
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::WasiCtxBuilder;

/// WASM runtime configuration
#[derive(Debug, Clone)]
pub struct WasmRuntimeConfig {
    /// Maximum execution time for WASM modules
    pub max_execution_time: Duration,
    /// Maximum memory allocation in bytes
    pub max_memory: Option<u64>,
    /// Enable WASI (WebAssembly System Interface)
    pub enable_wasi: bool,
    /// Enable debug output
    pub debug: bool,
}

impl Default for WasmRuntimeConfig {
    fn default() -> Self {
        Self {
            max_execution_time: Duration::from_secs(30),
            max_memory: Some(128 * 1024 * 1024),
            enable_wasi: true,
            debug: false,
        }
    }
}

/// WASM module execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmExecutionResult {
    /// Exit status
    pub status: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Memory used in bytes
    pub memory_used: Option<u64>,
}

/// WASM function argument
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WasmArgument {
    #[serde(rename = "string")]
    String(String),
    #[serde(rename = "number")]
    Number(f64),
    #[serde(rename = "boolean")]
    Boolean(bool),
    #[serde(rename = "array")]
    Array(Vec<WasmArgument>),
    #[serde(rename = "object")]
    Object(HashMap<String, WasmArgument>),
    #[serde(rename = "null")]
    Null,
}

/// WASM module
#[derive(Clone)]
pub struct WasmModule {
    bytes: Vec<u8>,
    manifest: SkillManifest,
}

impl WasmModule {
    /// Create a new WASM module from bytes
    pub fn new(bytes: Vec<u8>, manifest: SkillManifest) -> Result<Self> {
        if bytes.len() < 4 || &bytes[0..4] != b"\0asm" {
            anyhow::bail!("Invalid WASM file");
        }

        manifest
            .validate()
            .map_err(|e| anyhow::anyhow!("Invalid skill manifest: {}", e))?;

        Ok(Self { bytes, manifest })
    }

    /// Load a WASM module from a file
    pub fn from_file(path: PathBuf, manifest: SkillManifest) -> Result<Self> {
        let bytes = std::fs::read(&path)
            .map_err(|e| anyhow::anyhow!("Failed to read WASM file: {:?}", path).context(e))?;

        Self::new(bytes, manifest)
    }

    /// Get the manifest
    pub fn manifest(&self) -> &SkillManifest {
        &self.manifest
    }

    /// Get the WASM bytes
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// WASM runtime for executing WASM modules
pub struct WasmRuntime {
    config: WasmRuntimeConfig,
    sandbox_config: SandboxConfig,
    permission_set: PermissionSet,
}

impl WasmRuntime {
    /// Create a new WASM runtime
    pub fn new(config: WasmRuntimeConfig, sandbox_config: SandboxConfig) -> Self {
        Self {
            config,
            sandbox_config,
            permission_set: PermissionSet::new(),
        }
    }

    /// Create a runtime with default configuration
    pub fn default() -> Self {
        Self::new(WasmRuntimeConfig::default(), SandboxConfig::default())
    }

    /// Create a runtime with permissive configuration
    pub fn permissive() -> Self {
        Self::new(WasmRuntimeConfig::default(), SandboxConfig::permissive())
    }

    /// Set permissions for the runtime
    pub fn set_permissions(&mut self, permissions: PermissionSet) {
        self.permission_set = permissions;
    }

    /// Execute a WASM module
    pub fn execute_module(
        &self,
        module: &WasmModule,
        function: &str,
        args: Vec<WasmArgument>,
    ) -> Result<WasmExecutionResult> {
        let start_time = std::time::Instant::now();

        debug!("Executing WASM module: {}", module.manifest().id);
        debug!("Function: {}, Args: {:?}", function, args);

        self.check_permissions(module.manifest())?;

        let result = if self.config.enable_wasi {
            self.execute_wasi_module(module, function, args)?
        } else {
            self.execute_raw_module(module, function, args)?
        };

        let execution_time = start_time.elapsed();

        info!(
            "Wasm module executed: {} (status: {}, time: {}ms)",
            module.manifest().id,
            result.status,
            execution_time.as_millis()
        );

        Ok(WasmExecutionResult {
            status: result.status,
            stdout: result.stdout,
            stderr: result.stderr,
            execution_time_ms: execution_time.as_millis() as u64,
            memory_used: result.memory_used,
        })
    }

    /// Execute a WASM module with WASI support
    fn execute_wasi_module(
        &self,
        module: &WasmModule,
        function: &str,
        args: Vec<WasmArgument>,
    ) -> Result<WasmExecutionResult> {
        debug!("Executing WASI module: {}", module.manifest().id);

        let engine = Engine::default();
        let compiled_module = Module::from_binary(&engine, module.bytes())
            .context("Failed to compile WASI module")?;

        let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);
        preview1::add_to_linker_sync(&mut linker, |ctx| ctx)
            .context("Failed to configure WASI linker")?;

        let stdout_pipe = MemoryOutputPipe::new(1024 * 1024);
        let stderr_pipe = MemoryOutputPipe::new(1024 * 1024);

        let guest_args = self.build_guest_cli_args(module, function);
        let guest_env = self.build_guest_env(module, &args);

        let mut wasi = WasiCtxBuilder::new();
        wasi.stdout(stdout_pipe.clone());
        wasi.stderr(stderr_pipe.clone());
        wasi.args(&guest_args).context("Failed to set WASI args")?;
        wasi.envs(&guest_env).context("Failed to set WASI env")?;
        wasi.allow_blocking_current_thread(true);

        let wasi_ctx = wasi.build_p1();
        let mut store = Store::new(&engine, wasi_ctx);
        let instance = linker
            .instantiate(&mut store, &compiled_module)
            .context("Failed to instantiate WASI module")?;

        let entry = self.resolve_wasi_entry(function, module.manifest());
        let status = self.call_wasm_function(&mut store, &instance, &entry)?;

        Ok(WasmExecutionResult {
            status,
            stdout: Self::pipe_to_string(&stdout_pipe),
            stderr: Self::pipe_to_string(&stderr_pipe),
            execution_time_ms: 0,
            memory_used: None,
        })
    }

    /// Execute a WASM module without WASI support
    fn execute_raw_module(
        &self,
        module: &WasmModule,
        function: &str,
        _args: Vec<WasmArgument>,
    ) -> Result<WasmExecutionResult> {
        debug!("Executing raw WASM module: {}", module.manifest().id);

        let engine = Engine::default();
        let compiled_module = Module::from_binary(&engine, module.bytes())
            .context("Failed to compile raw WASM module")?;
        let mut store = Store::new(&engine, ());
        let linker: Linker<()> = Linker::new(&engine);
        let instance = linker
            .instantiate(&mut store, &compiled_module)
            .context("Failed to instantiate raw WASM module")?;

        let entry = self.resolve_raw_entry(function, module.manifest());
        let status = self.call_wasm_function(&mut store, &instance, &entry)?;

        Ok(WasmExecutionResult {
            status,
            stdout: String::new(),
            stderr: String::new(),
            execution_time_ms: 0,
            memory_used: None,
        })
    }

    fn build_guest_cli_args(&self, module: &WasmModule, function: &str) -> Vec<String> {
        vec![
            module.manifest().id.clone(),
            self.resolve_wasi_entry(function, module.manifest()),
        ]
    }

    fn build_guest_env(&self, module: &WasmModule, args: &[WasmArgument]) -> Vec<(String, String)> {
        vec![
            ("CEOCLAW_SKILL_ID".to_string(), module.manifest().id.clone()),
            (
                "CEOCLAW_SKILL_VERSION".to_string(),
                module.manifest().version.clone(),
            ),
            (
                "CEOCLAW_ARGS".to_string(),
                serde_json::to_string(&Self::normalize_args(args))
                    .unwrap_or_else(|_| "{}".to_string()),
            ),
        ]
    }

    fn normalize_args(args: &[WasmArgument]) -> HashMap<String, WasmArgument> {
        match args {
            [WasmArgument::Object(map)] => map.clone(),
            _ => {
                let mut normalized = HashMap::new();
                normalized.insert("args".to_string(), WasmArgument::Array(args.to_vec()));
                for (index, argument) in args.iter().cloned().enumerate() {
                    normalized.insert(format!("arg{}", index), argument);
                }
                normalized
            }
        }
    }

    fn resolve_wasi_entry(&self, function: &str, manifest: &SkillManifest) -> String {
        if function.is_empty() || function == manifest.entry_point || function == "main" {
            "_start".to_string()
        } else {
            function.to_string()
        }
    }

    fn resolve_raw_entry(&self, function: &str, manifest: &SkillManifest) -> String {
        if function.is_empty() {
            manifest.entry_point.clone()
        } else {
            function.to_string()
        }
    }

    fn call_wasm_function<T>(
        &self,
        store: &mut Store<T>,
        instance: &wasmtime::Instance,
        function_name: &str,
    ) -> Result<i32> {
        if let Ok(func) = instance.get_typed_func::<(), ()>(&mut *store, function_name) {
            func.call(&mut *store, ())
                .with_context(|| format!("Failed to call WASM function '{}'", function_name))?;
            return Ok(0);
        }

        if let Ok(func) = instance.get_typed_func::<(), i32>(&mut *store, function_name) {
            let status = func
                .call(&mut *store, ())
                .with_context(|| format!("Failed to call WASM function '{}'", function_name))?;
            return Ok(status);
        }

        anyhow::bail!(
            "WASM function '{}' must have signature () -> () or () -> i32",
            function_name
        )
    }

    fn pipe_to_string(pipe: &MemoryOutputPipe) -> String {
        String::from_utf8_lossy(&pipe.contents()).to_string()
    }

    /// Check if the module has the required permissions
    fn check_permissions(&self, manifest: &SkillManifest) -> Result<()> {
        for permission in &manifest.permissions {
            let perm =
                Permission::new(permission.permission_type.clone(), permission.scope.clone());

            let is_granted = self.permission_set.get_granted().iter().any(|p| {
                p.permission_type == perm.permission_type
                    && (p.scope.is_none() || p.scope == perm.scope)
            });

            if permission.required && !is_granted {
                anyhow::bail!(
                    "Required permission not granted: {}",
                    permission.permission_type
                );
            }
        }

        Ok(())
    }

    /// Get runtime configuration
    pub fn config(&self) -> &WasmRuntimeConfig {
        &self.config
    }

    /// Get sandbox configuration
    pub fn sandbox_config(&self) -> &SandboxConfig {
        &self.sandbox_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_module_creation() {
        let wasm_bytes = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

        let manifest = SkillManifest::new(
            "com.example.test".to_string(),
            "Test Skill".to_string(),
            "1.0.0".to_string(),
            "A test skill".to_string(),
            "Test Author".to_string(),
        );

        let result = WasmModule::new(wasm_bytes, manifest);
        assert!(result.is_ok());
    }

    #[test]
    fn test_wasm_module_invalid_magic() {
        let wasm_bytes = vec![0x00, 0x00, 0x00, 0x00];

        let manifest = SkillManifest::new(
            "com.example.test".to_string(),
            "Test Skill".to_string(),
            "1.0.0".to_string(),
            "A test skill".to_string(),
            "Test Author".to_string(),
        );

        let result = WasmModule::new(wasm_bytes, manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_wasm_runtime_creation() {
        let runtime = WasmRuntime::default();
        assert_eq!(runtime.config().max_execution_time.as_secs(), 30);
    }

    #[test]
    fn test_wasm_runtime_permissive() {
        let runtime = WasmRuntime::permissive();
        assert!(runtime.sandbox_config().enable_filesystem);
    }

    #[test]
    fn test_normalize_args_preserves_named_object() {
        let mut map = HashMap::new();
        map.insert(
            "name".to_string(),
            WasmArgument::String("Claude".to_string()),
        );

        let normalized = WasmRuntime::normalize_args(&[WasmArgument::Object(map.clone())]);
        assert_eq!(normalized.len(), 1);
        assert!(
            matches!(normalized.get("name"), Some(WasmArgument::String(value)) if value == "Claude")
        );
    }

    #[test]
    fn test_normalize_args_wraps_positional_values() {
        let normalized = WasmRuntime::normalize_args(&[
            WasmArgument::String("hello".to_string()),
            WasmArgument::Boolean(true),
        ]);

        assert!(normalized.contains_key("args"));
        assert!(
            matches!(normalized.get("arg0"), Some(WasmArgument::String(value)) if value == "hello")
        );
        assert!(matches!(
            normalized.get("arg1"),
            Some(WasmArgument::Boolean(true))
        ));
    }
}
