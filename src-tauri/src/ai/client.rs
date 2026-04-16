use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use uuid::Uuid;

use crate::utils::error::{AppError, Result};

/// Claude AI client wrapper
pub struct ClaudeClient {
    api_key: SecretString,
    model: String,
}

impl ClaudeClient {
    /// Create a new Claude client with the given API key
    ///
    /// # Errors
    ///
    /// Returns an error if client initialization fails
    pub fn new(api_key: SecretString) -> Result<Self> {
        info!("Claude client initialized");

        Ok(Self {
            api_key,
            model: "claude-3-sonnet-20240229".to_string(),
        })
    }

    /// Set the model to use
    pub fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    /// Send a message and return the response
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails
    pub async fn send_message(&self, message: &str, session_id: Uuid) -> Result<String> {
        let client = reqwest::Client::new();

        #[derive(Serialize)]
        struct Message {
            role: String,
            content: String,
        }

        #[derive(Serialize)]
        struct RequestBody {
            model: String,
            messages: Vec<Message>,
            max_tokens: usize,
        }

        #[derive(Deserialize)]
        struct ContentBlock {
            #[serde(rename = "type")]
            content_type: String,
            text: Option<String>,
        }

        #[derive(Deserialize)]
        struct Response {
            content: Vec<ContentBlock>,
        }

        let request_body = RequestBody {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: message.to_string(),
            }],
            max_tokens: 4096,
        };

        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", self.api_key.expose_secret())
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::Ai(format!("Failed to send request: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AppError::Ai(format!(
                "API error: {} - {}",
                status, error_text
            )));
        }

        let response_body: Response = response
            .json()
            .await
            .map_err(|e| AppError::Ai(format!("Failed to parse response: {e}")))?;

        let text = response_body
            .content
            .iter()
            .filter_map(|block| block.text.as_ref())
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");

        debug!("Received response for session {}", session_id);
        Ok(text)
    }
}

/// Message role in the conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl std::fmt::Display for MessageRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "user"),
            Self::Assistant => write!(f, "assistant"),
            Self::System => write!(f, "system"),
        }
    }
}

impl From<&str> for MessageRole {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "user" => Self::User,
            "assistant" => Self::Assistant,
            "system" => Self::System,
            _ => Self::User,
        }
    }
}

/// A message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub session_id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Message {
    pub fn new(session_id: Uuid, role: MessageRole, content: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            session_id,
            role,
            content,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let session_id = Uuid::new_v4();
        let msg = Message::new(session_id, MessageRole::User, "Hello".to_string());
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_message_role_display() {
        assert_eq!(MessageRole::User.to_string(), "user");
        assert_eq!(MessageRole::Assistant.to_string(), "assistant");
        assert_eq!(MessageRole::System.to_string(), "system");
    }

    #[test]
    fn test_message_role_from_str() {
        assert_eq!(MessageRole::from("user"), MessageRole::User);
        assert_eq!(MessageRole::from("assistant"), MessageRole::Assistant);
        assert_eq!(MessageRole::from("system"), MessageRole::System);
    }
}
