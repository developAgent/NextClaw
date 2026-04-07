use crate::channels::{Channel, ChannelConfig, ChannelHealth, ChannelManager, ChannelProvider};
use crate::utils::error::Result;
use tauri::State;
use tracing::info;

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
    channel: Channel,
    manager: State<'_, ChannelManager>,
) -> Result<()> {
    manager.add_channel(channel.clone()).await?;
    info!("Added channel: {}", channel.name);
    Ok(())
}

/// Update an existing channel
#[tauri::command]
pub async fn update_channel(
    channel: Channel,
    manager: State<'_, ChannelManager>,
) -> Result<()> {
    manager.update_channel(channel.clone()).await?;
    info!("Updated channel: {}", channel.name);
    Ok(())
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