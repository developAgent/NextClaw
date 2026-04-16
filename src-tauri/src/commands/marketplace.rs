//! Skill marketplace commands
//! Provides Tauri commands for integrating with the ClawHub skill marketplace

use crate::db::Database;
use crate::skills::host::WasmHost;
use crate::skills::manifest::SkillManifest;
use crate::skills::permissions::PermissionSet;
use crate::skills::runtime::WasmModule;
use crate::utils::error::{AppError, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::State;
use tracing::{debug, info};

/// Skill information from marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSkill {
    pub slug: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub icon: Option<String>,
    pub available: bool,
    pub installed: bool,
    pub installed_at: Option<String>,
    pub installed_path: Option<String>,
}

/// Search request for marketplace
#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<u32>,
}

/// Install request
#[derive(Debug, Deserialize)]
pub struct InstallRequest {
    pub slug: String,
    pub version: Option<String>,
}

/// Uninstall request
#[derive(Debug, Deserialize)]
pub struct UninstallRequest {
    pub slug: String,
}

#[derive(Debug, Clone)]
struct MarketplaceArtifact {
    manifest: SkillManifest,
    wasm_bytes: Vec<u8>,
    source_path: PathBuf,
}

#[derive(Debug, Clone)]
struct MarketplaceInstallRecord {
    slug: String,
    name: String,
    version: String,
    description: String,
    author: String,
    icon: Option<String>,
    available: bool,
    installed: bool,
    installed_at: Option<String>,
    installed_path: Option<String>,
    skill_id: Option<String>,
}

/// Marketplace manager
pub struct MarketplaceManager {
    base_url: String,
}

impl MarketplaceManager {
    pub fn new() -> Self {
        Self {
            base_url: "https://api.clawhub.ai".to_string(),
        }
    }

    fn builtin_skills(&self) -> Vec<MarketplaceSkill> {
        let _ = &self.base_url;

        vec![
            MarketplaceSkill {
                slug: "find-skills".to_string(),
                name: "Find Skills".to_string(),
                version: "1.0.0".to_string(),
                description: "Search and discover AI skills from various sources".to_string(),
                author: "ClawHub".to_string(),
                icon: Some("🔍".to_string()),
                available: false,
                installed: false,
                installed_at: None,
                installed_path: None,
            },
            MarketplaceSkill {
                slug: "self-improving-agent".to_string(),
                name: "Self-Improving Agent".to_string(),
                version: "1.0.0".to_string(),
                description: "An AI agent that learns and improves from feedback".to_string(),
                author: "ClawHub".to_string(),
                icon: Some("🧠".to_string()),
                available: false,
                installed: false,
                installed_at: None,
                installed_path: None,
            },
            MarketplaceSkill {
                slug: "brave-web-search".to_string(),
                name: "Brave Web Search".to_string(),
                version: "1.0.0".to_string(),
                description: "Web search using Brave Search API".to_string(),
                author: "ClawHub".to_string(),
                icon: Some("🌐".to_string()),
                available: false,
                installed: false,
                installed_at: None,
                installed_path: None,
            },
            MarketplaceSkill {
                slug: "tavily-search".to_string(),
                name: "Tavily Search".to_string(),
                version: "1.0.0".to_string(),
                description: "Web search using Tavily Search API".to_string(),
                author: "ClawHub".to_string(),
                icon: Some("🔎".to_string()),
                available: false,
                installed: false,
                installed_at: None,
                installed_path: None,
            },
        ]
    }

    fn installed_skill_map(
        &self,
        db: Arc<Database>,
    ) -> Result<HashMap<String, MarketplaceInstallRecord>> {
        let conn = db.conn();
        let conn_guard = conn.blocking_lock();

        let mut stmt = conn_guard
            .prepare(
                "SELECT slug, name, version, description, author, icon, available, installed, installed_at, installed_path, skill_id FROM skill_marketplace",
            )
            .map_err(|e| {
                AppError::Database(format!("Failed to query marketplace skills: {}", e))
            })?;

        let rows = stmt
            .query_map([], |row| {
                Ok(MarketplaceInstallRecord {
                    slug: row.get(0)?,
                    name: row.get(1)?,
                    version: row.get(2)?,
                    description: row.get(3)?,
                    author: row.get(4)?,
                    icon: row.get(5)?,
                    available: row.get::<_, i32>(6)? != 0,
                    installed: row.get::<_, i32>(7)? != 0,
                    installed_at: row.get(8)?,
                    installed_path: row.get(9)?,
                    skill_id: row.get(10)?,
                })
            })
            .map_err(|e| AppError::Database(format!("Failed to map marketplace skills: {}", e)))?;

        let installed = rows
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| {
                AppError::Database(format!("Failed to collect marketplace skills: {}", e))
            })?;

