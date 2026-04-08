use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Model information returned by Ollama
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaModel {
    /// Model name (e.g., "llama2:7b")
    pub name: String,
    /// Model size in GB
    pub size: f64,
    /// Quantization level
    pub quantization_level: Option<String>,
    /// Last modified timestamp
    pub modified_at: Option<DateTime<Utc>>,
}

/// Response from /api/tags endpoint
#[derive(Debug, Serialize, Deserialize)]
pub struct ListModelsResponse {
    pub models: Vec<OllamaModel>,
}

/// Detailed model information
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    /// License information
    pub license: Option<String>,
    /// Modelfile content
    pub modelfile: Option<String>,
    /// Parameters
    pub parameters: Option<String>,
    /// Template
    pub template: Option<String>,
    /// Details
    pub details: Option<ModelDetails>,
}

/// Model details
#[derive(Debug, Serialize, Deserialize)]
pub struct ModelDetails {
    pub format: Option<String>,
    pub family: Option<String>,
    pub families: Option<Vec<String>>,
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
}

/// Pull progress information
#[derive(Debug, Serialize, Deserialize)]
pub struct PullProgress {
    /// Current status
    pub status: String,
    /// Digest
    pub digest: Option<String>,
    /// Total bytes
    pub total: Option<u64>,
    /// Completed bytes
    pub completed: Option<u64>,
}

/// Chat message role
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaMessage {
    pub role: MessageRole,
    pub content: String,
}

/// Chat completion options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatOptions {
    /// Temperature for sampling (0.0 to 1.0)
    pub temperature: Option<f32>,
    /// Top-p sampling threshold (0.0 to 1.0)
    pub top_p: Option<f32>,
    /// Top-k sampling parameter
    pub top_k: Option<u32>,
    /// Maximum number of tokens to generate
    pub num_predict: Option<u32>,
    /// Stop sequences
    pub stop: Option<Vec<String>>,
    /// Presence penalty
    pub presence_penalty: Option<f32>,
    /// Frequency penalty
    pub frequency_penalty: Option<f32>,
}

/// Chat completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaChatRequest {
    /// Model name
    pub model: String,
    /// Messages
    pub messages: Vec<OllamaMessage>,
    /// Whether to stream responses
    #[serde(default = "default_stream")]
    pub stream: bool,
    /// Generation options
    pub options: Option<ChatOptions>,
}

fn default_stream() -> bool {
    false
}

impl OllamaChatRequest {
    pub fn new(model: String, messages: Vec<OllamaMessage>) -> Self {
        Self {
            model,
            messages,
            stream: false,
            options: None,
        }
    }

    pub fn with_options(mut self, options: ChatOptions) -> Self {
        self.options = Some(options);
        self
    }

    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }
}

/// Chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaChatResponse {
    /// Model name
    pub model: String,
    /// Generated message
    pub message: OllamaMessage,
    /// Time taken to generate
    pub total_duration: Option<u64>,
    /// Number of prompt tokens
    pub prompt_eval_count: Option<u32>,
    /// Time taken for prompt evaluation
    pub prompt_eval_duration: Option<u64>,
    /// Number of generated tokens
    pub eval_count: Option<u32>,
    /// Time taken for generation
    pub eval_duration: Option<u64>,
}

/// Streaming chat response chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaChatChunk {
    /// Model name
    pub model: String,
    /// Generated message chunk
    pub message: OllamaMessage,
    /// Done flag
    #[serde(default)]
    pub done: bool,
}

/// Generate request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    #[serde(default = "default_stream")]
    pub stream: bool,
    pub options: Option<ChatOptions>,
}

/// Generate response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub model: String,
    pub response: String,
    #[serde(default)]
    pub done: bool,
}

/// Embed request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedRequest {
    pub model: String,
    pub input: String,
}

/// Embed response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedResponse {
    pub embedding: Vec<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let message = OllamaMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
        };
        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("\"role\":\"user\""));
    }

    #[test]
    fn test_chat_request() {
        let request = OllamaChatRequest::new(
            "llama2".to_string(),
            vec![
                OllamaMessage {
                    role: MessageRole::System,
                    content: "You are a helpful assistant".to_string(),
                },
                OllamaMessage {
                    role: MessageRole::User,
                    content: "Hello".to_string(),
                },
            ],
        );

        assert_eq!(request.model, "llama2");
        assert_eq!(request.messages.len(), 2);
        assert!(!request.stream);
    }

    #[test]
    fn test_chat_options() {
        let options = ChatOptions {
            temperature: Some(0.7),
            top_p: Some(0.9),
            num_predict: Some(100),
            ..Default::default()
        };

        let json = serde_json::to_string(&options).unwrap();
        assert!(json.contains("\"temperature\":0.7"));
        assert!(json.contains("\"top_p\":0.9"));
    }
}