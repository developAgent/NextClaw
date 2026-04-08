//! Anthropic chat commands
//! Provides Tauri commands for Anthropic Claude API

use crate::providers::{AnthropicConfig, AnthropicProvider, AnthropicMessage, MessageCreateRequest, AnthropicMessageRole};
use crate::utils::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use tauri::State;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Anthropic message creation request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMessageRequestPayload {
    pub model: String,
    pub messages: Vec<AnthropicMessagePayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMessagePayload {
    pub role: String,
    pub content: String,
}

/// Anthropic message response payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicMessageResponsePayload {
    pub id: String,
    pub content: String,
    pub model: String,
    pub stop_reason: Option<String>,
    pub usage: Option<UsagePayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePayload {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

/// Available Anthropic models
const ANTHROPIC_MODELS: &[&str] = &[
    "claude-3-opus-20240229",
    "claude-3-sonnet-20240229",
    "claude-3-haiku-20240307",
    "claude-2.1",
    "claude-2.0",
    "claude-instant-1.2",
];

/// Anthropic client state
#[derive(Clone)]
pub struct AnthropicState {
    pub provider: Arc<Mutex<Option<AnthropicProvider>>>,
}

/// Create an Anthropic message
#[tauri::command]
pub async fn create_anthropic_message(
    request: AnthropicMessageRequestPayload,
    state: State<'_, AnthropicState>,
) -> Result<AnthropicMessageResponsePayload> {
    let provider = {
        let provider_guard = state.provider.lock().await;
        provider_guard.as_ref()
            .ok_or_else(|| AppError::Validation("Anthropic provider not configured".to_string()))?
            .clone()
    };

    // Convert payload to internal types
    let messages: Vec<AnthropicMessage> = request.messages
        .into_iter()
        .map(|m| AnthropicMessage {
            role: match m.role.as_str() {
                "user" => AnthropicMessageRole::User,
                "assistant" => AnthropicMessageRole::Assistant,
                _ => return Err(AppError::Validation(format!("Invalid role: {}", m.role))),
            },
            content: crate::providers::AnthropicMessageContent::Text(m.content),
        })
        .collect();

    let mut message_request = MessageCreateRequest::new(request.model, messages);

    if let Some(system) = request.system {
        message_request = message_request.with_system(system);
    }

    if let Some(max_tokens) = request.max_tokens {
        message_request = message_request.with_max_tokens(max_tokens);
    }

    if let Some(temperature) = request.temperature {
        message_request = message_request.with_temperature(temperature);
    }

    let response = tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async {
            provider.create_message(message_request).await
        })
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))??;

    // Extract text content
    let content = response.content
        .iter()
        .filter_map(|block| block.text.as_ref().cloned())
        .collect::<Vec<_>>()
        .join("");

    Ok(AnthropicMessageResponsePayload {
        id: response.id,
        content,
        model: response.model,
        stop_reason: response.stop_reason,
        usage: response.usage.map(|u| UsagePayload {
            input_tokens: u.input_tokens,
            output_tokens: u.output_tokens,
        }),
    })
}

/// List available Anthropic models
#[tauri::command]
pub async fn list_anthropic_models() -> Result<Vec<String>> {
    Ok(ANTHROPIC_MODELS.to_vec())
}

/// Validate Anthropic API key
#[tauri::command]
pub async fn validate_anthropic_api_key(api_key: String) -> Result<bool> {
    let config = AnthropicConfig::new(api_key)
        .with_model("claude-3-haiku-20240307");

    let provider = AnthropicProvider::new(config)?;

    let is_valid = tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(async {
            provider.validate_api_key().await
        })
    })
    .await
    .map_err(|e| AppError::Internal(format!("Task join error: {}", e)))??;

    Ok(is_valid)
}

/// Configure Anthropic provider
#[tauri::command]
pub async fn configure_anthropic(
    api_key: String,
    model: Option<String>,
    state: State<'_, AnthropicState>,
) -> Result<()> {
    let config = AnthropicConfig::new(api_key);
    let config = if let Some(model) = model {
        config.with_model(model)
    } else {
        config
    };

    let provider = AnthropicProvider::new(config)?;

    let mut provider_guard = state.provider.lock().await;
    *provider_guard = Some(Arc::new(provider));

    Ok(())
}

/// Get Anthropic configuration status
#[tauri::command]
pub async fn get_anthropic_status(state: State<'_, AnthropicState>) -> Result<bool> {
    let provider_guard = state.provider.lock().await;
    Ok(provider_guard.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anthropic_models_list() {
        let models = list_anthropic_models();
        assert!(models.is_ok());
        let model_list = models.unwrap();
        assert!(!model_list.is_empty());
        assert!(model_list.contains(&"claude-3-opus-20240229".to_string()));
    }
}