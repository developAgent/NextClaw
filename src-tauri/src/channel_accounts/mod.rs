//! Channel account management
//! Manages multiple accounts per channel provider

use crate::utils::error::{AppError, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

/// Channel account configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelAccount {
    pub id: String,
    pub channel_id: String,
    pub name: String,
    pub credentials: String, // JSON-serialized credentials
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Create channel account request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChannelAccountRequest {
    pub channel_id: String,
    pub name: String,
    pub credentials: serde_json::Value,
    pub is_default: bool,
}

/// Update channel account request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateChannelAccountRequest {
    pub id: String,
    pub name: Option<String>,
    pub credentials: Option<serde_json::Value>,
    pub is_default: Option<bool>,
}

impl ChannelAccount {
    pub fn new(channel_id: impl Into<String>, name: impl Into<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            channel_id: channel_id.into(),
            name: name.into(),
            credentials: "{}".to_string(),
            is_default: false,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn with_credentials(mut self, credentials: serde_json::Value) -> Result<Self> {
        self.credentials = serde_json::to_string(&credentials)
            .map_err(|e| AppError::Internal(format!("Failed to serialize credentials: {}", e)))?;
        Ok(self)
    }
}

/// Channel account manager
pub struct ChannelAccountManager {
    db: Arc<tokio::sync::Mutex<Connection>>,
}

impl ChannelAccountManager {
    pub fn new(db: Arc<tokio::sync::Mutex<Connection>>) -> Self {
        Self { db }
    }

    /// Create a new channel account
    pub async fn create_account(&self, request: CreateChannelAccountRequest) -> Result<ChannelAccount> {
        let account = ChannelAccount::new(&request.channel_id, &request.name)
            .with_credentials(request.credentials)?;

        let db = self.db.lock().await;

        // If this is set as default, clear other defaults for this channel
        if request.is_default {
            db.execute(
                "UPDATE channel_accounts SET is_default = 0 WHERE channel_id = ?1",
                params![&request.channel_id],
            ).map_err(|e| AppError::Database(format!("Failed to clear defaults: {}", e)))?;
        }

        let creds_json = serde_json::to_string(&request.credentials)
            .map_err(|e| AppError::Internal(format!("Failed to serialize credentials: {}", e)))?;

        db.execute(
            r#"
            INSERT INTO channel_accounts (id, channel_id, name, credentials, is_default, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                &account.id,
                &account.channel_id,
                &account.name,
                &creds_json,
                request.is_default as i32,
                &account.created_at,
                &account.updated_at,
            ],
        ).map_err(|e| AppError::Database(format!("Failed to create account: {}", e)))?;

        info!("Created channel account: {}", account.name);
        Ok(account)
    }

