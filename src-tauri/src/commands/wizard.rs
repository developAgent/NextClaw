//! 向导命令
//! 提供首次运行引导的 Tauri 命令

use crate::db::Database;
use crate::security::KeyStorage;
use crate::utils::error::{AppError, Result};
use rusqlite::params;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;

/// 向导状态
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WizardState {
    pub current_step: i32,
    pub completed: bool,
    pub language: String,
    pub ai_provider: Option<String>,
    pub api_key: Option<String>,
    pub api_key_provider: Option<String>,
    pub workspace_name: String,
    pub enabled_features: Vec<String>,
}

/// 获取向导状态
#[tauri::command]
pub async fn wizard_get_state(db: State<'_, Arc<Database>>) -> Result<WizardState> {
    let conn = db.conn();
    let conn = conn.lock().await;

    let mut stmt = conn.prepare(
        r#"
        SELECT current_step, completed, language, ai_provider, api_key_provider, workspace_name, enabled_features
        FROM setup_wizard_state
        WHERE id = 'wizard_state'
        "#,
    )?;

    let state = stmt.query_row([], |row| {
        let features_json: String = row.get(6)?;
        let features: Vec<String> = serde_json::from_str(&features_json).unwrap_or_default();

        Ok(WizardState {
            current_step: row.get(0)?,
            completed: row.get(1)?,
            language: row.get(2)?,
            ai_provider: row.get(3)?,
            api_key: None, // 不返回加密的 key
            api_key_provider: row.get(4)?,
            workspace_name: row.get(5)?,
            enabled_features: features,
        })
    }).map_err(|_| AppError::Database("Wizard state not found".to_string()))?;

    Ok(state)
}

/// 保存向导状态
#[tauri::command]
pub async fn wizard_save_state(
    db: State<'_, Arc<Database>>,
    state: String,
) -> Result<()> {
    let state: WizardState = serde_json::from_str(&state)
        .map_err(|e| AppError::Validation(format!("Invalid wizard state: {}", e)))?;

    let conn = db.conn();
    let conn = conn.lock().await;

    let features_json = serde_json::to_string(&state.enabled_features)
        .map_err(|e| AppError::Internal(format!("Failed to serialize features: {}", e)))?;

    conn.execute(
        r#"
        UPDATE setup_wizard_state
        SET current_step = ?1,
            language = ?2,
            ai_provider = ?3,
            api_key_provider = ?4,
            workspace_name = ?5,
            enabled_features = ?6,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = 'wizard_state'
        "#,
        params![
            state.current_step,
            state.language,
            state.ai_provider,
            state.api_key_provider,
            state.workspace_name,
            features_json,
        ],
    ).map_err(|e| AppError::Database(format!("Failed to save wizard state: {}", e)))?;

    Ok(())
}

/// 完成向导
#[tauri::command]
pub async fn wizard_complete(db: State<'_, Arc<Database>>) -> Result<()> {
    let conn = db.conn();
    let conn = conn.lock().await;

    conn.execute(
        r#"
        UPDATE setup_wizard_state
        SET completed = TRUE, current_step = 0, updated_at = CURRENT_TIMESTAMP
        WHERE id = 'wizard_state'
        "#,
        [],
    ).map_err(|e| AppError::Database(format!("Failed to complete wizard: {}", e)))?;

    tracing::info!("Setup wizard completed");
    Ok(())
}

/// 保存 API Key（加密存储）
#[tauri::command]
pub async fn wizard_save_api_key(
    db: State<'_, Arc<Database>>,
    provider: String,
    api_key: String,
) -> Result<()> {
    let storage = KeyStorage::new(db.inner().clone());
    storage.store(&provider, SecretString::new(api_key))?;

    // 更新向导状态中的 provider
    let conn = db.conn();
    let conn = conn.lock().await;
    conn.execute(
        r#"
        UPDATE setup_wizard_state
        SET ai_provider = ?1, api_key_provider = ?1, updated_at = CURRENT_TIMESTAMP
        WHERE id = 'wizard_state'
        "#,
        params![provider],
    ).map_err(|e| AppError::Database(format!("Failed to update provider: {}", e)))?;

    Ok(())
}

/// 重置向导状态（用于测试或重新引导）
#[tauri::command]
pub async fn wizard_reset(db: State<'_, Arc<Database>>) -> Result<()> {
    let conn = db.conn();
    let conn = conn.lock().await;

    conn.execute(
        r#"
        UPDATE setup_wizard_state
        SET current_step = 1, completed = FALSE, updated_at = CURRENT_TIMESTAMP
        WHERE id = 'wizard_state'
        "#,
        [],
    ).map_err(|e| AppError::Database(format!("Failed to reset wizard: {}", e)))?;

    Ok(())
}
