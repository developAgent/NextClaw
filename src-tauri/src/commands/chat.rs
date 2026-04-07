use crate::ai::client::{ClaudeClient, Message, MessageRole};
use crate::db::models::Session;
use crate::db::Database;
use crate::utils::error::Result;
use secrecy::SecretString;
use tauri::{State};
use uuid::Uuid;
use tracing::{debug, info};

/// Send a message to Claude and get a response
///
/// This is the core chat functionality
#[tauri::command]
pub async fn send_message(
    message: String,
    session_id: Uuid,
    db: State<'_, Database>,
    api_key: String,
) -> Result<String> {
    info!("Sending message to session {}", session_id);

    // Store user message
    let user_msg = Message::new(session_id, MessageRole::User, message.clone());
    store_message(&db, session_id, &user_msg).await?;

    // Get conversation history
    let history = get_session_history(&db, session_id).await?;

    // Create Claude client
    let secret_key = SecretString::new(api_key);
    let client = ClaudeClient::new(secret_key)?;

    // Send message and get response
    let response = client.send_message(&message, session_id).await?;

    // Store assistant message
    let assistant_msg = Message::new(session_id, MessageRole::Assistant, response.clone());
    store_message(&db, session_id, &assistant_msg).await?;

    debug!("Response received for session {}", session_id);
    Ok(response)
}

/// Get chat history for a session
#[tauri::command]
pub async fn get_chat_history(
    session_id: Uuid,
    db: State<'_, Database>,
) -> Result<Vec<Message>> {
    let messages = get_session_history(&db, session_id).await?;
    Ok(messages)
}

/// Create a new chat session
#[tauri::command]
pub async fn create_session(
    title: String,
    db: State<'_, Database>,
) -> Result<Session> {
    let session = Session::new(title);
    save_session(&db, &session).await?;
    info!("Created new session: {}", session.id);
    Ok(session)
}

/// List all chat sessions
#[tauri::command]
pub async fn list_sessions(
    db: State<'_, Database>,
) -> Result<Vec<Session>> {
    let sessions = list_all_sessions(&db).await?;
    Ok(sessions)
}

/// Delete a chat session
#[tauri::command]
pub async fn delete_session(
    session_id: Uuid,
    db: State<'_, Database>,
) -> Result<()> {
    db.execute(
        "DELETE FROM sessions WHERE id = ?1",
        &[&session_id.to_string()]
    ).map_err(|e| crate::utils::error::AppError::Database(e.to_string()))?;
    info!("Deleted session: {}", session_id);
    Ok(())
}

// Helper functions

async fn store_message(db: &Database, session_id: Uuid, message: &Message) -> Result<()> {
    db.execute(
        r#"
        INSERT INTO messages (id, session_id, role, content)
        VALUES (?1, ?2, ?3, ?4)
        "#,
        &[
            &message.id.to_string(),
            &session_id.to_string(),
            &message.role.as_str(),
            &message.content,
        ]
    ).map_err(|e| crate::utils::error::AppError::Database(e.to_string()))?;
    Ok(())
}

async fn get_session_history(db: &Database, session_id: Uuid) -> Result<Vec<Message>> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, role, content, created_at FROM messages WHERE session_id = ?1 ORDER BY created_at ASC"
    ).map_err(|e| crate::utils::error::AppError::Database(e.to_string()))?;

    let mut messages = Vec::new();
    let mut rows = stmt.query(&[&session_id.to_string()])
        .map_err(|e| crate::utils::error::AppError::Database(e.to_string()))?;

    while let Some(row) = rows.next().map_err(|e| crate::utils::error::AppError::Database(e.to_string()))? {
        let id: String = row.get(0)?;
        let role: String = row.get(1)?;
        let content: String = row.get(2)?;
        let created_at: String = row.get(3)?;

        messages.push(Message {
            id: Uuid::parse_str(&id).unwrap_or_default(),
            session_id,
            role: MessageRole::from(role.as_str()),
            content,
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at)
                .unwrap_or_default()
                .with_timezone(&chrono::Utc),
        });
    }

    Ok(messages)
}

async fn save_session(db: &Database, session: &Session) -> Result<()> {
    db.execute(
        r#"
        INSERT INTO sessions (id, title, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4)
        "#,
        &[
            &session.id.to_string(),
            &session.title,
            &session.created_at.to_rfc3339(),
            &session.updated_at.to_rfc3339(),
        ]
    ).map_err(|e| crate::utils::error::AppError::Database(e.to_string()))?;
    Ok(())
}

async fn list_all_sessions(db: &Database) -> Result<Vec<Session>> {
    let conn = db.conn();
    let mut stmt = conn.prepare(
        "SELECT id, title, created_at, updated_at FROM sessions ORDER BY updated_at DESC"
    ).map_err(|e| crate::utils::error::AppError::Database(e.to_string()))?;

    let mut sessions = Vec::new();
    let mut rows = stmt.query([])
        .map_err(|e| crate::utils::error::AppError::Database(e.to_string()))?;

    while let Some(row) = rows.next().map_err(|e| crate::utils::error::AppError::Database(e.to_string()))? {
        let id: String = row.get(0)?;
        let title: String = row.get(1)?;
        let created_at: String = row.get(2)?;
        let updated_at: String = row.get(3)?;

        sessions.push(Session {
            id: Uuid::parse_str(&id).unwrap_or_default(),
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