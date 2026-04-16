use crate::db::Database;
use crate::skills::manifest::SkillManifest;
use crate::skills::permissions::{Permission, PermissionChecker, PermissionSet};
use crate::skills::runtime::{WasmArgument, WasmExecutionResult, WasmModule, WasmRuntime};
use crate::skills::sandbox::SandboxConfig;
use crate::utils::error::{AppError, Result};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledSkill {
    pub manifest: SkillManifest,
    pub enabled: bool,
    pub permissions: PermissionSet,
}

/// WASM host configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmHostConfig {
    /// Maximum number of loaded modules
    pub max_loaded_modules: usize,
    /// Default sandbox configuration
    pub sandbox_config: SandboxConfig,
    /// Enable skill hot-reloading
    pub enable_hot_reload: bool,
}

impl Default for WasmHostConfig {
    fn default() -> Self {
        Self {
            max_loaded_modules: 100,
            sandbox_config: SandboxConfig::restrictive(),
            enable_hot_reload: false,
        }
    }
}

/// WASM host for managing skill execution
pub struct WasmHost {
    config: WasmHostConfig,
    db: Arc<Database>,
    permission_checker: Arc<Mutex<PermissionChecker>>,
    modules: Arc<RwLock<HashMap<String, WasmModule>>>,
    runtimes: Arc<RwLock<HashMap<String, Arc<WasmRuntime>>>>,
}

impl WasmHost {
    fn runtime_with_permissions(&self, permissions: PermissionSet) -> Arc<WasmRuntime> {
        let mut runtime = WasmRuntime::new(
            crate::skills::runtime::WasmRuntimeConfig::default(),
            self.config.sandbox_config.clone(),
        );
        runtime.set_permissions(permissions);
        Arc::new(runtime)
    }

    fn default_permissions_from_manifest(manifest: &SkillManifest) -> PermissionSet {
        let mut permissions = PermissionSet::new();

        for permission in &manifest.permissions {
            let permission_value =
                Permission::new(permission.permission_type.clone(), permission.scope.clone());

            if permission.required {
                permissions.grant(permission_value);
            }
        }

        permissions
    }