        Ok(installed
            .into_iter()
            .map(|skill| (skill.slug.clone(), skill))
            .collect())
    }

    fn project_dirs(&self) -> Result<ProjectDirs> {
        ProjectDirs::from("com", "ceoclaw", "CEOClaw")
            .ok_or_else(|| AppError::Config("Failed to get project directories".to_string()))
    }

    fn marketplace_artifact_base_dir(&self) -> Result<PathBuf> {
        Ok(self.project_dirs()?.data_dir().join("marketplace"))
    }

    fn artifact_candidate_dirs(&self, slug: &str) -> Result<Vec<PathBuf>> {
        let mut candidates = vec![self.marketplace_artifact_base_dir()?.join(slug)];

        if slug == "find-skills" {
            candidates.push(PathBuf::from("sdk/examples/hello-world"));
        }

        Ok(candidates)
    }

    fn artifact_exists_in_dir(&self, dir: &Path) -> bool {
        dir.join("manifest.json").exists() && dir.join("skill.wasm").exists()
    }

    fn artifact_available(&self, slug: &str) -> bool {
        self.artifact_candidate_dirs(slug)
            .map(|candidates| {
                candidates
                    .iter()
                    .any(|candidate| self.artifact_exists_in_dir(candidate))
            })
            .unwrap_or(false)
    }

    fn load_artifact_from_dir(&self, slug: &str, dir: &Path) -> Result<MarketplaceArtifact> {
        let manifest_path = dir.join("manifest.json");
        let wasm_path = dir.join("skill.wasm");

        if !manifest_path.exists() || !wasm_path.exists() {
            return Err(AppError::Validation(format!(
                "Marketplace artifact missing files for '{}': expected manifest.json and skill.wasm in {}",
                slug,
                dir.display()
            )));
        }

        let manifest_json = std::fs::read_to_string(&manifest_path).map_err(|e| {
            AppError::Io(format!("Failed to read {}: {}", manifest_path.display(), e))
        })?;
        let manifest: SkillManifest = serde_json::from_str(&manifest_json).map_err(|e| {
            AppError::Validation(format!(
                "Invalid marketplace manifest at {}: {}",
                manifest_path.display(),
                e
            ))
        })?;
        let wasm_bytes = std::fs::read(&wasm_path)
            .map_err(|e| AppError::Io(format!("Failed to read {}: {}", wasm_path.display(), e)))?;

        Ok(MarketplaceArtifact {
            manifest,
            wasm_bytes,
            source_path: dir.to_path_buf(),
        })
    }

    fn resolve_artifact(&self, slug: &str) -> Result<MarketplaceArtifact> {
        let candidates = self.artifact_candidate_dirs(slug)?;

        for candidate in &candidates {
            if self.artifact_exists_in_dir(candidate) {
                return self.load_artifact_from_dir(slug, candidate);
            }
        }

        let searched_paths = candidates
            .iter()
            .map(|candidate| candidate.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");

        Err(AppError::Validation(format!(
            "No marketplace artifact found for '{}'. Expected manifest.json and skill.wasm in one of: {}",
            slug, searched_paths
        )))
    }

    fn merge_skill_state(
        &self,
        builtin: MarketplaceSkill,
        installed: Option<&MarketplaceInstallRecord>,
    ) -> MarketplaceSkill {
        let available = self.artifact_available(&builtin.slug);

        if let Some(installed) = installed {
            MarketplaceSkill {
                slug: builtin.slug,
                name: installed.name.clone(),
                version: installed.version.clone(),
                description: installed.description.clone(),
                author: installed.author.clone(),
                icon: installed.icon.clone().or(builtin.icon),
                available: available || installed.available,
                installed: installed.installed,
                installed_at: installed.installed_at.clone(),
                installed_path: installed.installed_path.clone(),
            }
        } else {
            MarketplaceSkill {
                available,
                ..builtin
            }
        }
    }

    fn store_install_record(
        &self,
        db: Arc<Database>,
        skill: &MarketplaceSkill,
        install_path: &Path,
        skill_id: &str,
    ) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let conn = db.conn();
        let conn_guard = conn.blocking_lock();

        conn_guard
            .execute(
                r#"
                INSERT OR REPLACE INTO skill_marketplace (slug, name, version, description, author, icon, available, installed, installed_at, installed_path, skill_id)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                "#,
                rusqlite::params![
                    &skill.slug,
                    &skill.name,
                    &skill.version,
                    &skill.description,
                    &skill.author,
                    &skill.icon,
                    1,
                    1,
                    &now,
                    install_path.to_string_lossy().to_string(),
                    skill_id,
                ],
            )
            .map_err(|e| AppError::Database(format!("Failed to record installed skill: {}", e)))?;

        Ok(())
    }

    fn clear_install_record(&self, db: Arc<Database>, slug: &str) -> Result<()> {
        let conn = db.conn();
        let conn_guard = conn.blocking_lock();
        conn_guard
            .execute(
                r#"
                UPDATE skill_marketplace
                SET installed = 0,
                    installed_at = NULL,
                    installed_path = NULL,
                    skill_id = NULL,
                    available = ?2
                WHERE slug = ?1
                "#,
                rusqlite::params![&slug, i32::from(self.artifact_available(slug))],
            )
            .map_err(|e| {
                AppError::Database(format!("Failed to update skill uninstall state: {}", e))
            })?;
        Ok(())
    }

    /// Search skills in marketplace
    pub async fn search(
        &self,
        query: &str,
        limit: Option<u32>,
        db: Arc<Database>,
    ) -> Result<Vec<MarketplaceSkill>> {
        debug!("Searching marketplace for: {}", query);

        let installed_map = self.installed_skill_map(db)?;
        let q = query.to_lowercase();

        let filtered: Vec<MarketplaceSkill> = self
            .builtin_skills()
            .into_iter()
            .filter(|skill| {
                skill.name.to_lowercase().contains(&q)
                    || skill.description.to_lowercase().contains(&q)
                    || skill.slug.to_lowercase().contains(&q)
            })
            .map(|skill| self.merge_skill_state(skill, installed_map.get(&skill.slug)))
            .take(limit.unwrap_or(20) as usize)
            .collect();

        info!("Found {} skills matching '{}'", filtered.len(), query);
        Ok(filtered)
    }

    /// Get skill details
    pub async fn get_skill_details(
        &self,
        slug: &str,
        db: Arc<Database>,
    ) -> Result<MarketplaceSkill> {
        debug!("Getting skill details for: {}", slug);

        let installed_map = self.installed_skill_map(db)?;

        self.builtin_skills()
            .into_iter()
            .find(|skill| skill.slug == slug)
            .map(|skill| self.merge_skill_state(skill, installed_map.get(slug)))
            .ok_or_else(|| AppError::Validation(format!("Marketplace skill not found: {}", slug)))
    }

    /// Install a skill
    pub async fn install(&self, slug: &str, db: Arc<Database>, host: &WasmHost) -> Result<()> {
        info!("Installing skill: {}", slug);

        let skill = self.get_skill_details(slug, db.clone()).await?;
        let artifact = self.resolve_artifact(slug)?;
        let permissions = PermissionSet::new();
        let module =
            WasmModule::new(artifact.wasm_bytes, artifact.manifest.clone()).map_err(|e| {
                AppError::Internal(format!("Failed to create marketplace module: {}", e))
            })?;

        let skill_id = artifact.manifest.id.clone();
        let install_path = host.skill_storage_path(&skill_id)?;
        host.register_skill(module, permissions).await?;
        self.store_install_record(db, &skill, &install_path, &skill_id)?;

        info!(
            "Skill {} installed successfully from {}",
            slug,
            artifact.source_path.display()
        );
        Ok(())
    }

    /// Uninstall a skill
    pub async fn uninstall(&self, slug: &str, db: Arc<Database>, host: &WasmHost) -> Result<()> {
        info!("Uninstalling skill: {}", slug);

        let installed_map = self.installed_skill_map(db.clone())?;
        let record = installed_map.get(slug).ok_or_else(|| {
            AppError::Validation(format!("Marketplace skill not found: {}", slug))
        })?;

        let skill_id = record.skill_id.clone().ok_or_else(|| {
            AppError::Validation(format!("Marketplace skill '{}' is not installed", slug))
        })?;

        if host.is_skill_registered(&skill_id).await {
            host.unregister_skill(&skill_id).await?;
        }

        self.clear_install_record(db, slug)?;

        info!("Skill {} uninstalled successfully", slug);
        Ok(())
    }

    /// List all installed skills
    pub async fn list_installed(&self, db: Arc<Database>) -> Result<Vec<MarketplaceSkill>> {
        debug!("Listing installed skills");

        let conn = db.conn();
        let conn_guard = conn.blocking_lock();

        let mut stmt = conn_guard
            .prepare(
                "SELECT slug, name, version, description, author, icon, available, installed, installed_at, installed_path FROM skill_marketplace WHERE installed = 1 ORDER BY name COLLATE NOCASE ASC",
            )
            .map_err(|e| AppError::Database(format!("Failed to list installed skills: {}", e)))?;

        let skills = stmt
            .query_map([], |row| {
                Ok(MarketplaceSkill {
                    slug: row.get(0)?,
                    name: row.get(1)?,
                    version: row.get(2)?,
                    description: row.get(3)?,
                    author: row.get(4)?,
                    icon: row.get(5)?,
                    available: row.get::<_, i32>(6)? != 0,
                    installed: row.get::<_, i32>(7)? != 0,
                    installed_at: row.get(8)?,
                    installed_path: row.get(9)?,
                })
            })
            .map_err(|e| AppError::Database(format!("Failed to map skills: {}", e)))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| AppError::Database(format!("Failed to collect skills: {}", e)))?;

        Ok(skills)
    }
}

