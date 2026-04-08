use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Skill manifest - defines metadata and permissions for a WASM skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillManifest {
    /// Skill identifier (e.g., "com.example.file-organizer")
    pub id: String,
    /// Skill name
    pub name: String,
    /// Skill version (semver)
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Author information
    pub author: String,
    /// Skill permissions
    pub permissions: Vec<SkillPermission>,
    /// API version this skill requires
    pub api_version: String,
    /// Minimum CEOClaw version required
    pub min_ceoclaw_version: Option<String>,
    /// Skill entry point function name
    pub entry_point: String,
    /// Optional dependencies
    pub dependencies: Option<HashMap<String, String>>,
    /// Optional configuration schema
    pub config_schema: Option<serde_json::Value>,
}

impl SkillManifest {
    /// Create a new skill manifest
    pub fn new(
        id: String,
        name: String,
        version: String,
        description: String,
        author: String,
    ) -> Self {
        Self {
            id,
            name,
            version,
            description,
            author,
            permissions: Vec::new(),
            api_version: "1.0".to_string(),
            min_ceoclaw_version: None,
            entry_point: "main".to_string(),
            dependencies: None,
            config_schema: None,
        }
    }

    /// Add a permission to the manifest
    pub fn with_permission(mut self, permission: SkillPermission) -> Self {
        self.permissions.push(permission);
        self
    }

    /// Set the entry point function name
    pub fn with_entry_point(mut self, entry_point: String) -> Self {
        self.entry_point = entry_point;
        self
    }

    /// Validate the manifest
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Skill ID cannot be empty".to_string());
        }

        if self.name.is_empty() {
            return Err("Skill name cannot be empty".to_string());
        }

        if self.version.is_empty() {
            return Err("Skill version cannot be empty".to_string());
        }

        if self.author.is_empty() {
            return Err("Skill author cannot be empty".to_string());
        }

        // Validate semantic version
        semver::Version::parse(&self.version)
            .map_err(|e| format!("Invalid version: {}", e))?;

        // Validate permissions
        for permission in &self.permissions {
            permission.validate()?;
        }

        Ok(())
    }

    /// Get all required permissions
    pub fn get_permissions(&self) -> &[SkillPermission] {
        &self.permissions
    }

    /// Check if the skill requires a specific permission
    pub fn has_permission(&self, permission_type: &str) -> bool {
        self.permissions.iter().any(|p| p.permission_type == permission_type)
    }
}

/// Skill permission definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillPermission {
    /// Permission type (e.g., "file.read", "network.http")
    pub permission_type: String,
    /// Human-readable description
    pub description: String,
    /// Optional scope (e.g., "/path/to/dir" for file permissions)
    pub scope: Option<String>,
    /// Whether this permission is required or optional
    pub required: bool,
}

impl SkillPermission {
    /// Create a new permission
    pub fn new(permission_type: String, description: String) -> Self {
        Self {
            permission_type,
            description,
            scope: None,
            required: true,
        }
    }

    /// Add scope to the permission
    pub fn with_scope(mut self, scope: String) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Mark as optional
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// Validate the permission
    pub fn validate(&self) -> Result<(), String> {
        if self.permission_type.is_empty() {
            return Err("Permission type cannot be empty".to_string());
        }

        // Validate known permission types
        let known_types = vec![
            "file.read", "file.write", "file.delete", "file.execute",
            "network.http", "network.ws", "network.tcp", "network.udp",
            "system.exec", "system.env", "system.process",
            "clipboard.read", "clipboard.write",
            "notification.send",
            "database.read", "database.write",
        ];

        if !known_types.contains(&self.permission_type.as_str()) {
            return Err(format!("Unknown permission type: {}", self.permission_type));
        }

        Ok(())
    }
}

/// Predefined permission types
pub mod permissions {
    use super::SkillPermission;

    /// File read permission
    pub fn file_read(scope: Option<String>) -> SkillPermission {
        SkillPermission {
            permission_type: "file.read".to_string(),
            description: "Read files from the file system".to_string(),
            scope,
            required: true,
        }
    }

    /// File write permission
    pub fn file_write(scope: Option<String>) -> SkillPermission {
        SkillPermission {
            permission_type: "file.write".to_string(),
            description: "Write files to the file system".to_string(),
            scope,
            required: true,
        }
    }

    /// Network HTTP permission
    pub fn network_http(scope: Option<String>) -> SkillPermission {
        SkillPermission {
            permission_type: "network.http".to_string(),
            description: "Make HTTP requests".to_string(),
            scope,
            required: true,
        }
    }

    /// System execute permission
    pub fn system_exec() -> SkillPermission {
        SkillPermission {
            permission_type: "system.exec".to_string(),
            description: "Execute system commands".to_string(),
            scope: None,
            required: true,
        }
    }

    /// Environment variable access permission
    pub fn system_env() -> SkillPermission {
        SkillPermission {
            permission_type: "system.env".to_string(),
            description: "Read environment variables".to_string(),
            scope: None,
            required: true,
        }
    }

    /// Clipboard read permission
    pub fn clipboard_read() -> SkillPermission {
        SkillPermission {
            permission_type: "clipboard.read".to_string(),
            description: "Read clipboard content".to_string(),
            scope: None,
            required: true,
        }
    }

    /// Clipboard write permission
    pub fn clipboard_write() -> SkillPermission {
        SkillPermission {
            permission_type: "clipboard.write".to_string(),
            description: "Write to clipboard".to_string(),
            scope: None,
            required: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_manifest_validation() {
        let manifest = SkillManifest::new(
            "com.example.test".to_string(),
            "Test Skill".to_string(),
            "1.0.0".to_string(),
            "A test skill".to_string(),
            "Test Author".to_string(),
        );

        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_skill_manifest_invalid_version() {
        let manifest = SkillManifest::new(
            "com.example.test".to_string(),
            "Test Skill".to_string(),
            "invalid".to_string(),
            "A test skill".to_string(),
            "Test Author".to_string(),
        );

        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_permission_creation() {
        let perm = SkillPermission::new(
            "file.read".to_string(),
            "Read files".to_string(),
        );

        assert_eq!(perm.permission_type, "file.read");
        assert!(perm.required);
    }

    #[test]
    fn test_permission_with_scope() {
        let perm = SkillPermission::new(
            "file.read".to_string(),
            "Read files".to_string(),
        ).with_scope("/tmp".to_string());

        assert_eq!(perm.scope, Some("/tmp".to_string()));
    }
}