//! Chat V2 commands with OpenAI integration
//! Provides Tauri commands for AI chat functionality

use crate::providers::{OpenAIConfig, OpenAIProvider, ChatMessage, MessageRole, ChatCompletionRequest};
use crate::utils::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Chat completion request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequestPayload {
    pub model: String,
    pub messages: Vec<ChatMessagePayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessagePayload {
    pub role: String,
    pub content: String,
}

/// Chat completion response payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponsePayload {
    pub id: String,
    pub model: String,
    pub content: String,
    pub finish_reason: Option<String>,
    pub usage: Option<UsagePayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePayload {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Model info payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfoPayload {
    pub id: String,
    pub display_name: String,
    pub context_window: Option<u32>,
}

/// OpenAI client state
#[derive(Clone)]
pub struct OpenAIState {
    pub provider: Arc<Mutex<Option<Arc<OpenAIProvider>>>>,
}

/// Create a chat completion
#[tauri::command]
pub async fn create_chat_completion(
    request: ChatCompletionRequestPayload,
    state: State<'_, OpenAIState>,
) -> Result<ChatCompletionResponsePayload> {
    let provider = {
        let provider_guard = state.provider.lock().await;
        provider_guard.as_ref()
            .ok_or_else(|| AppError::Validation("OpenAI provider not configured".to_string()))?
            .clone()
    };

    // Convert payload to internal types
    let messages: Vec<ChatMessage> = request.messages
        .into_iter()
        .map(|m| -> Result<ChatMessage> {
            Ok(ChatMessage {
                role: match m.role.as_str() {
                    "system" => MessageRole::System,
                    "user" => MessageRole::User,
                    "assistant" => MessageRole::Assistant,
                    _ => return Err(AppError::Validation(format!("Invalid role: {}", m.role))),
                },
                content: m.content,
            })
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut completion_request = ChatCompletionRequest::new(request.model, messages);

    if let Some(temp) = request.temperature {
        completion_request = completion_request.with_temperature(temp);
    }

    if let Some(tokens) = request.max_tokens {
        completion_request = completion_request.with_max_tokens(tokens);
    }

    let response = tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async {
            provider.create_chat_completion(completion_request).await
        })
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))??;

    let choice = response.choices.first()
        .ok_or_else(|| AppError::Internal("No choices in response".to_string()))?;

    Ok(ChatCompletionResponsePayload {
        id: response.id,
        model: response.model,
        content: choice.message.content.clone(),
        finish_reason: choice.finish_reason.clone(),
        usage: response.usage.map(|u| UsagePayload {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        }),
    })
}

/// List available models
#[tauri::command]
pub async fn list_models(
    state: State<'_, OpenAIState>,
) -> Result<Vec<ModelInfoPayload>> {
    let provider = {
        let provider_guard = state.provider.lock().await;
        provider_guard.as_ref()
            .ok_or_else(|| AppError::Validation("OpenAI provider not configured".to_string()))?
            .clone()
    };

    let models_response = tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async {
            provider.list_models().await
        })
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))??;

    let models: Vec<ModelInfoPayload> = models_response.data
        .into_iter()
        .map(|m| ModelInfoPayload {
            id: m.id.clone(),
            display_name: m.id,
            context_window: None, // Could be fetched from API if available
        })
        .collect();

    Ok(models)
}

/// Validate API key
#[tauri::command]
pub async fn validate_api_key(
    api_key: String,
    base_url: Option<String>,
) -> Result<bool> {
    let config = OpenAIConfig::new(api_key)
        .with_base_url(base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()));

    let provider = OpenAIProvider::new(config)?;

    let is_valid = tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async {
            provider.validate_api_key().await
        })
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))??;

    Ok(is_valid)
}

/// Configure OpenAI provider
#[tauri::command]
pub async fn configure_openai(
    api_key: String,
    base_url: Option<String>,
    state: State<'_, OpenAIState>,
) -> Result<()> {
    let config = OpenAIConfig::new(api_key)
        .with_base_url(base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()));

    let provider = OpenAIProvider::new(config)?;

    let mut provider_guard = state.provider.lock().await;
    *provider_guard = Some(Arc::new(provider));

    Ok(())
}

/// Get OpenAI configuration status
#[tauri::command]
pub async fn get_openai_status(state: State<'_, OpenAIState>) -> Result<bool> {
    let provider_guard = state.provider.lock().await;
    Ok(provider_guard.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_payload_serialization() {
        let payload = ChatMessagePayload {
            role: "user".to_string(),
            content: "Hello".to_string(),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("user"));
        assert!(json.contains("Hello"));
    }
}