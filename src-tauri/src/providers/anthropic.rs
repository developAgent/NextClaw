//! Anthropic Provider implementation
//! Handles communication with Anthropic Claude API

use anyhow::{Context, Result};
use futures::StreamExt;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;
use tracing::{debug, info};

/// Anthropic Provider configuration
#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    pub api_key: String,
    pub base_url: Option<String>,
    pub model: String,
    pub version: String,
    pub timeout: Duration,
}

impl AnthropicConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: None,
            model: "claude-3-sonnet-20240229".to_string(),
            version: "2023-06-01".to_string(),
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

    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn get_base_url(&self) -> &str {
        self.base_url
            .as_deref()
            .unwrap_or("https://api.anthropic.com")
    }
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self::new("")
    }
}

/// Message role
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnthropicMessageRole {
    User,
    Assistant,
}

/// Message content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnthropicMessageContent {
    Text(String),
    Blocks(Vec<ContentBlock>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ImageSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    #[serde(rename = "media_type")]
    pub media_type: String,
    pub data: String,
}

/// Anthropic message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMessage {
    pub role: AnthropicMessageRole,
    pub content: AnthropicMessageContent,
}

impl AnthropicMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: AnthropicMessageRole::User,
            content: AnthropicMessageContent::Text(content.into()),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: AnthropicMessageRole::Assistant,
            content: AnthropicMessageContent::Text(content.into()),
        }
    }

    pub fn user_with_blocks(blocks: Vec<ContentBlock>) -> Self {
        Self {
            role: AnthropicMessageRole::User,
            content: AnthropicMessageContent::Blocks(blocks),
        }
    }
}

/// Message creation request
#[derive(Debug, Clone, Serialize)]
pub struct MessageCreateRequest {
    pub model: String,
    pub messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
}

impl MessageCreateRequest {
    pub fn new(model: impl Into<String>, messages: Vec<AnthropicMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            system: None,
            max_tokens: Some(4096),
            temperature: None,
            top_p: None,
            top_k: None,
            stream: None,
            stop_sequences: None,
        }
    }

    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = Some(tokens);
        self
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p);
        self
    }

    pub fn with_top_k(mut self, top_k: u32) -> Self {
        self.top_k = Some(top_k);
        self
    }

    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    pub fn with_stop_sequences(mut self, sequences: Vec<String>) -> Self {
        self.stop_sequences = Some(sequences);
        self
    }
}

