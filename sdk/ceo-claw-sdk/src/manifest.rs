use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Skill manifest builder
#[derive(Debug)]
pub struct SkillManifestBuilder {
    id: String,
    name: String,
    version: String,
    description: String,
    author: String,
    permissions: Vec<SkillPermission>,
    entry_point: String,
    api_version: String,
    dependencies: Option<HashMap<String, String>>,
    config_schema: Option<serde_json::Value>,
}

impl SkillManifestBuilder {
    /// Create a new skill manifest builder
    pub fn new(id: String, name: String, version: String) -> Self {
        Self {
            id,
            name,
            version,
            description: String::new(),
            author: String::new(),
            permissions: Vec::new(),
            entry_point: "main".to_string(),
            api_version: "1.0".to_string(),
            dependencies: None,
            config_schema: None,
        }
    }

    /// Set the description
    pub fn description(mut self, description: String) -> Self {
        self.description = description;
        self
    }

    /// Set the author
    pub fn author(mut self, author: String) -> Self {
        self.author = author;
        self
    }

    /// Add a permission
    pub fn permission(mut self, permission: SkillPermission) -> Self {
        self.permissions.push(permission);
        self
    }

    /// Add multiple permissions
    pub fn permissions(mut self, permissions: Vec<SkillPermission>) -> Self {
        self.permissions.extend(permissions);
        self
    }

    /// Set the entry point
    pub fn entry_point(mut self, entry_point: String) -> Self {
        self.entry_point = entry_point;
        self
    }

    /// Set the API version
    pub fn api_version(mut self, api_version: String) -> Self {
        self.api_version = api_version;
        self
    }

    /// Set dependencies
    pub fn dependencies(mut self, dependencies: HashMap<String, String>) -> Self {
        self.dependencies = Some(dependencies);
        self
    }

    /// Set config schema
    pub fn config_schema(mut self, schema: serde_json::Value) -> Self {
        self.config_schema = Some(schema);
        self
    }

    /// Build the manifest
    pub fn build(self) -> SkillManifest {
        SkillManifest {
            id: self.id,
            name: self.name,
            version: self.version,
            description: self.description,
            author: self.author,
            permissions: self.permissions,
            api_version: self.api_version,
            min_ceoclaw_version: None,
            entry_point: self.entry_point,
            dependencies: self.dependencies,
            config_schema: self.config_schema,
        }
    }
}

/// Permission builder
#[derive(Debug)]
pub struct PermissionBuilder {
    permission_type: String,
    description: String,
    scope: Option<String>,
    required: bool,
}

impl PermissionBuilder {
    /// Create a new permission builder
    pub fn new(permission_type: String, description: String) -> Self {
        Self {
            permission_type,
            description,
            scope: None,
            required: true,
        }
    }

    /// Set the scope
    pub fn scope(mut self, scope: String) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Mark as optional
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// Build the permission
    pub fn build(self) -> SkillPermission {
        SkillPermission {
            permission_type: self.permission_type,
            description: self.description,
            scope: self.scope,
            required: self.required,
        }
    }
}

/// Skill manifest
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
    /// Create a new skill manifest builder
    pub fn builder(id: String, name: String, version: String) -> SkillManifestBuilder {
        SkillManifestBuilder::new(id, name, version)
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
    pub fn required_permissions(&self) -> Vec<&SkillPermission> {
        self.permissions.iter().filter(|p| p.required).collect()
    }

    /// Get all optional permissions
    pub fn optional_permissions(&self) -> Vec<&SkillPermission> {
        self.permissions.iter().filter(|p| !p.required).collect()
    }

    /// Check if the skill requires a specific permission
    pub fn has_permission(&self, permission_type: &str) -> bool {
        self.permissions.iter().any(|p| p.permission_type == permission_type)
    }
}

/// Skill permission
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

    /// Create a permission builder
    pub fn builder(permission_type: String, description: String) -> PermissionBuilder {
        PermissionBuilder::new(permission_type, description)
    }

    /// Set scope
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

    /// File delete permission
    pub fn file_delete(scope: Option<String>) -> SkillPermission {
        SkillPermission {
            permission_type: "file.delete".to_string(),
            description: "Delete files from the file system".to_string(),
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
    fn test_manifest_builder() {
        let manifest = SkillManifest::builder(
            "com.example.test".to_string(),
            "Test Skill".to_string(),
            "1.0.0".to_string(),
        )
        .description("A test skill".to_string())
        .author("Test Author".to_string())
        .build();

        assert_eq!(manifest.id, "com.example.test");
        assert_eq!(manifest.name, "Test Skill");
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_permission_builder() {
        let permission = SkillPermission::builder(
            "file.read".to_string(),
            "Read files".to_string(),
        )
        .scope("/tmp".to_string())
        .optional()
        .build();

        assert_eq!(permission.permission_type, "file.read");
        assert_eq!(permission.scope, Some("/tmp".to_string()));
        assert!(!permission.required);
    }
}