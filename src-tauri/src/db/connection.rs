use rusqlite::{params, Connection, Result as SqliteResult};
use secrecy::{ExposeSecret, Secret, SecretString};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info};

use crate::utils::error::{AppError, Result};

/// Thread-safe database connection manager
#[derive(Clone)]
pub struct Database {
    conn: Arc<tokio::sync::Mutex<Connection>>,
}

impl Database {
    /// Create a new database connection
    ///
    /// # Errors
    ///
    /// Returns an error if database initialization or schema creation fails
    pub fn new(data_dir: &PathBuf) -> Result<Self> {
        // Ensure data directory exists
        std::fs::create_dir_all(data_dir)
            .map_err(|e| AppError::Database(format!("Failed to create data directory: {e}")))?;

        let db_path = data_dir.join("ceo-claw.db");
        debug!("Opening database at: {:?}", db_path);

        let conn = Connection::open(&db_path)
            .map_err(|e| AppError::Database(format!("Failed to open database: {e}")))?;

        let db = Self {
            conn: Arc::new(tokio::sync::Mutex::new(conn)),
        };
        db.init_schema()?;

        info!("Database initialized successfully");
        Ok(db)
    }

    /// Initialize database schema for ClawX functionality
    fn init_schema(&self) -> Result<()> {
        debug!("Initializing database schema");

        let conn = self.conn.blocking_lock();

        // --- Agents Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS agents (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                provider_id TEXT,
                model_id TEXT,
                system_prompt TEXT,
                temperature REAL,
                max_tokens INTEGER,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create agents table: {e}")))?;

        // --- Sessions Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                agent_id TEXT REFERENCES agents(id) ON DELETE SET NULL,
                title TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create sessions table: {e}")))?;

        // --- Messages Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create messages table: {e}")))?;

        // --- Channels Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS channels (
                id TEXT PRIMARY KEY,
                provider_type TEXT NOT NULL,
                name TEXT NOT NULL,
                config TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                priority INTEGER NOT NULL DEFAULT 0,
                health_status TEXT NOT NULL DEFAULT 'unknown',
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create channels table: {e}")))?;

        // --- Channel Accounts Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS channel_accounts (
                id TEXT PRIMARY KEY,
                channel_id TEXT NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
                name TEXT NOT NULL,
                credentials TEXT NOT NULL,
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create channel_accounts table: {e}")))?;

        // --- Cron Jobs Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS cron_jobs (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
                channel_account_id TEXT REFERENCES channel_accounts(id) ON DELETE SET NULL,
                cron_expression TEXT NOT NULL,
                message TEXT NOT NULL,
                target_config TEXT,
                enabled INTEGER NOT NULL DEFAULT 1,
                last_run TEXT,
                next_run TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create cron_jobs table: {e}")))?;

        // --- Cron Executions Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS cron_executions (
                id TEXT PRIMARY KEY,
                job_id TEXT NOT NULL REFERENCES cron_jobs(id) ON DELETE CASCADE,
                status TEXT NOT NULL,
                started_at TEXT NOT NULL,
                completed_at TEXT,
                output TEXT,
                error TEXT
            )
            "#,
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create cron_executions table: {e}")))?;

        // --- Skills Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS skills (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                description TEXT,
                author TEXT,
                enabled INTEGER NOT NULL DEFAULT 1,
                config TEXT,
                permissions_json TEXT,
                path TEXT,
                installed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create skills table: {e}")))?;

        let has_permissions_json = conn
            .prepare("PRAGMA table_info(skills)")
            .and_then(|mut stmt| {
                let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
                let mut found = false;
                for column in columns {
                    if column? == "permissions_json" {
                        found = true;
                        break;
                    }
                }
                Ok(found)
            })
            .map_err(|e| {
                AppError::Database(format!("Failed to inspect skills table schema: {e}"))
            })?;

        if !has_permissions_json {
            conn.execute("ALTER TABLE skills ADD COLUMN permissions_json TEXT", [])
                .map_err(|e| {
                    AppError::Database(format!("Failed to add skills.permissions_json column: {e}"))
                })?;
        }

