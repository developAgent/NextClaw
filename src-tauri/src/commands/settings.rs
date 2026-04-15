use crate::db::Database;
use crate::utils::config::Config;
use crate::utils::error::Result;
use secrecy::SecretString;
use tauri::State;
use tracing::{debug, info};

/// Get current application configuration as JSON string
///
/// Note: API keys are excluded for security
#[tauri::command]
pub fn get_config(
    db: State<'_, Database>,
) -> Result<String> {
    let config = Config::load()?;
    let api_key_exists = db.get_secret("api_key")?.is_some();

    // Return as JSON string to avoid serialization issues
    let config_json = serde_json::json!({
        "api": {
            "claudeModel": config.api.claude_model,
            "requestTimeoutSecs": config.api.request_timeout_secs,
            "maxRetries": config.api.max_retries,
            "apiKeyConfigured": api_key_exists
        },
        "commands": {
            "timeoutSecs": config.commands.timeout_secs,
            "allowShell": config.commands.allow_shell,
            "whitelist": config.commands.whitelist,
            "blacklist": config.commands.blacklist,
            "sandboxPath": config.commands.sandbox_path,
            "requireConfirmation": config.commands.require_confirmation
        },
        "ui": {
            "theme": config.ui.theme,
            "language": config.ui.language,
            "fontSize": config.ui.font_size,
            "showTimestamps": config.ui.show_timestamps,
            "maxHistory": config.ui.max_history
        }
    });

    Ok(config_json.to_string())
}

/// Update application configuration
#[tauri::command]
pub fn update_config(config_update: String) -> Result<()> {
    let update: ConfigUpdate = serde_json::from_str(&config_update)
        .map_err(|e| crate::utils::error::AppError::Validation(format!("Invalid config update: {e}")))?;

    let mut current_config = Config::load()?;

    // Update only provided fields
    if let Some(model) = update.claude_model {
        current_config.api.claude_model = model;
    }
    if let Some(timeout) = update.request_timeout_secs {
        current_config.api.request_timeout_secs = timeout;
    }
    if let Some(max_retries) = update.max_retries {
        current_config.api.max_retries = max_retries;
    }
    if let Some(timeout_secs) = update.timeout_secs {
        current_config.commands.timeout_secs = timeout_secs;
    }
    if let Some(allow_shell) = update.allow_shell {
        current_config.commands.allow_shell = allow_shell;
    }
    if let Some(whitelist) = update.whitelist {
        current_config.commands.whitelist = whitelist;
    }
    if let Some(blacklist) = update.blacklist {
        current_config.commands.blacklist = blacklist;
    }
    if let Some(sandbox_path) = update.sandbox_path {
        current_config.commands.sandbox_path = sandbox_path.into();
    }
    if let Some(require_confirmation) = update.require_confirmation {
        current_config.commands.require_confirmation = require_confirmation;
    }
    if let Some(theme) = update.theme {
        current_config.ui.theme = theme;
    }
    if let Some(language) = update.language {
        current_config.ui.language = language;
    }
    if let Some(font_size) = update.font_size {
        current_config.ui.font_size = font_size;
    }
    if let Some(show_timestamps) = update.show_timestamps {
        current_config.ui.show_timestamps = show_timestamps;
    }
    if let Some(max_history) = update.max_history {
        current_config.ui.max_history = max_history;
    }

    current_config.save()?;
    info!("Configuration updated");
    Ok(())
}

/// Set Claude API key
///
/// The key is stored encrypted in the database
#[tauri::command]
pub fn set_api_key(
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
pub fn delete_api_key(
    db: State<'_, Database>,
) -> Result<()> {
    db.delete_config("api_key")?;
    info!("API key deleted");
    Ok(())
}

// Data structures for config updates

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigUpdate {
    pub claude_model: Option<String>,
    pub request_timeout_secs: Option<u64>,
    pub max_retries: Option<u32>,
    pub timeout_secs: Option<u64>,
    pub allow_shell: Option<bool>,
    pub whitelist: Option<Vec<String>>,
    pub blacklist: Option<Vec<String>>,
    pub sandbox_path: Option<String>,
    pub require_confirmation: Option<bool>,
    pub theme: Option<String>,
    pub language: Option<String>,
    pub font_size: Option<u16>,
    pub show_timestamps: Option<bool>,
    pub max_history: Option<usize>,
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
            timeout_secs: None,
            allow_shell: None,
            whitelist: None,
            blacklist: None,
            sandbox_path: None,
            require_confirmation: None,
            theme: None,
            language: None,
            font_size: None,
            show_timestamps: None,
            max_history: None,
        };

        assert!(update.claude_model.is_some());
        assert!(update.request_timeout_secs.is_none());
    }
}