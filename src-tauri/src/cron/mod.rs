//! Cron job scheduler
//! Handles scheduling and execution of automated AI tasks

use crate::ai::client::{ClaudeClient, Message, MessageRole};
use crate::db::models::Session;
use crate::utils::error::{AppError, Result};
use chrono::{DateTime, Utc};
use cron::Schedule;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::Duration;
use tracing::{debug, info};
use uuid::Uuid;

/// Cron job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub agent_id: String,
    pub channel_account_id: Option<String>,
    pub cron_expression: String,
    pub message: String,
    pub target_config: Option<String>,
    pub enabled: bool,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Create cron job request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCronJobRequest {
    pub name: String,
    pub description: Option<String>,
    pub agent_id: String,
    pub channel_account_id: Option<String>,
    pub cron_expression: String,
    pub message: String,
    pub target_config: Option<String>,
    pub enabled: Option<bool>,
}

/// Update cron job request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCronJobRequest {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub agent_id: Option<String>,
    pub channel_account_id: Option<String>,
    pub cron_expression: Option<String>,
    pub message: Option<String>,
    pub target_config: Option<String>,
    pub enabled: Option<bool>,
}

/// Cron execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronExecution {
    pub id: String,
    pub job_id: String,
    pub status: CronExecutionStatus,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// Cron execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CronExecutionStatus {
    Pending,
    Running,
    Success,
    Failed,
}

impl CronJob {
    pub fn new(name: impl Into<String>, agent_id: impl Into<String>, cron_expr: impl Into<String>, message: impl Into<String>) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            description: None,
            agent_id: agent_id.into(),
            channel_account_id: None,
            cron_expression: cron_expr.into(),
            message: message.into(),
            target_config: None,
            enabled: true,
            last_run: None,
            next_run: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn with_channel_account(mut self, account_id: impl Into<String>) -> Self {
        self.channel_account_id = Some(account_id.into());
        self
    }

    pub fn with_target_config(mut self, config: impl Into<String>) -> Self {
        self.target_config = Some(config.into());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

impl CronExecution {
    pub fn new(job_id: impl Into<String>) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            job_id: job_id.into(),
            status: CronExecutionStatus::Pending,
            started_at: now.clone(),
            completed_at: None,
            output: None,
            error: None,
        }
    }

    pub fn with_status(mut self, status: CronExecutionStatus) -> Self {
        self.status = status;
        self
    }

    pub fn completed(mut self, output: Option<impl Into<String>>) -> Self {
        self.status = CronExecutionStatus::Success;
        self.completed_at = Some(Utc::now().to_rfc3339());
        self.output = output.map(|o| o.into());
        self
    }

    pub fn failed(mut self, error: impl Into<String>) -> Self {
        self.status = CronExecutionStatus::Failed;
        self.completed_at = Some(Utc::now().to_rfc3339());
        self.error = Some(error.into());
        self
    }

    pub fn started(mut self) -> Self {
        self.status = CronExecutionStatus::Running;
        self.started_at = Utc::now().to_rfc3339();
        self
    }
}

/// Cron scheduler
pub struct CronScheduler {
    pub db: Arc<Mutex<Connection>>,
    jobs: Arc<RwLock<HashMap<String, CronJob>>>,
    schedules: Arc<RwLock<HashMap<String, Schedule>>>,
    running: Arc<Mutex<bool>>,
}

