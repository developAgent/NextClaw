use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Channel provider types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelProvider {
    Claude,
    OpenAI,
    Gemini,
}

impl std::fmt::Display for ChannelProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Claude => write!(f, "claude"),
            Self::OpenAI => write!(f, "openai"),
            Self::Gemini => write!(f, "gemini"),
        }
    }
}

impl From<&str> for ChannelProvider {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "claude" => Self::Claude,
            "openai" => Self::OpenAI,
            "gemini" => Self::Gemini,
            _ => Self::Claude,
        }
    }
}

/// Channel health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelHealth {
    Unknown,
    Healthy,
    Degraded,
    Unhealthy,
}

impl std::fmt::Display for ChannelHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "unknown"),
            Self::Healthy => write!(f, "healthy"),
            Self::Degraded => write!(f, "degraded"),
            Self::Unhealthy => write!(f, "unhealthy"),
        }
    }
}

/// Channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub provider: ChannelProvider,
    pub model: String,
    pub api_key: Option<String>,
    pub api_base: Option<String>,
    pub priority: i32,
    pub enabled: bool,
    pub health_status: ChannelHealth,
    pub last_used: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Channel {
    pub fn new(
        name: String,
        provider: ChannelProvider,
        model: String,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            provider,
            model,
            api_key: None,
            api_base: None,
            priority: 0,
            enabled: true,
            health_status: ChannelHealth::Unknown,
            last_used: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    pub fn with_api_base(mut self, api_base: String) -> Self {
        self.api_base = Some(api_base);
        self
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn update_health(&mut self, status: ChannelHealth) {
        self.health_status = status;
        self.last_used = Some(chrono::Utc::now().timestamp());
        self.updated_at = chrono::Utc::now().timestamp();
    }
}

/// Channel configuration for Tauri
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    pub default_channel: Option<String>,
    pub failover_enabled: bool,
    pub health_check_interval: u64,
    pub max_retries: u32,
    pub timeout_secs: u64,
}

impl Default for ChannelConfig {
    fn default() -> Self {
        Self {
            default_channel: None,
            failover_enabled: true,
            health_check_interval: 300,
            max_retries: 3,
            timeout_secs: 30,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_creation() {
        let channel = Channel::new(
            "My Claude Channel".to_string(),
            ChannelProvider::Claude,
            "claude-3-sonnet-20240229".to_string(),
        );

        assert_eq!(channel.name, "My Claude Channel");
        assert_eq!(channel.provider, ChannelProvider::Claude);
        assert_eq!(channel.model, "claude-3-sonnet-20240229");
        assert!(channel.enabled);
    }

    #[test]
    fn test_channel_with_api_key() {
        let channel = Channel::new(
            "OpenAI Channel".to_string(),
            ChannelProvider::OpenAI,
            "gpt-4".to_string(),
        )
        .with_api_key("sk-test".to_string());

        assert_eq!(channel.api_key, Some("sk-test".to_string()));
    }

    #[test]
    fn test_channel_priority() {
        let mut channels = vec![
            Channel::new("Channel 1".to_string(), ChannelProvider::Claude, "claude-3-opus".to_string())
                .with_priority(1),
            Channel::new("Channel 2".to_string(), ChannelProvider::Claude, "claude-3-sonnet".to_string())
                .with_priority(2),
            Channel::new("Channel 3".to_string(), ChannelProvider::Claude, "claude-3-haiku".to_string())
                .with_priority(0),
        ];

        channels.sort_by(|a, b| b.priority.cmp(&a.priority));

        assert_eq!(channels[0].priority, 2);
        assert_eq!(channels[1].priority, 1);
        assert_eq!(channels[2].priority, 0);
    }
}
