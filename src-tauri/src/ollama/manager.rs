use super::{OllamaClient, OllamaClientConfig, OllamaModel, OllamaMessage, OllamaChatRequest};
use crate::db::Database;
use crate::utils::error::{AppError, Result};
use futures::StreamExt;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Ollama manager for managing local model operations
pub struct OllamaManager {
    client: Option<OllamaClient>,
    db: Arc<Database>,
    models: Arc<RwLock<Vec<OllamaModel>>>,
    is_connected: Arc<RwLock<bool>>,
}

impl OllamaManager {
    /// Create a new Ollama manager
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            client: None,
            db,
            models: Arc::new(RwLock::new(Vec::new())),
            is_connected: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize Ollama manager and connect to Ollama
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing Ollama manager");

        let config = OllamaClientConfig::default();
        let client = OllamaClient::new(config)
            .map_err(|e| AppError::Internal(format!("Failed to create Ollama client: {}", e)))?;

        // Check connection
        let connected = client.check_connection().await
            .map_err(|e| AppError::Internal(format!("Failed to check Ollama connection: {}", e)))?;

        if connected {
            self.client = Some(client);
            *self.is_connected.write().await = true;

            // Load available models
            self.refresh_models().await?;

            info!("Ollama manager initialized successfully");
        } else {
            warn!("Ollama is not running or not accessible");
            *self.is_connected.write().await = false;
        }

        Ok(())
    }

    /// Check if Ollama is connected
    pub async fn is_connected(&self) -> bool {
        *self.is_connected.read().await
    }

    /// Refresh the list of available models
    pub async fn refresh_models(&self) -> Result<()> {
        if let Some(client) = &self.client {
            let models = client.list_models().await
                .map_err(|e| AppError::Internal(format!("Failed to list models: {}", e)))?;

            let model_count = models.len();
            *self.models.write().await = models;
            info!("Refreshed {} models", model_count);
        }
        Ok(())
    }

    /// Get all available models
    pub async fn get_models(&self) -> Vec<OllamaModel> {
        self.models.read().await.clone()
    }

    /// Get a specific model by name
    pub async fn get_model(&self, name: &str) -> Option<OllamaModel> {
        let models = self.models.read().await;
        models.iter().find(|m| m.name == name).cloned()
    }

    /// Pull a model from Ollama registry
    pub async fn pull_model(&self, model_name: String) -> Result<()> {
        if let Some(client) = &self.client {
            client.pull_model(&model_name).await
                .map_err(|e| AppError::Internal(format!("Failed to pull model: {}", e)))?;

            // Refresh models after pulling
            self.refresh_models().await?;
            Ok(())
        } else {
            Err(AppError::Internal("Ollama client not initialized".to_string()))
        }
    }

    /// Delete a model
    pub async fn delete_model(&self, model_name: String) -> Result<()> {
        if let Some(client) = &self.client {
            client.delete_model(&model_name).await
                .map_err(|e| AppError::Internal(format!("Failed to delete model: {}", e)))?;

            // Refresh models after deleting
            self.refresh_models().await?;
            Ok(())
        } else {
            Err(AppError::Internal("Ollama client not initialized".to_string()))
        }
    }

    /// Send a chat completion request
    pub async fn chat(&self, request: OllamaChatRequest) -> Result<String> {
        if let Some(client) = &self.client {
            // Store conversation before consuming the request
            self.store_conversation(&request).await?;

            let response = client.chat(request).await
                .map_err(|e| AppError::Internal(format!("Chat request failed: {}", e)))?;

            Ok(response.message.content)
        } else {
            Err(AppError::Internal("Ollama client not initialized".to_string()))
        }
    }

    /// Send a streaming chat completion request
    pub async fn chat_stream(
        &self,
        request: OllamaChatRequest,
        mut callback: impl FnMut(String) + Send + 'static,
    ) -> Result<()> {
        if let Some(client) = &self.client {
            let mut stream = client.chat_stream(request).await
                .map_err(|e| AppError::Internal(format!("Streaming chat request failed: {}", e)))?;

            let mut full_response = String::new();

            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result
                    .map_err(|e| AppError::Internal(format!("Failed to read stream chunk: {}", e)))?;

                if !chunk.is_empty() {
                    callback(chunk.clone());
                    full_response.push_str(&chunk);
                }
            }

            Ok(())
        } else {
            Err(AppError::Internal("Ollama client not initialized".to_string()))
        }
    }

    /// Generate completion from a prompt
    pub async fn generate(&self, model: String, prompt: String) -> Result<String> {
        if let Some(client) = &self.client {
            client.generate(&model, &prompt).await
                .map_err(|e| AppError::Internal(format!("Generate request failed: {}", e)))
        } else {
            Err(AppError::Internal("Ollama client not initialized".to_string()))
        }
    }

    /// Embed text into vectors
    pub async fn embed(&self, model: String, input: String) -> Result<Vec<f32>> {
        if let Some(client) = &self.client {
            client.embed(&model, &input).await
                .map_err(|e| AppError::Internal(format!("Embed request failed: {}", e)))
        } else {
            Err(AppError::Internal("Ollama client not initialized".to_string()))
        }
    }

    /// Store conversation in database
    async fn store_conversation(&self, request: &OllamaChatRequest) -> Result<()> {
        // Create a session if one doesn't exist
        let session_id = Uuid::new_v4();

        // Create session
        let conn = self.db.conn();
        let conn_guard = conn.blocking_lock();
        conn_guard.execute(
            "INSERT INTO sessions (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![
                session_id.to_string(),
                format!("Ollama Chat: {}", request.model),
                chrono::Utc::now().to_rfc3339(),
                chrono::Utc::now().to_rfc3339(),
            ],
        ).map_err(|e| AppError::Database(e.to_string()))?;

        // Store messages
        for message in &request.messages {
            let msg_id = Uuid::new_v4();
            conn_guard.execute(
                "INSERT INTO messages (id, session_id, role, content, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    msg_id.to_string(),
                    session_id.to_string(),
                    serde_json::to_string(&message.role).unwrap(),
                    &message.content,
                    chrono::Utc::now().to_rfc3339(),
                ],
            ).map_err(|e| AppError::Database(e.to_string()))?;
        }

        debug!("Stored conversation in database");
        Ok(())
    }

    /// Set Ollama client configuration
    pub async fn set_config(&mut self, config: OllamaClientConfig) -> Result<()> {
        let client = OllamaClient::new(config)
            .map_err(|e| AppError::Internal(format!("Failed to create Ollama client: {}", e)))?;

        let connected = client.check_connection().await
            .map_err(|e| AppError::Internal(format!("Failed to check Ollama connection: {}", e)))?;

        if connected {
            self.client = Some(client);
            *self.is_connected.write().await = true;
            self.refresh_models().await?;
            info!("Ollama configuration updated successfully");
        } else {
            warn!("Failed to connect to Ollama with new configuration");
            *self.is_connected.write().await = false;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Ollama to be running
    async fn test_ollama_manager() {
        let db = Arc::new(Database::new(&std::path::PathBuf::from("/tmp/test")).unwrap());
        let mut manager = OllamaManager::new(db);

        manager.initialize().await.unwrap();
        assert!(manager.is_connected().await);

        let models = manager.get_models().await;
        assert!(!models.is_empty());
    }
}