use crate::db::connection::Database;
use crate::utils::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::info;
use uuid::Uuid;

const CURRENT_WORKSPACE_KEY: &str = "current_workspace_id";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub is_current: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateWorkspaceInput {
    pub name: String,
    pub description: Option<String>,
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|item| {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn ensure_default_workspace(db: &Database) -> Result<()> {
    let existing_count = db.transaction(|conn| {
        conn.query_row("SELECT COUNT(*) FROM workspaces", [], |row| row.get::<_, i64>(0))
    })?;

    if existing_count > 0 {
        return Ok(());
    }

    let now = chrono::Utc::now().to_rfc3339();
    let workspace_id = Uuid::new_v4().to_string();

    db.transaction(|conn| {
        conn.execute(
            r#"
            INSERT INTO workspaces (id, name, description, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            rusqlite::params![workspace_id, "Default Workspace", Option::<String>::None, now, now],
        )?;

        conn.execute(
            r#"
            INSERT OR REPLACE INTO settings (key, value, type, updated_at)
            VALUES (?1, ?2, 'string', ?3)
            "#,
            rusqlite::params![CURRENT_WORKSPACE_KEY, workspace_id, now],
        )?;

        Ok(())
    })?;

    info!("Created default workspace");
    Ok(())
}

fn get_current_workspace_id(db: &Database) -> Result<Option<String>> {
    ensure_default_workspace(db)?;

    let workspace_id = db.transaction(|conn| {
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let result = stmt.query_row(rusqlite::params![CURRENT_WORKSPACE_KEY], |row| {
            row.get::<_, String>(0)
        });

        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error),
        }
    })?;

    Ok(workspace_id)
}

fn map_workspace(row: &rusqlite::Row<'_>, current_workspace_id: Option<&str>) -> rusqlite::Result<WorkspaceDto> {
    let id = row.get::<_, String>(0)?;
    Ok(WorkspaceDto {
        is_current: current_workspace_id.is_some_and(|current_id| current_id == id),
        id,
        name: row.get(1)?,
        description: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

#[tauri::command]
pub fn list_workspaces(db: State<'_, Database>) -> Result<Vec<WorkspaceDto>> {
    ensure_default_workspace(&db)?;
    let current_workspace_id = get_current_workspace_id(&db)?;

    db.transaction(|conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, description, created_at, updated_at
            FROM workspaces
            ORDER BY datetime(updated_at) DESC, name COLLATE NOCASE ASC
            "#,
        )?;

        let rows = stmt.query_map([], |row| map_workspace(row, current_workspace_id.as_deref()))?;
        rows.collect::<rusqlite::Result<Vec<_>>>()
    })
}

#[tauri::command]
pub fn get_current_workspace(db: State<'_, Database>) -> Result<Option<WorkspaceDto>> {
    ensure_default_workspace(&db)?;
    let current_workspace_id = get_current_workspace_id(&db)?;

    let Some(current_workspace_id) = current_workspace_id else {
        return Ok(None);
    };

    let workspace = db.transaction(|conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, description, created_at, updated_at
            FROM workspaces
            WHERE id = ?1
            "#,
        )?;

        let result = stmt.query_row(rusqlite::params![&current_workspace_id], |row| {
            map_workspace(row, Some(&current_workspace_id))
        });

        match result {
            Ok(item) => Ok(Some(item)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(error),
        }
    })?;

    Ok(workspace)
}

#[tauri::command]
pub fn create_workspace(input: CreateWorkspaceInput, db: State<'_, Database>) -> Result<WorkspaceDto> {
    let name = input.name.trim();
    if name.is_empty() {
        return Err(AppError::Validation("Workspace name is required".to_string()));
    }

    ensure_default_workspace(&db)?;

    let existing = db.transaction(|conn| {
        conn.query_row(
            "SELECT COUNT(*) FROM workspaces WHERE lower(name) = lower(?1)",
            rusqlite::params![name],
            |row| row.get::<_, i64>(0),
        )
    })?;

    if existing > 0 {
        return Err(AppError::Validation("A workspace with this name already exists".to_string()));
    }

    let now = chrono::Utc::now().to_rfc3339();
    let workspace = WorkspaceDto {
        id: Uuid::new_v4().to_string(),
        name: name.to_string(),
        description: normalize_optional_string(input.description),
        is_current: false,
        created_at: now.clone(),
        updated_at: now.clone(),
    };

    db.transaction(|conn| {
        conn.execute(
            r#"
            INSERT INTO workspaces (id, name, description, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            rusqlite::params![
                &workspace.id,
                &workspace.name,
                &workspace.description,
                &workspace.created_at,
                &workspace.updated_at,
            ],
        )?;
        Ok(())
    })?;

    info!(workspace_id = %workspace.id, workspace_name = %workspace.name, "Created workspace");
    Ok(workspace)
}

#[tauri::command]
pub fn set_current_workspace(workspace_id: String, db: State<'_, Database>) -> Result<WorkspaceDto> {
    ensure_default_workspace(&db)?;

    let workspace = db.transaction(|conn| {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, description, created_at, updated_at
            FROM workspaces
            WHERE id = ?1
            "#,
        )?;

        stmt.query_row(rusqlite::params![&workspace_id], |row| map_workspace(row, Some(&workspace_id)))
    }).map_err(|error| match error {
        AppError::Database(message) if message.contains("Query returned no rows") => {
            AppError::Validation("Workspace not found".to_string())
        }
        other => other,
    })?;

    let now = chrono::Utc::now().to_rfc3339();
    db.transaction(|conn| {
        conn.execute(
            r#"
            INSERT OR REPLACE INTO settings (key, value, type, updated_at)
            VALUES (?1, ?2, 'string', ?3)
            "#,
            rusqlite::params![CURRENT_WORKSPACE_KEY, &workspace_id, now],
        )?;
        Ok(())
    })?;

    info!(workspace_id = %workspace_id, "Switched current workspace");
    Ok(workspace)
}
