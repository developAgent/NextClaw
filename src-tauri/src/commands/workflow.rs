use crate::workflow::{Workflow, WorkflowNode, WorkflowEdge, WorkflowVariable, WorkflowNodeType};
use crate::utils::error::{AppError, Result};
use tauri::State;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 工作流管理器
pub struct WorkflowManager {
    workflows: Arc<RwLock<Vec<Workflow>>>,
}

impl WorkflowManager {
    pub fn new() -> Self {
        Self {
            workflows: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 创建新工作流
    pub async fn create_workflow(&self, workflow: Workflow) -> Result<()> {
        let mut workflows = self.workflows.write().await;
        workflows.push(workflow);
        Ok(())
    }

    /// 获取所有工作流
    pub async fn get_all_workflows(&self) -> Result<Vec<Workflow>> {
        let workflows = self.workflows.read().await;
        Ok(workflows.clone())
    }

    /// 获取工作流
    pub async fn get_workflow(&self, id: &str) -> Result<Option<Workflow>> {
        let workflows = self.workflows.read().await;
        Ok(workflows.iter().find(|w| w.id == id).cloned())
    }

    /// 更新工作流
    pub async fn update_workflow(&self, workflow: Workflow) -> Result<()> {
        let mut workflows = self.workflows.write().await;
        if let Some(w) = workflows.iter_mut().find(|w| w.id == workflow.id) {
            *w = workflow;
            Ok(())
        } else {
            Err(AppError::Validation(format!("Workflow not found: {}", workflow.id)))
        }
    }

    /// 删除工作流
    pub async fn delete_workflow(&self, id: &str) -> Result<()> {
        let mut workflows = self.workflows.write().await;
        workflows.retain(|w| w.id != id);
        Ok(())
    }
}

impl Default for WorkflowManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建工作流
#[tauri::command]
pub async fn create_workflow(
    workflow: Workflow,
    manager: State<'_, Arc<WorkflowManager>>,
) -> Result<()> {
    manager.create_workflow(workflow).await
}

/// 获取所有工作流
#[tauri::command]
pub async fn get_all_workflows(
    manager: State<'_, Arc<WorkflowManager>>,
) -> Result<Vec<Workflow>> {
    manager.get_all_workflows().await
}

/// 获取工作流
#[tauri::command]
pub async fn get_workflow(
    id: String,
    manager: State<'_, Arc<WorkflowManager>>,
) -> Result<Option<Workflow>> {
    manager.get_workflow(&id).await
}

/// 更新工作流
#[tauri::command]
pub async fn update_workflow(
    workflow: Workflow,
    manager: State<'_, Arc<WorkflowManager>>,
) -> Result<()> {
    manager.update_workflow(workflow).await
}

/// 删除工作流
#[tauri::command]
pub async fn delete_workflow(
    id: String,
    manager: State<'_, Arc<WorkflowManager>>,
) -> Result<()> {
    manager.delete_workflow(&id).await
}