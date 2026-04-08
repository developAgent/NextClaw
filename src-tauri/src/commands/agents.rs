//! Agent management commands
//! Provides Tauri commands for agent CRUD operations

use crate::agents::{AgentManager, CreateAgentRequest, UpdateAgentRequest};
use crate::utils::error::Result;
use tauri::State;
use std::sync::Arc;

/// Create a new agent
#[tauri::command]
pub async fn create_agent(
    request: CreateAgentRequest,
    manager: State<'_, Arc<AgentManager>>,
) -> Result<crate::agents::Agent> {
    manager.create_agent(request).await
}

/// Get all agents
#[tauri::command]
pub async fn get_all_agents(
    manager: State<'_, Arc<AgentManager>>,
) -> Result<Vec<crate::agents::Agent>> {
    manager.get_all_agents().await
}

/// Get a specific agent
#[tauri::command]
pub async fn get_agent(
    id: String,
    manager: State<'_, Arc<AgentManager>>,
) -> Result<Option<crate::agents::Agent>> {
    manager.get_agent(&id).await
}

/// Update an agent
#[tauri::command]
pub async fn update_agent(
    request: UpdateAgentRequest,
    manager: State<'_, Arc<AgentManager>>,
) -> Result<crate::agents::Agent> {
    manager.update_agent(request).await
}

/// Delete an agent
#[tauri::command]
pub async fn delete_agent(
    id: String,
    manager: State<'_, Arc<AgentManager>>,
) -> Result<()> {
    manager.delete_agent(&id).await
}

/// Clone an agent
#[tauri::command]
pub async fn clone_agent(
    id: String,
    new_name: Option<String>,
    manager: State<'_, Arc<AgentManager>>,
) -> Result<crate::agents::Agent> {
    let original = manager.get_agent(&id).await?
        .ok_or_else(|| crate::utils::error::AppError::Validation(format!("Agent not found: {}", id)))?;

    let request = CreateAgentRequest {
        name: new_name.unwrap_or_else(|| format!("{} (Copy)", original.name)),
        description: original.description,
        provider_id: original.provider_id,
        model_id: original.model_id,
        system_prompt: original.system_prompt,
        temperature: original.temperature,
        max_tokens: original.max_tokens,
    };

    manager.create_agent(request).await
}