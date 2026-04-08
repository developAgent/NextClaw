use crate::types::{SkillArgs, WasmArgument};
use std::collections::HashMap;
use uuid::Uuid;

/// Skill context - provides information about the skill execution environment
#[derive(Debug, Clone)]
pub struct SkillContext {
    /// Unique execution ID
    pub execution_id: String,
    /// Skill ID
    pub skill_id: String,
    /// Skill version
    pub skill_version: String,
    /// Request ID from CEOClaw
    pub request_id: Option<String>,
    /// User ID (if available)
    pub user_id: Option<String>,
    /// Session ID (if available)
    pub session_id: Option<String>,
    /// Environment variables
    pub env: HashMap<String, String>,
    /// Custom context data
    pub custom: HashMap<String, String>,
}

impl SkillContext {
    /// Create a new skill context
    pub fn new() -> Self {
        Self {
            execution_id: Uuid::new_v4().to_string(),
            skill_id: Self::get_env_var("CEOCLAW_SKILL_ID", "unknown".to_string()),
            skill_version: Self::get_env_var("CEOCLAW_SKILL_VERSION", "1.0.0".to_string()),
            request_id: std::env::var("CEOCLAW_REQUEST_ID").ok(),
            user_id: std::env::var("CEOCLAW_USER_ID").ok(),
            session_id: std::env::var("CEOCLAW_SESSION_ID").ok(),
            env: Self::load_env_vars(),
            custom: HashMap::new(),
        }
    }

    /// Get an environment variable with a default value
    fn get_env_var(name: &str, default: String) -> String {
        std::env::var(name).unwrap_or(default)
    }

    /// Load environment variables with CEOCLAW_ prefix
    fn load_env_vars() -> HashMap<String, String> {
        std::env::vars()
            .filter(|(k, _)| k.starts_with("CEOCLAW_"))
            .map(|(k, v)| (k.clone(), v))
            .collect()
    }

    /// Get an environment variable
    pub fn get_env(&self, name: &str) -> Option<&String> {
        self.env.get(name)
    }

    /// Set a custom context value
    pub fn set_custom(&mut self, key: String, value: String) {
        self.custom.insert(key, value);
    }

    /// Get a custom context value
    pub fn get_custom(&self, key: &str) -> Option<&String> {
        self.custom.get(key)
    }

    /// Get the execution ID
    pub fn execution_id(&self) -> &str {
        &self.execution_id
    }

    /// Get the skill ID
    pub fn skill_id(&self) -> &str {
        &self.skill_id
    }

    /// Get the skill version
    pub fn skill_version(&self) -> &str {
        &self.skill_version
    }

    /// Get the request ID
    pub fn request_id(&self) -> Option<&String> {
        self.request_id.as_ref()
    }

    /// Get the user ID
    pub fn user_id(&self) -> Option<&String> {
        self.user_id.as_ref()
    }

    /// Get the session ID
    pub fn session_id(&self) -> Option<&String> {
        self.session_id.as_ref()
    }
}

impl Default for SkillContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating SkillContext with custom values
#[derive(Debug)]
pub struct ContextBuilder {
    context: SkillContext,
}

impl ContextBuilder {
    /// Create a new context builder
    pub fn new() -> Self {
        Self {
            context: SkillContext::new(),
        }
    }

    /// Set the skill ID
    pub fn with_skill_id(mut self, skill_id: String) -> Self {
        self.context.skill_id = skill_id;
        self
    }

    /// Set the skill version
    pub fn with_skill_version(mut self, version: String) -> Self {
        self.context.skill_version = version;
        self
    }

    /// Set the request ID
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.context.request_id = Some(request_id);
        self
    }

    /// Set the user ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.context.user_id = Some(user_id);
        self
    }

    /// Set the session ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.context.session_id = Some(session_id);
        self
    }

    /// Add a custom value
    pub fn with_custom(mut self, key: String, value: String) -> Self {
        self.context.set_custom(key, value);
        self
    }

    /// Build the context
    pub fn build(self) -> SkillContext {
        self.context
    }
}

impl Default for ContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let context = SkillContext::new();
        assert!(!context.execution_id.is_empty());
    }

    #[test]
    fn test_context_builder() {
        let context = ContextBuilder::new()
            .with_skill_id("test.skill".to_string())
            .with_skill_version("1.0.0".to_string())
            .with_custom("key".to_string(), "value".to_string())
            .build();

        assert_eq!(context.skill_id, "test.skill");
        assert_eq!(context.skill_version, "1.0.0");
        assert_eq!(context.get_custom("key"), Some(&"value".to_string()));
    }
}