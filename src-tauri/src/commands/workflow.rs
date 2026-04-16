use crate::recorder::Recorder;
use crate::skills::host::WasmHost;
use crate::utils::error::Result;
use crate::workflow::{Workflow, WorkflowExecutionResult, WorkflowManager};
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn create_workflow(
    workflow: Workflow,
    manager: State<'_, Arc<WorkflowManager>>,
) -> Result<Workflow> {
    manager.create_workflow(workflow)
}

#[tauri::command]
pub fn get_all_workflows(manager: State<'_, Arc<WorkflowManager>>) -> Result<Vec<Workflow>> {
    manager.get_all_workflows()
}

#[tauri::command]
pub fn get_workflow(
    id: String,
    manager: State<'_, Arc<WorkflowManager>>,
) -> Result<Option<Workflow>> {
    manager.get_workflow(&id)
}

#[tauri::command]
pub fn update_workflow(
    workflow: Workflow,
    manager: State<'_, Arc<WorkflowManager>>,
) -> Result<Workflow> {
    manager.update_workflow(workflow)
}

#[tauri::command]
pub async fn execute_workflow(
    id: String,
    app: AppHandle,
    manager: State<'_, Arc<WorkflowManager>>,
    recorder: State<'_, Arc<Recorder>>,
    host: State<'_, Arc<WasmHost>>,
) -> Result<WorkflowExecutionResult> {
    manager
        .execute_workflow(&id, &app, recorder.inner().as_ref(), host.inner().as_ref())
        .await
}

#[tauri::command]
pub fn delete_workflow(id: String, manager: State<'_, Arc<WorkflowManager>>) -> Result<()> {
    manager.delete_workflow(&id)
}
