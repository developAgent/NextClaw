use anthropic_rs::{
    error::{AnthropicError, AuthenticationError, RateLimitError, ValidationError},
    Claude,
};

use async_trait::async_trait;
use futures::StreamExt;
use secrecy::{Secret, SecretString};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::utils::error::{AppError, Result};

/// Claude AI client wrapper with streaming support
pub struct ClaudeClient {
    client: Claude,
    model: String,
}

impl ClaudeClient {
    /// Create a new Claude client with the given API key
    ///
    /// # Errors
    ///
    /// Returns an error if the API key is invalid or client initialization fails
    pub fn new(api_key: SecretString) -> Result<Self> {
        let client = Claude::new(api_key);
        info!("Claude client initialized");

        Ok(Self {
            client,
            model: "claude-3-sonnet-20240229".to_string(),
        })
    }

    /// Set the model to use
    #[must_use]
    pub const fn with_model(mut self, model: String) -> Self {
        self.model = model;
        self
    }

    /// Send a message and return the response (non-streaming)
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails
    pub async fn send_message(&self, message: &str, session_id: Uuid) -> Result<String> {
        let response = self
            .client
            .message()
            .with_system("You are CEOClaw, an AI assistant that helps users execute commands and automate tasks on their computer. Always be helpful, clear, and safe.")
            .with_user(message)
            .with_model(&self.model)
            .await
            .map_err(|e| match e {
                AnthropicError::Authentication(_) => {
                    AppError::Authentication("Invalid API key".to_string())
                }
                AnthropicError::RateLimit(_) => AppError::RateLimit("Rate limit exceeded".to_string()),
                AnthropicError::Validation(e) => AppError::Validation(e.to_string()),
                _ => AppError::Ai(e.to_string()),
            })?;

        let content = response.content();
        let text = content.iter().filter_map(|block| block.as_text()).collect::<Vec<_>>().join("\n");

        debug!("Received response for session {}", session_id);
        Ok(text)
    }

    /// Send a message with streaming response
    ///
    /// Returns a stream of text chunks as they arrive
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails to start
    pub async fn send_message_streaming(
        &self,
        message: &str,
        _session_id: Uuid,
        mut callback: impl FnMut(String),
    ) -> Result<()> {
        let mut stream = self
            .client
            .message()
            .with_system("You are CEOClaw, an AI assistant that helps users execute commands and automate tasks on their computer. Always be helpful, clear, and safe.")
            .with_user(message)
            .with_model(&self.model)
            .stream()
            .await
            .map_err(|e| match e {
                AnthropicError::Authentication(_) => {
                    AppError::Authentication("Invalid API key".to_string())
                }
                AnthropicError::RateLimit(_) => AppError::RateLimit("Rate limit exceeded".to_string()),
                AnthropicError::Validation(e) => AppError::Validation(e.to_string()),
                _ => AppError::Ai(e.to_string()),
            })?;

        info!("Starting streaming response");

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    if let Some(text) = chunk.as_text() {
                        callback(text);
                    }
                }
                Err(e) => {
                    error!("Stream error: {}", e);
                    return Err(AppError::Ai(e.to_string()));
                }
            }
        }

        info!("Streaming completed");
        Ok(())
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

/// A message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Message {
    pub fn new(role: MessageRole, content: String) -> Self {
        Self {
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
        let msg = Message::new(MessageRole::User, "Hello".to_string());
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "Hello");
    }

    #[test]
    fn test_message_role_display() {
        assert_eq!(MessageRole::User.to_string(), "user");
        assert_eq!(MessageRole::Assistant.to_string(), "assistant");
        assert_eq!(MessageRole::System.to_string(), "system");
    }
}