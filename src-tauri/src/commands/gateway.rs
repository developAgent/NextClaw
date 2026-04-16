//! Gateway management commands
//! Provides Tauri commands for managing the OpenClaw Gateway process

use crate::db::Database;
use crate::utils::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

/// Gateway status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayStatus {
    pub state: GatewayState,
    pub port: u16,
    pub pid: Option<u32>,
    pub started_at: Option<String>,
}

/// Gateway state enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GatewayState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

/// Gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub auto_start: bool,
    pub token: Option<String>,
    pub port: u16,
    pub proxy_enabled: bool,
    pub proxy_server: Option<String>,
    pub proxy_http_server: Option<String>,
    pub proxy_https_server: Option<String>,
    pub proxy_all_server: Option<String>,
    pub proxy_bypass_rules: Option<String>,
}

/// Gateway manager
pub struct GatewayManager {
    status: Arc<Mutex<GatewayStatus>>,
    process: Arc<Mutex<Option<Child>>>,
    config: Arc<Mutex<GatewayConfig>>,
    stopping: Arc<AtomicBool>,
}

impl GatewayManager {
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(GatewayStatus {
                state: GatewayState::Stopped,
                port: 18789,
                pid: None,
                started_at: None,
            })),
            process: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(GatewayConfig {
                auto_start: true,
                token: None,
                port: 18789,
                proxy_enabled: false,
                proxy_server: None,
                proxy_http_server: None,
                proxy_https_server: None,
                proxy_all_server: None,
                proxy_bypass_rules: None,
            })),
            stopping: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the gateway process
    pub async fn start(&self, db: Arc<Database>) -> Result<()> {
        let mut status = self.status.lock().await;
        if status.state != GatewayState::Stopped {
            return Err(AppError::Validation("Gateway is not stopped".to_string()));
        }

        drop(status);

        info!("Starting OpenClaw Gateway process");

        // Update status to starting
        {
            let mut status = self.status.lock().await;
            status.state = GatewayState::Starting;
        }

        // Load configuration from database
        let config = self.load_config(db.clone()).await?;

        // Prepare command
        let mut cmd = Command::new("openclaw");
        cmd.args(["gateway", "start"])
            .arg("--port")
            .arg(config.port.to_string());

        // Add token if configured
        if let Some(token) = &config.token {
            cmd.arg("--token").arg(token);
        }

        // Add proxy settings if enabled
        if config.proxy_enabled {
            if let Some(proxy) = &config.proxy_server {
                cmd.env("HTTP_PROXY", proxy);
                cmd.env("HTTPS_PROXY", proxy);
            }
            if let Some(http_proxy) = &config.proxy_http_server {
                cmd.env("HTTP_PROXY", http_proxy);
            }
            if let Some(https_proxy) = &config.proxy_https_server {
                cmd.env("HTTPS_PROXY", https_proxy);
            }
            if let Some(all_proxy) = &config.proxy_all_server {
                cmd.env("ALL_PROXY", all_proxy);
            }
            if let Some(bypass) = &config.proxy_bypass_rules {
                cmd.env("NO_PROXY", bypass);
            }
        }

        // Start the process
        match cmd.spawn() {
            Ok(child) => {
                let pid = child.id();
                debug!("Gateway process started with PID: {:?}", pid);

                // Store process
                *self.process.lock().await = Some(child);

                // Update status
                {
                    let mut status = self.status.lock().await;
                    status.state = GatewayState::Running;
                    status.pid = Some(pid);
                    status.port = config.port;
                    status.started_at = Some(chrono::Utc::now().to_rfc3339());
                }

                info!("Gateway started successfully on port {}", config.port);
                Ok(())
            }
            Err(e) => {
                // Update status to error
                {
                    let mut status = self.status.lock().await;
                    status.state = GatewayState::Error;
                }
                Err(AppError::Internal(format!(
                    "Failed to start gateway: {}",
                    e
                )))
            }
        }
    }

    /// Stop the gateway process
    pub async fn stop(&self) -> Result<()> {
        let mut status = self.status.lock().await;
        if status.state != GatewayState::Running {
            return Err(AppError::Validation("Gateway is not running".to_string()));
        }

        drop(status);

        info!("Stopping OpenClaw Gateway process");

        // Set stopping flag
        self.stopping.store(true, Ordering::SeqCst);

        // Update status to stopping
        {
            let mut status = self.status.lock().await;
            status.state = GatewayState::Stopping;
        }

        // Kill the process
        {
            let mut process_guard = self.process.lock().await;
            if let Some(mut child) = process_guard.take() {
                match child.kill() {
                    Ok(_) => {
                        debug!("Gateway process killed");
                    }
                    Err(e) => {
                        warn!("Failed to kill gateway process: {}", e);
                    }
                }
            }
        }

        // Reset stopping flag
        self.stopping.store(false, Ordering::SeqCst);

        // Update status
        {
            let mut status = self.status.lock().await;
            status.state = GatewayState::Stopped;
            status.pid = None;
            status.started_at = None;
        }

        info!("Gateway stopped successfully");
        Ok(())
    }

    /// Restart the gateway process
    pub async fn restart(&self, db: Arc<Database>) -> Result<()> {
        info!("Restarting OpenClaw Gateway");

        if self.status.lock().await.state == GatewayState::Running {
            self.stop().await?;
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        self.start(db).await?;

        info!("Gateway restarted successfully");
        Ok(())
    }

    /// Get the current gateway status
    pub async fn get_status(&self) -> GatewayStatus {
        self.status.lock().await.clone()
    }

    /// Load configuration from database
    async fn load_config(&self, db: Arc<Database>) -> Result<GatewayConfig> {
        let conn = db.conn();
        let conn_guard = conn.blocking_lock();

        let mut stmt = conn_guard
            .prepare("SELECT * FROM gateway_config WHERE id = 'main'")
            .map_err(|e| AppError::Database(format!("Failed to query gateway config: {}", e)))?;

        let result = stmt.query_row([], |row| {
            Ok(GatewayConfig {
                auto_start: row.get::<_, i32>(3)? != 0,
                token: row.get(4)?,
                port: row.get::<_, i32>(5)? as u16,
                proxy_enabled: row.get::<_, i32>(6)? != 0,
                proxy_server: row.get(7)?,
                proxy_http_server: row.get(8)?,
                proxy_https_server: row.get(9)?,
                proxy_all_server: row.get(10)?,
                proxy_bypass_rules: row.get(11)?,
            })
        });

        match result {
            Ok(config) => Ok(config),
            Err(_) => {
                // Return default config if not found
                Ok(GatewayConfig {
                    auto_start: true,
                    token: None,
                    port: 18789,
                    proxy_enabled: false,
                    proxy_server: None,
                    proxy_http_server: None,
                    proxy_https_server: None,
                    proxy_all_server: None,
                    proxy_bypass_rules: None,
                })
            }
        }
    }

    /// Get gateway configuration from database and refresh cache
    pub async fn get_config(&self, db: Arc<Database>) -> Result<GatewayConfig> {
        let config = self.load_config(db).await?;
        *self.config.lock().await = config.clone();
        Ok(config)
    }

    /// Save configuration to database
    async fn save_config(&self, config: &GatewayConfig, db: Arc<Database>) -> Result<()> {
        let conn = db.conn();
        let conn_guard = conn.blocking_lock();
        let now = chrono::Utc::now().to_rfc3339();

        conn_guard.execute(
            r#"
            INSERT OR REPLACE INTO gateway_config (
                id, auto_start, token, port, proxy_enabled, proxy_server,
                proxy_http_server, proxy_https_server, proxy_all_server, proxy_bypass_rules, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
            rusqlite::params![
                "main",
                config.auto_start as i32,
                &config.token,
                config.port as i32,
                config.proxy_enabled as i32,
                &config.proxy_server,
                &config.proxy_http_server,
                &config.proxy_https_server,
                &config.proxy_all_server,
                &config.proxy_bypass_rules,
                &now,
            ],
        ).map_err(|e| AppError::Database(format!("Failed to save gateway config: {}", e)))?;

        // Update cached config
        *self.config.lock().await = config.clone();

        info!("Gateway configuration saved");
        Ok(())
    }

    /// Check gateway health
    pub async fn check_health(&self) -> Result<bool> {
        let status = self.status.lock().await;
        if status.state != GatewayState::Running {
            return Ok(false);
        }

        // Try to connect to the gateway
        let url = format!("http://127.0.0.1:{}/health", status.port);
        match reqwest::get(&url).await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Generate a new token
    pub fn generate_token() -> String {
        use uuid::Uuid;
        Uuid::new_v4().to_string()
    }
}

impl Default for GatewayManager {
    fn default() -> Self {
        Self::new()
    }
}

// Tauri commands

/// Get gateway status
#[tauri::command]
pub async fn get_gateway_status(manager: State<'_, Arc<GatewayManager>>) -> Result<GatewayStatus> {
    Ok(manager.get_status().await)
}

/// Start gateway
#[tauri::command]
pub async fn start_gateway(
    db: State<'_, Arc<Database>>,
    manager: State<'_, Arc<GatewayManager>>,
) -> Result<()> {
    manager.start(db.inner().clone()).await
}

/// Stop gateway
#[tauri::command]
pub async fn stop_gateway(manager: State<'_, Arc<GatewayManager>>) -> Result<()> {
    manager.stop().await
}

/// Restart gateway
#[tauri::command]
pub async fn restart_gateway(
    db: State<'_, Arc<Database>>,
    manager: State<'_, Arc<GatewayManager>>,
) -> Result<()> {
    manager.restart(db.inner().clone()).await
}

/// Check gateway health
#[tauri::command]
pub async fn check_gateway_health(manager: State<'_, Arc<GatewayManager>>) -> Result<bool> {
    manager.check_health().await
}

/// Get gateway configuration
#[tauri::command]
pub async fn get_gateway_config(
    db: State<'_, Arc<Database>>,
    manager: State<'_, Arc<GatewayManager>>,
) -> Result<GatewayConfig> {
    manager.get_config(db.inner().clone()).await
}

/// Update gateway configuration
#[tauri::command]
pub async fn update_gateway_config(
    config: GatewayConfig,
    db: State<'_, Arc<Database>>,
    manager: State<'_, Arc<GatewayManager>>,
) -> Result<()> {
    manager.save_config(&config, db.inner().clone()).await
}

/// Generate new gateway token
#[tauri::command]
pub fn generate_new_token() -> String {
    GatewayManager::generate_token()
}

/// Get control UI URL
#[tauri::command]
pub async fn get_control_ui_url(manager: State<'_, Arc<GatewayManager>>) -> Result<String> {
    let status = manager.get_status().await;
    let token = manager.config.lock().await.token.clone();

    if status.state != GatewayState::Running {
        return Err(AppError::Validation("Gateway is not running".to_string()));
    }

    Ok(format!(
        "http://127.0.0.1:{}/?token={}",
        status.port,
        token.unwrap_or_default()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token() {
        let token = GatewayManager::generate_token();
        assert!(!token.is_empty());
        assert!(token.len() == 36); // UUID v4 length
    }
}
