//! Agent management
//! Provides CRUD operations for AI agents

use crate::utils::error::{AppError, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub created_at: String,
    pub updated_at: String,
}

/// Create agent request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub description: Option<String>,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

/// Update agent request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAgentRequest {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub provider_id: Option<String>,
    pub model_id: Option<String>,
    pub system_prompt: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

impl Agent {
    pub fn new(name: impl Into<String>) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            description: None,
            provider_id: None,
            model_id: None,
            system_prompt: None,
            temperature: None,
            max_tokens: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider_id = Some(provider.into());
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model_id = Some(model.into());
        self
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = Some(tokens);
        self
    }
}

/// Agent manager
pub struct AgentManager {
    db: Arc<tokio::sync::Mutex<Connection>>,
}

impl AgentManager {
    pub fn new(db: Arc<tokio::sync::Mutex<Connection>>) -> Self {
        Self { db }
    }

    /// Create a new agent
    pub async fn create_agent(&self, request: CreateAgentRequest) -> Result<Agent> {
        let agent = Agent::new(&request.name)
            .with_description(request.description.unwrap_or_default())
            .with_provider(request.provider_id.unwrap_or_default())
            .with_model(request.model_id.unwrap_or_default())
            .with_system_prompt(request.system_prompt.unwrap_or_default());

        let db = self.db.lock().await;
        db.execute(
            r#"
            INSERT INTO agents (id, name, description, provider_id, model_id, system_prompt, temperature, max_tokens, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                &agent.id,
                &agent.name,
                &agent.description,
                &agent.provider_id,
                &agent.model_id,
                &agent.system_prompt,
                agent.temperature,
                agent.max_tokens,
                &agent.created_at,
                &agent.updated_at,
            ],
        ).map_err(|e| AppError::Database(format!("Failed to create agent: {}", e)))?;

        info!("Created agent: {}", agent.name);
        Ok(agent)
    }

    /// Get all agents
    pub async fn get_all_agents(&self) -> Result<Vec<Agent>> {
        let db = self.db.lock().await;
        let mut stmt = db.prepare(
            "SELECT id, name, description, provider_id, model_id, system_prompt, temperature, max_tokens, created_at, updated_at
             FROM agents ORDER BY created_at DESC"
        ).map_err(|e| AppError::Database(format!("Failed to query agents: {}", e)))?;

        let agents = stmt
            .query_map([], |row| {
                #[allow(clippy::too_many_arguments)]
                Ok(Agent {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    provider_id: row.get(3)?,
                    model_id: row.get(4)?,
                    system_prompt: row.get(5)?,
                    temperature: row.get(6)?,
                    max_tokens: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })
            .map_err(|e| AppError::Database(format!("Failed to query agents: {}", e)))?
            .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()
            .map_err(|e| AppError::Database(format!("Failed to map agents: {}", e)))?;

        debug!("Retrieved {} agents", agents.len());
        Ok(agents)
    }

    /// Get a specific agent
    pub async fn get_agent(&self, id: &str) -> Result<Option<Agent>> {
        let db = self.db.lock().await;
        let mut stmt = db.prepare(
            "SELECT id, name, description, provider_id, model_id, system_prompt, temperature, max_tokens, created_at, updated_at
             FROM agents WHERE id = ?1"
        ).map_err(|e| AppError::Database(format!("Failed to query agent: {}", e)))?;

        let agent = stmt
            .query_row(params![id], |row| {
                Ok(Agent {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    provider_id: row.get(3)?,
                    model_id: row.get(4)?,
                    system_prompt: row.get(5)?,
                    temperature: row.get(6)?,
                    max_tokens: row.get(7)?,
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                })
            })
            .ok();

        Ok(agent)
    }

    /// Update an agent
    pub async fn update_agent(&self, request: UpdateAgentRequest) -> Result<Agent> {
        let db = self.db.lock().await;

        // Build dynamic update query
        let mut updates = vec![];
        let mut params: Vec<Box<dyn rusqlite::ToSql + Send>> = vec![];

        if let Some(name) = &request.name {
            updates.push("name = ?");
            params.push(Box::new(name.clone()));
        }
        if let Some(desc) = &request.description {
            updates.push("description = ?");
            params.push(Box::new(desc.clone()));
        }
        if let Some(provider) = &request.provider_id {
            updates.push("provider_id = ?");
            params.push(Box::new(provider.clone()));
        }
        if let Some(model) = &request.model_id {
            updates.push("model_id = ?");
            params.push(Box::new(model.clone()));
        }
        if let Some(prompt) = &request.system_prompt {
            updates.push("system_prompt = ?");
            params.push(Box::new(prompt.clone()));
        }
        if let Some(temp) = request.temperature {
            updates.push("temperature = ?");
            params.push(Box::new(temp));
        }
        if let Some(tokens) = request.max_tokens {
            updates.push("max_tokens = ?");
            params.push(Box::new(tokens));
        }

        updates.push("updated_at = ?");
        let now = chrono::Utc::now().to_rfc3339();
        params.push(Box::new(now.clone()));
        params.push(Box::new(request.id.clone()));

        let query = format!(
            "UPDATE agents SET {} WHERE id = ?{}",
            updates.join(", "),
            params.len() - updates.len() - 1
        );

        let count = db
            .execute(&query, rusqlite::params_from_iter(params))
            .map_err(|e| AppError::Database(format!("Failed to update agent: {}", e)))?;

        if count == 0 {
            return Err(AppError::Validation(format!(
                "Agent not found: {}",
                request.id
            )));
        }

        // Return updated agent
        self.get_agent(&request.id)
            .await?
            .ok_or_else(|| AppError::Internal("Failed to retrieve updated agent".to_string()))
    }

    /// Delete an agent
    pub async fn delete_agent(&self, id: &str) -> Result<()> {
        let db = self.db.lock().await;

        let count = db
            .execute("DELETE FROM agents WHERE id = ?1", params![id])
            .map_err(|e| AppError::Database(format!("Failed to delete agent: {}", e)))?;

        if count == 0 {
            return Err(AppError::Validation(format!("Agent not found: {}", id)));
        }

        info!("Deleted agent: {}", id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new("Test Agent")
            .with_description("A test agent")
            .with_temperature(0.7);

        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.description, Some("A test agent".to_string()));
        assert_eq!(agent.temperature, Some(0.7));
    }
}
