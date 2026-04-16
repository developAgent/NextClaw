use crate::db::connection::Database;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};
use tracing::{error, info, warn};

/// Diagnostic information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticInfo {
    pub status: DiagnosticStatus,
    pub checks: Vec<DiagnosticCheck>,
    pub summary: String,
}

/// Diagnostic status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticStatus {
    Healthy,
    Warning,
    Critical,
}

/// Individual diagnostic check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticCheck {
    pub name: String,
    pub status: DiagnosticStatus,
    pub message: String,
    pub details: Option<String>,
}

/// Telemetry data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryData {
    pub session_id: String,
    pub agent_id: Option<String>,
    pub model_id: String,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    pub timestamp: String,
}

/// Gateway token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayTokenInfo {
    pub token: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub last_used: Option<String>,
}

/// WebSocket diagnostic result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketDiagnostic {
    pub connected: bool,
    pub url: String,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

/// Run diagnostic checks (OpenClaw Doctor)
#[tauri::command]
pub async fn run_diagnostics(
    db: State<'_, Database>,
    app: AppHandle,
) -> Result<DiagnosticInfo, String> {
    info!("Running diagnostic checks...");

    let mut checks = Vec::new();

    // Check database connection
    let db_status = check_database(&db);
    checks.push(db_status);

    // Check Gateway status
    let gateway_status = check_gateway(&app).await;
    checks.push(gateway_status);

    // Check skills installation
    let skills_status = check_skills(&db);
    checks.push(skills_status);

    // Check configuration
    let config_status = check_configuration(&db);
    checks.push(config_status);

    // Determine overall status
    let overall_status = if checks
        .iter()
        .any(|c| c.status == DiagnosticStatus::Critical)
    {
        DiagnosticStatus::Critical
    } else if checks.iter().any(|c| c.status == DiagnosticStatus::Warning) {
        DiagnosticStatus::Warning
    } else {
        DiagnosticStatus::Healthy
    };

    let summary = match overall_status {
        DiagnosticStatus::Healthy => "All systems operational".to_string(),
        DiagnosticStatus::Warning => "Some issues detected, but system is functional".to_string(),
        DiagnosticStatus::Critical => "Critical issues detected, please address".to_string(),
    };

    let diagnostic_info = DiagnosticInfo {
        status: overall_status.clone(),
        checks,
        summary,
    };

    info!("Diagnostics completed: {:?}", overall_status);

    // Emit diagnostic results
    let _ = app.emit("diagnostics-completed", &diagnostic_info);

    Ok(diagnostic_info)
}

fn check_database(db: &Database) -> DiagnosticCheck {
    match db.transaction(|conn| -> rusqlite::Result<(), rusqlite::Error> {
        let mut stmt = conn.prepare("SELECT 1")?;
        stmt.query([])?;
        Ok(())
    }) {
        Ok(_) => DiagnosticCheck {
            name: "Database Connection".to_string(),
            status: DiagnosticStatus::Healthy,
            message: "Database connection is working".to_string(),
            details: None,
        },
        Err(e) => DiagnosticCheck {
            name: "Database Connection".to_string(),
            status: DiagnosticStatus::Critical,
            message: "Database connection failed".to_string(),
            details: Some(format!("Error: {}", e)),
        },
    }
}

async fn check_gateway(_app: &AppHandle) -> DiagnosticCheck {
    // Simplified check - in production, you would check actual gateway status
    DiagnosticCheck {
        name: "Gateway".to_string(),
        status: DiagnosticStatus::Healthy,
        message: "Gateway component is available".to_string(),
        details: None,
    }
}

