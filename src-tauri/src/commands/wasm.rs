use crate::skills::host::{InstalledSkill, WasmHost};
use crate::skills::manifest::SkillManifest;
use crate::skills::permissions::PermissionSet;
use crate::skills::runtime::WasmArgument;
use crate::utils::error::Result;
use std::sync::Arc;
use tauri::State;
use tracing::info;

/// Check if WASM host is initialized
#[tauri::command]
pub async fn wasm_host_initialized(host: State<'_, Arc<WasmHost>>) -> Result<bool> {
    Ok(host.list_skills().await.len() > 0)
}

/// List all installed WASM skills
#[tauri::command]
pub async fn wasm_list_skills(host: State<'_, Arc<WasmHost>>) -> Result<Vec<InstalledSkill>> {
    host.list_installed_skills().await
}

/// Get a specific skill manifest
#[tauri::command]
pub async fn wasm_get_skill_manifest(
    skill_id: String,
    host: State<'_, Arc<WasmHost>>,
) -> Result<Option<SkillManifest>> {
    Ok(host.get_skill_manifest(&skill_id).await)
}

/// Execute a WASM skill function
#[tauri::command]
pub async fn wasm_execute_skill(
    skill_id: String,
    function: String,
    args: Vec<WasmArgument>,
    host: State<'_, Arc<WasmHost>>,
) -> Result<crate::skills::runtime::WasmExecutionResult> {
    host.execute_skill(&skill_id, &function, args).await
}

/// Register a new WASM skill from base64 encoded WASM and manifest
#[tauri::command]
pub async fn wasm_register_skill(
    wasm_base64: String,
    manifest_json: String,
    permissions_json: String,
    host: State<'_, Arc<WasmHost>>,
) -> Result<()> {
    use base64::prelude::BASE64_STANDARD;
    use base64::Engine;

    // Decode WASM from base64
    let wasm_bytes = BASE64_STANDARD
        .decode(wasm_base64)
        .map_err(|e| crate::utils::error::AppError::Validation(format!("Invalid base64: {}", e)))?;

    // Parse manifest
    let manifest: SkillManifest = serde_json::from_str(&manifest_json).map_err(|e| {
        crate::utils::error::AppError::Validation(format!("Invalid manifest: {}", e))
    })?;

    // Parse permissions
    let permissions: PermissionSet = serde_json::from_str(&permissions_json).map_err(|e| {
        crate::utils::error::AppError::Validation(format!("Invalid permissions: {}", e))
    })?;

    // Create module
    let module =
        crate::skills::runtime::WasmModule::new(wasm_bytes, manifest.clone()).map_err(|e| {
            crate::utils::error::AppError::Internal(format!("Failed to create module: {}", e))
        })?;

    // Register skill
    host.register_skill(module, permissions)
        .await
        .map_err(|e| {
            crate::utils::error::AppError::Internal(format!("Failed to register skill: {}", e))
        })?;

    info!("Registered WASM skill: {}", manifest.name);
    Ok(())
}

/// Unregister a WASM skill
#[tauri::command]
pub async fn wasm_unregister_skill(skill_id: String, host: State<'_, Arc<WasmHost>>) -> Result<()> {
    host.unregister_skill(&skill_id).await
}

/// Set WASM skill enabled state
#[tauri::command]
pub async fn wasm_set_skill_enabled(
    skill_id: String,
    enabled: bool,
    host: State<'_, Arc<WasmHost>>,
) -> Result<()> {
    host.set_skill_enabled(&skill_id, enabled).await
}

/// Check if a skill is registered
#[tauri::command]
pub async fn wasm_is_skill_registered(
    skill_id: String,
    host: State<'_, Arc<WasmHost>>,
) -> Result<bool> {
    Ok(host.is_skill_registered(&skill_id).await)
}
