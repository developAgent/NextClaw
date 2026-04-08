use crate::skills::permissions::{Permission, PermissionChecker, PermissionError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, warn};

/// Sandbox configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Enable file system access
    pub enable_filesystem: bool,
    /// Allowed file system paths
    pub allowed_paths: Vec<PathBuf>,
    /// Enable network access
    pub enable_network: bool,
    /// Allowed network domains
    pub allowed_domains: Vec<String>,
    /// Enable system command execution
    pub enable_system_exec: bool,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: Option<u64>,
    /// Maximum memory usage in bytes
    pub max_memory_mb: Option<u64>,
    /// Enable clipboard access
    pub enable_clipboard: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enable_filesystem: false,
            allowed_paths: Vec::new(),
            enable_network: false,
            allowed_domains: Vec::new(),
            enable_system_exec: false,
            max_execution_time_ms: Some(30000), // 30 seconds default
            max_memory_mb: Some(128), // 128MB default
            enable_clipboard: false,
        }
    }
}

impl SandboxConfig {
    /// Create a restrictive sandbox configuration
    pub fn restrictive() -> Self {
        Self::default()
    }

    /// Create a permissive sandbox configuration
    pub fn permissive() -> Self {
        Self {
            enable_filesystem: true,
            allowed_paths: vec![PathBuf::from("/")],
            enable_network: true,
            allowed_domains: Vec::new(), // All domains
            enable_system_exec: false,
            max_execution_time_ms: Some(60000), // 60 seconds
            max_memory_mb: Some(256), // 256MB
            enable_clipboard: true,
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if let Some(max_time) = self.max_execution_time_ms {
            if max_time == 0 {
                return Err("Max execution time must be greater than 0".to_string());
            }
        }

        if let Some(max_memory) = self.max_memory_mb {
            if max_memory == 0 {
                return Err("Max memory must be greater than 0".to_string());
            }
        }

        if self.enable_filesystem && self.allowed_paths.is_empty() {
            return Err("Filesystem access requires allowed paths".to_string());
        }

        Ok(())
    }
}

/// Sandbox for isolating skill execution
pub struct Sandbox {
    config: SandboxConfig,
    skill_id: String,
    permission_checker: Arc<PermissionChecker>,
}

impl Sandbox {
    /// Create a new sandbox
    pub fn new(
        config: SandboxConfig,
        skill_id: String,
        permission_checker: Arc<PermissionChecker>,
    ) -> Result<Self, String> {
        config.validate()?;

        Ok(Self {
            config,
            skill_id,
            permission_checker,
        })
    }

    /// Check if a file operation is allowed
    pub fn check_file_access(&self, path: &PathBuf, operation: &str) -> Result<(), PermissionError> {
        if !self.config.enable_filesystem {
            return Err(PermissionError::Denied {
                permission: format!("file.{}", operation),
            });
        }

        // Check if path is in allowed paths
        let is_allowed = self.config.allowed_paths.iter().any(|allowed| {
            path.starts_with(allowed)
        });

        if !is_allowed {
            return Err(PermissionError::Denied {
                permission: format!("file.{} (path not allowed)", operation),
            });
        }

        // Check permission with permission checker
        let permission = Permission::new(
            format!("file.{}", operation),
            Some(path.to_string_lossy().to_string()),
        );

        self.permission_checker.check_permission(&self.skill_id, &permission)
    }

    /// Check if a network request is allowed
    pub fn check_network_access(&self, domain: &str) -> Result<(), PermissionError> {
        if !self.config.enable_network {
            return Err(PermissionError::Denied {
                permission: "network.http".to_string(),
            });
        }

        // Check if domain is in allowed domains
        if !self.config.allowed_domains.is_empty() {
            let is_allowed = self.config.allowed_domains.iter().any(|allowed| {
                domain.ends_with(allowed) || domain == *allowed
            });

            if !is_allowed {
                return Err(PermissionError::Denied {
                    permission: format!("network.http (domain '{}' not allowed)", domain),
                });
            }
        }

        // Check permission with permission checker
        let permission = Permission::new(
            "network.http".to_string(),
            Some(domain.to_string()),
        );

        self.permission_checker.check_permission(&self.skill_id, &permission)
    }

    /// Check if system command execution is allowed
    pub fn check_system_exec(&self, command: &str) -> Result<(), PermissionError> {
        if !self.config.enable_system_exec {
            return Err(PermissionError::Denied {
                permission: "system.exec".to_string(),
            });
        }

        // Check permission with permission checker
        let permission = Permission::new(
            "system.exec".to_string(),
            Some(command.to_string()),
        );

        self.permission_checker.check_permission(&self.skill_id, &permission)
    }

    /// Check if clipboard access is allowed
    pub fn check_clipboard_access(&self, operation: &str) -> Result<(), PermissionError> {
        if !self.config.enable_clipboard {
            return Err(PermissionError::Denied {
                permission: format!("clipboard.{}", operation),
            });
        }

        // Check permission with permission checker
        let permission = Permission::new(
            format!("clipboard.{}", operation),
            None,
        );

        self.permission_checker.check_permission(&self.skill_id, &permission)
    }

    /// Get sandbox configuration
    pub fn config(&self) -> &SandboxConfig {
        &self.config
    }

    /// Get skill ID
    pub fn skill_id(&self) -> &str {
        &self.skill_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert!(!config.enable_filesystem);
        assert!(!config.enable_network);
        assert!(!config.enable_system_exec);
    }

    #[test]
    fn test_sandbox_config_permissive() {
        let config = SandboxConfig::permissive();
        assert!(config.enable_filesystem);
        assert!(config.enable_network);
        assert!(!config.enable_system_exec);
    }

    #[test]
    fn test_sandbox_config_validation() {
        let mut config = SandboxConfig::default();
        config.max_execution_time_ms = Some(0);

        assert!(config.validate().is_err());
    }

    #[test]
    fn test_sandbox_file_access() {
        let config = SandboxConfig {
            enable_filesystem: true,
            allowed_paths: vec![PathBuf::from("/tmp")],
            ..Default::default()
        };

        let permission_checker = Arc::new(PermissionChecker::new());
        let mut checker_ref = permission_checker.clone();

        let mut permissions = crate::skills::permissions::PermissionSet::new();
        permissions.grant(Permission::new(
            "file.read".to_string(),
            Some("/tmp".to_string()),
        ));

        // We need to add permissions to the checker
        // This is a simplified test

        let sandbox = Sandbox::new(config, "test.skill".to_string(), permission_checker);
        assert!(sandbox.is_ok());
    }
}