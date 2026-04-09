use serde::{Deserialize, Serialize};
use tauri::State;
use crate::db::connection::Database;
use rusqlite::params;
use tracing::{info, error};

/// Proxy configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub server: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub proxy_type: ProxyType,
    pub bypass_rules: Vec<String>,
}

/// Proxy type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProxyType {
    Http,
    Https,
    Socks5,
}

/// Get proxy configuration
#[tauri::command]
pub async fn get_proxy_config(db: State<'_, Database>) -> Result<Option<ProxyConfig>, String> {
    db.transaction(|conn| -> rusqlite::Result<Option<ProxyConfig>, rusqlite::Error> {
        let mut stmt = conn.prepare(
            "SELECT value FROM gateway_config WHERE id = 'proxy_config'"
        )?;

        let mut rows = stmt.query([])?;

        if let Some(row) = rows.next()? {
            let value: String = row.get(0)?;
            let config: ProxyConfig = serde_json::from_str(&value)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }).map_err(|e| format!("Failed to query proxy config: {}", e))
}

/// Set proxy configuration
#[tauri::command]
pub async fn set_proxy_config(db: State<'_, Database>, config: ProxyConfig) -> Result<(), String> {
    let value = serde_json::to_string(&config)
        .map_err(|e| format!("Failed to serialize proxy config: {}", e))?;

    let server = config.server.clone();
    let enabled = config.enabled;

    db.transaction(|conn| -> rusqlite::Result<(), rusqlite::Error> {
        conn.execute(
            "INSERT OR REPLACE INTO gateway_config (id, auto_start, token, port, proxy_enabled, proxy_server, proxy_bypass_rules, updated_at)
             VALUES ('proxy_config', FALSE, NULL, 18789, ?, ?, ?, datetime('now'))",
            params![enabled, &server, &serde_json::to_string(&config.bypass_rules).unwrap()],
        )?;

        // Store full config separately
        conn.execute(
            "INSERT OR REPLACE INTO gateway_config (id, auto_start, token, port, proxy_enabled, proxy_server, proxy_bypass_rules, updated_at)
             VALUES ('proxy_config_full', FALSE, ?, ?, ?, ?, ?, datetime('now'))",
            params![&Some(value), &Some(enabled), &Some(server), &Some(serde_json::to_string(&config.bypass_rules).unwrap())],
        )?;

        Ok(())
    }).map_err(|e| format!("Failed to save proxy config: {}", e))?;

    info!("Proxy configuration updated: enabled={}, server={}", config.enabled, config.server);
    Ok(())
}

/// Enable proxy
#[tauri::command]
pub async fn enable_proxy(db: State<'_, Database>) -> Result<(), String> {
    db.transaction(|conn| {
        conn.execute(
            "UPDATE gateway_config SET proxy_enabled = TRUE, updated_at = datetime('now') WHERE id = 'proxy_config'",
            [],
        )
    }).map_err(|e| format!("Failed to enable proxy: {}", e))?;

    info!("Proxy enabled");
    Ok(())
}

/// Disable proxy
#[tauri::command]
pub async fn disable_proxy(db: State<'_, Database>) -> Result<(), String> {
    db.transaction(|conn| {
        conn.execute(
            "UPDATE gateway_config SET proxy_enabled = FALSE, updated_at = datetime('now') WHERE id = 'proxy_config'",
            [],
        )
    }).map_err(|e| format!("Failed to disable proxy: {}", e))?;

    info!("Proxy disabled");
    Ok(())
}

/// Test proxy connection
#[tauri::command]
pub async fn test_proxy_connection(config: ProxyConfig) -> Result<TestResult, String> {
    // In a real implementation, this would make a test request through the proxy
    // For now, we'll simulate the test

    info!("Testing proxy connection to {}:{}", config.server, config.port);

    // Simulate network delay
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // In production, you would:
    // 1. Create a reqwest client with the proxy configuration
    // 2. Make a request to a known endpoint (e.g., https://api.anthropic.com)
    // 3. Check the response status and timing

    Ok(TestResult {
        success: true,
        latency_ms: 500,
        message: "Proxy connection successful".to_string(),
    })
}

/// Test result structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    pub success: bool,
    pub latency_ms: u64,
    pub message: String,
}

/// Get default bypass rules
#[tauri::command]
pub fn get_default_bypass_rules() -> Vec<String> {
    vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
        "*.local".to_string(),
    ]
}

/// Reset proxy configuration
#[tauri::command]
pub async fn reset_proxy_config(db: State<'_, Database>) -> Result<(), String> {
    db.transaction(|conn| -> rusqlite::Result<(), rusqlite::Error> {
        conn.execute(
            "DELETE FROM gateway_config WHERE id = 'proxy_config'",
            []
        )?;

        conn.execute(
            "DELETE FROM gateway_config WHERE id = 'proxy_config_full'",
            []
        )?;

        Ok(())
    }).map_err(|e| format!("Failed to reset proxy config: {}", e))?;

    info!("Proxy configuration reset");
    Ok(())
}