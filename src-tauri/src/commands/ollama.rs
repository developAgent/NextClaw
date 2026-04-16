use crate::ollama::models::MessageRole;
use crate::ollama::{OllamaChatRequest, OllamaManager, OllamaMessage};
use crate::utils::error::Result;
use tauri::State;
use tracing::info;

/// Check if Ollama is connected
#[tauri::command]
pub async fn ollama_check_connection(manager: State<'_, OllamaManager>) -> Result<bool> {
    Ok(manager.is_connected().await)
}

/// List all available Ollama models
#[tauri::command]
pub async fn ollama_list_models(
    manager: State<'_, OllamaManager>,
) -> Result<Vec<crate::ollama::OllamaModel>> {
    Ok(manager.get_models().await)
}

/// Refresh the list of available models
#[tauri::command]
pub async fn ollama_refresh_models(manager: State<'_, OllamaManager>) -> Result<()> {
    manager.refresh_models().await
}

/// Get a specific model by name
#[tauri::command]
pub async fn ollama_get_model(
    name: String,
    manager: State<'_, OllamaManager>,
) -> Result<Option<crate::ollama::OllamaModel>> {
    Ok(manager.get_model(&name).await)
}

/// Pull a model from Ollama registry
#[tauri::command]
pub async fn ollama_pull_model(
    model_name: String,
    manager: State<'_, OllamaManager>,
) -> Result<()> {
    manager.pull_model(model_name).await
}

/// Delete a model
#[tauri::command]
pub async fn ollama_delete_model(
    model_name: String,
    manager: State<'_, OllamaManager>,
) -> Result<()> {
    manager.delete_model(model_name).await
}

/// Send a chat completion request to Ollama
#[tauri::command]
pub async fn ollama_chat(
    model: String,
    messages: Vec<OllamaMessage>,
    manager: State<'_, OllamaManager>,
) -> Result<String> {
    let request = OllamaChatRequest::new(model, messages);
    manager.chat(request).await
}

/// Generate completion from a prompt
#[tauri::command]
pub async fn ollama_generate(
    model: String,
    prompt: String,
    manager: State<'_, OllamaManager>,
) -> Result<String> {
    manager.generate(model, prompt).await
}

/// Embed text into vectors
#[tauri::command]
pub async fn ollama_embed(
    model: String,
    input: String,
    manager: State<'_, OllamaManager>,
) -> Result<Vec<f32>> {
    manager.embed(model, input).await
}
