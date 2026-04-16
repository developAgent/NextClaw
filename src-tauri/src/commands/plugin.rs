use crate::db::Database;
use crate::utils::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub enabled: bool,
    pub config: Option<String>,
    pub installed_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
struct PluginInstallMetadata {
    name: Option<String>,
    version: Option<String>,
    author: Option<String>,
    description: Option<String>,
    config: Option<String>,
    enabled: Option<bool>,
}

fn plugin_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Plugin> {
    Ok(Plugin {
        id: row.get(0)?,
        name: row.get(1)?,
        version: row.get(2)?,
        author: row.get(3)?,
        description: row.get(4)?,
        enabled: row.get(5)?,
        config: row.get(6)?,
        installed_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn parse_install_request(url: &str) -> Result<(String, PluginInstallMetadata)> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err(AppError::Validation(
            "Plugin source cannot be empty".to_string(),
        ));
    }

    if let Ok(metadata) = serde_json::from_str::<PluginInstallMetadata>(trimmed) {
        let name = metadata
            .name
            .clone()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| AppError::Validation("Plugin name is required".to_string()))?;
        return Ok((name, metadata));
    }

    let normalized = trimmed.replace('\\', "/");
    let leaf = normalized
        .rsplit('/')
        .find(|segment| !segment.trim().is_empty())
        .unwrap_or(trimmed)
        .trim();
    let without_extension = leaf.strip_suffix(".json").unwrap_or(leaf);
    let name = without_extension.trim();

    if name.is_empty() {
        return Err(AppError::Validation(
            "Failed to derive plugin name from source".to_string(),
        ));
    }

    Ok((
        name.to_string(),
        PluginInstallMetadata {
            name: Some(name.to_string()),
            version: None,
            author: None,
            description: None,
            config: None,
            enabled: None,
        },
    ))
}

fn build_plugin(url: &str) -> Result<Plugin> {
    let (name, metadata) = parse_install_request(url)?;
    let now = chrono::Utc::now().timestamp_millis();

    Ok(Plugin {
        id: Uuid::new_v4().to_string(),
        name,
        version: metadata.version.unwrap_or_else(|| "1.0.0".to_string()),
        author: metadata.author,
        description: metadata
            .description
            .or_else(|| Some(url.trim().to_string())),
        enabled: metadata.enabled.unwrap_or(true),
        config: metadata.config,
        installed_at: now,
        updated_at: now,
    })
}

/// Get all plugins
#[tauri::command]
pub async fn get_all_plugins(db: State<'_, Arc<Database>>) -> Result<Vec<Plugin>> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();

    let mut stmt = conn_guard
        .prepare(
            "SELECT id, name, version, author, description, enabled, config, installed_at, updated_at FROM plugins ORDER BY name COLLATE NOCASE ASC",
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    let plugins = stmt
        .query_map([], plugin_from_row)
        .map_err(|e| AppError::Database(e.to_string()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(plugins)
}

/// Get a specific plugin
#[tauri::command]
pub async fn get_plugin(id: String, db: State<'_, Arc<Database>>) -> Result<Option<Plugin>> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();

    let mut stmt = conn_guard
        .prepare(
            "SELECT id, name, version, author, description, enabled, config, installed_at, updated_at FROM plugins WHERE id = ?1",
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    let plugin = stmt.query_row([id], plugin_from_row).ok();

    Ok(plugin)
}

/// Install a plugin
#[tauri::command]
pub async fn install_plugin(url: String, db: State<'_, Arc<Database>>) -> Result<Plugin> {
    info!("Installing plugin from: {}", url);

    let plugin = build_plugin(&url)?;
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();

    conn_guard
        .execute(
            r#"
            INSERT INTO plugins (id, name, version, author, description, enabled, config, installed_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            rusqlite::params![
                &plugin.id,
                &plugin.name,
                &plugin.version,
                &plugin.author,
                &plugin.description,
                plugin.enabled,
                &plugin.config,
                plugin.installed_at,
                plugin.updated_at,
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(plugin)
}

/// Enable a plugin
#[tauri::command]
pub async fn enable_plugin(id: String, db: State<'_, Arc<Database>>) -> Result<()> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();

    conn_guard
        .execute(
            "UPDATE plugins SET enabled = 1, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![chrono::Utc::now().timestamp_millis(), id],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    info!("Enabled plugin: {}", id);
    Ok(())
}

/// Disable a plugin
#[tauri::command]
pub async fn disable_plugin(id: String, db: State<'_, Arc<Database>>) -> Result<()> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();

    conn_guard
        .execute(
            "UPDATE plugins SET enabled = 0, updated_at = ?1 WHERE id = ?2",
            rusqlite::params![chrono::Utc::now().timestamp_millis(), id],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    info!("Disabled plugin: {}", id);
    Ok(())
}

/// Uninstall a plugin
#[tauri::command]
pub async fn uninstall_plugin(id: String, db: State<'_, Arc<Database>>) -> Result<()> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();

    conn_guard
        .execute("DELETE FROM plugins WHERE id = ?1", [&id])
        .map_err(|e| AppError::Database(e.to_string()))?;

    info!("Uninstalled plugin: {}", id);
    Ok(())
}
