use crate::db::models::CommandExecution;
use crate::db::Database;
use crate::exec::shell::{ExecutionResult, ShellExecutor};
use crate::utils::error::Result;
use tauri::State;
use uuid::Uuid;
use tracing::{debug, info, warn};

/// Execute a system command
///
/// # Safety
///
/// Commands are validated against whitelist/blacklist before execution
#[tauri::command]
pub async fn execute_command(
    command: String,
    session_id: Uuid,
    whitelist: Vec<String>,
    blacklist: Vec<String>,
    timeout_secs: u64,
    db: State<'_, Database>,
) -> Result<CommandExecution> {
    info!("Executing command: {}", command);

    // Create executor
    let executor = ShellExecutor::new(whitelist, blacklist, None, timeout_secs);

    // Execute command
    let result = executor.execute(&command).await?;

    // Record execution
    let mut execution = CommandExecution::new(session_id, command);
    execution.exit_code = result.exit_code;
    execution.stdout = Some(result.stdout.clone());
    execution.stderr = Some(result.stderr.clone());
    execution.duration_ms = Some(result.duration_ms as i64);

    save_execution(&db, &execution).await?;

    if result.success() {
        debug!("Command executed successfully");
        Ok(execution)
    } else {
        warn!("Command failed: {:?}", result);
        Ok(execution)
    }
}

/// Get command execution history for a session
#[tauri::command]
pub async fn get_command_history(
    session_id: Uuid,
    db: State<'_, Database>,
) -> Result<Vec<CommandExecution>> {
    let executions = get_session_executions(&db, session_id).await?;
    Ok(executions)
}

/// Check if a command requires confirmation before execution
#[tauri::command]
pub async fn check_command_confirmation(
    command: String,
    whitelist: Vec<String>,
    blacklist: Vec<String>,
) -> Result<bool> {
    let executor = ShellExecutor::new(whitelist, blacklist, None, 300);
    Ok(executor.requires_confirmation(&command))
}

// Helper functions

async fn save_execution(db: &Database, execution: &CommandExecution) -> Result<()> {
    db.execute(
        r#"
        INSERT INTO command_executions (id, session_id, command, exit_code, stdout, stderr, duration_ms)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        &[
            &execution.id.to_string(),
            &execution.session_id.to_string(),
            &execution.command,
            &execution.exit_code.map(|c| c.to_string()),
            &execution.stdout.clone(),
            &execution.stderr.clone(),
            &execution.duration_ms.map(|d| d.to_string()),
        ]
    ).map_err(|e| crate::utils::error::AppError::Database(e.to_string()))?;
    Ok(())
}

async fn get_session_executions(db: &Database, session_id: Uuid) -> Result<Vec<CommandExecution>> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, command, exit_code, stdout, stderr, duration_ms, created_at
         FROM command_executions
         WHERE session_id = ?1
         ORDER BY created_at DESC"
    ).map_err(|e| crate::utils::error::AppError::Database(e.to_string()))?;

    let mut executions = Vec::new();
    let mut rows = stmt.query(&[&session_id.to_string()])
        .map_err(|e| crate::utils::error::AppError::Database(e.to_string()))?;

    while let Some(row) = rows.next().map_err(|e| crate::utils::error::AppError::Database(e.to_string()))? {
        let id: String = row.get(0)?;
        let command: String = row.get(1)?;
        let exit_code: Option<String> = row.get(2)?;
        let stdout: Option<String> = row.get(3)?;
        let stderr: Option<String> = row.get(4)?;
        let duration_ms: Option<String> = row.get(5)?;
        let created_at: String = row.get(6)?;

        executions.push(CommandExecution {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            session_id,
            command,
            exit_code: exit_code.and_then(|s| s.parse().ok()),
            stdout,
            stderr,
            duration_ms: duration_ms.and_then(|s| s.parse().ok()),
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                .unwrap_or_default()
                .with_timezone(&chrono::Utc),
        });
    }

    Ok(executions)
}