use serde::{Deserialize, Serialize};

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

fn default_channel_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

fn default_channel_enabled() -> bool {
    true
}

fn default_channel_priority() -> i32 {
    0
}

fn default_channel_health() -> ChannelHealth {
    ChannelHealth::Unknown
}

fn default_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    #[serde(default = "default_channel_id")]
    pub id: String,
    pub provider_type: String,
    pub name: String,
    #[serde(default)]
    pub config: serde_json::Value,
    #[serde(default = "default_channel_enabled")]
    pub enabled: bool,
    #[serde(default = "default_channel_priority")]
    pub priority: i32,
    #[serde(default = "default_channel_health")]
    pub health_status: ChannelHealth,
    #[serde(default = "default_timestamp")]
    pub created_at: String,
    #[serde(default = "default_timestamp")]
    pub updated_at: String,
}

impl Channel {
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now().to_rfc3339();
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
    fn test_channel_creation_defaults() {
        let channel: Channel = serde_json::from_value(serde_json::json!({
            "provider_type": "openai",
            "name": "My OpenAI Channel",
            "config": {"base_url": "https://api.openai.com"}
        }))
        .expect("channel should deserialize");

        assert_eq!(channel.name, "My OpenAI Channel");
        assert_eq!(channel.provider_type, "openai");
        assert!(channel.enabled);
        assert_eq!(channel.priority, 0);
        assert_eq!(channel.health_status, ChannelHealth::Unknown);
    }

    #[test]
    fn test_channel_priority_sorting() {
        let mut channels = vec![
            Channel {
                id: "1".to_string(),
                provider_type: "openai".to_string(),
                name: "Channel 1".to_string(),
                config: serde_json::json!({}),
                enabled: true,
                priority: 1,
                health_status: ChannelHealth::Unknown,
                created_at: default_timestamp(),
                updated_at: default_timestamp(),
            },
            Channel {
                id: "2".to_string(),
                provider_type: "anthropic".to_string(),
                name: "Channel 2".to_string(),
                config: serde_json::json!({}),
                enabled: true,
                priority: 2,
                health_status: ChannelHealth::Unknown,
                created_at: default_timestamp(),
                updated_at: default_timestamp(),
            },
            Channel {
                id: "3".to_string(),
                provider_type: "custom".to_string(),
                name: "Channel 3".to_string(),
                config: serde_json::json!({}),
                enabled: true,
                priority: 0,
                health_status: ChannelHealth::Unknown,
                created_at: default_timestamp(),
                updated_at: default_timestamp(),
            },
        ];

        channels.sort_by(|a, b| b.priority.cmp(&a.priority));

        assert_eq!(channels[0].priority, 2);
        assert_eq!(channels[1].priority, 1);
        assert_eq!(channels[2].priority, 0);
    }
}
