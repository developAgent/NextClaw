use super::types::{Channel, ChannelConfig, ChannelHealth};
use crate::db::Database;
use crate::utils::error::{AppError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Channel manager for managing multiple AI channels
pub struct ChannelManager {
    db: Arc<Database>,
    channels: Arc<RwLock<HashMap<String, Channel>>>,
    config: Arc<RwLock<ChannelConfig>>,
}

impl ChannelManager {
    /// Create a new channel manager
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            channels: Arc::new(RwLock::new(HashMap::new())),
            config: Arc::new(RwLock::new(ChannelConfig::default())),
        }
    }

    /// Initialize channel manager by loading channels from database (blocking version for setup)
    pub fn initialize_blocking(&self) -> Result<()> {
        info!("Initializing channel manager");

        let conn = self.db.conn();
        let conn_guard = conn.blocking_lock();
        let mut stmt = conn_guard
            .prepare(
                "SELECT id, provider_type, name, config, enabled, priority, health_status, created_at, updated_at FROM channels",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let channels_list = stmt
            .query_map([], |row| {
                let health_status = match row.get::<_, String>(6)?.as_str() {
                    "healthy" => ChannelHealth::Healthy,
                    "degraded" => ChannelHealth::Degraded,
                    "unhealthy" => ChannelHealth::Unhealthy,
                    _ => ChannelHealth::Unknown,
                };

                let config_text: String = row.get(3)?;
                let config =
                    serde_json::from_str(&config_text).unwrap_or_else(|_| serde_json::json!({}));

                Ok(Channel {
                    id: row.get(0)?,
                    provider_type: row.get(1)?,
                    name: row.get(2)?,
                    config,
                    enabled: row.get::<_, i32>(4)? != 0,
                    priority: row.get(5)?,
                    health_status,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut channels = self.channels.blocking_write();
        channels.clear();
        for channel in channels_list {
            channels.insert(channel.id.clone(), channel);
        }

        info!("Loaded {} channels from database", channels.len());
        Ok(())
    }

    /// Get all channels
    pub async fn get_all_channels(&self) -> Result<Vec<Channel>> {
        let channels = self.channels.read().await;
        let mut channels_vec: Vec<Channel> = channels.values().cloned().collect();
        channels_vec.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(channels_vec)
    }

    /// Get a specific channel by ID
    pub async fn get_channel(&self, id: &str) -> Result<Option<Channel>> {
        let channels = self.channels.read().await;
        Ok(channels.get(id).cloned())
    }

    /// Get the best available channel
    pub async fn get_best_channel(&self) -> Result<Option<Channel>> {
        let channels = self.get_all_channels().await?;
        let enabled_channels: Vec<&Channel> = channels.iter().filter(|c| c.enabled).collect();

        if enabled_channels.is_empty() {
            warn!("No enabled channels available");
            return Ok(None);
        }

        for channel in &enabled_channels {
            if channel.health_status == ChannelHealth::Healthy {
                debug!("Selected healthy channel: {}", channel.name);
                return Ok(Some((*channel).clone()));
            }
        }

        warn!("No healthy channels, using first enabled channel");
        Ok(enabled_channels.first().map(|c| (*c).clone()))
    }

    /// Add a new channel
    pub async fn add_channel(&self, mut channel: Channel) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        if channel.created_at.is_empty() {
            channel.created_at = now.clone();
        }
        channel.updated_at = now;

        let config_json = serde_json::to_string(&channel.config).map_err(|e| {
            AppError::Validation(format!("Failed to serialize channel config: {e}"))
        })?;

        let conn = self.db.conn();
        let conn_guard = conn.blocking_lock();
        conn_guard
            .execute(
                r#"
                INSERT INTO channels (id, provider_type, name, config, enabled, priority, health_status, created_at, updated_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
                rusqlite::params![
                    &channel.id,
                    &channel.provider_type,
                    &channel.name,
                    &config_json,
                    channel.enabled as i32,
                    &channel.priority,
                    &channel.health_status.to_string(),
                    &channel.created_at,
                    &channel.updated_at,
                ],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut channels = self.channels.write().await;
        let channel_name = channel.name.clone();
        channels.insert(channel.id.clone(), channel);

        info!("Added channel: {}", channel_name);
        Ok(())
    }

    /// Update an existing channel
    pub async fn update_channel(&self, mut channel: Channel) -> Result<()> {
        channel.touch();

        let config_json = serde_json::to_string(&channel.config).map_err(|e| {
            AppError::Validation(format!("Failed to serialize channel config: {e}"))
        })?;

        let conn = self.db.conn();
        let conn_guard = conn.blocking_lock();
        conn_guard
            .execute(
                r#"
                UPDATE channels
                SET provider_type = ?1, name = ?2, config = ?3, enabled = ?4, priority = ?5, health_status = ?6, updated_at = ?7
                WHERE id = ?8
                "#,
                rusqlite::params![
                    &channel.provider_type,
                    &channel.name,
                    &config_json,
                    channel.enabled as i32,
                    &channel.priority,
                    &channel.health_status.to_string(),
                    &channel.updated_at,
                    &channel.id,
                ],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut channels = self.channels.write().await;
        let channel_name = channel.name.clone();
        channels.insert(channel.id.clone(), channel);

        info!("Updated channel: {}", channel_name);
        Ok(())
    }

    /// Delete a channel
    pub async fn delete_channel(&self, id: &str) -> Result<()> {
        let conn = self.db.conn();
        let conn_guard = conn.blocking_lock();
        conn_guard
            .execute("DELETE FROM channels WHERE id = ?1", rusqlite::params![id])
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut channels = self.channels.write().await;
        channels.remove(id);

        info!("Deleted channel: {}", id);
        Ok(())
    }

    /// Set default channel
    pub async fn set_default_channel(&self, id: &str) -> Result<()> {
        let mut config = self.config.write().await;
        config.default_channel = Some(id.to_string());
        Ok(())
    }

    /// Get default channel
    pub async fn get_default_channel(&self) -> Result<Option<Channel>> {
        let config = self.config.read().await;
        if let Some(id) = &config.default_channel {
            self.get_channel(id).await
        } else {
            self.get_best_channel().await
        }
    }

    /// Check channel health
    pub async fn check_channel_health(&self, id: &str) -> Result<ChannelHealth> {
        let channel = self
            .get_channel(id)
            .await?
            .ok_or_else(|| AppError::Validation(format!("Channel not found: {}", id)))?;

        let health = if channel.config.is_null() {
            ChannelHealth::Degraded
        } else {
            ChannelHealth::Healthy
        };

        if let Some(mut ch) = self.get_channel(id).await? {
            ch.health_status = health;
            ch.touch();
            self.update_channel(ch).await?;
        }

        Ok(health)
    }

    /// Get channel configuration
    pub async fn get_config(&self) -> ChannelConfig {
        self.config.read().await.clone()
    }

    /// Update channel configuration
    pub async fn update_config(&self, config: ChannelConfig) -> Result<()> {
        let mut config_mut = self.config.write().await;
        *config_mut = config;
        Ok(())
    }
}
