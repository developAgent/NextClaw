use crate::agents::{Agent, AgentManager};
use crate::ai::client::{Message, MessageRole};
use crate::db::models::Session;
use crate::db::Database;
use crate::providers::{
    AnthropicConfig, AnthropicMessage, AnthropicProvider, ChatCompletionRequest, ChatMessage,
    MessageCreateRequest, MessageRole as ProviderMessageRole, OpenAIConfig, OpenAIProvider,
};
use crate::utils::error::{AppError, Result};
use secrecy::ExposeSecret;
use tauri::State;
use tracing::{debug, info};
use uuid::Uuid;

use std::sync::Arc;

/// Send a message through the session's bound agent and get a response
#[tauri::command]
pub async fn send_message(
    message: String,
    session_id: String,
    agent_id: Option<String>,
    db: State<'_, Database>,
    agent_manager: State<'_, Arc<AgentManager>>,
) -> Result<String> {
    info!("Sending message to session {}", session_id);

    let session_uuid = Uuid::parse_str(&session_id)
        .map_err(|e| AppError::Validation(format!("Invalid session ID: {e}")))?;

    let mut session = get_session(&db, session_uuid)?
        .ok_or_else(|| AppError::Validation(format!("Session not found: {session_id}")))?;

    let requested_agent_id = normalize_optional_string(agent_id);
    let resolved_agent_id = session
        .agent_id
        .clone()
        .or(requested_agent_id)
        .ok_or_else(|| {
            AppError::Validation("Please select an agent before sending a message".to_string())
        })?;

    let agent = agent_manager
        .get_agent(&resolved_agent_id)
        .await?
        .ok_or_else(|| AppError::Validation(format!("Agent not found: {resolved_agent_id}")))?;

    if session.agent_id.as_deref() != Some(agent.id.as_str()) {
        attach_agent_to_session(&db, session_uuid, &agent.id)?;
        session.agent_id = Some(agent.id.clone());
    }

    let existing_history = get_session_history(&db, session_uuid)?;
    if existing_history.is_empty() {
        if let Some(system_prompt) = normalize_optional_string(agent.system_prompt.clone()) {
            let system_msg = Message::new(session_uuid, MessageRole::System, system_prompt);
            store_message(&db, session_uuid, &system_msg)?;
        }
    }

    let user_msg = Message::new(session_uuid, MessageRole::User, message);
    store_message(&db, session_uuid, &user_msg)?;

    let history = get_session_history(&db, session_uuid)?;
    let response = generate_agent_response(&db, &agent, &history).await?;

    let assistant_msg = Message::new(session_uuid, MessageRole::Assistant, response.clone());
    store_message(&db, session_uuid, &assistant_msg)?;

    debug!("Response received for session {}", session_id);
    Ok(response)
}

/// Get chat history for a session
#[tauri::command]
pub fn get_chat_history(session_id: String, db: State<'_, Database>) -> Result<Vec<Message>> {
    let session_uuid = Uuid::parse_str(&session_id)
        .map_err(|e| AppError::Validation(format!("Invalid session ID: {e}")))?;

    get_session_history(&db, session_uuid)
}

/// Create a new chat session
#[tauri::command]
pub fn create_session(
    title: String,
    agent_id: Option<String>,
    db: State<'_, Database>,
) -> Result<Session> {
    let session = Session::new(title, normalize_optional_string(agent_id));
    save_session(&db, &session)?;
    info!("Created new session: {}", session.id);
    Ok(session)
}

/// List all chat sessions
#[tauri::command]
pub fn list_sessions(db: State<'_, Database>) -> Result<Vec<Session>> {
    list_all_sessions(&db)
}

