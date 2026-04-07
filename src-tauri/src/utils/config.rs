use crate::utils::error::{AppError, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::info;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Data directory for storing application data
    pub data_dir: PathBuf,
    /// API configuration
    pub api: ApiConfig,
    /// Command execution configuration
    pub commands: CommandConfig,
    /// UI configuration
    pub ui: UiConfig,
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Claude API key (stored in database, not this file)
    pub claude_model: String,
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
    /// Maximum retries for failed requests
    pub max_retries: u32,
}

/// Command execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandConfig {
    /// Maximum command execution time in seconds
    pub timeout_secs: u64,
    /// Allow shell commands
    pub allow_shell: bool,
    /// Command whitelist (empty means all allowed)
    pub whitelist: Vec<String>,
    /// Blacklist (commands that are always blocked)
    pub blacklist: Vec<String>,
    /// Path sandbox (empty means no restriction)
    pub sandbox_path: PathBuf,
    /// Require confirmation for dangerous commands
    pub require_confirmation: bool,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Theme (light, dark, auto)
    pub theme: String,
    /// Font size
    pub font_size: u16,
    /// Show timestamps in chat
    pub show_timestamps: bool,
    /// Maximum message history length
    pub max_history: usize,
}

impl Default for Config {
    fn default() -> Self {
        let project_dirs = ProjectDirs::from("com", "ceoclaw", "CEOClaw")
            .expect("Failed to get project directories");

        let data_dir = project_dirs.data_dir().to_path_buf();

        Self {
            data_dir,
            api: ApiConfig {
                claude_model: "claude-3-sonnet-20240229".to_string(),
                request_timeout_secs: 120,
                max_retries: 3,
            },
            commands: CommandConfig {
                timeout_secs: 300,
                allow_shell: true,
                whitelist: vec![],
                blacklist: vec![
                    "rm -rf /".to_string(),
                    "rm -rf /*".to_string(),
                    "dd if=/dev/zero".to_string(),
                    "mkfs".to_string(),
                    "format C:".to_string(),
                    "shutdown /p".to_string(),
                    "shutdown -h now".to_string(),
                ],
                sandbox_path: PathBuf::new(), // No sandbox by default
                require_confirmation: true,
            },
            ui: UiConfig {
                theme: "dark".to_string(),
                font_size: 14,
                show_timestamps: true,
                max_history: 1000,
            },
        }
    }
}

impl Config {
    /// Load configuration from disk or return defaults
    ///
    /// # Errors
    ///
    /// Returns an error if configuration file is corrupted
    pub fn load() -> Result<Self> {
        let project_dirs = ProjectDirs::from("com", "ceoclaw", "CEOClaw")
            .ok_or_else(|| AppError::Config("Failed to get project directories".to_string()))?;

        let config_dir = project_dirs.config_dir();
        std::fs::create_dir_all(config_dir)
            .map_err(|e| AppError::Config(format!("Failed to create config directory: {e}")))?;

        let config_path = config_dir.join("config.toml");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)
                .map_err(|e| AppError::Config(format!("Failed to read config file: {e}")))?;

            toml::from_str(&content)
                .map_err(|e| AppError::Config(format!("Failed to parse config: {e}")))
        } else {
            let config = Self::default();
            config.save()?;
            info!("Created default configuration");
            Ok(config)
        }
    }

    /// Save configuration to disk
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails
    pub fn save(&self) -> Result<()> {
        let project_dirs = ProjectDirs::from("com", "ceoclaw", "CEOClaw")
            .ok_or_else(|| AppError::Config("Failed to get project directories".to_string()))?;

        let config_dir = project_dirs.config_dir();
        std::fs::create_dir_all(config_dir)
            .map_err(|e| AppError::Config(format!("Failed to create config directory: {e}")))?;

        let config_path = config_dir.join("config.toml");
        let content = toml::to_string_pretty(self)
            .map_err(|e| AppError::Config(format!("Failed to serialize config: {e}")))?;

        std::fs::write(&config_path, content)
            .map_err(|e| AppError::Config(format!("Failed to write config file: {e}")))?;

        info!("Configuration saved to {:?}", config_path);
        Ok(())
    }

    /// Ensure data directory exists
    ///
    /// # Errors
    ///
    /// Returns an error if directory creation fails
    pub fn ensure_data_dir(&self) -> Result<()> {
        std::fs::create_dir_all(&self.data_dir)
            .map_err(|e| AppError::Config(format!("Failed to create data directory: {e}")))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.api.claude_model, "claude-3-sonnet-20240229");
        assert_eq!(config.api.request_timeout_secs, 120);
        assert!(config.commands.require_confirmation);
        assert!(!config.commands.blacklist.is_empty());
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let parsed: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.api.claude_model, parsed.api.claude_model);
    }
}