/// Message response
#[derive(Debug, Clone, Deserialize)]
pub struct MessageResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub response_type: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Stream event
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
pub enum StreamEvent {
    #[serde(rename = "message_start")]
    MessageStart {
        message: MessageStart,
    },
    #[serde(rename = "content_block_start")]
    ContentBlockStart {
        index: u32,
        content_block: ContentBlock,
    },
    #[serde(rename = "content_block_delta")]
    ContentBlockDelta {
        index: u32,
        delta: Delta,
    },
    #[serde(rename = "content_block_stop")]
    ContentBlockStop {
        index: u32,
    },
    #[serde(rename = "message_delta")]
    MessageDelta {
        delta: MessageDeltaInfo,
        usage: Option<Usage>,
    },
    #[serde(rename = "message_stop")]
    MessageStop {
        #[serde(rename = "stop_reason")]
        reason: Option<String>,
    },
    #[serde(rename = "error")]
    Error {
        error: ErrorInfo,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageStart {
    pub id: String,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub role: String,
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub stop_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Delta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageDeltaInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorInfo {
    #[serde(rename = "type")]
    pub error_type: String,
    pub message: String,
}

/// Anthropic Provider
pub struct AnthropicProvider {
    config: AnthropicConfig,
    client: Client,
}

impl AnthropicProvider {
    pub fn new(config: AnthropicConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { config, client })
    }

    /// Get the endpoint URL
    fn get_endpoint(&self, path: &str) -> String {
        format!("{}/{}", self.config.get_base_url().trim_end_matches('/'), path)
    }

    /// Create a message
    pub async fn create_message(&self, request: MessageCreateRequest) -> Result<MessageResponse> {
        let url = self.get_endpoint("v1/messages");

        debug!("Creating message with model: {}", request.model);

        let response = self.client
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", &self.config.version)
            .header("Content-Type", "application/json")
            .header("dangerously-allow-browser", "true")
            .json(&request)
            .send()
            .await
            .context("Failed to send message request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Message creation failed: {} - {}", status, body);
        }

        let message: MessageResponse = response
            .json()
            .await
            .context("Failed to parse message response")?;

        Ok(message)
    }

    /// Create a streaming message
    pub async fn create_message_stream(
        &self,
        request: MessageCreateRequest,
    ) -> Result<impl futures::Stream<Item = Result<StreamEvent>>> {
        let url = self.get_endpoint("v1/messages");

        debug!("Creating streaming message with model: {}", request.model);

        let mut request_with_stream = request;
        request_with_stream.stream = Some(true);

        let response = self.client
            .post(&url)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", &self.config.version)
            .header("Content-Type", "application/json")
            .header("dangerously-allow-browser", "true")
            .json(&request_with_stream)
            .send()
            .await
            .context("Failed to send streaming request")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Streaming message failed: {} - {}", status, body);
        }

        let stream = response.bytes_stream();

        // Parse SSE stream
        let stream = stream.map(|chunk_result: Result<bytes::Bytes, reqwest::Error>| {
            chunk_result.and_then(|bytes| {
                let text = String::from_utf8_lossy(&bytes);
                Ok(text)
            })
        });

        Ok(futures::stream::unfold(stream, |mut stream| async move {
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                for line in chunk.lines() {
                    if line.starts_with("data: ") {
                        let data = &line[6..];
                        if let Ok(event) = serde_json::from_str::<StreamEvent>(data) {
                            return Some((Ok(event), stream));
                        }
                    }
                }
            }
            None
        })
        .filter_map(|r| async { r }))
    }

    /// Validate API key by making a simple request
    pub async fn validate_api_key(&self) -> Result<bool> {
        let test_request = MessageCreateRequest::new(
            "claude-3-haiku-20240307",
            vec![AnthropicMessage::user("Hi")],
        )
        .with_max_tokens(10);

        match self.create_message(test_request).await {
            Ok(_) => Ok(true),
            Err(e) => {
                // Check if it's an auth error
                let error_msg = e.to_string().to_lowercase();
                if error_msg.contains("401") || error_msg.contains("403") || error_msg.contains("unauthorized") {
                    Ok(false)
                } else {
                    // Other errors might still mean the key is valid but the request failed
                    Ok(true)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = AnthropicConfig::new("test-key")
            .with_model("claude-3-opus-20240229")
            .with_version("2024-01-01");

        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.model, "claude-3-opus-20240229");
        assert_eq!(config.version, "2024-01-01");
    }

    #[test]
    fn test_message_creation() {
        let message = AnthropicMessage::user("Hello");
        assert!(matches!(message.role, AnthropicMessageRole::User));

        let blocks = vec![ContentBlock {
            block_type: "text".to_string(),
            text: Some("Hello".to_string()),
            source: None,
        }];
        let user_blocks = AnthropicMessage::user_with_blocks(blocks);
        assert!(matches!(user_blocks.role, AnthropicMessageRole::User));
    }

    #[test]
    fn test_request_creation() {
        let request = MessageCreateRequest::new(
            "claude-3",
            vec![AnthropicMessage::user("Test")],
        )
        .with_max_tokens(100)
        .with_temperature(0.7)
        .with_system("You are helpful");

        assert_eq!(request.model, "claude-3");
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.temperature, Some(0.7));
        assert_eq!(request.system, Some("You are helpful".to_string()));
    }
}