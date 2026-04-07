use crate::db::Database;
use crate::utils::config::Config;
use crate::utils::error::Result;
use secrecy::SecretString;
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, info};

/// Get current application configuration
///
/// Note: API keys are excluded for security
#[tauri::command]
pub async fn get_config(
    db: State<'_, Database>,
) -> Result<ConfigSafe> {
    let config = Config::load()?;
    let api_key_exists = db.get_secret("api_key")?.is_some();

    Ok(ConfigSafe {
        api: ApiConfigSafe {
            claude_model: config.api.claude_model,
            request_timeout_secs: config.api.request_timeout_secs,
            max_retries: config.api.max_retries,
            api_key_configured: api_key_exists,
        },
        commands: config.commands.clone(),
        ui: config.ui.clone(),
    })
}

/// Update application configuration
#[tauri::command]
pub async fn update_config(
    config: ConfigUpdate,
) -> Result<()> {
    let mut current_config = Config::load()?;

    // Update only provided fields
    if let Some(model) = config.claude_model {
        current_config.api.claude_model = model;
    }
    if let Some(timeout) = config.request_timeout_secs {
        current_config.api.request_timeout_secs = timeout;
    }
    if let Some(max_retries) = config.max_retries {
        current_config.api.max_retries = max_retries;
    }
    if let Some(whitelist) = config.whitelist {
        current_config.commands.whitelist = whitelist;
    }
    if let Some(blacklist) = config.blacklist {
        current_config.commands.blacklist = blacklist;
    }
    if let Some(sandbox_path) = config.sandbox_path {
        current_config.commands.sandbox_path = sandbox_path.into();
    }
    if let Some(require_confirmation) = config.require_confirmation {
        current_config.commands.require_confirmation = require_confirmation;
    }

    current_config.save()?;
    info!("Configuration updated");
    Ok(())
}

/// Set Claude API key
///
/// The key is stored encrypted in the database
#[tauri::command]
pub async fn set_api_key(
    api_key: String,
    db: State<'_, Database>,
) -> Result<()> {
    let secret = SecretString::new(api_key);
    db.set_secret("api_key", secret)?;
    info!("API key updated");
    Ok(())
}

/// Delete Claude API key
#[tauri::command]
pub async fn delete_api_key(
    db: State<'_, Database>,
) -> Result<()> {
    db.delete_config("api_key")?;
    info!("API key deleted");
    Ok(())
}

// Data structures for Tauri commands

/// Configuration safe for transmission (no secrets)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSafe {
    pub api: ApiConfigSafe,
    pub commands: crate::utils::config::CommandConfig,
    pub ui: crate::utils::config::UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfigSafe {
    pub claude_model: String,
    pub request_timeout_secs: u64,
    pub max_retries: u32,
    pub api_key_configured: bool,
}

/// Configuration update (only updatable fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigUpdate {
    pub claude_model: Option<String>,
    pub request_timeout_secs: Option<u64>,
    pub max_retries: Option<u32>,
    pub whitelist: Option<Vec<String>>,
    pub blacklist: Option<Vec<String>>,
    pub sandbox_path: Option<String>,
    pub require_confirmation: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_update_partial() {
        let update = ConfigUpdate {
            claude_model: Some("claude-3-opus-20240229".to_string()),
            request_timeout_secs: None,
            max_retries: None,
            whitelist: None,
            blacklist: None,
            sandbox_path: None,
            require_confirmation: None,
        };

        assert!(update.claude_model.is_some());
        assert!(update.request_timeout_secs.is_none());
    }
}