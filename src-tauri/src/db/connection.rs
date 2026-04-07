use duckdb::{Connection, Result as DuckResult};
use secrecy::{Secret, SecretString};
use std::path::PathBuf;
use tracing::{debug, error, info};

use crate::utils::error::{AppError, Result};

/// Database connection manager
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create a new database connection
    ///
    /// # Errors
    ///
    /// Returns an error if database initialization or schema creation fails
    pub fn new(data_dir: &PathBuf) -> Result<Self> {
        let db_path = data_dir.join("ceo-claw.db");
        debug!("Opening database at: {:?}", db_path);

        let conn = Connection::open(&db_path).map_err(|e| AppError::Database(format!("Failed to open database: {e}")))?;

        let db = Self { conn };
        db.init_schema()?;

        info!("Database initialized successfully");
        Ok(db)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        debug!("Initializing database schema");

        // Sessions table
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id UUID PRIMARY KEY,
                title TEXT NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create sessions table: {e}")))?;

        // Messages table
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id UUID PRIMARY KEY,
                session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create messages table: {e}")))?;

        // Command executions table
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS command_executions (
                id UUID PRIMARY KEY,
                session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                command TEXT NOT NULL,
                exit_code INTEGER,
                stdout TEXT,
                stderr TEXT,
                duration_ms INTEGER,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create command_executions table: {e}")))?;

        // File operations table
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS file_operations (
                id UUID PRIMARY KEY,
                session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                operation TEXT NOT NULL,
                path TEXT NOT NULL,
                success BOOLEAN NOT NULL,
                error TEXT,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create file_operations table: {e}")))?;

        // Config table
        self.conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                is_secret BOOLEAN NOT NULL DEFAULT FALSE,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create config table: {e}")))?;

        // Create indexes
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_messages_session_id ON messages(session_id)",
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create messages index: {e}")))?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_commands_session_id ON command_executions(session_id)",
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create commands index: {e}")))?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_file_ops_session_id ON file_operations(session_id)",
            [],
        ).map_err(|e| AppError::Database(format!("Failed to create file_ops index: {e}")))?;

        debug!("Database schema initialized");
        Ok(())
    }

    /// Get a reference to the connection
    #[must_use]
    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Execute a query with parameters
    pub fn execute(&self, sql: &str, params: &[&dyn duckdb::ToSql]) -> DuckResult<usize> {
        self.conn.execute(sql, params)
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
        let mut stmt = self.conn.prepare("SELECT value, is_secret FROM config WHERE key = ?1")
            .map_err(|e| AppError::Database(format!("Failed to prepare config query: {e}")))?;

        let mut result = stmt.query([key])
            .map_err(|e| AppError::Database(format!("Failed to query config: {e}")))?;

        if let Some(row) = result.next()? {
            let value: String = row.get(0)?;
            let is_secret: bool = row.get(1)?;

            if is_secret {
                // Return secret values wrapped
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
        self.conn.execute(
            r#"
            INSERT INTO config (key, value, is_secret)
            VALUES (?1, ?2, ?3)
            ON CONFLICT (key) DO UPDATE SET
                value = excluded.value,
                is_secret = excluded.is_secret,
                updated_at = CURRENT_TIMESTAMP
            "#,
            [key, value, &is_secret.to_string()],
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
        let mut stmt = self.conn.prepare("SELECT value FROM config WHERE key = ?1 AND is_secret = TRUE")
            .map_err(|e| AppError::Database(format!("Failed to prepare secret query: {e}")))?;

        let mut result = stmt.query([key])
            .map_err(|e| AppError::Database(format!("Failed to query secret: {e}")))?;

        if let Some(row) = result.next()? {
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
        self.conn.execute("DELETE FROM config WHERE key = ?1", [key])
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