impl CronScheduler {
    pub fn new(db: Arc<Mutex<Connection>>) -> Self {
        Self {
            db,
            jobs: Arc::new(RwLock::new(HashMap::new())),
            schedules: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// Load jobs from database
    pub async fn load_jobs(&self) -> Result<()> {
        let db = self.db.lock().await;
        let mut stmt = db.prepare(
            "SELECT id, name, description, agent_id, channel_account_id, cron_expression, message, target_config, enabled, last_run, next_run, created_at, updated_at
             FROM cron_jobs ORDER BY created_at"
        ).map_err(|e| AppError::Database(format!("Failed to load jobs: {}", e)))?;

        let jobs = stmt
            .query_map([], |row| {
                #[allow(clippy::too_many_arguments)]
                Ok(CronJob {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    agent_id: row.get(3)?,
                    channel_account_id: row.get(4)?,
                    cron_expression: row.get(5)?,
                    message: row.get(6)?,
                    target_config: row.get(7)?,
                    enabled: row.get::<_, i32>(8)? != 0,
                    last_run: row.get(9)?,
                    next_run: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            })
            .map_err(|e| AppError::Database(format!("Failed to load jobs: {}", e)))?
            .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()
            .map_err(|e| AppError::Database(format!("Failed to collect jobs: {}", e)))?;

        let mut jobs_map = self.jobs.write().await;
        jobs_map.clear();
        for job in jobs {
            jobs_map.insert(job.id.clone(), job);
        }

        info!("Loaded {} cron jobs", jobs_map.len());
        Ok(())
    }

    /// Create a new cron job
    pub async fn create_job(&self, request: CreateCronJobRequest) -> Result<CronJob> {
        let mut job = CronJob::new(&request.name, &request.agent_id, &request.cron_expression, &request.message);
        job.description = normalize_optional_string(request.description);
        job.channel_account_id = normalize_optional_string(request.channel_account_id);
        job.target_config = normalize_optional_string(request.target_config);
        job.enabled = request.enabled.unwrap_or(true);

        // Parse and validate cron expression
        let schedule = Schedule::from_str(&job.cron_expression)
            .map_err(|e| AppError::Validation(format!("Invalid cron expression: {}", e)))?;

        // Calculate next run time
        let now = Utc::now();
        let next_run = schedule.upcoming(Utc).next();
        let next_run_str = next_run
            .map(|dt: DateTime<chrono::Utc>| dt.to_rfc3339())
            .unwrap_or_else(|| Utc::now().to_rfc3339());

        let db = self.db.lock().await;

        let next_run_sql = next_run_str.as_str();

        db.execute(
            r#"
            INSERT INTO cron_jobs (id, name, description, agent_id, channel_account_id, cron_expression, message, target_config, enabled, last_run, next_run, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
            params![
                &job.id,
                &job.name,
                &job.description,
                &job.agent_id,
                &job.channel_account_id,
                &job.cron_expression,
                &job.message,
                &job.target_config,
                job.enabled as i32,
                &job.last_run,
                &next_run_sql,
                &job.created_at,
                &job.updated_at,
            ],
        ).map_err(|e| AppError::Database(format!("Failed to create job: {}", e)))?;

        // Update job with next run time
        let job_with_next = CronJob {
            next_run: Some(next_run_str),
            ..job.clone()
        };

        // Update in-memory cache
        let mut jobs = self.jobs.write().await;
        jobs.insert(job_with_next.id.clone(), job_with_next.clone());

        // Update schedule cache
        let mut schedules = self.schedules.write().await;
        schedules.insert(job_with_next.id.clone(), schedule);

        info!("Created cron job: {}", job_with_next.name);
        Ok(job_with_next)
    }

    /// Get all jobs
    pub async fn get_all_jobs(&self) -> Result<Vec<CronJob>> {
        let jobs = self.jobs.read().await;
        Ok(jobs.values().cloned().collect())
    }

    /// Get a specific job
    pub async fn get_job(&self, id: &str) -> Result<Option<CronJob>> {
        let jobs = self.jobs.read().await;
        Ok(jobs.get(id).cloned())
    }

    /// Update a job
    pub async fn update_job(&self, request: UpdateCronJobRequest) -> Result<CronJob> {
        let mut job = self.get_job(&request.id).await?
            .ok_or_else(|| AppError::Validation(format!("Job not found: {}", request.id)))?;

        // Update fields
        if let Some(name) = request.name {
            job.name = name;
        }
        if let Some(desc) = request.description {
            job.description = normalize_optional_string(Some(desc));
        }
        if let Some(agent_id) = request.agent_id {
            job.agent_id = agent_id;
        }
        if let Some(account_id) = request.channel_account_id {
            job.channel_account_id = normalize_optional_string(Some(account_id));
        }
        if let Some(cron_expr) = request.cron_expression {
            job.cron_expression = cron_expr.clone();

            // Re-parse schedule
            let schedule = Schedule::from_str(&job.cron_expression)
                .map_err(|e| AppError::Validation(format!("Invalid cron expression: {}", e)))?;

            // Update next run time
            let now = Utc::now();
            let next_run = schedule.upcoming(Utc).next();
            let next_run_str = next_run
                .map(|dt: DateTime<chrono::Utc>| dt.to_rfc3339())
                .unwrap_or_else(|| Utc::now().to_rfc3339());
            job.next_run = Some(next_run_str);

            // Update schedule cache
            let mut schedules = self.schedules.write().await;
            schedules.insert(job.id.clone(), schedule);
        }
        if let Some(message) = request.message {
            job.message = message;
        }
        if let Some(config) = request.target_config {
            job.target_config = normalize_optional_string(Some(config));
        }
        if let Some(enabled) = request.enabled {
            job.enabled = enabled;
        }

        job.updated_at = Utc::now().to_rfc3339();

        // Save to database
        let db = self.db.lock().await;
        db.execute(
            r#"
            UPDATE cron_jobs
            SET name = ?1, description = ?2, agent_id = ?3, channel_account_id = ?4,
                cron_expression = ?5, message = ?6, target_config = ?7, enabled = ?8,
                last_run = ?9, next_run = ?10, updated_at = ?11
            WHERE id = ?12
            "#,
            params![
                &job.name,
                &job.description,
                &job.agent_id,
                &job.channel_account_id,
                &job.cron_expression,
                &job.message,
                &job.target_config,
                job.enabled as i32,
                &job.last_run,
                &job.next_run,
                &job.updated_at,
                &job.id,
            ],
        ).map_err(|e| AppError::Database(format!("Failed to update job: {}", e)))?;

        // Update in-memory cache
        let mut jobs = self.jobs.write().await;
        jobs.insert(job.id.clone(), job.clone());

        info!("Updated cron job: {}", job.name);
        Ok(job)
    }

    /// Delete a job
    pub async fn delete_job(&self, id: &str) -> Result<()> {
        let db = self.db.lock().await;

        let count = db.execute("DELETE FROM cron_jobs WHERE id = ?1", params![id])
            .map_err(|e| AppError::Database(format!("Failed to delete job: {}", e)))?;

        if count == 0 {
            return Err(AppError::Validation(format!("Job not found: {}", id)));
        }

        // Remove from cache
        let mut jobs = self.jobs.write().await;
        jobs.remove(id);

        let mut schedules = self.schedules.write().await;
        schedules.remove(id);

        info!("Deleted cron job: {}", id);
        Ok(())
    }

    /// Start the scheduler
    pub async fn start(&self) {
        let running = self.running.clone();
        if *running.lock().await {
            return;
        }

        *running.lock().await = true;
        info!("Cron scheduler started");

        let jobs_cache = self.jobs.clone();
        let schedules_cache = self.schedules.clone();
        let db_cache = self.db.clone();

        tokio::spawn(async move {
            loop {
                if !*running.lock().await {
                    break;
                }

                // Check for jobs that need to run
                let now = Utc::now();

                let jobs = jobs_cache.read().await;
                let jobs_to_run: Vec<(String, CronJob)> = jobs
                    .iter()
                    .filter(|(_, job)| {
                        job.enabled
                        && if let Some(next_run) = &job.next_run {
                            // Parse next_run and check if it's time
                            if let Ok(dt) = DateTime::parse_from_rfc3339(next_run) {
                                dt <= now
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    })
                    .map(|(id, job)| (id.clone(), job.clone()))
                    .collect();
                drop(jobs);

                if !jobs_to_run.is_empty() {
                    debug!("Found {} jobs to run", jobs_to_run.len());

                    for (job_id, job) in jobs_to_run {
                        // Execute job in a separate task
                        let db = db_cache.clone();
                        let jobs_inner = jobs_cache.clone();
                        let schedules = schedules_cache.clone();
                        let scheduler = Self {
                            db: db_cache.clone(),
                            jobs: jobs_cache.clone(),
                            schedules: schedules_cache.clone(),
                            running: running.clone(),
                        };

                        tokio::spawn(async move {
                            // Update job state
                            {
                                let mut jobs = jobs_inner.write().await;
                                if let Some(job) = jobs.get_mut(&job_id) {
                                    job.last_run = Some(Utc::now().to_rfc3339());

                                    // Calculate next run time
                                    if let Some(schedule) = {
                                        let schedules_read = schedules.read().await;
                                        schedules_read.get(&job_id).cloned()
                                    } {
                                        let next_run = schedule.upcoming(Utc).next();
                                        let next_run_str = next_run
                                            .map(|dt: DateTime<chrono::Utc>| dt.to_rfc3339())
                                            .unwrap_or_else(|| Utc::now().to_rfc3339());
                                        job.next_run = Some(next_run_str);
                                    }
                                }
                            }

                            // Create execution record
                            let execution = CronExecution::new(&job_id).started();

                            // Save execution start
                            db.lock().await.execute(
                                r#"
                                INSERT INTO cron_executions (id, job_id, status, started_at, completed_at, output, error)
                                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                                "#,
                                params![
                                    &execution.id,
                                    &execution.job_id,
                                    "running",
                                    &execution.started_at,
                                    None::<&str>,
                                    None::<&str>,
                                    None::<&str>,
                                ],
                            ).ok();

                            // Execute the job
                            let result = scheduler.execute_job(&job).await;

                            // Update execution record
                            let (status, output, error) = match result {
                                Ok(output) => ("success", Some(output), None::<String>),
                                Err(e) => ("failed", None::<String>, Some(e.to_string())),
                            };

                            db.lock().await.execute(
                                r#"
                                UPDATE cron_executions
                                SET status = ?1, completed_at = ?2, output = ?3, error = ?4
                                WHERE id = ?5
                                "#,
                                params![status, &Utc::now().to_rfc3339(), output, error, &execution.id],
                            ).ok();

                            info!("Cron job {} completed with status: {}", job_id, status);
                        });
                    }
                }

                // Sleep for a minute before next check
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        });
    }

    /// Stop the scheduler
    pub async fn stop(&self) {
        *self.running.lock().await = false;
        info!("Cron scheduler stopped");
    }

    /// Execute a cron job
    pub async fn execute_job(&self, job: &CronJob) -> Result<String> {
        debug!("Executing cron job: {}", job.name);

        let agent = self.get_agent(&job.agent_id).await?
            .ok_or_else(|| AppError::Validation(format!("Agent not found: {}", job.agent_id)))?;

        let session = self.create_cron_session(job, &agent).await?;
        let session_id = session.id;

        if let Some(system_prompt) = agent.system_prompt.as_deref().filter(|prompt| !prompt.trim().is_empty()) {
            self.store_message(
                session_id,
                &Message::new(session_id, MessageRole::System, system_prompt.to_string()),
            )
            .await?;
        }

        let user_message = Message::new(session_id, MessageRole::User, job.message.clone());
        self.store_message(session_id, &user_message).await?;

        let response = self.generate_response(job, &agent, session_id).await?;

        let assistant_message = Message::new(session_id, MessageRole::Assistant, response.clone());
        self.store_message(session_id, &assistant_message).await?;

        let delivered_response = self.deliver_response(job, &response).await?;
        self.update_job_after_execution(job, &delivered_response).await?;

        Ok(delivered_response)
    }

    async fn generate_response(&self, job: &CronJob, agent: &crate::agents::Agent, session_id: Uuid) -> Result<String> {
        match agent.provider_id.as_deref().map(str::trim).filter(|provider| !provider.is_empty()) {
            Some("anthropic") | Some("claude") | None => {
                let api_key = self.get_api_key()?;
                let secret_key = secrecy::SecretString::new(api_key);
                let client = ClaudeClient::new(secret_key)
                    .map(|client| {
                        if let Some(model_id) = agent.model_id.clone().filter(|model| !model.trim().is_empty()) {
                            client.with_model(model_id)
                        } else {
                            client
                        }
                    })?;

                client.send_message(&job.message, session_id).await
            }
            Some(provider) => Err(AppError::Validation(format!(
                "Cron jobs currently support only Anthropic/Claude agents, got provider: {}",
                provider
            ))),
        }
    }

    async fn deliver_response(&self, job: &CronJob, response: &str) -> Result<String> {
        if let Some(account_id) = job.channel_account_id.as_deref().filter(|value| !value.trim().is_empty()) {
            let account = self.get_channel_account(account_id).await?
                .ok_or_else(|| AppError::Validation(format!("Channel account not found: {}", account_id)))?;
            let target_config = self.parse_target_config(job.target_config.as_deref())?;
            let delivery_summary = self.dispatch_to_channel_account(job, &account, response, target_config.as_ref()).await?;
            return Ok(format!("{}\n\n{}", response, delivery_summary));
        }

        Ok(response.to_string())
    }

    async fn get_agent(&self, agent_id: &str) -> Result<Option<crate::agents::Agent>> {
        let db = self.db.lock().await;
        let mut stmt = db.prepare(
            "SELECT id, name, description, provider_id, model_id, system_prompt, temperature, max_tokens, created_at, updated_at FROM agents WHERE id = ?1"
        ).map_err(|e| AppError::Database(format!("Failed to query agent: {}", e)))?;

        let agent = stmt
            .query_row(params![agent_id], |row| {
                Ok(crate::agents::Agent {
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

    async fn get_channel_account(&self, account_id: &str) -> Result<Option<crate::channel_accounts::ChannelAccount>> {
        let db = self.db.lock().await;
        let mut stmt = db.prepare(
            "SELECT id, channel_id, name, credentials, is_default, created_at, updated_at FROM channel_accounts WHERE id = ?1"
        ).map_err(|e| AppError::Database(format!("Failed to query channel account: {}", e)))?;

        let account = stmt
            .query_row(params![account_id], |row| {
                Ok(crate::channel_accounts::ChannelAccount {
                    id: row.get(0)?,
                    channel_id: row.get(1)?,
                    name: row.get(2)?,
                    credentials: row.get(3)?,
                    is_default: row.get::<_, i32>(4)? != 0,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .ok();

        Ok(account)
    }

    async fn get_channel(&self, channel_id: &str) -> Result<Option<crate::channels::types::Channel>> {
        let db = self.db.lock().await;
        let mut stmt = db.prepare(
            "SELECT id, provider_type, name, config, enabled, priority, health_status, created_at, updated_at FROM channels WHERE id = ?1"
        ).map_err(|e| AppError::Database(format!("Failed to query channel: {}", e)))?;

        let channel = stmt
            .query_row(params![channel_id], |row| {
                let config_text: String = row.get(3)?;
                let config = serde_json::from_str(&config_text)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    ))?;

                let health_status = match row.get::<_, String>(6)?.as_str() {
                    "healthy" => crate::channels::types::ChannelHealth::Healthy,
                    "degraded" => crate::channels::types::ChannelHealth::Degraded,
                    "unhealthy" => crate::channels::types::ChannelHealth::Unhealthy,
                    _ => crate::channels::types::ChannelHealth::Unknown,
                };

                Ok(crate::channels::types::Channel {
                    id: row.get(0)?,
                    provider_type: row.get(1)?,
                    name: row.get(2)?,
                    config,
                    enabled: row.get::<_, i32>(4)? != 0,
                    priority: row.get(5)?,
                    health_status,
                    created_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })
            .ok();

        Ok(channel)
    }

    fn parse_target_config(&self, raw: Option<&str>) -> Result<Option<serde_json::Value>> {
        let Some(raw) = raw.map(str::trim).filter(|value| !value.is_empty()) else {
            return Ok(None);
        };

        let parsed = serde_json::from_str(raw)
            .map_err(|e| AppError::Validation(format!("Invalid target config JSON: {}", e)))?;
        Ok(Some(parsed))
    }

    async fn dispatch_to_channel_account(
        &self,
        job: &CronJob,
        account: &crate::channel_accounts::ChannelAccount,
        response: &str,
        target_config: Option<&serde_json::Value>,
    ) -> Result<String> {
        let credentials: serde_json::Value = serde_json::from_str(&account.credentials)
            .map_err(|e| AppError::Validation(format!("Invalid channel account credentials JSON: {}", e)))?;
        let channel = self.get_channel(&account.channel_id).await?
            .ok_or_else(|| AppError::Validation(format!("Channel not found for account: {}", account.channel_id)))?;

        let provider = target_config
            .and_then(|config| config.get("provider"))
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .or_else(|| {
                credentials
                    .get("provider")
                    .and_then(serde_json::Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
            })
            .unwrap_or_else(|| channel.provider_type.as_str());

        let mut payload = serde_json::json!({
            "job_id": job.id,
            "job_name": job.name,
            "agent_id": job.agent_id,
            "channel_id": channel.id,
            "channel_name": channel.name,
            "channel_provider": channel.provider_type,
            "channel_account_id": account.id,
            "channel_account_name": account.name,
            "message": job.message,
            "response": response,
            "executed_at": Utc::now().to_rfc3339(),
        });

        if let Some(config) = target_config {
            payload["target_config"] = config.clone();
        }

        match provider {
            "webhook" | "custom" => {
                let webhook_url = self.get_string_config(
                    target_config,
                    Some(&credentials),
                    Some(&channel.config),
                    &["webhook_url", "url", "endpoint", "base_url"],
                )
                .ok_or_else(|| AppError::Validation(format!(
                    "Channel account {} is missing a webhook endpoint for cron delivery",
                    account.id
                )))?;

                let content_type = self
                    .get_string_config(
                        target_config,
                        Some(&credentials),
                        Some(&channel.config),
                        &["content_type"],
                    )
                    .unwrap_or_else(|| "application/json".to_string());

                let method = self
                    .get_string_config(
                        target_config,
                        Some(&credentials),
                        Some(&channel.config),
                        &["method"],
                    )
                    .unwrap_or_else(|| "POST".to_string())
                    .to_uppercase();

                let mut headers = HeaderMap::new();
                headers.insert(
                    CONTENT_TYPE,
                    HeaderValue::from_str(&content_type).map_err(|e| {
                        AppError::Validation(format!("Invalid content type header for cron delivery: {}", e))
                    })?,
                );
                self.apply_auth_headers(&mut headers, target_config, Some(&credentials), Some(&channel.config))?;
                self.apply_custom_headers(&mut headers, target_config, Some(&credentials), Some(&channel.config))?;

                let client = reqwest::Client::new();
                let request = match method.as_str() {
                    "POST" => client.post(&webhook_url),
                    "PUT" => client.put(&webhook_url),
                    "PATCH" => client.patch(&webhook_url),
                    other => {
                        return Err(AppError::Validation(format!(
                            "Unsupported cron delivery HTTP method: {}",
                            other
                        )))
                    }
                };

                let result = request
                    .headers(headers)
                    .json(&payload)
                    .send()
                    .await
                    .map_err(|e| AppError::Execution(format!("Failed to send cron webhook delivery: {}", e)))?;

                if !result.status().is_success() {
                    let status = result.status();
                    let body = result.text().await.unwrap_or_else(|_| String::new());
                    return Err(AppError::Execution(format!(
                        "Cron webhook delivery failed: {} {}",
                        status,
                        body
                    )));
                }

                Ok(format!(
                    "Delivered to {} channel '{}' via {} {}",
                    provider,
                    account.name,
                    method,
                    webhook_url
                ))
            }
            "slack" => {
                let webhook_url = self.get_string_config(
                    target_config,
                    Some(&credentials),
                    Some(&channel.config),
                    &["webhook_url", "slack_webhook_url"],
                )
                .ok_or_else(|| AppError::Validation(format!(
                    "Slack channel account {} is missing webhook_url",
                    account.id
                )))?;

                let text = format!("*{}*\n{}", job.name, response);
                let slack_payload = serde_json::json!({
                    "text": text,
                    "metadata": payload,
                });

                let result = reqwest::Client::new()
                    .post(&webhook_url)
                    .json(&slack_payload)
                    .send()
                    .await
                    .map_err(|e| AppError::Execution(format!("Failed to send Slack cron delivery: {}", e)))?;

                if !result.status().is_success() {
                    let status = result.status();
                    let body = result.text().await.unwrap_or_else(|_| String::new());
                    return Err(AppError::Execution(format!(
                        "Slack cron delivery failed: {} {}",
                        status,
                        body
                    )));
                }

                Ok(format!("Delivered to Slack channel '{}'", account.name))
            }
            other => Err(AppError::Validation(format!(
                "Unsupported cron delivery provider: {}",
                other
            ))),
        }
    }

    fn get_string_config(
        &self,
        target_config: Option<&serde_json::Value>,
        credentials: Option<&serde_json::Value>,
        channel_config: Option<&serde_json::Value>,
        keys: &[&str],
    ) -> Option<String> {
        [target_config, credentials, channel_config]
            .into_iter()
            .flatten()
            .find_map(|value| {
                keys.iter().find_map(|key| {
                    value
                        .get(*key)
                        .and_then(serde_json::Value::as_str)
                        .map(str::trim)
                        .filter(|candidate| !candidate.is_empty())
                        .map(ToString::to_string)
                })
            })
    }

    fn get_object_config<'a>(
        &self,
        target_config: Option<&'a serde_json::Value>,
        credentials: Option<&'a serde_json::Value>,
        channel_config: Option<&'a serde_json::Value>,
        keys: &[&str],
    ) -> Option<&'a serde_json::Map<String, serde_json::Value>> {
        [target_config, credentials, channel_config]
            .into_iter()
            .flatten()
            .find_map(|value| {
                keys.iter().find_map(|key| value.get(*key).and_then(serde_json::Value::as_object))
            })
    }

    fn apply_auth_headers(
        &self,
        headers: &mut HeaderMap,
        target_config: Option<&serde_json::Value>,
        credentials: Option<&serde_json::Value>,
        channel_config: Option<&serde_json::Value>,
    ) -> Result<()> {
        if let Some(token) = self.get_string_config(target_config, credentials, channel_config, &["bearer_token", "token", "api_token"]) {
            let auth_value = HeaderValue::from_str(&format!("Bearer {}", token))
                .map_err(|e| AppError::Validation(format!("Invalid bearer token header for cron delivery: {}", e)))?;
            headers.insert(AUTHORIZATION, auth_value);
            return Ok(());
        }

        if let Some(token) = self.get_string_config(target_config, credentials, channel_config, &["authorization"]) {
            let auth_value = HeaderValue::from_str(&token)
                .map_err(|e| AppError::Validation(format!("Invalid authorization header for cron delivery: {}", e)))?;
            headers.insert(AUTHORIZATION, auth_value);
        }

        Ok(())
    }

    fn apply_custom_headers(
        &self,
        headers: &mut HeaderMap,
        target_config: Option<&serde_json::Value>,
        credentials: Option<&serde_json::Value>,
        channel_config: Option<&serde_json::Value>,
    ) -> Result<()> {
        if let Some(custom_headers) = self.get_object_config(target_config, credentials, channel_config, &["headers"]) {
            for (key, value) in custom_headers {
                let header_value = value.as_str().ok_or_else(|| {
                    AppError::Validation(format!("Custom header '{}' for cron delivery must be a string", key))
                })?;
                let header_name = HeaderName::from_str(key)
                    .map_err(|e| AppError::Validation(format!("Invalid cron delivery header name '{}': {}", key, e)))?;
                let header_value = HeaderValue::from_str(header_value)
                    .map_err(|e| AppError::Validation(format!("Invalid cron delivery header '{}' value: {}", key, e)))?;
                headers.insert(header_name, header_value);
            }
        }

        Ok(())
    }

    async fn create_cron_session(&self, job: &CronJob, agent: &crate::agents::Agent) -> Result<Session> {
        let title = format!("[Cron] {} - {}", job.name, agent.name);
        let session = Session::new(title, Some(agent.id.clone()));
        let db = self.db.lock().await;

        db.execute(
            "INSERT INTO sessions (id, agent_id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                &session.id.to_string(),
                &agent.id,
                &session.title,
                &session.created_at.to_rfc3339(),
                &session.updated_at.to_rfc3339(),
            ],
        ).map_err(|e| AppError::Database(format!("Failed to create cron session: {}", e)))?;

        Ok(session)
    }

    async fn store_message(&self, session_id: Uuid, message: &Message) -> Result<()> {
        let db = self.db.lock().await;
        db.execute(
            r#"
            INSERT INTO messages (id, session_id, role, content)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![
                &message.id.to_string(),
                &session_id.to_string(),
                &message.role.to_string(),
                &message.content,
            ],
        ).map_err(|e| AppError::Database(format!("Failed to store cron message: {}", e)))?;

        Ok(())
    }

    async fn update_job_after_execution(&self, job: &CronJob, output: &str) -> Result<()> {
        let now = Utc::now();
        let next_run = Schedule::from_str(&job.cron_expression)
            .ok()
            .and_then(|schedule| schedule.upcoming(Utc).next())
            .map(|dt| dt.to_rfc3339());

        {
            let db = self.db.lock().await;
            db.execute(
                r#"
                UPDATE cron_jobs
                SET last_run = ?1, next_run = ?2, updated_at = ?3
                WHERE id = ?4
                "#,
                params![&now.to_rfc3339(), &next_run, &now.to_rfc3339(), &job.id],
            ).map_err(|e| AppError::Database(format!("Failed to update cron job after execution: {}", e)))?;
        }

        let mut jobs = self.jobs.write().await;
        if let Some(job_mut) = jobs.get_mut(&job.id) {
            job_mut.last_run = Some(now.to_rfc3339());
            job_mut.next_run = next_run;
            job_mut.updated_at = now.to_rfc3339();
        }

        debug!("Cron job {} produced {} chars of output", job.id, output.len());
        Ok(())
    }

    /// Get execution history for a job
    pub async fn get_executions(&self, job_id: &str, limit: Option<u32>) -> Result<Vec<CronExecution>> {
        let db = self.db.lock().await;
        let limit_clause = limit.map(|l| format!("LIMIT {}", l)).unwrap_or_else(|| String::new());

        let mut stmt = db.prepare(
            format!(
                "SELECT id, job_id, status, started_at, completed_at, output, error
                 FROM cron_executions WHERE job_id = ?1 ORDER BY started_at DESC {}",
                limit_clause
            ).as_str()
        ).map_err(|e| AppError::Database(format!("Failed to query executions: {}", e)))?;

        let executions = stmt
            .query_map(params![job_id], |row| {
                let status_str: String = row.get(2)?;
                let status = match status_str.as_str() {
                    "pending" => CronExecutionStatus::Pending,
                    "running" => CronExecutionStatus::Running,
                    "success" => CronExecutionStatus::Success,
                    "failed" => CronExecutionStatus::Failed,
                    _ => CronExecutionStatus::Failed,
                };
                Ok(CronExecution {
                    id: row.get(0)?,
                    job_id: row.get(1)?,
                    status,
                    started_at: row.get(3)?,
                    completed_at: row.get(4)?,
                    output: row.get(5)?,
                    error: row.get(6)?,
                })
            })
            .map_err(|e| AppError::Database(format!("Failed to query executions: {}", e)))?
            .collect::<std::result::Result<Vec<_>, rusqlite::Error>>()
            .map_err(|e| AppError::Database(format!("Failed to collect executions: {}", e)))?;

        debug!("Retrieved {} executions for job {}", executions.len(), job_id);
        Ok(executions)
    }
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value
        .map(|raw| raw.trim().to_string())
        .filter(|raw| !raw.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cron_job_creation() {
        let job = CronJob::new(
            "Test Job",
            "agent-1",
            "0 */5 * * * *", // Every 5 hours
            "Test message",
        );

        assert_eq!(job.name, "Test Job");
        assert_eq!(job.cron_expression, "0 */5 * * * *");
        assert!(job.enabled);
    }

    #[test]
    fn test_execution_creation() {
        let execution = CronExecution::new("job-1");

        assert!(matches!(execution.status, CronExecutionStatus::Pending));

        let completed = execution.completed(Some("Test output"));
        assert!(matches!(completed.status, CronExecutionStatus::Success));
        assert_eq!(completed.output, Some("Test output".to_string()));
    }

    #[test]
    fn test_normalize_optional_string() {
        assert_eq!(normalize_optional_string(None), None);
        assert_eq!(normalize_optional_string(Some("   ".to_string())), None);
        assert_eq!(
            normalize_optional_string(Some("  value  ".to_string())),
            Some("value".to_string())
        );
    }

    #[test]
    fn test_get_string_config_uses_priority_order() {
        let db = Arc::new(Mutex::new(Connection::open_in_memory().expect("in-memory db should open")));
        let scheduler = CronScheduler::new(db);
        let target = serde_json::json!({ "webhook_url": "https://target.example" });
        let credentials = serde_json::json!({ "webhook_url": "https://credentials.example" });
        let channel = serde_json::json!({ "webhook_url": "https://channel.example" });

        assert_eq!(
            scheduler.get_string_config(Some(&target), Some(&credentials), Some(&channel), &["webhook_url"]),
            Some("https://target.example".to_string())
        );
        assert_eq!(
            scheduler.get_string_config(None, Some(&credentials), Some(&channel), &["webhook_url"]),
            Some("https://credentials.example".to_string())
        );
        assert_eq!(
            scheduler.get_string_config(None, None, Some(&channel), &["webhook_url"]),
            Some("https://channel.example".to_string())
        );
    }
}