//! Channel account management commands
//! Provides Tauri commands for managing channel accounts

use crate::channel_accounts::{ChannelAccountManager, CreateChannelAccountRequest, UpdateChannelAccountRequest};
use crate::utils::error::Result;
use tauri::State;
use std::sync::Arc;

/// Create a new channel account
#[tauri::command]
pub async fn create_channel_account(
    request: CreateChannelAccountRequest,
    manager: State<'_, Arc<ChannelAccountManager>>,
) -> Result<crate::channel_accounts::ChannelAccount> {
    manager.create_account(request).await
}

/// Get all accounts
#[tauri::command]
pub async fn get_all_channel_accounts(
    manager: State<'_, Arc<ChannelAccountManager>>,
) -> Result<Vec<crate::channel_accounts::ChannelAccount>> {
    manager.get_all_accounts().await
}

/// Get accounts for a specific channel
#[tauri::command]
pub async fn get_channel_accounts(
    channel_id: String,
    manager: State<'_, Arc<ChannelAccountManager>>,
) -> Result<Vec<crate::channel_accounts::ChannelAccount>> {
    manager.get_accounts_for_channel(&channel_id).await
}

/// Get a specific account
#[tauri::command]
pub async fn get_channel_account(
    id: String,
    manager: State<'_, Arc<ChannelAccountManager>>,
) -> Result<Option<crate::channel_accounts::ChannelAccount>> {
    manager.get_account(&id).await
}

/// Update an account
#[tauri::command]
pub async fn update_channel_account(
    request: UpdateChannelAccountRequest,
    manager: State<'_, Arc<ChannelAccountManager>>,
) -> Result<crate::channel_accounts::ChannelAccount> {
    manager.update_account(request).await
}

/// Delete an account
#[tauri::command]
pub async fn delete_channel_account(
    id: String,
    manager: State<'_, Arc<ChannelAccountManager>>,
) -> Result<()> {
    manager.delete_account(&id).await
}

/// Set account as default for its channel
#[tauri::command]
pub async fn set_default_channel_account(
    account_id: String,
    manager: State<'_, Arc<ChannelAccountManager>>,
) -> Result<()> {
    manager.set_default_account(&account_id).await
}