    /// Get all accounts for a channel
    pub async fn get_accounts_for_channel(&self, channel_id: &str) -> Result<Vec<ChannelAccount>> {
        let db = self.db.lock().await;
        let mut stmt = db.prepare(
            "SELECT id, channel_id, name, credentials, is_default, created_at, updated_at
             FROM channel_accounts WHERE channel_id = ?1 ORDER BY is_default DESC, created_at"
        ).map_err(|e| AppError::Database(format!("Failed to query accounts: {}", e)))?;

        let accounts = stmt
            .query_map(params![channel_id], |row| {
                Ok(ChannelAccount {
                    id: row.get(0)?,
                    channel_id: row.get(1)?,
                    name: row.get(2)?,
                    credentials: row.get(3)?,
                    is_default: row.get::<i32, _>(4)? != 0,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Database(format!("Failed to map accounts: {}", e)))?;

        Ok(accounts)
    }

    /// Get all accounts
    pub async fn get_all_accounts(&self) -> Result<Vec<ChannelAccount>> {
        let db = self.db.lock().await;
        let mut stmt = db.prepare(
            "SELECT id, channel_id, name, credentials, is_default, created_at, updated_at
             FROM channel_accounts ORDER BY channel_id, is_default DESC, created_at"
        ).map_err(|e| AppError::Database(format!("Failed to query accounts: {}", e)))?;

        let accounts = stmt
            .query_map([], |row| {
                Ok(ChannelAccount {
                    id: row.get(0)?,
                    channel_id: row.get(1)?,
                    name: row.get(2)?,
                    credentials: row.get(3)?,
                    is_default: row.get::<i32, _>(4)? != 0,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Database(format!("Failed to map accounts: {}", e)))?;

        Ok(accounts)
    }

    /// Get a specific account
    pub async fn get_account(&self, id: &str) -> Result<Option<ChannelAccount>> {
        let db = self.db.lock().await;
        let mut stmt = db.prepare(
            "SELECT id, channel_id, name, credentials, is_default, created_at, updated_at
             FROM channel_accounts WHERE id = ?1"
        ).map_err(|e| AppError::Database(format!("Failed to query account: {}", e)))?;

        let account = stmt
            .query_row(params![id], |row| {
                Ok(ChannelAccount {
                    id: row.get(0)?,
                    channel_id: row.get(1)?,
                    name: row.get(2)?,
                    credentials: row.get(3)?,
                    is_default: row.get::<i32, _>(4)? != 0,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .ok();

        Ok(account)
    }

    /// Update an account
    pub async fn update_account(&self, request: UpdateChannelAccountRequest) -> Result<ChannelAccount> {
        let db = self.db.lock().await;

        // If setting as default, clear other defaults for the same channel
        if let Some(true) = request.is_default {
            if let Some(account) = self.get_account(&request.id).await? {
                db.execute(
                    "UPDATE channel_accounts SET is_default = 0 WHERE channel_id = ?1",
                    params![&account.channel_id],
                ).map_err(|e| AppError::Database(format!("Failed to clear defaults: {}", e)))?;
            }
        }

        // Build dynamic update query
        let mut updates = vec![];
        let mut params = vec![];

        if let Some(name) = &request.name {
            updates.push("name = ?");
            params.push(name as &dyn rusqlite::ToSql);
        }
        if let Some(creds) = &request.credentials {
            updates.push("credentials = ?");
            let creds_json = serde_json::to_string(creds)
                .map_err(|e| AppError::Internal(format!("Failed to serialize credentials: {}", e)))?;
            params.push(creds_json);
        }
        if let Some(is_default) = request.is_default {
            updates.push("is_default = ?");
            params.push(&(is_default as i32));
        }
        updates.push("updated_at = ?");
        params.push(&chrono::Utc::now().to_rfc3339());
        params.push(&request.id);

        let query = format!(
            "UPDATE channel_accounts SET {} WHERE id = ?{}",
            updates.join(", "),
            params.len() - updates.len() - 1
        );

        let count = db.execute(&query, rusqlite::params_from_iter(params))
            .map_err(|e| AppError::Database(format!("Failed to update account: {}", e)))?;

        if count == 0 {
            return Err(AppError::Validation(format!("Account not found: {}", request.id)));
        }

        self.get_account(&request.id).await?.ok_or_else(|| {
            AppError::Internal("Failed to retrieve updated account".to_string())
        })
    }

    /// Delete an account
    pub async fn delete_account(&self, id: &str) -> Result<()> {
        let db = self.db.lock().await;

        let count = db.execute("DELETE FROM channel_accounts WHERE id = ?1", params![id])
            .map_err(|e| AppError::Database(format!("Failed to delete account: {}", e)))?;

        if count == 0 {
            return Err(AppError::Validation(format!("Account not found: {}", id)));
        }

        info!("Deleted channel account: {}", id);
        Ok(())
    }

    /// Set account as default for its channel
    pub async fn set_default_account(&self, account_id: &str) -> Result<()> {
        let account = self.get_account(account_id).await?
            .ok_or_else(|| AppError::Validation(format!("Account not found: {}", account_id)))?;

        let db = self.db.lock().await;

        db.execute(
            "UPDATE channel_accounts SET is_default = 0 WHERE channel_id = ?1",
            params![&account.channel_id]
        ).map_err(|e| AppError::Database(format!("Failed to clear defaults: {}", e)))?;

        db.execute(
            "UPDATE channel_accounts SET is_default = 1 WHERE id = ?1",
            params![account_id]
        ).map_err(|e| AppError::Database(format!("Failed to set default: {}", e)))?;

        info!("Set default account: {} for channel: {}", account_id, account.channel_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let creds = serde_json::json!({"api_key": "test"});
        let account = ChannelAccount::new("channel-1", "Test Account")
            .with_credentials(creds);

        assert!(account.is_ok());
        let acc = account.unwrap();
        assert_eq!(acc.name, "Test Account");
    }
}