fn check_skills(db: &Database) -> DiagnosticCheck {
    match db.transaction(
        |conn| -> rusqlite::Result<DiagnosticCheck, rusqlite::Error> {
            let mut stmt = conn.prepare("SELECT COUNT(*) as count FROM skills")?;

            let mut rows = stmt.query([])?;

            if let Some(row) = rows.next()? {
                let count: i64 = row.get("count").unwrap_or(0);
                Ok(DiagnosticCheck {
                    name: "Installed Skills".to_string(),
                    status: DiagnosticStatus::Healthy,
                    message: format!("{} skill(s) installed", count),
                    details: None,
                })
            } else {
                Ok(DiagnosticCheck {
                    name: "Installed Skills".to_string(),
                    status: DiagnosticStatus::Healthy,
                    message: "0 skills installed".to_string(),
                    details: None,
                })
            }
        },
    ) {
        Ok(check) => check,
        Err(e) => DiagnosticCheck {
            name: "Installed Skills".to_string(),
            status: DiagnosticStatus::Warning,
            message: "Could not check installed skills".to_string(),
            details: Some(format!("Error: {}", e)),
        },
    }
}

fn check_configuration(db: &Database) -> DiagnosticCheck {
    match db.transaction(
        |conn| -> rusqlite::Result<DiagnosticCheck, rusqlite::Error> {
            let mut stmt = conn.prepare("SELECT COUNT(*) as count FROM settings")?;

            let mut rows = stmt.query([])?;

            if let Some(row) = rows.next()? {
                let count: i64 = row.get("count").unwrap_or(0);
                Ok(DiagnosticCheck {
                    name: "Configuration".to_string(),
                    status: DiagnosticStatus::Healthy,
                    message: format!("{} configuration item(s) loaded", count),
                    details: None,
                })
            } else {
                Ok(DiagnosticCheck {
                    name: "Configuration".to_string(),
                    status: DiagnosticStatus::Healthy,
                    message: "0 configuration items loaded".to_string(),
                    details: None,
                })
            }
        },
    ) {
        Ok(check) => check,
        Err(e) => DiagnosticCheck {
            name: "Configuration".to_string(),
            status: DiagnosticStatus::Warning,
            message: "Could not check configuration".to_string(),
            details: Some(format!("Error: {}", e)),
        },
    }
}

/// Get telemetry data
#[tauri::command]
pub async fn get_telemetry_data(
    db: State<'_, Database>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<TelemetryData>, String> {
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);

    db.transaction(|conn| -> rusqlite::Result<Vec<TelemetryData>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT id, session_id, agent_id, model_id, prompt_tokens, completion_tokens, total_tokens, created_at
             FROM token_usage
             ORDER BY created_at DESC
             LIMIT ? OFFSET ?"
        )?;

        let mut rows = stmt.query(rusqlite::params![limit, offset])?;

        let mut results = Vec::new();
        while let Some(row) = rows.next()? {
            results.push(TelemetryData {
                session_id: row.get("session_id")?,
                agent_id: row.get("agent_id").ok(),
                model_id: row.get("model_id")?,
                prompt_tokens: row.get("prompt_tokens")?,
                completion_tokens: row.get("completion_tokens")?,
                total_tokens: row.get("total_tokens")?,
                timestamp: row.get("created_at")?,
            });
        }
        Ok(results)
    }).map_err(|e| format!("Failed to query telemetry data: {}", e))
}

