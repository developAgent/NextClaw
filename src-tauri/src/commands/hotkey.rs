use crate::db::Database;
use crate::hotkeys::{HotkeyRegistry, RegisteredHotkey};
use crate::utils::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Hotkey {
    pub id: String,
    pub action: String,
    pub key_combination: String,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize)]
pub struct RegisteredHotkeysResponse {
    pub registered: Vec<RegisteredHotkey>,
}

fn validate_action(action: &str) -> Result<()> {
    let normalized = action.trim();
    if normalized.is_empty() {
        return Err(AppError::Validation(
            "Hotkey action is required".to_string(),
        ));
    }

    Ok(())
}

fn normalize_key_combination(key_combination: &str) -> Result<String> {
    let normalized = key_combination
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| part.to_ascii_uppercase())
        .collect::<Vec<_>>()
        .join("+");

    if normalized.is_empty() {
        return Err(AppError::Validation(
            "Hotkey key combination is required".to_string(),
        ));
    }

    Ok(normalized)
}

fn read_hotkeys(db: &Database) -> Result<Vec<Hotkey>> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();

    let mut stmt = conn_guard
        .prepare(
            "SELECT id, action, key_combination, enabled, created_at, updated_at FROM hotkeys ORDER BY created_at DESC",
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    let hotkeys = stmt
        .query_map([], |row| {
            Ok(Hotkey {
                id: row.get(0)?,
                action: row.get(1)?,
                key_combination: row.get(2)?,
                enabled: row.get::<_, i64>(3)? != 0,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| AppError::Database(e.to_string()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(hotkeys)
}

async fn sync_registry(db: &Database, registry: &HotkeyRegistry) -> Result<Vec<RegisteredHotkey>> {
    registry.load_enabled_from_db(db).await
}

/// Get all hotkeys
#[tauri::command]
pub async fn get_all_hotkeys(db: State<'_, Arc<Database>>) -> Result<Vec<Hotkey>> {
    read_hotkeys(db.inner().as_ref())
}

/// Add a new hotkey
#[tauri::command]
pub async fn add_hotkey(
    action: String,
    key_combination: String,
    db: State<'_, Arc<Database>>,
    registry: State<'_, Arc<HotkeyRegistry>>,
) -> Result<Hotkey> {
    validate_action(&action)?;
    let normalized_key_combination = normalize_key_combination(&key_combination)?;

    let conn = db.conn();
    let conn_guard = conn.blocking_lock();

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().timestamp();

    conn_guard
        .execute(
            "INSERT INTO hotkeys (id, action, key_combination, enabled, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![&id, action.trim(), &normalized_key_combination, 1, now, now],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    drop(conn_guard);
    sync_registry(db.inner().as_ref(), registry.inner().as_ref()).await?;

    info!(
        "Added hotkey: {} -> {}",
        normalized_key_combination,
        action.trim()
    );

    Ok(Hotkey {
        id,
        action: action.trim().to_string(),
        key_combination: normalized_key_combination,
        enabled: true,
        created_at: now,
        updated_at: now,
    })
}

/// Update an existing hotkey
#[tauri::command]
pub async fn update_hotkey(
    id: String,
    action: Option<String>,
    key_combination: Option<String>,
    enabled: Option<bool>,
    db: State<'_, Arc<Database>>,
    registry: State<'_, Arc<HotkeyRegistry>>,
) -> Result<()> {
    if action.is_none() && key_combination.is_none() && enabled.is_none() {
        return Err(AppError::Validation(
            "No hotkey fields were provided for update".to_string(),
        ));
    }

    let conn = db.conn();
    let conn_guard = conn.blocking_lock();

    let existing = conn_guard
        .query_row(
            "SELECT action, key_combination, enabled FROM hotkeys WHERE id = ?1",
            [&id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)? != 0,
                ))
            },
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    let next_action = match action {
        Some(value) => {
            validate_action(&value)?;
            value.trim().to_string()
        }
        None => existing.0,
    };

    let next_key_combination = match key_combination {
        Some(value) => normalize_key_combination(&value)?,
        None => existing.1,
    };

    let next_enabled = enabled.unwrap_or(existing.2);

    let hotkey_id = id.clone();
    conn_guard
        .execute(
            "UPDATE hotkeys SET action = ?1, key_combination = ?2, enabled = ?3, updated_at = ?4 WHERE id = ?5",
            rusqlite::params![next_action, next_key_combination, if next_enabled { 1 } else { 0 }, chrono::Utc::now().timestamp(), hotkey_id],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    drop(conn_guard);
    sync_registry(db.inner().as_ref(), registry.inner().as_ref()).await?;

    info!("Updated hotkey: {}", id);
    Ok(())
}

/// Delete a hotkey
#[tauri::command]
pub async fn delete_hotkey(
    id: String,
    db: State<'_, Arc<Database>>,
    registry: State<'_, Arc<HotkeyRegistry>>,
) -> Result<()> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();

    conn_guard
        .execute("DELETE FROM hotkeys WHERE id = ?1", [&id])
        .map_err(|e| AppError::Database(e.to_string()))?;

    drop(conn_guard);
    registry.remove(&id).await;

    info!("Deleted hotkey: {}", id);
    Ok(())
}

/// Sync enabled hotkeys into the runtime registry
#[tauri::command]
pub async fn register_hotkeys(
    db: State<'_, Arc<Database>>,
    registry: State<'_, Arc<HotkeyRegistry>>,
) -> Result<RegisteredHotkeysResponse> {
    let registered = sync_registry(db.inner().as_ref(), registry.inner().as_ref()).await?;

    for hotkey in &registered {
        info!(
            "Registered hotkey: {} -> {}",
            hotkey.key_combination, hotkey.action
        );
    }

    Ok(RegisteredHotkeysResponse { registered })
}

/// Get runtime registered hotkeys
#[tauri::command]
pub async fn get_registered_hotkeys(
    registry: State<'_, Arc<HotkeyRegistry>>,
) -> Result<RegisteredHotkeysResponse> {
    Ok(RegisteredHotkeysResponse {
        registered: registry.list().await,
    })
}
