use crate::db::Database;
use crate::recorder::{PlaybackResult, Recorder};
use crate::skills::host::WasmHost;
use crate::skills::runtime::WasmArgument;
use crate::utils::error::{AppError, Result};
use evalexpr::{
    eval_boolean_with_context, ContextWithMutableVariables, HashMapContext, Value as EvalValue,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::time::{sleep, Duration};
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    pub label: String,
    pub node_type: WorkflowNodeType,
    pub position_x: f64,
    pub position_y: f64,
    pub config: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowVariable {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowNodeType {
    Trigger,
    Action,
    Condition,
    Delay,
    Agent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
    pub variables: Vec<WorkflowVariable>,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Workflow {
    pub fn new(name: String, description: String) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            nodes: Vec::new(),
            edges: Vec::new(),
            variables: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNodeExecution {
    pub node_id: String,
    pub label: String,
    pub node_type: WorkflowNodeType,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionResult {
    pub workflow_id: String,
    pub workflow_name: String,
    pub status: String,
    pub executed_nodes: usize,
    pub skipped_nodes: usize,
    pub failed_nodes: usize,
    pub finished_at: i64,
    pub node_results: Vec<WorkflowNodeExecution>,
}

#[derive(Debug)]
pub struct WorkflowManager {
    db: Arc<Database>,
}

struct NodeExecutionOutcome {
    status: String,
    message: String,
    should_continue: bool,
}

impl WorkflowManager {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub fn create_workflow(&self, mut workflow: Workflow) -> Result<Workflow> {
        workflow.name = workflow.name.trim().to_string();
        workflow.description = workflow.description.trim().to_string();
        workflow.updated_at = chrono::Utc::now().timestamp();

        if workflow.id.trim().is_empty() {
            workflow.id = Uuid::new_v4().to_string();
        }

        if workflow.name.is_empty() {
            return Err(AppError::Validation(
                "Workflow name is required".to_string(),
            ));
        }

        if workflow.created_at == 0 {
            workflow.created_at = workflow.updated_at;
        }

        self.persist_workflow(&workflow)?;
        info!("Created workflow: {}", workflow.name);
        Ok(workflow)
    }

    pub fn get_all_workflows(&self) -> Result<Vec<Workflow>> {
        let conn = self.db.conn();
        let conn_guard = conn.blocking_lock();
        let mut stmt = conn_guard
            .prepare(
                "SELECT id, name, description, nodes_json, edges_json, variables_json, created_at, updated_at FROM workflows ORDER BY updated_at DESC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        stmt.query_map([], |row| {
            Ok(Workflow {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                nodes: serde_json::from_str::<Vec<WorkflowNode>>(&row.get::<_, String>(3)?)
                    .unwrap_or_default(),
                edges: serde_json::from_str::<Vec<WorkflowEdge>>(&row.get::<_, String>(4)?)
                    .unwrap_or_default(),
                variables: serde_json::from_str::<Vec<WorkflowVariable>>(&row.get::<_, String>(5)?)
                    .unwrap_or_default(),
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| AppError::Database(e.to_string()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| AppError::Database(e.to_string()))
    }

    pub fn get_workflow(&self, id: &str) -> Result<Option<Workflow>> {
        let conn = self.db.conn();
        let conn_guard = conn.blocking_lock();
        let mut stmt = conn_guard
            .prepare(
                "SELECT id, name, description, nodes_json, edges_json, variables_json, created_at, updated_at FROM workflows WHERE id = ?1",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let result = stmt.query_row([id], |row| {
            Ok(Workflow {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                nodes: serde_json::from_str::<Vec<WorkflowNode>>(&row.get::<_, String>(3)?)
                    .unwrap_or_default(),
                edges: serde_json::from_str::<Vec<WorkflowEdge>>(&row.get::<_, String>(4)?)
                    .unwrap_or_default(),
                variables: serde_json::from_str::<Vec<WorkflowVariable>>(&row.get::<_, String>(5)?)
                    .unwrap_or_default(),
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        });

        match result {
            Ok(workflow) => Ok(Some(workflow)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(AppError::Database(error.to_string())),
        }
    }

    pub fn update_workflow(&self, mut workflow: Workflow) -> Result<Workflow> {
        workflow.name = workflow.name.trim().to_string();
        workflow.description = workflow.description.trim().to_string();

        if workflow.name.is_empty() {
            return Err(AppError::Validation(
                "Workflow name is required".to_string(),
            ));
        }

        let existing = self
            .get_workflow(&workflow.id)?
            .ok_or_else(|| AppError::Validation(format!("Workflow not found: {}", workflow.id)))?;

        workflow.created_at = existing.created_at;
        workflow.updated_at = chrono::Utc::now().timestamp();
        self.persist_workflow(&workflow)?;
        info!("Updated workflow: {}", workflow.name);
        Ok(workflow)
    }

    pub async fn execute_workflow(
        &self,
        id: &str,
        app_handle: &AppHandle,
        recorder: &Recorder,
        wasm_host: &WasmHost,
    ) -> Result<WorkflowExecutionResult> {
        let workflow = self
            .get_workflow(id)?
            .ok_or_else(|| AppError::Validation(format!("Workflow not found: {id}")))?;

        if workflow.nodes.is_empty() {
            return Err(AppError::Validation(format!(
                "Workflow '{}' has no nodes",
                workflow.name
            )));
        }

        let trigger_ids = workflow
            .nodes
            .iter()
            .filter(|node| matches!(node.node_type, WorkflowNodeType::Trigger))
            .map(|node| node.id.clone())
            .collect::<Vec<_>>();

        if trigger_ids.is_empty() {
            return Err(AppError::Validation(format!(
                "Workflow '{}' has no trigger node",
                workflow.name
            )));
        }

        let node_map = workflow
            .nodes
            .iter()
            .cloned()
            .map(|node| (node.id.clone(), node))
            .collect::<HashMap<_, _>>();

        let mut outgoing = HashMap::<String, Vec<String>>::new();
        for edge in &workflow.edges {
            outgoing
                .entry(edge.source.clone())
                .or_default()
                .push(edge.target.clone());
        }

        let mut queue = VecDeque::from(trigger_ids);
        let mut visited = HashSet::<String>::new();
        let mut node_results = Vec::<WorkflowNodeExecution>::new();
        let mut overall_status = "completed".to_string();

        while let Some(node_id) = queue.pop_front() {
            if !visited.insert(node_id.clone()) {
                continue;
            }

            let Some(node) = node_map.get(&node_id).cloned() else {
                node_results.push(WorkflowNodeExecution {
                    node_id,
                    label: "Unknown node".to_string(),
                    node_type: WorkflowNodeType::Action,
                    status: "failed".to_string(),
                    message: "Workflow edge points to a missing node".to_string(),
                });
                overall_status = "failed".to_string();
                break;
            };

            match execute_node(&workflow, &node, app_handle, recorder, wasm_host).await {
                Ok(outcome) => {
                    let should_continue = outcome.should_continue;
                    node_results.push(WorkflowNodeExecution {
                        node_id: node.id.clone(),
                        label: node.label.clone(),
                        node_type: node.node_type.clone(),
                        status: outcome.status,
                        message: outcome.message,
                    });

                    if should_continue {
                        if let Some(next_nodes) = outgoing.get(&node.id) {
                            for next_node_id in next_nodes {
                                queue.push_back(next_node_id.clone());
                            }
                        }
                    }
                }
                Err(error) => {
                    node_results.push(WorkflowNodeExecution {
                        node_id: node.id.clone(),
                        label: node.label.clone(),
                        node_type: node.node_type.clone(),
                        status: "failed".to_string(),
                        message: error.to_string(),
                    });
                    overall_status = "failed".to_string();
                    break;
                }
            }
        }

        let executed_nodes = node_results
            .iter()
            .filter(|result| result.status == "executed")
            .count();
        let skipped_nodes = node_results
            .iter()
            .filter(|result| result.status == "skipped")
            .count();
        let failed_nodes = node_results
            .iter()
            .filter(|result| result.status == "failed")
            .count();

        if failed_nodes > 0 {
            overall_status = "failed".to_string();
        }

        let result = WorkflowExecutionResult {
            workflow_id: workflow.id.clone(),
            workflow_name: workflow.name.clone(),
            status: overall_status,
            executed_nodes,
            skipped_nodes,
            failed_nodes,
            finished_at: chrono::Utc::now().timestamp(),
            node_results,
        };

        let _ = app_handle.emit("workflow-executed", &result);
        info!("Executed workflow: {}", workflow.name);
        Ok(result)
    }

    pub fn delete_workflow(&self, id: &str) -> Result<()> {
        let conn = self.db.conn();
        let conn_guard = conn.blocking_lock();
        conn_guard
            .execute("DELETE FROM workflows WHERE id = ?1", [id])
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    fn persist_workflow(&self, workflow: &Workflow) -> Result<()> {
        let conn = self.db.conn();
        let conn_guard = conn.blocking_lock();
        let nodes_json = serde_json::to_string(&workflow.nodes)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let edges_json = serde_json::to_string(&workflow.edges)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let variables_json = serde_json::to_string(&workflow.variables)
            .map_err(|e| AppError::Internal(e.to_string()))?;

        conn_guard
            .execute(
                "INSERT OR REPLACE INTO workflows (id, name, description, nodes_json, edges_json, variables_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    workflow.id,
                    workflow.name,
                    workflow.description,
                    nodes_json,
                    edges_json,
                    variables_json,
                    workflow.created_at,
                    workflow.updated_at,
                ],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }
}

async fn execute_node(
    workflow: &Workflow,
    node: &WorkflowNode,
    app_handle: &AppHandle,
    recorder: &Recorder,
    wasm_host: &WasmHost,
) -> Result<NodeExecutionOutcome> {
    match node.node_type {
        WorkflowNodeType::Trigger => execute_trigger_node(node),
        WorkflowNodeType::Delay => execute_delay_node(node).await,
        WorkflowNodeType::Condition => execute_condition_node(workflow, node),
        WorkflowNodeType::Action => {
            execute_action_node(node, app_handle, recorder, wasm_host).await
        }
        WorkflowNodeType::Agent => Err(AppError::Validation(format!(
            "Workflow node '{}' uses unsupported node type 'agent'",
            node.label
        ))),
    }
}

fn execute_trigger_node(node: &WorkflowNode) -> Result<NodeExecutionOutcome> {
    let config = parse_node_config(node)?;
    let event_name = config
        .get("event")
        .and_then(Value::as_str)
        .unwrap_or("manual");

    Ok(NodeExecutionOutcome {
        status: "executed".to_string(),
        message: format!("Entered trigger '{}' via manual run", event_name),
        should_continue: true,
    })
}

async fn execute_delay_node(node: &WorkflowNode) -> Result<NodeExecutionOutcome> {
    let config = parse_node_config(node)?;
    let duration_ms = parse_duration_ms(&config)?;
    sleep(Duration::from_millis(duration_ms)).await;

    Ok(NodeExecutionOutcome {
        status: "executed".to_string(),
        message: format!("Waited {duration_ms} ms"),
        should_continue: true,
    })
}

fn execute_condition_node(
    workflow: &Workflow,
    node: &WorkflowNode,
) -> Result<NodeExecutionOutcome> {
    let config = parse_node_config(node)?;
    let expression = if let Some(expression) = config.as_str() {
        expression.trim().to_string()
    } else {
        config
            .get("expression")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .ok_or_else(|| {
                AppError::Validation(format!(
                    "Condition node '{}' requires an expression",
                    node.label
                ))
            })?
    };

    let mut context = HashMapContext::new();
    for variable in &workflow.variables {
        if variable.key.trim().is_empty() {
            continue;
        }

        context
            .set_value(variable.key.clone(), parse_variable_value(&variable.value))
            .map_err(|error| AppError::Validation(error.to_string()))?;
    }

    let passes = eval_boolean_with_context(&expression, &context)
        .map_err(|error| AppError::Validation(format!("Condition evaluation failed: {error}")))?;

    Ok(NodeExecutionOutcome {
        status: if passes {
            "executed".to_string()
        } else {
            "skipped".to_string()
        },
        message: if passes {
            format!("Condition matched: {expression}")
        } else {
            format!("Condition returned false: {expression}")
        },
        should_continue: passes,
    })
}

async fn execute_action_node(
    node: &WorkflowNode,
    app_handle: &AppHandle,
    recorder: &Recorder,
    wasm_host: &WasmHost,
) -> Result<NodeExecutionOutcome> {
    let config = parse_node_config(node)?;
    let action_type = config
        .get("type")
        .and_then(Value::as_str)
        .map(|value| value.trim().to_ascii_lowercase())
        .ok_or_else(|| {
            AppError::Validation(format!("Action node '{}' requires a type", node.label))
        })?;

    match action_type.as_str() {
        "emit_event" | "emit" => {
            let event_name = config
                .get("event")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    AppError::Validation(format!(
                        "Action node '{}' requires an event name",
                        node.label
                    ))
                })?;
            let payload = config.get("payload").cloned().unwrap_or(Value::Null);
            app_handle
                .emit(event_name, payload)
                .map_err(|error| AppError::Execution(error.to_string()))?;

            Ok(NodeExecutionOutcome {
                status: "executed".to_string(),
                message: format!("Emitted event '{event_name}'"),
                should_continue: true,
            })
        }
        "playback_recording" | "recorder_playback" => {
            let recording_id = config
                .get("recording_id")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    AppError::Validation(format!(
                        "Action node '{}' requires recording_id",
                        node.label
                    ))
                })?;
            let playback_result = recorder.playback_recording(recording_id).await?;

            Ok(NodeExecutionOutcome {
                status: "executed".to_string(),
                message: summarize_playback_result(&playback_result),
                should_continue: true,
            })
        }
        "execute_skill" | "skill" => {
            let skill_id = config
                .get("skill_id")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    AppError::Validation(format!(
                        "Action node '{}' requires skill_id",
                        node.label
                    ))
                })?;
            let function = config
                .get("function")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    AppError::Validation(format!(
                        "Action node '{}' requires function",
                        node.label
                    ))
                })?;
            let args = config
                .get("args")
                .and_then(Value::as_array)
                .map(|values| values.iter().map(json_to_wasm_argument).collect::<Vec<_>>())
                .unwrap_or_default();

            let execution_result = wasm_host.execute_skill(skill_id, function, args).await?;
            let stderr = execution_result.stderr.trim();
            let message = if stderr.is_empty() {
                format!(
                    "Executed skill '{}::{}' (status {}, {} ms)",
                    skill_id, function, execution_result.status, execution_result.execution_time_ms
                )
            } else {
                format!(
                    "Executed skill '{}::{}' (status {}, {} ms, stderr: {})",
                    skill_id,
                    function,
                    execution_result.status,
                    execution_result.execution_time_ms,
                    stderr
                )
            };

            Ok(NodeExecutionOutcome {
                status: "executed".to_string(),
                message,
                should_continue: true,
            })
        }
        other => Err(AppError::Validation(format!(
            "Unsupported workflow action type: {other}. Supported types: emit_event, playback_recording, execute_skill"
        ))),
    }
}

fn parse_node_config(node: &WorkflowNode) -> Result<Value> {
    let Some(raw) = node
        .config
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(Value::Object(Default::default()));
    };

    serde_json::from_str::<Value>(raw).or_else(|_| Ok(Value::String(raw.to_string())))
}

fn parse_duration_ms(config: &Value) -> Result<u64> {
    config
        .as_u64()
        .or_else(|| config.get("duration_ms").and_then(Value::as_u64))
        .or_else(|| config.get("ms").and_then(Value::as_u64))
        .ok_or_else(|| {
            AppError::Validation(
                "Delay node requires a numeric duration_ms or ms value".to_string(),
            )
        })
}

fn parse_variable_value(value: &str) -> EvalValue {
    match serde_json::from_str::<Value>(value) {
        Ok(Value::Bool(boolean)) => EvalValue::Boolean(boolean),
        Ok(Value::Number(number)) => {
            if let Some(int) = number.as_i64() {
                EvalValue::Int(int)
            } else if let Some(float) = number.as_f64() {
                EvalValue::Float(float)
            } else {
                EvalValue::String(value.to_string())
            }
        }
        Ok(Value::String(string)) => EvalValue::String(string),
        Ok(Value::Null) => EvalValue::Empty,
        Ok(other) => EvalValue::String(other.to_string()),
        Err(_) => EvalValue::String(value.to_string()),
    }
}

fn summarize_playback_result(result: &PlaybackResult) -> String {
    if result.warnings.is_empty() {
        format!(
            "Replayed recording with {} executed, {} skipped, {} failed events",
            result.replayed_events, result.skipped_events, result.failed_events
        )
    } else {
        warn!(
            "Workflow recorder playback emitted warnings for {}: {:?}",
            result.recording_id, result.warnings
        );
        format!(
            "Replayed recording with {} executed, {} skipped, {} failed events ({})",
            result.replayed_events,
            result.skipped_events,
            result.failed_events,
            result.warnings.join("; ")
        )
    }
}

fn json_to_wasm_argument(value: &Value) -> WasmArgument {
    match value {
        Value::Null => WasmArgument::Null,
        Value::Bool(boolean) => WasmArgument::Boolean(*boolean),
        Value::Number(number) => WasmArgument::Number(number.as_f64().unwrap_or_default()),
        Value::String(string) => WasmArgument::String(string.clone()),
        Value::Array(values) => {
            WasmArgument::Array(values.iter().map(json_to_wasm_argument).collect())
        }
        Value::Object(map) => WasmArgument::Object(
            map.iter()
                .map(|(key, value)| (key.clone(), json_to_wasm_argument(value)))
                .collect(),
        ),
    }
}
