use rusqlite::{Connection, Result as SqliteResult};
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

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        debug!("Initializing database schema");

        let conn = self.conn.blocking_lock();

        // Sessions table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create sessions table: {e}")))?;

        // Messages table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create messages table: {e}")))?;

        // Command executions table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS command_executions (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                command TEXT NOT NULL,
                exit_code INTEGER,
                stdout TEXT,
                stderr TEXT,
                duration_ms INTEGER,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create command_executions table: {e}")))?;

        // File operations table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS file_operations (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                operation TEXT NOT NULL,
                path TEXT NOT NULL,
                success INTEGER NOT NULL DEFAULT 1,
                error TEXT,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create file_operations table: {e}")))?;

        // Config table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                is_secret INTEGER NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create config table: {e}")))?;

        // Channels table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS channels (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                provider TEXT NOT NULL,
                model TEXT NOT NULL,
                api_key TEXT,
                api_base TEXT,
                priority INTEGER DEFAULT 0,
                enabled INTEGER NOT NULL DEFAULT 1,
                health_status TEXT DEFAULT 'unknown',
                last_used INTEGER,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create channels table: {e}")))?;

        // Plugins table
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
        ).map_err(|e| AppError::Database(format!("Failed to create plugins table: {e}")))?;

        // Hotkeys table
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
        ).map_err(|e| AppError::Database(format!("Failed to create hotkeys table: {e}")))?;

        // Themes table
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS themes (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                mode TEXT NOT NULL,
                accent_color TEXT,
                window_opacity REAL,
                blur_enabled INTEGER NOT NULL DEFAULT 0,
                custom_css TEXT,
                is_default INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL
            )
            "#,
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create themes table: {e}")))?;

        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_session_id ON messages(session_id)",
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create messages index: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_commands_session_id ON command_executions(session_id)",
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create commands index: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_file_ops_session_id ON file_operations(session_id)",
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create file_ops index: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_channels_priority ON channels(priority)",
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create channels index: {e}")))?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_hotkeys_action ON hotkeys(action)",
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create hotkeys index: {e}")))?;

        debug!("Database schema initialized");
        Ok(())
    }

    /// Get a reference to the connection
    pub fn conn(&self) -> Arc<tokio::sync::Mutex<Connection>> {
        self.conn.clone()
    }

    /// Execute a query with parameters
    pub fn execute(&self, sql: &str, params: &[&dyn rusqlite::ToSql]) -> SqliteResult<usize> {
        let conn = self.conn.blocking_lock();
        conn.execute(sql, params)
    }
}

/// Configuration storage with secret handling
impl Database {
    /// Get a configuration value
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails
    pub fn get_config(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.blocking_lock();
        let mut stmt = conn.prepare("SELECT value, is_secret FROM config WHERE key = ?1")
            .map_err(|e| AppError::Database(format!("Failed to prepare config query: {e}")))?;

        let mut result = stmt.query([key])
            .map_err(|e| AppError::Database(format!("Failed to query config: {e}")))?;

        if let Ok(Some(row)) = result.next()
            .map_err(|e| AppError::Database(format!("Failed to get config row: {e}")))
        {
            let value: String = row.get(0)?;
            let is_secret: i32 = row.get(1)?;

            if is_secret != 0 {
                Ok(Some("***SECRET***".to_string()))
            } else {
                Ok(Some(value))
            }
        } else {
            Ok(None)
        }
    }

    /// Set a configuration value
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails
    pub fn set_config(&self, key: &str, value: &str, is_secret: bool) -> Result<()> {
        let conn = self.conn.blocking_lock();
        conn.execute(
            r#"
            INSERT INTO config (key, value, is_secret)
            VALUES (?1, ?2, ?3)
            ON CONFLICT (key) DO UPDATE SET
                value = excluded.value,
                is_secret = excluded.is_secret,
                updated_at = CURRENT_TIMESTAMP
            "#,
            rusqlite::params![key, value, if is_secret { 1 } else { 0 }],
        ).map_err(|e| AppError::Database(format!("Failed to set config: {e}")))?;

        debug!("Config '{}' updated", key);
        Ok(())
    }

    /// Get a secret configuration value
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails
    pub fn get_secret(&self, key: &str) -> Result<Option<SecretString>> {
        let conn = self.conn.blocking_lock();
        let mut stmt = conn.prepare("SELECT value FROM config WHERE key = ?1 AND is_secret = 1")
            .map_err(|e| AppError::Database(format!("Failed to prepare secret query: {e}")))?;

        let mut result = stmt.query([key])
            .map_err(|e| AppError::Database(format!("Failed to query secret: {e}")))?;

        if let Ok(Some(row)) = result.next()
            .map_err(|e| AppError::Database(format!("Failed to get secret row: {e}")))
        {
            let value: String = row.get(0)?;
            Ok(Some(SecretString::new(value)))
        } else {
            Ok(None)
        }
    }

    /// Set a secret configuration value
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails
    pub fn set_secret(&self, key: &str, value: SecretString) -> Result<()> {
        self.set_config(key, value.expose_secret(), true)
    }

    /// Delete a configuration value
    ///
    /// # Errors
    ///
    /// Returns an error if the query fails
    pub fn delete_config(&self, key: &str) -> Result<()> {
        let conn = self.conn.blocking_lock();
        conn.execute("DELETE FROM config WHERE key = ?1", [key])
            .map_err(|e| AppError::Database(format!("Failed to delete config: {e}")))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_database_creation() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().to_path_buf();
        let db = Database::new(&data_dir);
        assert!(db.is_ok());
    }

    #[test]
    fn test_config_operations() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().to_path_buf();
        let db = Database::new(&data_dir).unwrap();

        // Set config
        db.set_config("test_key", "test_value", false).unwrap();

        // Get config
        let value = db.get_config("test_key").unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        // Update config
        db.set_config("test_key", "new_value", false).unwrap();
        let value = db.get_config("test_key").unwrap();
        assert_eq!(value, Some("new_value".to_string()));

        // Delete config
        db.delete_config("test_key").unwrap();
        let value = db.get_config("test_key").unwrap();
        assert!(value.is_none());
    }

    #[test]
    fn test_secret_operations() {
        let temp_dir = TempDir::new().unwrap();
        let data_dir = temp_dir.path().to_path_buf();
        let db = Database::new(&data_dir).unwrap();

        // Set secret
        let secret = SecretString::new("my_secret_value".to_string());
        db.set_secret("api_key", secret).unwrap();

        // Get secret
        let retrieved = db.get_secret("api_key").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().expose_secret(), "my_secret_value");

        // Regular get should mask it
        let masked = db.get_config("api_key").unwrap();
        assert_eq!(masked, Some("***SECRET***".to_string()));
    }
}