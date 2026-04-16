use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use thiserror::Error;

/// Permission definition
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct Permission {
    /// Permission type (e.g., "file.read", "network.http")
    pub permission_type: String,
    /// Optional scope (e.g., "/path/to/dir" for file permissions)
    pub scope: Option<String>,
}

impl Permission {
    /// Create a new permission
    pub fn new(permission_type: String, scope: Option<String>) -> Self {
        Self {
            permission_type,
            scope,
        }
    }

    /// Check if this permission matches another one
    pub fn matches(&self, other: &Permission) -> bool {
        if self.permission_type != other.permission_type {
            return false;
        }

        match (&self.scope, &other.scope) {
            (None, None) => true,
            (Some(s1), Some(s2)) => s1 == s2,
            _ => false,
        }
    }
}

/// Permission set - a collection of permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSet {
    /// Granted permissions
    pub granted: HashSet<Permission>,
    /// Denied permissions
    pub denied: HashSet<Permission>,
}

impl PermissionSet {
    /// Create a new permission set
    pub fn new() -> Self {
        Self {
            granted: HashSet::new(),
            denied: HashSet::new(),
        }
    }

    /// Add a granted permission
    pub fn grant(&mut self, permission: Permission) {
        self.denied.remove(&permission);
        self.granted.insert(permission);
    }

    /// Add a denied permission
    pub fn deny(&mut self, permission: Permission) {
        self.granted.remove(&permission);
        self.denied.insert(permission);
    }

    /// Check if a permission is granted
    pub fn is_granted(&self, permission: &Permission) -> bool {
        self.granted.iter().any(|p| p.matches(permission))
            && !self.denied.iter().any(|p| p.matches(permission))
    }

    /// Get all granted permissions
    pub fn get_granted(&self) -> &HashSet<Permission> {
        &self.granted
    }

    /// Get all denied permissions
    pub fn get_denied(&self) -> &HashSet<Permission> {
        &self.denied
    }

    /// Merge with another permission set
    pub fn merge(&mut self, other: PermissionSet) {
        for perm in other.granted {
            self.granted.insert(perm);
        }
        for perm in other.denied {
            self.denied.insert(perm);
        }
    }

    /// Remove all permissions
    pub fn clear(&mut self) {
        self.granted.clear();
        self.denied.clear();
    }
}

impl Default for PermissionSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Permission error
#[derive(Error, Debug)]
pub enum PermissionError {
    #[error("Permission denied: {permission}")]
    Denied { permission: String },

    #[error("Permission not found: {permission}")]
    NotFound { permission: String },

    #[error("Invalid permission: {reason}")]
    Invalid { reason: String },

    #[error("Permission check failed: {reason}")]
    CheckFailed { reason: String },
}

/// Permission checker
pub struct PermissionChecker {
    permission_sets: HashMap<String, PermissionSet>,
}

impl PermissionChecker {
    /// Create a new permission checker
    pub fn new() -> Self {
        Self {
            permission_sets: HashMap::new(),
        }
    }

    /// Add or update a permission set for a skill
    pub fn set_permissions(&mut self, skill_id: String, permissions: PermissionSet) {
        self.permission_sets.insert(skill_id, permissions);
    }

    /// Get permissions for a skill
    pub fn get_permissions(&self, skill_id: &str) -> Option<&PermissionSet> {
        self.permission_sets.get(skill_id)
    }

    /// Check if a skill has a specific permission
    pub fn check_permission(
        &self,
        skill_id: &str,
        permission: &Permission,
    ) -> Result<(), PermissionError> {
        let permission_set =
            self.permission_sets
                .get(skill_id)
                .ok_or_else(|| PermissionError::NotFound {
                    permission: format!("Skill '{}' not found", skill_id),
                })?;

        if permission_set.is_granted(permission) {
            Ok(())
        } else {
            Err(PermissionError::Denied {
                permission: permission.permission_type.clone(),
            })
        }
    }

    /// Check multiple permissions at once
    pub fn check_permissions(
        &self,
        skill_id: &str,
        permissions: &[Permission],
    ) -> Result<(), PermissionError> {
        for permission in permissions {
            self.check_permission(skill_id, permission)?;
        }
        Ok(())
    }

    /// Remove a skill's permissions
    pub fn remove_permissions(&mut self, skill_id: &str) {
        self.permission_sets.remove(skill_id);
    }

    /// List all skills with permissions
    pub fn list_skills(&self) -> Vec<&str> {
        self.permission_sets.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for PermissionChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permission_set_grant() {
        let mut set = PermissionSet::new();
        let perm = Permission::new("file.read".to_string(), None);

        set.grant(perm.clone());
        assert!(set.is_granted(&perm));
    }

    #[test]
    fn test_permission_set_deny() {
        let mut set = PermissionSet::new();
        let perm = Permission::new("file.read".to_string(), None);

        set.deny(perm.clone());
        assert!(!set.is_granted(&perm));
    }

    #[test]
    fn test_permission_checker() {
        let mut checker = PermissionChecker::new();
        let mut permissions = PermissionSet::new();
        permissions.grant(Permission::new("file.read".to_string(), None));

        checker.set_permissions("test.skill".to_string(), permissions);

        let result = checker.check_permission(
            "test.skill",
            &Permission::new("file.read".to_string(), None),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_permission_checker_denied() {
        let mut checker = PermissionChecker::new();
        let mut permissions = PermissionSet::new();
        permissions.grant(Permission::new("file.read".to_string(), None));

        checker.set_permissions("test.skill".to_string(), permissions);

        let result = checker.check_permission(
            "test.skill",
            &Permission::new("file.write".to_string(), None),
        );

        assert!(result.is_err());
    }
}
