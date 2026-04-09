use rusqlite::{Connection, params, Result as SqliteResult};
use secrecy::{ExposeSecret, Secret, SecretString};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, info};

use crate::utils::error::{AppError, Result};

/// Thread-safe database connection manager
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
        ).map_err(|e| AppError::Database(format!("Failed to create agents table: {e}")))?;

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
        ).map_err(|e| AppError::Database(format!("Failed to create sessions table: {e}")))?;

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
        ).map_err(|e| AppError::Database(format!("Failed to create messages table: {e}")))?;

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
        ).map_err(|e| AppError::Database(format!("Failed to create channels table: {e}")))?;

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
        ).map_err(|e| AppError::Database(format!("Failed to create channel_accounts table: {e}")))?;

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
        ).map_err(|e| AppError::Database(format!("Failed to create cron_jobs table: {e}")))?;

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
        ).map_err(|e| AppError::Database(format!("Failed to create cron_executions table: {e}")))?;

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
                path TEXT,
                installed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create skills table: {e}")))?;

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
        ).map_err(|e| AppError::Database(format!("Failed to create models table: {e}")))?;

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
        ).map_err(|e| AppError::Database(format!("Failed to create settings table: {e}")))?;

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
        ).map_err(|e| AppError::Database(format!("Failed to create secure_storage table: {e}")))?;

        // --- Indexes for better performance ---
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_session_id ON messages(session_id)",
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create messages index: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_sessions_agent_id ON sessions(agent_id)",
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create sessions index: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_cron_jobs_enabled ON cron_jobs(enabled)",
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create cron_jobs index: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_skills_enabled ON skills(enabled)",
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create skills index: {e}")))?;

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