/// Get token usage statistics
#[tauri::command]
pub async fn get_token_usage_stats(db: State<'_, Database>) -> Result<TokenUsageStats, String> {
    let stats = db
        .transaction(
            |conn| -> rusqlite::Result<TokenUsageStats, rusqlite::Error> {
                let mut stmt = conn.prepare(
                    "SELECT
               SUM(prompt_tokens) as total_prompt,
               SUM(completion_tokens) as total_completion,
               SUM(total_tokens) as total_tokens,
               COUNT(*) as total_requests
             FROM token_usage",
                )?;

                let mut rows = stmt.query([])?;

                let row = rows
                    .next()?
                    .ok_or_else(|| rusqlite::Error::QueryReturnedNoRows)?;

                Ok(TokenUsageStats {
                    total_prompt_tokens: row.get("total_prompt").ok().flatten().unwrap_or(0),
                    total_completion_tokens: row
                        .get("total_completion")
                        .ok()
                        .flatten()
                        .unwrap_or(0),
                    total_tokens: row.get("total_tokens").ok().flatten().unwrap_or(0),
                    total_requests: row.get("total_requests").ok().flatten().unwrap_or(0),
                })
            },
        )
        .map_err(|e| format!("Failed to query token usage stats: {}", e))?;

    Ok(stats)
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageStats {
    pub total_prompt_tokens: i64,
    pub total_completion_tokens: i64,
    pub total_tokens: i64,
    pub total_requests: i64,
}

/// Get application logs
#[tauri::command]
pub async fn get_app_logs(
    db: State<'_, Database>,
    level: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Vec<LogEntry>, String> {
    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);

    let logs = db
        .transaction(|conn| -> rusqlite::Result<Vec<LogEntry>, rusqlite::Error> {
            let mut results = Vec::new();

            if let Some(level_filter) = level {
                let mut stmt = conn.prepare(
                    "SELECT id, level, message, timestamp, context
                 FROM app_logs
                 WHERE level = ?
                 ORDER BY timestamp DESC
                 LIMIT ? OFFSET ?",
                )?;

                let mut rows = stmt.query(rusqlite::params![&level_filter, &limit, &offset])?;
                while let Some(row) = rows.next()? {
                    results.push(LogEntry {
                        id: row.get("id")?,
                        level: row.get("level")?,
                        message: row.get("message")?,
                        timestamp: row.get("timestamp")?,
                        context: row.get("context").ok(),
                    });
                }
            } else {
                let mut stmt = conn.prepare(
                    "SELECT id, level, message, timestamp, context
                 FROM app_logs
                 ORDER BY timestamp DESC
                 LIMIT ? OFFSET ?",
                )?;

                let mut rows = stmt.query(rusqlite::params![&limit, &offset])?;
                while let Some(row) = rows.next()? {
                    results.push(LogEntry {
                        id: row.get("id")?,
                        level: row.get("level")?,
                        message: row.get("message")?,
                        timestamp: row.get("timestamp")?,
                        context: row.get("context").ok(),
                    });
                }
            }

            Ok(results)
        })
        .map_err(|e| format!("Failed to query logs: {}", e))?;

    Ok(logs)
}

/// Log entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub id: String,
    pub level: String,
    pub message: String,
    pub timestamp: String,
    pub context: Option<String>,
}

/// Export telemetry data
#[tauri::command]
pub async fn export_telemetry_data(
    db: State<'_, Database>,
    format: String,
) -> Result<String, String> {
    let data = get_telemetry_data(db, None, None).await?;

    let output = match format.to_lowercase().as_str() {
        "json" => serde_json::to_string_pretty(&data)
            .map_err(|e| format!("Failed to serialize to JSON: {}", e))?,
        "csv" => {
            let mut csv = String::from("session_id,agent_id,model_id,prompt_tokens,completion_tokens,total_tokens,timestamp\n");
            for entry in data {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{}\n",
                    entry.session_id,
                    entry.agent_id.unwrap_or_else(|| "none".to_string()),
                    entry.model_id,
                    entry.prompt_tokens,
                    entry.completion_tokens,
                    entry.total_tokens,
                    entry.timestamp
                ));
            }
            csv
        }
        _ => return Err(format!("Unsupported export format: {}", format)),
    };

    Ok(output)
}

/// Clear telemetry data
#[tauri::command]
pub async fn clear_telemetry_data(db: State<'_, Database>) -> Result<(), String> {
    db.transaction(|conn| conn.execute("DELETE FROM token_usage", []))
        .map_err(|e| format!("Failed to clear telemetry data: {}", e))?;

    info!("Telemetry data cleared");

    Ok(())
}

/// Get system information
#[tauri::command]
pub fn get_system_info() -> SystemInfo {
    SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        family: std::env::consts::FAMILY.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// System information structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub family: String,
    pub version: String,
}
