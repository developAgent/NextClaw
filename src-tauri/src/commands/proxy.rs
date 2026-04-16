use crate::db::connection::Database;
use reqwest::Proxy;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{error, info};

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
    db.transaction(
        |conn| -> rusqlite::Result<Option<ProxyConfig>, rusqlite::Error> {
            let mut stmt =
                conn.prepare("SELECT value FROM gateway_config WHERE id = 'proxy_config'")?;

            let mut rows = stmt.query([])?;

            if let Some(row) = rows.next()? {
                let value: String = row.get(0)?;
                let config: ProxyConfig = serde_json::from_str(&value)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
                Ok(Some(config))
            } else {
                Ok(None)
            }
        },
    )
    .map_err(|e| format!("Failed to query proxy config: {}", e))
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

    info!(
        "Proxy configuration updated: enabled={}, server={}",
        config.enabled, config.server
    );
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
    if !config.enabled {
        return Err("Proxy is disabled".to_string());
    }

    let server = config.server.trim();
    if server.is_empty() {
        return Err("Proxy server is required".to_string());
    }

    if config.port == 0 {
        return Err("Proxy port must be greater than 0".to_string());
    }

    let proxy_url = match config.proxy_type {
        ProxyType::Http => format!("http://{}:{}", server, config.port),
        ProxyType::Https => format!("https://{}:{}", server, config.port),
        ProxyType::Socks5 => format!("socks5h://{}:{}", server, config.port),
    };

    info!("Testing proxy connection to {}", proxy_url);

    let proxy =
        Proxy::all(&proxy_url).map_err(|e| format!("Invalid proxy configuration: {}", e))?;
    let proxy = if let Some(username) = config.username.as_deref() {
        if username.is_empty() {
            proxy
        } else {
            proxy.basic_auth(username, config.password.as_deref().unwrap_or(""))
        }
    } else {
        proxy
    };

    let client = reqwest::Client::builder()
        .proxy(proxy)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("Failed to create proxy client: {}", e))?;

    let start = std::time::Instant::now();
    let response = client
        .get("https://api.anthropic.com")
        .send()
        .await
        .map_err(|e| {
            error!("Proxy connection test failed: {}", e);
            format!("Proxy connection failed: {}", e)
        })?;

    let latency_ms = start.elapsed().as_millis() as u64;
    let status = response.status();

    if status.is_success() || status.as_u16() == 401 || status.as_u16() == 403 {
        Ok(TestResult {
            success: true,
            latency_ms,
            message: format!("Proxy connection successful ({})", status),
        })
    } else {
        Err(format!(
            "Proxy responded with unexpected status: {}",
            status
        ))
    }
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
        conn.execute("DELETE FROM gateway_config WHERE id = 'proxy_config'", [])?;

        conn.execute(
            "DELETE FROM gateway_config WHERE id = 'proxy_config_full'",
            [],
        )?;

        Ok(())
    })
    .map_err(|e| format!("Failed to reset proxy config: {}", e))?;

    info!("Proxy configuration reset");
    Ok(())
}
