use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};
use std::time::Duration;

/// Ollama client configuration
#[derive(Debug, Clone)]
pub struct OllamaClientConfig {
    /// Base URL for Ollama API (default: http://localhost:11434)
    pub base_url: String,
    /// Request timeout (default: 120 seconds)
    pub timeout: Duration,
}

impl Default for OllamaClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            timeout: Duration::from_secs(120),
        }
    }
}

/// Ollama API client
pub struct OllamaClient {
    client: reqwest::Client,
    config: OllamaClientConfig,
}

impl OllamaClient {
    /// Create a new Ollama client
    pub fn new(config: OllamaClientConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self { client, config })
    }

    /// Create client with default configuration
    pub fn default() -> Result<Self> {
        Self::new(OllamaClientConfig::default())
    }

    /// Check if Ollama is running and accessible
    pub async fn check_connection(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.config.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to connect to Ollama")?;

        Ok(response.status().is_success())
    }

    /// List all available models
    pub async fn list_models(&self) -> Result<Vec<crate::ollama::models::OllamaModel>> {
        let url = format!("{}/api/tags", self.config.base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to list models")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to list models: {}", response.status());
        }

        let models_response: crate::ollama::models::ListModelsResponse = response
            .json()
            .await
            .context("Failed to parse models response")?;

        Ok(models_response.models)
    }

    /// Get information about a specific model
    pub async fn show_model_info(&self, model_name: &str) -> Result<crate::ollama::models::ModelInfo> {
        let url = format!("{}/api/show", self.config.base_url);
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({ "name": model_name }))
            .send()
            .await
            .context("Failed to get model info")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get model info: {}", response.status());
        }

        let model_info: crate::ollama::models::ModelInfo = response
            .json()
            .await
            .context("Failed to parse model info")?;

        Ok(model_info)
    }

    /// Pull a model from Ollama registry
    pub async fn pull_model(&self, model_name: &str) -> Result<crate::ollama::models::PullProgress> {
        let url = format!("{}/api/pull", self.config.base_url);
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({ "name": model_name, "stream": false }))
            .send()
            .await
            .context("Failed to pull model")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to pull model: {}", response.status());
        }

        let progress: crate::ollama::models::PullProgress = response
            .json()
            .await
            .context("Failed to parse pull response")?;

        info!("Pulled model: {}", model_name);
        Ok(progress)
    }

    /// Delete a model
    pub async fn delete_model(&self, model_name: &str) -> Result<()> {
        let url = format!("{}/api/delete", self.config.base_url);
        let response = self.client
            .delete(&url)
            .json(&serde_json::json!({ "name": model_name }))
            .send()
            .await
            .context("Failed to delete model")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to delete model: {}", response.status());
        }

        info!("Deleted model: {}", model_name);
        Ok(())
    }

    /// Create a model from a modelfile
    pub async fn create_model(&self, name: &str, modelfile: &str) -> Result<()> {
        let url = format!("{}/api/create", self.config.base_url);
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "name": name,
                "modelfile": modelfile,
                "stream": false
            }))
            .send()
            .await
            .context("Failed to create model")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to create model: {}", response.status());
        }

        info!("Created model: {}", name);
        Ok(())
    }

    /// Send a chat completion request
    pub async fn chat(&self, request: crate::ollama::models::OllamaChatRequest) -> Result<crate::ollama::models::OllamaChatResponse> {
        let url = format!("{}/api/chat", self.config.base_url);
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "model": request.model,
                "messages": request.messages,
                "stream": request.stream,
                "options": request.options,
            }))
            .send()
            .await
            .context("Failed to send chat request")?;

        if !response.status().is_success() {
            anyhow::bail!("Chat request failed: {}", response.status());
        }

        let chat_response: crate::ollama::models::OllamaChatResponse = response
            .json()
            .await
            .context("Failed to parse chat response")?;

        debug!("Chat response received from model: {}", request.model);
        Ok(chat_response)
    }

    /// Send a chat completion request with streaming
    pub async fn chat_stream(
        &self,
        request: crate::ollama::models::OllamaChatRequest,
    ) -> Result<impl futures::Stream<Item = Result<String>>> {
        let url = format!("{}/api/chat", self.config.base_url);

        let mut request_obj = serde_json::json!({
            "model": request.model,
            "messages": request.messages,
            "stream": true,
        });

        if let Some(options) = request.options {
            request_obj["options"] = serde_json::to_value(options)?;
        }

        let response = self.client
            .post(&url)
            .json(&request_obj)
            .send()
            .await
            .context("Failed to send streaming chat request")?;

        if !response.status().is_success() {
            anyhow::bail!("Streaming chat request failed: {}", response.status());
        }

        let stream = response.bytes_stream().map(|bytes| {
            bytes.context("Failed to read stream")?
                .split(|&b| b == b'\n')
                .filter_map(|line| {
                    if line.is_empty() {
                        None
                    } else {
                        Some(String::from_utf8(line.to_vec()).context("Invalid UTF-8"))
                    }
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .map(|line| {
                    let chunk: serde_json::Value = serde_json::from_str(&line)
                        .context("Failed to parse JSON chunk")?;
                    chunk["message"]["content"]
                        .as_str()
                        .map(|s| Ok(s.to_string()))
                        .unwrap_or_else(|| Ok(String::new()))
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?
        }).flatten();

        Ok(futures::stream::iter(stream))
    }

    /// Generate completion from a prompt
    pub async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let url = format!("{}/api/generate", self.config.base_url);
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "model": model,
                "prompt": prompt,
                "stream": false,
            }))
            .send()
            .await
            .context("Failed to send generate request")?;

        if !response.status().is_success() {
            anyhow::bail!("Generate request failed: {}", response.status());
        }

        let generate_response: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse generate response")?;

        let response_text = generate_response["response"]
            .as_str()
            .context("No response in generate result")?;

        Ok(response_text.to_string())
    }

    /// Embed text into vectors
    pub async fn embed(&self, model: &str, input: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/embed", self.config.base_url);
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "model": model,
                "input": input,
            }))
            .send()
            .await
            .context("Failed to send embed request")?;

        if !response.status().is_success() {
            anyhow::bail!("Embed request failed: {}", response.status());
        }

        let embed_response: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse embed response")?;

        let embedding = embed_response["embedding"]
            .as_array()
            .context("No embedding in result")?;

        let vector = embedding
            .iter()
            .map(|v| v.as_f64().context("Not a float")?.map(|f| f as f32))
            .collect::<Result<Vec<_>>>()?;

        Ok(vector)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Ollama to be running
    async fn test_ollama_connection() {
        let client = OllamaClient::default().unwrap();
        let is_connected = client.check_connection().await.unwrap();
        assert!(is_connected);
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_models() {
        let client = OllamaClient::default().unwrap();
        let models = client.list_models().await.unwrap();
        assert!(!models.is_empty());
    }
}