/// Delete a chat session
#[tauri::command]
pub fn delete_session(session_id: String, db: State<'_, Database>) -> Result<()> {
    let session_uuid = Uuid::parse_str(&session_id)
        .map_err(|e| AppError::Validation(format!("Invalid session ID: {e}")))?;

    let conn = db.conn();
    let conn_guard = conn.blocking_lock();
    conn_guard
        .execute(
            "DELETE FROM sessions WHERE id = ?1",
            [&session_uuid.to_string()],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
    info!("Deleted session: {}", session_id);
    Ok(())
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn default_model_for_provider(provider: &str) -> &'static str {
    match provider {
        "openai" => "gpt-4o-mini",
        _ => "claude-3-sonnet-20240229",
    }
}

async fn generate_agent_response(
    db: &Database,
    agent: &Agent,
    history: &[Message],
) -> Result<String> {
    let provider = agent
        .provider_id
        .as_deref()
        .map(str::trim)
        .filter(|provider| !provider.is_empty())
        .unwrap_or("anthropic")
        .to_lowercase();

    let model = agent
        .model_id
        .as_deref()
        .map(str::trim)
        .filter(|model| !model.is_empty())
        .unwrap_or(default_model_for_provider(&provider))
        .to_string();

    let api_key = db
        .get_secret("api_key")?
        .ok_or_else(|| AppError::Authentication("API key is not configured".to_string()))?;
    let api_key = api_key.expose_secret().to_string();

    match provider.as_str() {
        "openai" => generate_openai_response(api_key, model, agent, history).await,
        "anthropic" | "claude" => generate_anthropic_response(api_key, model, agent, history).await,
        other => Err(AppError::Validation(format!(
            "Unsupported agent provider for chat: {other}"
        ))),
    }
}

async fn generate_openai_response(
    api_key: String,
    model: String,
    agent: &Agent,
    history: &[Message],
) -> Result<String> {
    let provider = OpenAIProvider::new(OpenAIConfig::new(api_key).with_model(model.clone()))
        .map_err(|e| AppError::Ai(format!("Failed to initialize OpenAI provider: {e}")))?;

    let messages = history
        .iter()
        .map(|message| ChatMessage {
            role: match message.role {
                MessageRole::System => ProviderMessageRole::System,
                MessageRole::User => ProviderMessageRole::User,
                MessageRole::Assistant => ProviderMessageRole::Assistant,
            },
            content: message.content.clone(),
        })
        .collect();

    let mut request = ChatCompletionRequest::new(model, messages);
    if let Some(temperature) = agent.temperature {
        request = request.with_temperature(temperature);
    }
    if let Some(max_tokens) = agent.max_tokens {
        request = request.with_max_tokens(max_tokens);
    }

    let response = provider
        .create_chat_completion(request)
        .await
        .map_err(|e| AppError::Ai(format!("OpenAI chat request failed: {e}")))?;

    response
        .choices
        .first()
        .map(|choice| choice.message.content.clone())
        .ok_or_else(|| AppError::Ai("OpenAI response did not include a message".to_string()))
}

async fn generate_anthropic_response(
    api_key: String,
    model: String,
    agent: &Agent,
    history: &[Message],
) -> Result<String> {
    let provider = AnthropicProvider::new(AnthropicConfig::new(api_key).with_model(model.clone()))
        .map_err(|e| AppError::Ai(format!("Failed to initialize Anthropic provider: {e}")))?;

    let system_prompt = history
        .iter()
        .filter(|message| matches!(message.role, MessageRole::System))
        .map(|message| message.content.trim())
        .filter(|content| !content.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n");

    let messages = history
        .iter()
        .filter_map(|message| match message.role {
            MessageRole::System => None,
            MessageRole::User => Some(AnthropicMessage::user(message.content.clone())),
            MessageRole::Assistant => Some(AnthropicMessage::assistant(message.content.clone())),
        })
        .collect();

    let mut request = MessageCreateRequest::new(model, messages);
    if !system_prompt.is_empty() {
        request = request.with_system(system_prompt);
    }
    if let Some(temperature) = agent.temperature {
        request = request.with_temperature(temperature);
    }
    if let Some(max_tokens) = agent.max_tokens {
        request = request.with_max_tokens(max_tokens);
    }

    let response = provider
        .create_message(request)
        .await
        .map_err(|e| AppError::Ai(format!("Anthropic chat request failed: {e}")))?;

    let content = response
        .content
        .into_iter()
        .filter_map(|block| match block {
            crate::providers::anthropic::ContentBlock {
                text: Some(text), ..
            } => Some(text),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");

    if content.trim().is_empty() {
        return Err(AppError::Ai(
            "Anthropic response did not include text content".to_string(),
        ));
    }

    Ok(content)
}

fn get_session(db: &Database, session_id: Uuid) -> Result<Option<Session>> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();
    let mut stmt = conn_guard
        .prepare("SELECT id, agent_id, title, created_at, updated_at FROM sessions WHERE id = ?1")
        .map_err(|e| AppError::Database(e.to_string()))?;

    let session = stmt
        .query_row([&session_id.to_string()], |row| {
            Ok(Session {
                id: Uuid::parse_str(&row.get::<_, String>(0)?).unwrap_or_default(),
                agent_id: row.get(1)?,
                title: row.get(2)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap_or_default()
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .unwrap_or_default()
                    .with_timezone(&chrono::Utc),
            })
        })
        .ok();

    Ok(session)
}

fn attach_agent_to_session(db: &Database, session_id: Uuid, agent_id: &str) -> Result<()> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();
    conn_guard
        .execute(
            "UPDATE sessions SET agent_id = ?1, updated_at = ?2 WHERE id = ?3",
            rusqlite::params![
                agent_id,
                &chrono::Utc::now().to_rfc3339(),
                &session_id.to_string()
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

fn touch_session(db: &Database, session_id: Uuid) -> Result<()> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();
    conn_guard
        .execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
            rusqlite::params![&chrono::Utc::now().to_rfc3339(), &session_id.to_string()],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

fn store_message(db: &Database, session_id: Uuid, message: &Message) -> Result<()> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();
    conn_guard
        .execute(
            r#"
            INSERT INTO messages (id, session_id, role, content)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            rusqlite::params![
                &message.id.to_string(),
                &session_id.to_string(),
                &message.role.to_string(),
                &message.content,
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
    drop(conn_guard);
    touch_session(db, session_id)?;
    Ok(())
}

fn get_session_history(db: &Database, session_id: Uuid) -> Result<Vec<Message>> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();
    let mut stmt = conn_guard
        .prepare(
            "SELECT id, role, content, created_at FROM messages WHERE session_id = ?1 ORDER BY created_at ASC",
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    let mut messages = Vec::new();
    let mut rows = stmt
        .query([&session_id.to_string()])
        .map_err(|e| AppError::Database(e.to_string()))?;

    while let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
        let id: String = row.get(0).map_err(|e| AppError::Database(e.to_string()))?;
        let role: String = row.get(1).map_err(|e| AppError::Database(e.to_string()))?;
        let content: String = row.get(2).map_err(|e| AppError::Database(e.to_string()))?;
        let created_at: String = row.get(3).map_err(|e| AppError::Database(e.to_string()))?;

        messages.push(Message {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            session_id,
            role: MessageRole::from(role.as_str()),
            content,
            timestamp: chrono::DateTime::parse_from_rfc3339(&created_at)
                .unwrap_or_default()
                .with_timezone(&chrono::Utc),
        });
    }

    Ok(messages)
}

fn save_session(db: &Database, session: &Session) -> Result<()> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();
    conn_guard
        .execute(
            r#"
            INSERT INTO sessions (id, agent_id, title, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            rusqlite::params![
                &session.id.to_string(),
                session.agent_id.as_deref(),
                &session.title,
                &session.created_at.to_rfc3339(),
                &session.updated_at.to_rfc3339(),
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

fn list_all_sessions(db: &Database) -> Result<Vec<Session>> {
    let conn = db.conn();
    let conn_guard = conn.blocking_lock();
    let mut stmt = conn_guard
        .prepare(
            "SELECT id, agent_id, title, created_at, updated_at FROM sessions ORDER BY updated_at DESC",
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

    let mut sessions = Vec::new();
    let mut rows = stmt
        .query([])
        .map_err(|e| AppError::Database(e.to_string()))?;

    while let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
        let id: String = row.get(0).map_err(|e| AppError::Database(e.to_string()))?;
        let agent_id: Option<String> = row.get(1).map_err(|e| AppError::Database(e.to_string()))?;
        let title: String = row.get(2).map_err(|e| AppError::Database(e.to_string()))?;
        let created_at: String = row.get(3).map_err(|e| AppError::Database(e.to_string()))?;
        let updated_at: String = row.get(4).map_err(|e| AppError::Database(e.to_string()))?;

        sessions.push(Session {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            agent_id,
            title,
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                .unwrap_or_default()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)
                .unwrap_or_default()
                .with_timezone(&chrono::Utc),
        });
    }

    Ok(sessions)
}
