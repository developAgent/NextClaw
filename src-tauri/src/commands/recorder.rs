use crate::recorder::{PlaybackResult, RecordedEvent, Recorder, Recording, RecordingState};
use crate::utils::error::Result;
use std::sync::Arc;
use tauri::State;

#[tauri::command]
pub async fn start_recording(
    name: String,
    description: String,
    recorder: State<'_, Arc<Recorder>>,
) -> Result<Recording> {
    recorder.start_recording(name, description).await
}

#[tauri::command]
pub async fn stop_recording(recorder: State<'_, Arc<Recorder>>) -> Result<Recording> {
    recorder.stop_recording().await
}

#[tauri::command]
pub async fn pause_recording(recorder: State<'_, Arc<Recorder>>) -> Result<()> {
    recorder.pause_recording().await
}

#[tauri::command]
pub async fn resume_recording(recorder: State<'_, Arc<Recorder>>) -> Result<()> {
    recorder.resume_recording().await
}

#[tauri::command]
pub async fn add_event(event: RecordedEvent, recorder: State<'_, Arc<Recorder>>) -> Result<()> {
    recorder.add_event(event).await
}

#[tauri::command]
pub async fn get_recording_state(recorder: State<'_, Arc<Recorder>>) -> RecordingState {
    recorder.get_state().await
}

#[tauri::command]
pub async fn get_current_recording(recorder: State<'_, Arc<Recorder>>) -> Option<Recording> {
    recorder.get_current_recording().await
}

#[tauri::command]
pub async fn list_recordings(recorder: State<'_, Arc<Recorder>>) -> Result<Vec<Recording>> {
    recorder.list_recordings()
}

#[tauri::command]
pub async fn get_recording(
    id: String,
    recorder: State<'_, Arc<Recorder>>,
) -> Result<Option<Recording>> {
    recorder.get_recording(&id)
}

#[tauri::command]
pub async fn delete_recording(id: String, recorder: State<'_, Arc<Recorder>>) -> Result<()> {
    recorder.delete_recording(&id)
}

#[tauri::command]
pub async fn playback_recording(
    id: String,
    recorder: State<'_, Arc<Recorder>>,
) -> Result<PlaybackResult> {
    recorder.playback_recording(&id).await
}