impl Default for MarketplaceManager {
    fn default() -> Self {
        Self::new()
    }
}

// Tauri commands

/// Search marketplace skills
#[tauri::command]
pub async fn search_marketplace(
    request: SearchRequest,
    db: State<'_, Arc<Database>>,
    manager: State<'_, Arc<MarketplaceManager>>,
) -> Result<Vec<MarketplaceSkill>> {
    manager
        .search(&request.query, request.limit, db.inner().clone())
        .await
}

/// Get skill details
#[tauri::command]
pub async fn get_skill_details(
    slug: String,
    db: State<'_, Arc<Database>>,
    manager: State<'_, Arc<MarketplaceManager>>,
) -> Result<MarketplaceSkill> {
    manager.get_skill_details(&slug, db.inner().clone()).await
}

/// Install skill
#[tauri::command]
pub async fn install_skill(
    slug: String,
    db: State<'_, Arc<Database>>,
    manager: State<'_, Arc<MarketplaceManager>>,
    host: State<'_, Arc<WasmHost>>,
) -> Result<()> {
    manager
        .install(&slug, db.inner().clone(), host.inner().as_ref())
        .await
}

/// Uninstall skill
#[tauri::command]
pub async fn uninstall_skill(
    slug: String,
    db: State<'_, Arc<Database>>,
    manager: State<'_, Arc<MarketplaceManager>>,
    host: State<'_, Arc<WasmHost>>,
) -> Result<()> {
    manager
        .uninstall(&slug, db.inner().clone(), host.inner().as_ref())
        .await
}

/// List all installed skills
#[tauri::command]
pub async fn list_installed_skills(
    db: State<'_, Arc<Database>>,
    manager: State<'_, Arc<MarketplaceManager>>,
) -> Result<Vec<MarketplaceSkill>> {
    manager.list_installed(db.inner().clone()).await
}

/// Get skill categories
#[tauri::command]
pub async fn get_skill_categories() -> Result<Vec<String>> {
    Ok(vec![
        "Productivity".to_string(),
        "Research".to_string(),
        "Communication".to_string(),
        "Development".to_string(),
        "Data Processing".to_string(),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_availability_without_wasm() {
        let manager = MarketplaceManager::new();
        assert!(!manager.artifact_available("find-skills"));
    }
}
