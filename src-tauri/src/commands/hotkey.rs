use crate::db::Database;
use crate::utils::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;
use tracing::info;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hotkey {
    pub id: String,
    pub action: String,
    pub key_combination: String,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Get all hotkeys
#[tauri::command]
pub async fn get_all_hotkeys(db: State<'_, Arc<Database>>) -> Result<Vec<Hotkey>> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();

    let mut stmt = conn_guard
        .prepare("SELECT id, action, key_combination, enabled, created_at, updated_at FROM hotkeys")
        .map_err(|e| AppError::Database(e.to_string()))?;

    let hotkeys = stmt.query_map([], |row| {
        Ok(Hotkey {
            id: row.get(0)?,
            action: row.get(1)?,
            key_combination: row.get(2)?,
            enabled: row.get(3)?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }).map_err(|e| AppError::Database(e.to_string()))?
    .collect::<std::result::Result<Vec<_>, _>>()
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(hotkeys)
}

/// Add a new hotkey
#[tauri::command]
pub async fn add_hotkey(
    action: String,
    key_combination: String,
    db: State<'_, Arc<Database>>,
) -> Result<Hotkey> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    conn_guard.execute(
        "INSERT INTO hotkeys (id, action, key_combination, enabled, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![&id, &action, &key_combination, true, now, now],
    ).map_err(|e| AppError::Database(e.to_string()))?;

    info!("Added hotkey: {} -> {}", key_combination, action);

    Ok(Hotkey {
        id,
        action,
        key_combination,
        enabled: true,
        created_at: now,
        updated_at: now,
    })
}

/// Update an existing hotkey
#[tauri::command]
pub async fn update_hotkey(
    id: String,
    key_combination: String,
    db: State<'_, Arc<Database>>,
) -> Result<()> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();

    conn_guard.execute(
        "UPDATE hotkeys SET key_combination = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![key_combination, chrono::Utc::now().timestamp(), id],
    ).map_err(|e| AppError::Database(e.to_string()))?;

    info!("Updated hotkey: {}", id);
    Ok(())
}

/// Delete a hotkey
#[tauri::command]
pub async fn delete_hotkey(id: String, db: State<'_, Arc<Database>>) -> Result<()> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();

    conn_guard.execute("DELETE FROM hotkeys WHERE id = ?1", [&id])
        .map_err(|e| AppError::Database(e.to_string()))?;

    info!("Deleted hotkey: {}", id);
    Ok(())
}

/// Register all hotkeys (placeholder - needs platform-specific implementation)
#[tauri::command]
pub async fn register_hotkeys(db: State<'_, Arc<Database>>) -> Result<()> {
    let hotkeys = get_all_hotkeys(db).await?;

    for hotkey in hotkeys {
        if hotkey.enabled {
            info!("Would register hotkey: {} -> {}", hotkey.key_combination, hotkey.action);
            // TODO: Platform-specific hotkey registration
        }
    }

    Ok(())
}