use crate::channels::{Channel, ChannelConfig, ChannelHealth, ChannelManager};
use crate::utils::error::Result;
use secrecy::SecretString;
use tauri::State;
use uuid::Uuid;
use tracing::{debug, info};

/// Get all channels
#[tauri::command]
pub async fn get_all_channels(
    manager: State<'_, ChannelManager>,
) -> Result<Vec<Channel>> {
    manager.get_all_channels().await
}

/// Get a specific channel
#[tauri::command]
pub async fn get_channel(
    id: String,
    manager: State<'_, ChannelManager>,
) -> Result<Option<Channel>> {
    manager.get_channel(&id).await
}

/// Add a new channel
#[tauri::command]
pub async fn add_channel(
    name: String,
    provider: String,
    model: String,
    api_key: Option<String>,
    api_base: Option<String>,
    priority: Option<i32>,
    manager: State<'_, ChannelManager>,
) -> Result<Channel> {
    let provider = match provider.to_lowercase().as_str() {
        "claude" => crate::channels::ChannelProvider::Claude,
        "openai" => crate::channels::ChannelProvider::OpenAI,
        "gemini" => crate::channels::ChannelProvider::Gemini,
        _ => return Err(crate::utils::error::AppError::Validation(
            format!("Unknown provider: {}", provider)
        )),
    };

    let mut channel = Channel::new(name, provider, model);

    if let Some(key) = api_key {
        channel = channel.with_api_key(key);
    }

    if let Some(base) = api_base {
        channel = channel.with_api_base(base);
    }

    if let Some(prio) = priority {
        channel = channel.with_priority(prio);
    }

    manager.add_channel(channel.clone()).await?;
    info!("Added channel: {}", channel.name);
    Ok(channel)
}

/// Update an existing channel
#[tauri::command]
pub async fn update_channel(
    id: String,
    name: Option<String>,
    model: Option<String>,
    api_key: Option<String>,
    api_base: Option<String>,
    priority: Option<i32>,
    enabled: Option<bool>,
    manager: State<'_, ChannelManager>,
) -> Result<Channel> {
    let mut channel = manager.get_channel(&id).await?
        .ok_or_else(|| crate::utils::error::AppError::Validation(format!("Channel not found: {}", id)))?;

    if let Some(n) = name {
        channel.name = n;
    }
    if let Some(m) = model {
        channel.model = m;
    }
    if let Some(key) = api_key {
        channel.api_key = Some(key);
    }
    if let Some(base) = api_base {
        channel.api_base = Some(base);
    }
    if let Some(prio) = priority {
        channel.priority = prio;
    }
    if let Some(en) = enabled {
        channel.enabled = en;
    }
    channel.updated_at = chrono::Utc::now().timestamp();

    manager.update_channel(channel.clone()).await?;
    info!("Updated channel: {}", channel.name);
    Ok(channel)
}

/// Delete a channel
#[tauri::command]
pub async fn delete_channel(
    id: String,
    manager: State<'_, ChannelManager>,
) -> Result<()> {
    manager.delete_channel(&id).await?;
    info!("Deleted channel: {}", id);
    Ok(())
}

/// Set default channel
#[tauri::command]
pub async fn set_default_channel(
    id: String,
    manager: State<'_, ChannelManager>,
) -> Result<()> {
    manager.set_default_channel(&id).await?;
    Ok(())
}

/// Get default channel
#[tauri::command]
pub async fn get_default_channel(
    manager: State<'_, ChannelManager>,
) -> Result<Option<Channel>> {
    manager.get_default_channel().await
}

/// Check channel health
#[tauri::command]
pub async fn check_channel_health(
    id: String,
    manager: State<'_, ChannelManager>,
) -> Result<ChannelHealth> {
    manager.check_channel_health(&id).await
}

/// Get channel configuration
#[tauri::command]
pub async fn get_channel_config(
    manager: State<'_, ChannelManager>,
) -> Result<ChannelConfig> {
    Ok(manager.get_config().await)
}

/// Update channel configuration
#[tauri::command]
pub async fn update_channel_config(
    config: ChannelConfig,
    manager: State<'_, ChannelManager>,
) -> Result<()> {
    manager.update_config(config).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_commands() {
        // Test channel commands
    }
}