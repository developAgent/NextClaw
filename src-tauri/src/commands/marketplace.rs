//! Skill marketplace commands
//! Provides Tauri commands for integrating with the ClawHub skill marketplace

use crate::db::Database;
use crate::utils::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Arc;
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

    /// Search skills in marketplace
    pub async fn search(&self, query: &str, limit: Option<u32>) -> Result<Vec<MarketplaceSkill>> {
        debug!("Searching marketplace for: {}", query);

        // Simulate marketplace search for now
        // In production, this would call the actual ClawHub API
        let skills = vec![
            MarketplaceSkill {
                slug: "find-skills".to_string(),
                name: "Find Skills".to_string(),
                version: "1.0.0".to_string(),
                description: "Search and discover AI skills from various sources".to_string(),
                author: "ClawHub".to_string(),
                icon: Some("🔍".to_string()),
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
                installed: false,
                installed_at: None,
                installed_path: None,
            },
        ];

        // Filter by query
        let filtered: Vec<MarketplaceSkill> = skills
            .into_iter()
            .filter(|skill| {
                let q = query.to_lowercase();
                skill.name.to_lowercase().contains(&q)
                    || skill.description.to_lowercase().contains(&q)
                    || skill.slug.to_lowercase().contains(&q)
            })
            .take(limit.unwrap_or(20) as usize)
            .collect();

        info!("Found {} skills matching '{}'", filtered.len(), query);
        Ok(filtered)
    }

    /// Get skill details
    pub async fn get_skill_details(&self, slug: &str) -> Result<MarketplaceSkill> {
        debug!("Getting skill details for: {}", slug);

        // Simulate getting skill details
        Ok(MarketplaceSkill {
            slug: slug.to_string(),
            name: "Sample Skill".to_string(),
            version: "1.0.0".to_string(),
            description: "A sample skill description".to_string(),
            author: "Unknown".to_string(),
            icon: Some("📦".to_string()),
            installed: false,
            installed_at: None,
            installed_path: None,
        })
    }

    /// Install a skill
    pub async fn install(&self, slug: &str, db: Arc<Database>) -> Result<()> {
        info!("Installing skill: {}", slug);

        let conn = db.conn();
        let conn_guard = conn.blocking_lock();
        let now = chrono::Utc::now().to_rfc3339();

        // Get skill details
        let skill = self.get_skill_details(slug).await?;

        // Insert into marketplace table
        conn_guard.execute(
            r#"
            INSERT OR REPLACE INTO skill_marketplace (slug, name, version, description, author, icon, installed, installed_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            rusqlite::params![
                &skill.slug,
                &skill.name,
                &skill.version,
                &skill.description,
                &skill.author,
                &skill.icon,
                1, // installed
                &now,
            ],
        ).map_err(|e| AppError::Database(format!("Failed to record installed skill: {}", e)))?;

        info!("Skill {} installed successfully", slug);
        Ok(())
    }

    /// Uninstall a skill
    pub async fn uninstall(&self, slug: &str, db: Arc<Database>) -> Result<()> {
        info!("Uninstalling skill: {}", slug);

        let conn = db.conn();
        let conn_guard = conn.blocking_lock();

        conn_guard.execute(
            "DELETE FROM skill_marketplace WHERE slug = ?1",
            [&slug],
        ).map_err(|e| AppError::Database(format!("Failed to uninstall skill: {}", e)))?;

        info!("Skill {} uninstalled successfully", slug);
        Ok(())
    }

    /// List all installed skills
    pub async fn list_installed(&self, db: Arc<Database>) -> Result<Vec<MarketplaceSkill>> {
        debug!("Listing installed skills");

        let conn = db.conn();
        let conn_guard = conn.blocking_lock();

        let mut stmt = conn_guard
            .prepare("SELECT * FROM skill_marketplace WHERE installed = 1")
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
                    installed: row.get::<_, i32>(6)? != 0,
                    installed_at: row.get(7)?,
                    installed_path: row.get(8)?,
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
    manager: State<'_, Arc<MarketplaceManager>>,
) -> Result<Vec<MarketplaceSkill>> {
    manager.search(&request.query, request.limit).await
}

/// Get skill details
#[tauri::command]
pub async fn get_skill_details(
    slug: String,
    manager: State<'_, Arc<MarketplaceManager>>,
) -> Result<MarketplaceSkill> {
    manager.get_skill_details(&slug).await
}

/// Install skill
#[tauri::command]
pub async fn install_skill(
    slug: String,
    db: State<'_, Arc<Database>>,
    manager: State<'_, Arc<MarketplaceManager>>,
) -> Result<()> {
    manager.install(&slug, db.inner().clone()).await
}

/// Uninstall skill
#[tauri::command]
pub async fn uninstall_skill(
    slug: String,
    db: State<'_, Arc<Database>>,
    manager: State<'_, Arc<MarketplaceManager>>,
) -> Result<()> {
    manager.uninstall(&slug, db.inner().clone()).await
}

/// List installed skills
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

    #[tokio::test]
    async fn test_marketplace_search() {
        let manager = MarketplaceManager::new();
        let results = manager.search("search", Some(5)).await.unwrap();
        assert!(!results.is_empty());
    }
}