    /// Create a new WASM host
    pub fn new(config: WasmHostConfig, db: Arc<Database>) -> Self {
        Self {
            config,
            db,
            permission_checker: Arc::new(Mutex::new(PermissionChecker::new())),
            modules: Arc::new(RwLock::new(HashMap::new())),
            runtimes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a WASM host with default configuration
    pub fn default(db: Arc<Database>) -> Self {
        Self::new(WasmHostConfig::default(), db)
    }

    /// Initialize the WASM host and load installed skills
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing WASM host");

        // Load installed skills from database
        self.load_installed_skills().await?;

        info!("WASM host initialized successfully");
        Ok(())
    }

    /// Load installed skills from database
    async fn load_installed_skills(&self) -> Result<()> {
        let installed_skills = self.list_installed_skills().await?;

        let mut loaded_count = 0;
        for installed_skill in installed_skills {
            if installed_skill.enabled {
                self.load_skill_into_memory(&installed_skill.manifest, installed_skill.permissions)
                    .await?;
                loaded_count += 1;
            }
        }

        info!("Loaded {} WASM skills", loaded_count);
        Ok(())
    }

    /// List all installed skills
    pub async fn list_installed_skills(&self) -> Result<Vec<InstalledSkill>> {
        let conn = self.db.conn();
        let conn_guard = conn.blocking_lock();

        let mut stmt = conn_guard
            .prepare("SELECT config, enabled, permissions_json FROM skills ORDER BY name COLLATE NOCASE ASC")
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, bool>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut installed_skills = Vec::new();
        for row in rows {
            let (manifest_json, enabled, permissions_json) =
                row.map_err(|e| AppError::Database(e.to_string()))?;
            let Some(manifest_json) = manifest_json else {
                continue;
            };

            let manifest: SkillManifest = serde_json::from_str(&manifest_json)
                .map_err(|e| AppError::Database(e.to_string()))?;
            let permissions = permissions_json
                .as_deref()
                .map(serde_json::from_str::<PermissionSet>)
                .transpose()
                .map_err(|e| AppError::Database(e.to_string()))?
                .unwrap_or_else(|| Self::default_permissions_from_manifest(&manifest));
            installed_skills.push(InstalledSkill {
                manifest,
                enabled,
                permissions,
            });
        }

        Ok(installed_skills)
    }

    pub async fn set_skill_enabled(&self, skill_id: &str, enabled: bool) -> Result<()> {
        let installed_skill = self
            .list_installed_skills()
            .await?
            .into_iter()
            .find(|skill| skill.manifest.id == skill_id)
            .ok_or_else(|| AppError::Validation(format!("Skill not found: {}", skill_id)))?;

        if enabled {
            self.load_skill_into_memory(&installed_skill.manifest, installed_skill.permissions)
                .await?;
        } else {
            self.unload_skill_from_memory(skill_id).await;
        }

        let conn = self.db.conn();
        let conn_guard = conn.blocking_lock();
        conn_guard
            .execute(
                "UPDATE skills SET enabled = ?1, updated_at = ?2 WHERE id = ?3",
                rusqlite::params![enabled, chrono::Utc::now().timestamp(), skill_id],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    /// Register a new skill
    pub async fn register_skill(
        &self,
        module: WasmModule,
        permissions: PermissionSet,
    ) -> Result<()> {
        let skill_id = module.manifest().id.clone();

        let modules = self.modules.read().await;
        if modules.len() >= self.config.max_loaded_modules {
            return Err(AppError::Internal(
                "Maximum number of loaded modules reached".to_string(),
            ));
        }
        drop(modules);

        self.persist_skill_files(&module)?;

        let mut modules = self.modules.write().await;
        modules.insert(skill_id.clone(), module.clone());
        drop(modules);

        {
            let mut checker = self.permission_checker.lock().await;
            checker.set_permissions(skill_id.clone(), permissions.clone());
        }

        let runtime = self.runtime_with_permissions(permissions);
        let mut runtimes = self.runtimes.write().await;
        runtimes.insert(skill_id.clone(), runtime);

        self.store_skill_in_db(&module, &permissions).await?;

        info!("Registered skill: {}", module.manifest().name);
        Ok(())
    }

    async fn load_skill_into_memory(
        &self,
        manifest: &SkillManifest,
        permissions: PermissionSet,
    ) -> Result<()> {
        let skill_path = self.get_skill_path(&manifest.id)?;
        if !skill_path.exists() {
            return Err(AppError::Validation(format!(
                "Skill files not found: {}",
                manifest.id
            )));
        }

        let module = self.load_skill_from_directory(&skill_path).await?;

        let mut modules = self.modules.write().await;
        modules.insert(manifest.id.clone(), module);
        drop(modules);

        {
            let mut checker = self.permission_checker.lock().await;
            checker.set_permissions(manifest.id.clone(), permissions.clone());
        }

        let runtime = self.runtime_with_permissions(permissions);
        let mut runtimes = self.runtimes.write().await;
        runtimes.insert(manifest.id.clone(), runtime);

        debug!("Loaded skill: {}", manifest.name);
        Ok(())
    }

    async fn unload_skill_from_memory(&self, skill_id: &str) {
        let mut modules = self.modules.write().await;
        modules.remove(skill_id);
        drop(modules);

        let mut runtimes = self.runtimes.write().await;
        runtimes.remove(skill_id);
    }

    /// Unregister a skill
    pub async fn unregister_skill(&self, skill_id: &str) -> Result<()> {
        self.unload_skill_from_memory(skill_id).await;

        // Remove from database
        let conn = self.db.conn();
        let conn_guard = conn.blocking_lock();
        conn_guard
            .execute("DELETE FROM skills WHERE id = ?1", [skill_id])
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Remove permissions
        {
            let mut checker = self.permission_checker.lock().await;
            checker.remove_permissions(skill_id);
        }

        info!("Unregistered skill: {}", skill_id);
        Ok(())
    }

    /// Execute a skill function
    pub async fn execute_skill(
        &self,
        skill_id: &str,
        function: &str,
        args: Vec<WasmArgument>,
    ) -> Result<WasmExecutionResult> {
        // Get the module
        let module = {
            let guard = self.modules.read().await;
            match guard.get(skill_id) {
                Some(m) => m.clone(),
                None => {
                    return Err(AppError::Validation(format!(
                        "Skill not found: {}",
                        skill_id
                    )))
                }
            }
        };

        // Get the runtime
        let runtime = {
            let guard = self.runtimes.read().await;
            match guard.get(skill_id) {
                Some(r) => Arc::clone(r),
                None => {
                    return Err(AppError::Validation(format!(
                        "Runtime not found for skill: {}",
                        skill_id
                    )))
                }
            }
        };

        let function_owned = function.to_string();

        // Execute
        tokio::task::spawn_blocking(move || {
            runtime
                .execute_module(&module, &function_owned, args)
                .map_err(|e| AppError::Internal(format!("Skill execution failed: {}", e)))
        })
        .await
        .context("Failed to join execution task")?
    }

    /// Get skill manifest
    pub async fn get_skill_manifest(&self, skill_id: &str) -> Option<SkillManifest> {
        let modules = self.modules.read().await;
        modules.get(skill_id).map(|m| m.manifest().clone())
    }

    /// List all registered skills
    pub async fn list_skills(&self) -> Vec<SkillManifest> {
        let modules = self.modules.read().await;
        modules.values().map(|m| m.manifest().clone()).collect()
    }

    /// Check if a skill is registered
    pub async fn is_skill_registered(&self, skill_id: &str) -> bool {
        let modules = self.modules.read().await;
        modules.contains_key(skill_id)
    }

    /// Load a skill from a file
    async fn load_skill_from_directory(&self, path: &Path) -> Result<WasmModule> {
        // Read the manifest file
        let manifest_path = path.join("manifest.json");
        let manifest_json = std::fs::read_to_string(&manifest_path)
            .context(format!("Failed to read manifest: {:?}", manifest_path))?;

        let manifest: SkillManifest =
            serde_json::from_str(&manifest_json).context("Failed to parse manifest")?;

        // Read the WASM file
        let wasm_path = path.join("skill.wasm");
        let wasm_bytes = std::fs::read(&wasm_path)
            .context(format!("Failed to read WASM file: {:?}", wasm_path))?;

        Ok(WasmModule::new(wasm_bytes, manifest)?)
    }

    pub fn skill_storage_path(&self, skill_id: &str) -> Result<PathBuf> {
        self.get_skill_path(skill_id)
    }

    /// Store a skill in the database
    async fn store_skill_in_db(
        &self,
        module: &WasmModule,
        permissions: &PermissionSet,
    ) -> Result<()> {
        let conn = self.db.conn();
        let conn_guard = conn.blocking_lock();

        let manifest = module.manifest();
        let now = chrono::Utc::now().timestamp();
        let permissions_json =
            serde_json::to_string(permissions).map_err(|e| AppError::Internal(e.to_string()))?;

        conn_guard.execute(
            r#"
            INSERT OR REPLACE INTO skills (id, name, version, description, author, enabled, config, permissions_json, installed_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            rusqlite::params![
                &manifest.id,
                &manifest.name,
                &manifest.version,
                &manifest.description,
                &manifest.author,
                true,
                serde_json::to_string(&manifest).ok(),
                permissions_json,
                now,
                now,
            ],
        ).map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    fn persist_skill_files(&self, module: &WasmModule) -> Result<()> {
        let skill_path = self.get_skill_path(&module.manifest().id)?;
        self.persist_skill_files_to_path(module, &skill_path)
    }

    pub fn persist_skill_files_to_path(
        &self,
        module: &WasmModule,
        skill_path: &Path,
    ) -> Result<()> {
        std::fs::create_dir_all(skill_path).map_err(|e| AppError::Io(e.to_string()))?;

        let manifest_path = skill_path.join("manifest.json");
        let wasm_path = skill_path.join("skill.wasm");

        std::fs::write(
            manifest_path,
            serde_json::to_vec_pretty(module.manifest())
                .map_err(|e| AppError::Internal(e.to_string()))?,
        )
        .map_err(|e| AppError::Io(e.to_string()))?;

        std::fs::write(wasm_path, module.bytes()).map_err(|e| AppError::Io(e.to_string()))?;
        Ok(())
    }

    /// Get the path for a skill
    fn get_skill_path(&self, skill_id: &str) -> Result<PathBuf> {
        let project_dirs = directories::ProjectDirs::from("com", "ceoclaw", "CEOClaw")
            .ok_or_else(|| AppError::Config("Failed to get project directories".to_string()))?;

        let skills_dir = project_dirs.data_dir().join("skills");
        Ok(skills_dir.join(skill_id))
    }

    /// Get the permission checker
    pub async fn permission_checker(&self) -> Arc<Mutex<PermissionChecker>> {
        self.permission_checker.clone()
    }

    /// Get the configuration
    pub fn config(&self) -> &WasmHostConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_wasm_host_registration() {
        // Create a minimal valid WASM module
        let wasm_bytes = vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00];

        let manifest = SkillManifest::new(
            "com.example.test".to_string(),
            "Test Skill".to_string(),
            "1.0.0".to_string(),
            "A test skill".to_string(),
            "Test Author".to_string(),
        );

        let module = WasmModule::new(wasm_bytes, manifest).unwrap();
        let permissions = PermissionSet::new();

        // This test requires a database
        // let db = Arc::new(Database::new(&PathBuf::from("/tmp/test")).unwrap());
        // let host = WasmHost::default(db);
        // host.register_skill(module, permissions).await.unwrap();
    }
}
