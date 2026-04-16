use crate::db::Database;
use crate::utils::error::{AppError, Result};
use enigo::{
    Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings as EnigoSettings,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingEvent {
    pub id: String,
    pub event_type: String,
    pub payload: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: String,
    pub events: Vec<RecordingEvent>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Recording {
    pub fn new(name: String, description: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            status: "recording".to_string(),
            events: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_event(&mut self, event: RecordedEvent) {
        let now = chrono::Utc::now().timestamp();
        self.events.push(RecordingEvent {
            id: Uuid::new_v4().to_string(),
            event_type: event.event_type,
            payload: event.payload,
            created_at: now,
        });
        self.updated_at = now;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum RecordingState {
    Idle,
    Recording {
        started_at: u64,
    },
    Paused {
        paused_at: u64,
    },
    Stopped,
    PlayingBack {
        recording_id: String,
        started_at: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedEvent {
    pub event_type: String,
    pub payload: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackResult {
    pub recording_id: String,
    pub replayed_events: usize,
    pub skipped_events: usize,
    pub failed_events: usize,
    pub warnings: Vec<String>,
    pub finished_at: i64,
}

#[derive(Debug)]
pub struct Recorder {
    db: Arc<Database>,
    state: Arc<RwLock<RecordingState>>,
    current_recording: Arc<RwLock<Option<Recording>>>,
}

impl Recorder {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            state: Arc::new(RwLock::new(RecordingState::Idle)),
            current_recording: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn start_recording(&self, name: String, description: String) -> Result<Recording> {
        let mut state = self.state.write().await;
        if !matches!(*state, RecordingState::Idle | RecordingState::Stopped) {
            return Err(AppError::Validation(
                "Cannot start recording - recorder is not idle".to_string(),
            ));
        }

        let recording = Recording::new(name.trim().to_string(), description.trim().to_string());
        let mut current = self.current_recording.write().await;
        *current = Some(recording.clone());
        *state = RecordingState::Recording {
            started_at: chrono::Utc::now().timestamp_millis() as u64,
        };

        info!("Started recording: {}", recording.name);
        Ok(recording)
    }

    pub async fn stop_recording(&self) -> Result<Recording> {
        let mut state = self.state.write().await;
        if !matches!(
            *state,
            RecordingState::Recording { .. } | RecordingState::Paused { .. }
        ) {
            return Err(AppError::Validation("No recording in progress".to_string()));
        }

        let mut current = self.current_recording.write().await;
        let mut recording = current
            .take()
            .ok_or_else(|| AppError::Internal("No recording in progress".to_string()))?;

        recording.status = "stopped".to_string();
        recording.updated_at = chrono::Utc::now().timestamp();
        self.persist_recording(&recording)?;

        *state = RecordingState::Stopped;
        info!("Stopped recording: {}", recording.name);
        Ok(recording)
    }

    pub async fn pause_recording(&self) -> Result<()> {
        let mut state = self.state.write().await;
        if matches!(*state, RecordingState::Recording { .. }) {
            *state = RecordingState::Paused {
                paused_at: chrono::Utc::now().timestamp_millis() as u64,
            };
            info!("Paused recording");
            Ok(())
        } else {
            Err(AppError::Validation("No recording in progress".to_string()))
        }
    }

    pub async fn resume_recording(&self) -> Result<()> {
        let mut state = self.state.write().await;
        if matches!(*state, RecordingState::Paused { .. }) {
            *state = RecordingState::Recording {
                started_at: chrono::Utc::now().timestamp_millis() as u64,
            };
            info!("Resumed recording");
            Ok(())
        } else {
            Err(AppError::Validation("Recording is not paused".to_string()))
        }
    }

    pub async fn add_event(&self, event: RecordedEvent) -> Result<()> {
        let state = self.state.read().await;
        if !matches!(*state, RecordingState::Recording { .. }) {
            return Err(AppError::Validation("No recording in progress".to_string()));
        }
        drop(state);

        let mut current = self.current_recording.write().await;
        if let Some(recording) = current.as_mut() {
            recording.add_event(event);
            debug!("Added event to recording {}", recording.id);
            Ok(())
        } else {
            Err(AppError::Internal("No recording in progress".to_string()))
        }
    }

    pub async fn get_state(&self) -> RecordingState {
        self.state.read().await.clone()
    }

    pub async fn get_current_recording(&self) -> Option<Recording> {
        self.current_recording.read().await.clone()
    }

    pub fn list_recordings(&self) -> Result<Vec<Recording>> {
        let conn = self.db.conn();
        let conn_guard = conn.blocking_lock();
        let mut stmt = conn_guard
            .prepare(
                "SELECT id, name, description, status, events_json, created_at, updated_at FROM recordings ORDER BY updated_at DESC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        stmt.query_map([], |row| {
            let events_json: String = row.get(4)?;
            let events =
                serde_json::from_str::<Vec<RecordingEvent>>(&events_json).unwrap_or_default();
            Ok(Recording {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                status: row.get(3)?,
                events,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| AppError::Database(e.to_string()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| AppError::Database(e.to_string()))
    }

    pub fn get_recording(&self, id: &str) -> Result<Option<Recording>> {
        let conn = self.db.conn();
        let conn_guard = conn.blocking_lock();
        let mut stmt = conn_guard
            .prepare(
                "SELECT id, name, description, status, events_json, created_at, updated_at FROM recordings WHERE id = ?1",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let result = stmt.query_row([id], |row| {
            let events_json: String = row.get(4)?;
            let events =
                serde_json::from_str::<Vec<RecordingEvent>>(&events_json).unwrap_or_default();
            Ok(Recording {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                status: row.get(3)?,
                events,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            })
        });

        match result {
            Ok(recording) => Ok(Some(recording)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(AppError::Database(error.to_string())),
        }
    }

    pub fn delete_recording(&self, id: &str) -> Result<()> {
        let conn = self.db.conn();
        let conn_guard = conn.blocking_lock();
        conn_guard
            .execute("DELETE FROM recordings WHERE id = ?1", [id])
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn playback_recording(&self, id: &str) -> Result<PlaybackResult> {
        let recording = self
            .get_recording(id)?
            .ok_or_else(|| AppError::Validation(format!("Recording not found: {id}")))?;

        {
            let mut state = self.state.write().await;
            *state = RecordingState::PlayingBack {
                recording_id: recording.id.clone(),
                started_at: chrono::Utc::now().timestamp_millis() as u64,
            };
        }

        let playback_result = async {
            let mut enigo = Enigo::new(&EnigoSettings::default()).map_err(|error| {
                AppError::Execution(format!("Failed to initialize input playback: {error}"))
            })?;

            let mut replayed_events = 0_usize;
            let mut skipped_events = 0_usize;
            let mut warnings = Vec::new();

            for event in &recording.events {
                match execute_recording_event(&mut enigo, event).await? {
                    EventExecutionOutcome::Executed => {
                        replayed_events += 1;
                    }
                    EventExecutionOutcome::Skipped(reason) => {
                        skipped_events += 1;
                        let warning = format!("{}: {}", event.event_type, reason);
                        warn!("Skipped recording event {}: {}", event.id, warning);
                        warnings.push(warning);
                    }
                }
            }

            Ok::<PlaybackResult, AppError>(PlaybackResult {
                recording_id: recording.id.clone(),
                replayed_events,
                skipped_events,
                failed_events: 0,
                warnings,
                finished_at: chrono::Utc::now().timestamp(),
            })
        }
        .await;

        let mut state = self.state.write().await;
        *state = RecordingState::Idle;
        drop(state);

        match playback_result {
            Ok(result) => {
                info!("Playback finished: {}", recording.name);
                Ok(result)
            }
            Err(error) => {
                warn!("Playback failed for {}: {}", recording.name, error);
                Err(error)
            }
        }
    }

    fn persist_recording(&self, recording: &Recording) -> Result<()> {
        let conn = self.db.conn();
        let conn_guard = conn.blocking_lock();
        let events_json = serde_json::to_string(&recording.events)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        conn_guard
            .execute(
                "INSERT OR REPLACE INTO recordings (id, name, description, status, events_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    recording.id,
                    recording.name,
                    recording.description,
                    recording.status,
                    events_json,
                    recording.created_at,
                    recording.updated_at,
                ],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}

#[derive(Debug)]
enum EventExecutionOutcome {
    Executed,
    Skipped(String),
}

async fn execute_recording_event(
    enigo: &mut Enigo,
    event: &RecordingEvent,
) -> Result<EventExecutionOutcome> {
    match event.event_type.as_str() {
        "delay" | "sleep" | "wait" => execute_delay_event(&event.payload).await,
        "text" => execute_text_event(enigo, &event.payload),
        "key" | "keyboard" => execute_key_event(enigo, &event.payload),
        "mouse_move" | "move" => execute_mouse_move_event(enigo, &event.payload),
        "mouse_button" | "click" => execute_mouse_button_event(enigo, &event.payload),
        other => Ok(EventExecutionOutcome::Skipped(format!(
            "unsupported event type '{other}'"
        ))),
    }
}

async fn execute_delay_event(payload: &str) -> Result<EventExecutionOutcome> {
    let value = parse_payload_value(payload)?;
    let Some(duration_ms) = value
        .as_u64()
        .or_else(|| value.get("duration_ms").and_then(Value::as_u64))
        .or_else(|| value.get("ms").and_then(Value::as_u64))
    else {
        return Ok(EventExecutionOutcome::Skipped(
            "delay payload must be a number or contain duration_ms/ms".to_string(),
        ));
    };

    tokio::time::sleep(Duration::from_millis(duration_ms)).await;
    Ok(EventExecutionOutcome::Executed)
}

fn execute_text_event(enigo: &mut Enigo, payload: &str) -> Result<EventExecutionOutcome> {
    let value = parse_payload_value(payload)?;
    let Some(text) = value.as_str().map(ToOwned::to_owned).or_else(|| {
        value
            .get("text")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
    }) else {
        return Ok(EventExecutionOutcome::Skipped(
            "text payload must be a string or contain a text field".to_string(),
        ));
    };

    enigo
        .text(&text)
        .map_err(|error| AppError::Execution(format!("Failed to type text: {error}")))?;
    Ok(EventExecutionOutcome::Executed)
}

fn execute_key_event(enigo: &mut Enigo, payload: &str) -> Result<EventExecutionOutcome> {
    let value = parse_payload_value(payload)?;
    let Some(key_name) = value
        .get("key")
        .and_then(Value::as_str)
        .or_else(|| value.as_str())
    else {
        return Ok(EventExecutionOutcome::Skipped(
            "key payload must be a string or contain a key field".to_string(),
        ));
    };

    let Some(key) = map_key(key_name) else {
        return Ok(EventExecutionOutcome::Skipped(format!(
            "unsupported key '{key_name}'"
        )));
    };

    let direction = value
        .get("direction")
        .and_then(Value::as_str)
        .map(parse_direction)
        .transpose()?
        .unwrap_or(Direction::Click);

    enigo
        .key(key, direction)
        .map_err(|error| AppError::Execution(format!("Failed to simulate key input: {error}")))?;
    Ok(EventExecutionOutcome::Executed)
}

fn execute_mouse_move_event(enigo: &mut Enigo, payload: &str) -> Result<EventExecutionOutcome> {
    let value = parse_payload_value(payload)?;
    let Some(x) = value.get("x").and_then(Value::as_i64) else {
        return Ok(EventExecutionOutcome::Skipped(
            "mouse move payload must contain numeric x and y".to_string(),
        ));
    };
    let Some(y) = value.get("y").and_then(Value::as_i64) else {
        return Ok(EventExecutionOutcome::Skipped(
            "mouse move payload must contain numeric x and y".to_string(),
        ));
    };

    let coordinate = value
        .get("coordinate")
        .and_then(Value::as_str)
        .map(parse_coordinate)
        .transpose()?
        .unwrap_or(Coordinate::Abs);

    enigo
        .move_mouse(x as i32, y as i32, coordinate)
        .map_err(|error| AppError::Execution(format!("Failed to move mouse: {error}")))?;
    Ok(EventExecutionOutcome::Executed)
}

fn execute_mouse_button_event(enigo: &mut Enigo, payload: &str) -> Result<EventExecutionOutcome> {
    let value = parse_payload_value(payload)?;

    if let Some(x) = value.get("x").and_then(Value::as_i64) {
        let Some(y) = value.get("y").and_then(Value::as_i64) else {
            return Ok(EventExecutionOutcome::Skipped(
                "mouse button payload with x must also contain y".to_string(),
            ));
        };

        enigo
            .move_mouse(x as i32, y as i32, Coordinate::Abs)
            .map_err(|error| {
                AppError::Execution(format!("Failed to move mouse before click: {error}"))
            })?;
    }

    let button_name = value
        .get("button")
        .and_then(Value::as_str)
        .unwrap_or("left");
    let Some(button) = map_button(button_name) else {
        return Ok(EventExecutionOutcome::Skipped(format!(
            "unsupported mouse button '{button_name}'"
        )));
    };

    let direction = value
        .get("direction")
        .and_then(Value::as_str)
        .map(parse_direction)
        .transpose()?
        .unwrap_or(Direction::Click);

    enigo.button(button, direction).map_err(|error| {
        AppError::Execution(format!("Failed to simulate mouse button input: {error}"))
    })?;
    Ok(EventExecutionOutcome::Executed)
}

fn parse_payload_value(payload: &str) -> Result<Value> {
    serde_json::from_str::<Value>(payload).map_err(|error| {
        AppError::Validation(format!("Invalid recording event payload JSON: {error}"))
    })
}

fn parse_direction(value: &str) -> Result<Direction> {
    match value.trim().to_ascii_lowercase().as_str() {
        "click" | "tap" => Ok(Direction::Click),
        "press" | "down" => Ok(Direction::Press),
        "release" | "up" => Ok(Direction::Release),
        other => Err(AppError::Validation(format!(
            "Unsupported input direction: {other}"
        ))),
    }
}

fn parse_coordinate(value: &str) -> Result<Coordinate> {
    match value.trim().to_ascii_lowercase().as_str() {
        "abs" | "absolute" => Ok(Coordinate::Abs),
        "rel" | "relative" => Ok(Coordinate::Rel),
        other => Err(AppError::Validation(format!(
            "Unsupported coordinate mode: {other}"
        ))),
    }
}

fn map_button(value: &str) -> Option<Button> {
    match value.trim().to_ascii_lowercase().as_str() {
        "left" => Some(Button::Left),
        "middle" => Some(Button::Middle),
        "right" => Some(Button::Right),
        _ => None,
    }
}

fn map_key(value: &str) -> Option<Key> {
    let normalized = value.trim().to_ascii_lowercase();
    let key = match normalized.as_str() {
        "ctrl" | "control" => Key::Control,
        "alt" | "option" => Key::Alt,
        "shift" => Key::Shift,
        "meta" | "super" | "command" | "windows" => Key::Meta,
        "enter" | "return" => Key::Return,
        "space" => Key::Space,
        "tab" => Key::Tab,
        "escape" | "esc" => Key::Escape,
        "up" | "uparrow" | "arrowup" => Key::UpArrow,
        "down" | "downarrow" | "arrowdown" => Key::DownArrow,
        "left" | "leftarrow" | "arrowleft" => Key::LeftArrow,
        "right" | "rightarrow" | "arrowright" => Key::RightArrow,
        "backspace" => Key::Backspace,
        other => {
            if let Some(function_number) = other.strip_prefix('f') {
                return match function_number {
                    "1" => Some(Key::F1),
                    "2" => Some(Key::F2),
                    "3" => Some(Key::F3),
                    "4" => Some(Key::F4),
                    "5" => Some(Key::F5),
                    "6" => Some(Key::F6),
                    "7" => Some(Key::F7),
                    "8" => Some(Key::F8),
                    "9" => Some(Key::F9),
                    "10" => Some(Key::F10),
                    "11" => Some(Key::F11),
                    "12" => Some(Key::F12),
                    _ => None,
                };
            }

            let mut chars = other.chars();
            match (chars.next(), chars.next()) {
                (Some(character), None) => Key::Unicode(character),
                _ => return None,
            }
        }
    };

    Some(key)
}
