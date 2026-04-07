use crate::db::Database;
use crate::utils::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;
use tracing::info;
use std::sync::Arc;

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

/// Get all plugins
#[tauri::command]
pub async fn get_all_plugins(db: State<'_, Arc<Database>>) -> Result<Vec<Plugin>> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();

    let mut stmt = conn_guard
        .prepare("SELECT id, name, version, author, description, enabled, config, installed_at, updated_at FROM plugins")
        .map_err(|e| AppError::Database(e.to_string()))?;

    let plugins = stmt.query_map([], |row| {
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
    }).map_err(|e| AppError::Database(e.to_string()))?
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
        .prepare("SELECT id, name, version, author, description, enabled, config, installed_at, updated_at FROM plugins WHERE id = ?1")
        .map_err(|e| AppError::Database(e.to_string()))?;

    let plugin = stmt.query_row([id], |row| {
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
    }).ok();

    Ok(plugin)
}

/// Install a plugin (placeholder)
#[tauri::command]
pub async fn install_plugin(url: String) -> Result<Plugin> {
    info!("Installing plugin from: {}", url);
    Err(AppError::Internal("Plugin marketplace coming soon".to_string()))
}

/// Enable a plugin
#[tauri::command]
pub async fn enable_plugin(id: String, db: State<'_, Arc<Database>>) -> Result<()> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();

    conn_guard.execute(
        "UPDATE plugins SET enabled = 1, updated_at = ?1 WHERE id = ?2",
        rusqlite::params![chrono::Utc::now().timestamp(), id],
    ).map_err(|e| AppError::Database(e.to_string()))?;

    info!("Enabled plugin: {}", id);
    Ok(())
}

/// Disable a plugin
#[tauri::command]
pub async fn disable_plugin(id: String, db: State<'_, Arc<Database>>) -> Result<()> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();

    conn_guard.execute(
        "UPDATE plugins SET enabled = 0, updated_at = ?1 WHERE id = ?2",
        rusqlite::params![chrono::Utc::now().timestamp(), id],
    ).map_err(|e| AppError::Database(e.to_string()))?;

    info!("Disabled plugin: {}", id);
    Ok(())
}

/// Uninstall a plugin
#[tauri::command]
pub async fn uninstall_plugin(id: String, db: State<'_, Arc<Database>>) -> Result<()> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();

    conn_guard.execute("DELETE FROM plugins WHERE id = ?1", [&id])
        .map_err(|e| AppError::Database(e.to_string()))?;

    info!("Uninstalled plugin: {}", id);
    Ok(())
}