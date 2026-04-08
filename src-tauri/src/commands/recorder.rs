use crate::recorder::{Recording, RecordingState, RecordedEvent};
use crate::utils::error::Result;
use tauri::State;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// 录制管理器
pub struct Recorder {
    state: Arc<RwLock<RecordingState>>,
    current_recording: Arc<RwLock<Option<Recording>>>,
}

impl Recorder {
    /// 创建新的录制管理器
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(RecordingState::Idle)),
            current_recording: Arc::new(RwLock::new(None)),
        }
    }

    /// 开始录制
    pub async fn start_recording(&self, name: String, description: String) -> Result<Recording> {
        let mut state = self.state.write().await;

        if !matches!(*state, RecordingState::Idle) {
            return Err(crate::utils::error::AppError::Validation(
                "Cannot start recording - recorder is not idle".to_string(),
            ));
        }

        let recording = Recording::new(name, description);

        let mut current = self.current_recording.write().await;
        *current = Some(recording.clone());

        *state = RecordingState::Recording {
            started_at: chrono::Utc::now().timestamp_millis() as u64,
        };

        info!("Started recording: {}", recording.name);
        Ok(recording)
    }

    /// 停止录制
    pub async fn stop_recording(&self) -> Result<Recording> {
        let mut state = self.state.write().await;

        let recording = if let RecordingState::Recording { .. } = *state {
            let mut current = self.current_recording.write().await;
            let recording = current.take()
                .ok_or_else(|| crate::utils::error::AppError::Internal("No recording in progress".to_string()))?;

            recording
        } else {
            return Err(crate::utils::error::AppError::Validation(
                "No recording in progress".to_string(),
            ));
        };

        *state = RecordingState::Stopped;

        info!("Stopped recording: {}", recording.name);
        Ok(recording)
    }

    /// 暂停录制
    pub async fn pause_recording(&self) -> Result<()> {
        let mut state = self.state.write().await;

        if let RecordingState::Recording { .. } = *state {
            *state = RecordingState::Paused {
                paused_at: chrono::Utc::now().timestamp_millis() as u64,
            };
            info!("Paused recording");
            Ok(())
        } else {
            Err(crate::utils::error::AppError::Validation(
                "No recording in progress".to_string(),
            ))
        }
    }

    /// 恢复录制
    pub async fn resume_recording(&self) -> Result<()> {
        let mut state = self.state.write().await;

        if let RecordingState::Paused { .. } = *state {
            *state = RecordingState::Recording {
                started_at: chrono::Utc::now().timestamp_millis() as u64,
            };
            info!("Resumed recording");
            Ok(())
        } else {
            Err(crate::utils::error::AppError::Validation(
                "Recording is not paused".to_string(),
            ))
        }
    }

    /// 添加事件到当前录制
    pub async fn add_event(&self, event: RecordedEvent) -> Result<()> {
        let state = self.state.read().await;

        if !matches!(*state, RecordingState::Recording { .. }) {
            return Err(crate::utils::error::AppError::Validation(
                "No recording in progress".to_string(),
            ));
        }

        let mut current = self.current_recording.write().await;
        if let Some(recording) = current.as_mut() {
            recording.add_event(event);
            debug!("Added event: {}", event.id());
            Ok(())
        } else {
            Err(crate::utils::error::AppError::Internal(
                "No recording in progress".to_string(),
            ))
        }
    }

    /// 获取当前录制状态
    pub async fn get_state(&self) -> RecordingState {
        self.state.read().await.clone()
    }

    /// 获取当前录制
    pub async fn get_current_recording(&self) -> Option<Recording> {
        self.current_recording.read().await.clone()
    }
}

impl Default for Recorder {
    fn default() -> Self {
        Self::new()
    }
}

/// 开始录制
#[tauri::command]
pub async fn start_recording(
    name: String,
    description: String,
    recorder: State<'_, Arc<Recorder>>,
) -> Result<Recording> {
    recorder.start_recording(name, description).await
}

/// 停止录制
#[tauri::command]
pub async fn stop_recording(
    recorder: State<'_, Arc<Recorder>>,
) -> Result<Recording> {
    recorder.stop_recording().await
}

/// 暂停录制
#[tauri::command]
pub async fn pause_recording(
    recorder: State<'_, Arc<Recorder>>,
) -> Result<()> {
    recorder.pause_recording().await
}

/// 恢复录制
#[tauri::command]
pub async fn resume_recording(
    recorder: State<'_, Arc<Recorder>>,
) -> Result<()> {
    recorder.resume_recording().await
}

/// 添加事件
#[tauri::command]
pub async fn add_event(
    event: RecordedEvent,
    recorder: State<'_, Arc<Recorder>>,
) -> Result<()> {
    recorder.add_event(event).await
}

/// 获取录制状态
#[tauri::command]
pub async fn get_recording_state(
    recorder: State<'_, Arc<Recorder>>,
) -> RecordingState {
    recorder.get_state().await
}

/// 获取当前录制
#[tauri::command]
pub async fn get_current_recording(
    recorder: State<'_, Arc<Recorder>>,
) -> Option<Recording> {
    recorder.get_current_recording().await
}