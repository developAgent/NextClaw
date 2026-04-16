use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Database models and DTOs

/// Chat session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub agent_id: Option<String>,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Session {
    #[must_use]
    pub fn new(title: String, agent_id: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            agent_id,
            title,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

impl Message {
    #[must_use]
    pub fn new(session_id: Uuid, role: MessageRole, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            role,
            content,
            created_at: Utc::now(),
        }
    }
}

/// Message role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl From<&str> for MessageRole {
    fn from(s: &str) -> Self {
        match s {
            "user" => Self::User,
            "assistant" => Self::Assistant,
            "system" => Self::System,
            _ => Self::User,
        }
    }
}

impl MessageRole {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Assistant => "assistant",
            Self::System => "system",
        }
    }
}

/// Command execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecution {
    pub id: Uuid,
    pub session_id: Uuid,
    pub command: String,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub duration_ms: Option<i64>,
    pub created_at: DateTime<Utc>,
}

impl CommandExecution {
    #[must_use]
    pub fn new(session_id: Uuid, command: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            command,
            exit_code: None,
            stdout: None,
            stderr: None,
            duration_ms: None,
            created_at: Utc::now(),
        }
    }

    pub fn success(&self) -> bool {
        self.exit_code.map_or(false, |code| code == 0)
    }
}

/// File operation record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub id: Uuid,
    pub session_id: Uuid,
    pub operation: FileOperationType,
    pub path: String,
    pub success: bool,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl FileOperation {
    #[must_use]
    pub fn new(session_id: Uuid, operation: FileOperationType, path: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            operation,
            path,
            success: true,
            error: None,
            created_at: Utc::now(),
        }
    }
}

/// File operation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileOperationType {
    Read,
    Write,
    Delete,
    List,
    Move,
    Copy,
}

impl FileOperationType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::List => "list",
            Self::Move => "move",
            Self::Copy => "copy",
        }
    }
}

impl From<&str> for FileOperationType {
    fn from(s: &str) -> Self {
        match s {
            "read" => Self::Read,
            "write" => Self::Write,
            "delete" => Self::Delete,
            "list" => Self::List,
            "move" => Self::Move,
            "copy" => Self::Copy,
            _ => Self::Read,
        }
    }
}

/// Configuration value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValue {
    pub key: String,
    pub value: String,
    pub is_secret: bool,
    pub updated_at: DateTime<Utc>,
}

/// Session with message count (for list views)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub message_count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new("Test Session".to_string(), None);
        assert_eq!(session.title, "Test Session");
        assert!(session.agent_id.is_none());
        assert!(session.created_at <= Utc::now());
    }

    #[test]
    fn test_message_creation() {
        let session_id = Uuid::new_v4();
        let message = Message::new(session_id, MessageRole::User, "Hello".to_string());
        assert_eq!(message.session_id, session_id);
        assert_eq!(message.role, MessageRole::User);
        assert_eq!(message.content, "Hello");
    }

    #[test]
    fn test_command_execution() {
        let session_id = Uuid::new_v4();
        let cmd = CommandExecution::new(session_id, "ls -la".to_string());
        assert_eq!(cmd.command, "ls -la");
        assert!(!cmd.success()); // No exit code yet
    }

    #[test]
    fn test_message_role_conversion() {
        assert_eq!(MessageRole::from("user"), MessageRole::User);
        assert_eq!(MessageRole::from("assistant"), MessageRole::Assistant);
        assert_eq!(MessageRole::from("system"), MessageRole::System);
    }

    #[test]
    fn test_file_operation_type() {
        assert_eq!(FileOperationType::Read.as_str(), "read");
        assert_eq!(FileOperationType::Write.as_str(), "write");
        assert_eq!(FileOperationType::from("read"), FileOperationType::Read);
    }
}
