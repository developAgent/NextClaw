use super::types::{Channel, ChannelConfig, ChannelHealth, ChannelProvider};
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

        // Create a channel from database row data
        fn create_channel_from_row(row: &rusqlite::Row) -> Result<Channel> {
            let id: String = row.get(0).map_err(|e| AppError::Database(e.to_string()))?;
            let name: String = row.get(1).map_err(|e| AppError::Database(e.to_string()))?;
            let provider_str: String = row.get(2).map_err(|e| AppError::Database(e.to_string()))?;
            let model: String = row.get(3).map_err(|e| AppError::Database(e.to_string()))?;
            let api_key: Option<String> = row.get(4).map_err(|e| AppError::Database(e.to_string()))?;
            let api_base: Option<String> = row.get(5).map_err(|e| AppError::Database(e.to_string()))?;
            let priority: i32 = row.get(6).map_err(|e| AppError::Database(e.to_string()))?;
            let enabled: i32 = row.get(7).map_err(|e| AppError::Database(e.to_string()))?;
            let health_str: String = row.get(8).map_err(|e| AppError::Database(e.to_string()))?;
            let last_used: Option<i64> = row.get(9).map_err(|e| AppError::Database(e.to_string()))?;
            let created_at: i64 = row.get(10).map_err(|e| AppError::Database(e.to_string()))?;
            let updated_at: i64 = row.get(11).map_err(|e| AppError::Database(e.to_string()))?;

            let health = match health_str.as_str() {
                "healthy" => ChannelHealth::Healthy,
                "degraded" => ChannelHealth::Degraded,
                "unhealthy" => ChannelHealth::Unhealthy,
                _ => ChannelHealth::Unknown,
            };

            Ok(Channel {
                id,
                name,
                provider: ChannelProvider::from(provider_str.as_str()),
                model,
                api_key,
                api_base,
                priority,
                enabled: enabled != 0,
                health_status: health,
                last_used,
                created_at,
                updated_at,
            })
        }

        // Load channels from database
        let conn = self.db.conn();
        let conn_guard = conn.blocking_lock();
        let mut stmt = conn_guard
            .prepare("SELECT id, name, provider, model, api_key, api_base, priority, enabled, health_status, last_used, created_at, updated_at FROM channels")
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut channels_list = Vec::new();
        let mut rows = stmt.query([])
            .map_err(|e| AppError::Database(e.to_string()))?;

        while let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
            let channel = create_channel_from_row(row)?;
            channels_list.push(channel);
        }

        // Store channels using blocking lock
        let mut channels = self.channels.blocking_write();
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
        let enabled_channels: Vec<&Channel> = channels.iter()
            .filter(|c| c.enabled)
            .collect();

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
    pub async fn add_channel(&self, channel: Channel) -> Result<()> {
        let conn = self.db.conn();
        let conn_guard = conn.blocking_lock();
        conn_guard.execute(
            r#"
            INSERT INTO channels (id, name, provider, model, api_key, api_base, priority, enabled, health_status, last_used, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            rusqlite::params![
                &channel.id,
                &channel.name,
                &channel.provider.to_string(),
                &channel.model,
                &channel.api_key,
                &channel.api_base,
                &channel.priority,
                channel.enabled as i32,
                &channel.health_status.to_string(),
                &channel.last_used,
                &channel.created_at,
                &channel.updated_at,
            ],
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let mut channels = self.channels.write().await;
        let channel_name = channel.name.clone();
        channels.insert(channel.id.clone(), channel);

        info!("Added channel: {}", channel_name);
        Ok(())
    }

    /// Update an existing channel
    pub async fn update_channel(&self, channel: Channel) -> Result<()> {
        let conn = self.db.conn();
        let conn_guard = conn.blocking_lock();
        conn_guard.execute(
            r#"
            UPDATE channels
            SET name = ?1, provider = ?2, model = ?3, api_key = ?4, api_base = ?5, priority = ?6, enabled = ?7, health_status = ?8, last_used = ?9, updated_at = ?10
            WHERE id = ?11
            "#,
            rusqlite::params![
                &channel.name,
                &channel.provider.to_string(),
                &channel.model,
                &channel.api_key,
                &channel.api_base,
                &channel.priority,
                channel.enabled as i32,
                &channel.health_status.to_string(),
                &channel.last_used,
                channel.updated_at,
                &channel.id,
            ],
        ).map_err(|e| AppError::Database(e.to_string()))?;

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
        conn_guard.execute("DELETE FROM channels WHERE id = ?1", rusqlite::params![id])
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
        let channel = self.get_channel(id).await?
            .ok_or_else(|| AppError::Validation(format!("Channel not found: {}", id)))?;

        let health = if channel.api_key.is_some() {
            ChannelHealth::Healthy
        } else {
            ChannelHealth::Degraded
        };

        if let Some(mut ch) = self.get_channel(id).await? {
            ch.update_health(health);
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