        // --- Models Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS models (
                id TEXT PRIMARY KEY,
                provider_type TEXT NOT NULL,
                name TEXT NOT NULL,
                display_name TEXT NOT NULL,
                context_window INTEGER,
                pricing TEXT,
                capabilities TEXT
            )
            "#,
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create models table: {e}")))?;

        // --- Settings Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                type TEXT NOT NULL DEFAULT 'string',
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create settings table: {e}")))?;

        // --- Secure Storage (API Keys) Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS secure_storage (
                id TEXT PRIMARY KEY,
                service TEXT NOT NULL,
                account_id TEXT,
                encrypted_value TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create secure_storage table: {e}")))?;

        // --- Gateway Config Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS gateway_config (
                id TEXT PRIMARY KEY,
                auto_start INTEGER NOT NULL DEFAULT 1,
                token TEXT,
                port INTEGER NOT NULL DEFAULT 18789,
                proxy_enabled INTEGER NOT NULL DEFAULT 0,
                proxy_server TEXT,
                proxy_http_server TEXT,
                proxy_https_server TEXT,
                proxy_all_server TEXT,
                proxy_bypass_rules TEXT,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create gateway_config table: {e}")))?;

        // --- Skill Marketplace Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS skill_marketplace (
                slug TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version TEXT,
                description TEXT,
                author TEXT,
                icon TEXT,
                available INTEGER NOT NULL DEFAULT 0,
                installed INTEGER NOT NULL DEFAULT 0,
                installed_at TEXT,
                installed_path TEXT,
                skill_id TEXT
            )
            "#,
            [],
        )
        .map_err(|e| {
            AppError::Database(format!("Failed to create skill_marketplace table: {e}"))
        })?;

        let skill_marketplace_columns = conn
            .prepare("PRAGMA table_info(skill_marketplace)")
            .and_then(|mut stmt| {
                let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;
                columns.collect::<std::result::Result<Vec<_>, _>>()
            })
            .map_err(|e| {
                AppError::Database(format!(
                    "Failed to inspect skill_marketplace table schema: {e}"
                ))
            })?;

        if !skill_marketplace_columns
            .iter()
            .any(|column| column == "available")
        {
            conn.execute(
                "ALTER TABLE skill_marketplace ADD COLUMN available INTEGER NOT NULL DEFAULT 0",
                [],
            )
            .map_err(|e| {
                AppError::Database(format!(
                    "Failed to add skill_marketplace.available column: {e}"
                ))
            })?;
        }

        if !skill_marketplace_columns
            .iter()
            .any(|column| column == "skill_id")
        {
            conn.execute("ALTER TABLE skill_marketplace ADD COLUMN skill_id TEXT", [])
                .map_err(|e| {
                    AppError::Database(format!(
                        "Failed to add skill_marketplace.skill_id column: {e}"
                    ))
                })?;
        }

        // --- Plugins Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS plugins (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version TEXT NOT NULL,
                author TEXT,
                description TEXT,
                enabled INTEGER NOT NULL DEFAULT 1,
                config TEXT,
                installed_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create plugins table: {e}")))?;

        // --- Recordings Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS recordings (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL,
                events_json TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create recordings table: {e}")))?;

        // --- Workflows Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS workflows (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                nodes_json TEXT NOT NULL,
                edges_json TEXT NOT NULL,
                variables_json TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create workflows table: {e}")))?;

        // --- Hotkeys Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS hotkeys (
                id TEXT PRIMARY KEY,
                action TEXT NOT NULL,
                key_combination TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create hotkeys table: {e}")))?;

        // --- Skill Config Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS skill_config (
                skill_key TEXT PRIMARY KEY,
                api_key TEXT,
                env_vars TEXT,
                enabled INTEGER NOT NULL DEFAULT 1,
                config_path TEXT,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create skill_config table: {e}")))?;

        // --- Token Usage Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS token_usage (
                id TEXT PRIMARY KEY,
                session_id TEXT,
                agent_id TEXT,
                model_id TEXT,
                prompt_tokens INTEGER NOT NULL DEFAULT 0,
                completion_tokens INTEGER NOT NULL DEFAULT 0,
                total_tokens INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create token_usage table: {e}")))?;

        // --- App Logs Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS app_logs (
                id TEXT PRIMARY KEY,
                level TEXT NOT NULL,
                message TEXT NOT NULL,
                timestamp TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                context TEXT
            )
            "#,
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create app_logs table: {e}")))?;

        // --- Workspaces Table ---
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS workspaces (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create workspaces table: {e}")))?;

        // --- Indexes for better performance ---
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_session_id ON messages(session_id)",
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create messages index: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sessions_agent_id ON sessions(agent_id)",
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create sessions index: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_cron_jobs_enabled ON cron_jobs(enabled)",
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create cron_jobs index: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_skills_enabled ON skills(enabled)",
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create skills index: {e}")))?;

        // --- New indexes for ClawX functionality ---
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_gateway_config ON gateway_config(auto_start)",
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create gateway_config index: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_skill_marketplace_installed ON skill_marketplace(installed)",
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create skill_marketplace index: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_plugins_enabled ON plugins(enabled)",
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create plugins enabled index: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_plugins_updated_at ON plugins(updated_at)",
            [],
        )
        .map_err(|e| {
            AppError::Database(format!("Failed to create plugins updated_at index: {e}"))
        })?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_recordings_updated_at ON recordings(updated_at)",
            [],
        )
        .map_err(|e| {
            AppError::Database(format!("Failed to create recordings updated_at index: {e}"))
        })?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_workflows_updated_at ON workflows(updated_at)",
            [],
        )
        .map_err(|e| {
            AppError::Database(format!("Failed to create workflows updated_at index: {e}"))
        })?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_hotkeys_enabled ON hotkeys(enabled)",
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create hotkeys enabled index: {e}")))?;

        conn.execute(
            "CREATE UNIQUE INDEX IF NOT EXISTS idx_hotkeys_key_combination ON hotkeys(key_combination)",
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create hotkeys key combination index: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_skill_config_enabled ON skill_config(enabled)",
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create skill_config index: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_token_usage_session ON token_usage(session_id)",
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create token_usage index: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_token_usage_agent ON token_usage(agent_id)",
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create token_usage index: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_app_logs_timestamp ON app_logs(timestamp)",
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create app_logs index: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_app_logs_level ON app_logs(level)",
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create app_logs index: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_workspaces_name ON workspaces(name)",
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to create workspaces index: {e}")))?;

        debug!("Database schema initialized successfully");
        Ok(())
    }

    /// Get a reference to the database connection
    pub fn connection(&self) -> Arc<tokio::sync::Mutex<Connection>> {
        Arc::clone(&self.conn)
    }

    /// Get the connection (alias for backward compatibility)
    pub fn conn(&self) -> Arc<tokio::sync::Mutex<Connection>> {
        self.connection()
    }

    /// Execute a transaction
    pub fn transaction<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&Connection) -> SqliteResult<R>,
    {
        let conn = self.conn.blocking_lock();
        f(&conn).map_err(|e| AppError::Database(e.to_string()))
    }

    /// Get a secret from secure storage
    pub fn get_secret(&self, key: &str) -> Result<Option<SecretString>> {
        let conn = self.conn.blocking_lock();
        let mut stmt = conn
            .prepare("SELECT encrypted_value FROM secure_storage WHERE id = ?1")
            .map_err(|e| AppError::Database(format!("Failed to get secret: {}", e)))?;

        let result = stmt
            .query_row(params![key], |row| row.get::<_, String>(0))
            .ok();

        Ok(result.map(|s| SecretString::new(s)))
    }

    /// Set a secret in secure storage
    pub fn set_secret(&self, key: &str, secret: SecretString) -> Result<()> {
        let conn = self.conn.blocking_lock();
        let now = chrono::Utc::now().to_rfc3339();

        conn.execute(
            r#"
            INSERT OR REPLACE INTO secure_storage (id, service, account_id, encrypted_value, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
            params![
                key,
                "system",
                Option::<&str>::None,
                secret.expose_secret(),
                &now,
                &now,
            ],
        )
        .map_err(|e| AppError::Database(format!("Failed to set secret: {}", e)))?;

        Ok(())
    }

    /// Delete a secret from secure storage
    pub fn delete_config(&self, key: &str) -> Result<()> {
        let conn = self.conn.blocking_lock();
        conn.execute("DELETE FROM secure_storage WHERE id = ?1", params![key])
            .map_err(|e| AppError::Database(format!("Failed to delete config: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_creation() {
        let db = Database::new(&std::path::PathBuf::from("/tmp/test_db"));
        assert!(db.is_ok());
    }
}
