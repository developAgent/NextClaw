//! OpenAI Provider implementation
//! Handles communication with OpenAI-compatible APIs

use anyhow::{Context, Result};
use futures::{Stream, StreamExt};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tracing::{debug, info};

/// OpenAI Provider configuration
#[derive(Debug, Clone)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: String,
    pub timeout: Duration,
}

impl OpenAIConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: None,
            model: "gpt-4o-mini".to_string(),
            timeout: Duration::from_secs(120),
        }
    }

    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn get_base_url(&self) -> &str {
        self.base_url
            .as_deref()
            .unwrap_or("https://api.openai.com/v1")
    }
}

impl Default for OpenAIConfig {
    fn default() -> Self {
        Self::new("")
    }
}

/// Message role
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }
}

/// Chat completion request
#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

impl ChatCompletionRequest {
    pub fn new(model: impl Into<String>, messages: Vec<ChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            temperature: None,
            max_tokens: None,
            stream: None,
        }
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = Some(tokens);
        self
    }

    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }
}

/// Chat completion response
#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Chat completion delta (for streaming)
#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionDelta {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<DeltaChoice>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeltaChoice {
    pub index: u32,
    pub delta: Delta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Delta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

/// Error response
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorDetail {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub code: Option<String>,
}

/// Available models
#[derive(Debug, Clone, Deserialize)]
pub struct Model {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelsResponse {
    pub object: String,
    pub data: Vec<Model>,
}

/// OpenAI Provider
pub struct OpenAIProvider {
    config: OpenAIConfig,
    client: Client,
}

impl OpenAIProvider {
    pub fn new(config: OpenAIConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { config, client })
    }

    /// Get the base URL
    fn get_endpoint(&self, path: &str) -> String {
        format!("{}/{}", self.config.get_base_url().trim_end_matches('/'), path)
    }

    /// List available models
    pub async fn list_models(&self) -> Result<ModelsResponse> {
        let url = self.get_endpoint("models");

        debug!("Listing models from: {}", url);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await
            .context("Failed to send list models request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Failed to list models: {} - {}", status, body);
        }

        let models: ModelsResponse = response
            .json()
            .await
            .context("Failed to parse models response")?;

        Ok(models)
    }

    /// Create a chat completion
    pub async fn create_chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        let url = self.get_endpoint("chat/completions");
        let model = request.model.clone();

        debug!("Creating chat completion with model: {}", model);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send chat completion request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Chat completion failed: {} - {}", status, body);
        }

        let completion: ChatCompletionResponse = response
            .json()
            .await
            .context("Failed to parse chat completion response")?;

        Ok(completion)
    }

    /// Create a streaming chat completion
    pub async fn create_chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<impl Stream<Item = Result<ChatCompletionDelta>>> {
        let url = self.get_endpoint("chat/completions");
        let model = request.model.clone();

        debug!("Creating streaming chat completion with model: {}", model);

        let mut request_with_stream = request;
        request_with_stream.stream = Some(true);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_with_stream)
            .send()
            .await
            .context("Failed to send streaming request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Streaming completion failed: {} - {}", status, body);
        }

        let stream = response.bytes_stream();

        // Parse SSE stream
        let stream = stream.map(|chunk| {
            chunk.and_then(|bytes| {
                let text = String::from_utf8_lossy(&bytes).into_owned();
                Ok(text)
            })
        });

        Ok(futures::stream::unfold(stream, |mut stream| async {
            while let Some(chunk_result) = stream.next().await {
                let chunk = match chunk_result {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                for line in chunk.lines() {
                    if line.starts_with("data: ") {
                        let data = &line[6..];
                        if data == "[DONE]" {
                            return None;
                        }
                        if let Ok(delta) = serde_json::from_str::<ChatCompletionDelta>(data) {
                            return Some((Ok(delta), stream));
                        }
                    }
                }
            }
            None
        })
        .map(|r| Some(r))
        .filter_map(futures::future::ready))
    }

    /// Validate API key by making a simple request
    pub async fn validate_api_key(&self) -> Result<bool> {
        let url = self.get_endpoint("models");

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}

// Placeholder for Stream trait (would need async-stream or similar)
// This is a simplified version

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = OpenAIConfig::new("test-key")
            .with_model("gpt-4")
            .with_base_url("https://api.custom.com/v1");

        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.get_base_url(), "https://api.custom.com/v1");
    }

    #[test]
    fn test_message_creation() {
        let system = ChatMessage::system("You are a helpful assistant");
        let user = ChatMessage::user("Hello");
        let assistant = ChatMessage::assistant("Hi there!");

        assert!(matches!(system.role, MessageRole::System));
        assert!(matches!(user.role, MessageRole::User));
        assert!(matches!(assistant.role, MessageRole::Assistant));
    }

    #[test]
    fn test_request_creation() {
        let request = ChatCompletionRequest::new(
            "gpt-4",
            vec![
                ChatMessage::system("You are helpful"),
                ChatMessage::user("Hello"),
            ],
        )
        .with_temperature(0.7)
        .with_max_tokens(100);

        assert_eq!(request.model, "gpt-4");
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.temperature, Some(0.7));
        assert_eq!(request.max_tokens, Some(100));
    }
}