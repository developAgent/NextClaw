//! Cron job scheduler
//! Handles scheduling and execution of automated AI tasks

use crate::utils::error::{AppError, Result};
use chrono::{DateTime, Utc};
use cron::Schedule;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::Duration;
use tracing::{debug, info};

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
                Ok(CronJob {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    agent_id: row.get(3)?,
                    channel_account_id: row.get(4)?,
                    cron_expression: row.get(5)?,
                    message: row.get(6)?,
                    target_config: row.get(7)?,
                    enabled: row.get::<i32, _>(8)? != 0,
                    last_run: row.get(9)?,
                    next_run: row.get(10)?,
                    created_at: row.get(11)?,
                    updated_at: row.get(12)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Database(format!("Failed to map jobs: {}", e)))?;

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
        let job = CronJob::new(&request.name, &request.agent_id, &request.cron_expression, &request.message)
            .with_description(request.description.unwrap_or_default())
            .with_channel_account(request.channel_account_id.unwrap_or_default())
            .with_target_config(request.target_config.unwrap_or_default());

        if !request.enabled.unwrap_or(true) {
            job.disabled();
        }

        // Parse and validate cron expression
        let schedule = Schedule::from_str(&job.cron_expression)
            .map_err(|e| AppError::Validation(format!("Invalid cron expression: {}", e)))?;

        // Calculate next run time
        let now = Utc::now();
        let next_run = schedule.upcoming(Utc).next();
        let next_run_str = next_run
            .map(|dt| dt.with_timezone(&chrono::Utc).to_rfc3339())
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
            ..job
        };

        // Update in-memory cache
        let mut jobs = self.jobs.write().await;
        jobs.insert(job.id.clone(), job_with_next.clone());

        // Update schedule cache
        let mut schedules = self.schedules.write().await;
        schedules.insert(job.id.clone(), schedule);

        info!("Created cron job: {}", job.name);
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
        let mut job = self.get_job(&request.id)?
            .ok_or_else(|| AppError::Validation(format!("Job not found: {}", request.id)))?;

        // Update fields
        if let Some(name) = request.name {
            job.name = name;
        }
        if let Some(desc) = request.description {
            job.description = Some(desc);
        }
        if let Some(agent_id) = request.agent_id {
            job.agent_id = agent_id;
        }
        if let Some(account_id) = request.channel_account_id {
            job.channel_account_id = Some(account_id);
        }
        if let Some(cron_expr) = request.cron_expression {
            job.cron_expression = cron_expr;

            // Re-parse schedule
            let schedule = Schedule::from_str(&cron_expr)
                .map_err(|e| AppError::Validation(format!("Invalid cron expression: {}", e)))?;

            // Update next run time
            let now = Utc::now();
            let next_run = schedule.upcoming(Utc).next();
            let next_run_str = next_run
                .map(|dt| dt.with_timezone(&chrono::Utc).to_rfc3339())
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
            job.target_config = Some(config);
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

        tokio::spawn(async move {
            loop {
                if !*running.lock().await {
                    break;
                }

                // Check for jobs that need to run
                let now = Utc::now();

                let jobs = self.jobs.read().await;
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

                if !jobs_to_run.is_empty() {
                    debug!("Found {} jobs to run", jobs_to_run.len());

                    for (job_id, job) in jobs_to_run {
                        // Execute job in a separate task
                        let db = self.db.clone();
                        let jobs_cache = self.jobs.clone();
                        let schedules_cache = self.schedules.clone();
                        let running_clone = running.clone();

                        tokio::spawn(async move {
                            // Update job state
                            {
                                let mut jobs = jobs_cache.write().await;
                                if let Some(job) = jobs.get_mut(&job_id) {
                                    job.last_run = Some(Utc::now().to_rfc3339());

                                    // Calculate next run time
                                    if let Some(schedule) = {
                                        let schedules = schedules_cache.read().await;
                                        schedules.get(&job_id).cloned()
                                    } {
                                        let next_run = schedule.upcoming(Utc).next();
                                        let next_run_str = next_run
                                            .map(|dt| dt.with_timezone(&chrono::Utc).to_rfc3339())
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
                            let result = Self::execute_job(&job).await;

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
    pub async fn execute_job(job: &CronJob) -> Result<String> {
        debug!("Executing cron job: {}", job.name);

        // TODO: Execute the actual job
        // This would call the agent with the message
        // and send the result to the target

        Ok(format!("Job '{}' executed at {}", job.name, Utc::now().to_rfc3339()))
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
            )
        ).map_err(|e| AppError::Database(format!("Failed to query executions: {}", e)))?;

        let executions = stmt
            .query_map(params![job_id], |row| {
                Ok(CronExecution {
                    id: row.get(0)?,
                    job_id: row.get(1)?,
                    status: match row.get::<&str, _>(2)?.as_str() {
                    "pending" => CronExecutionStatus::Pending,
                    "running" => CronExecutionStatus::Running,
                    "success" => CronExecutionStatus::Success,
                    "failed" => CronExecutionStatus::Failed,
                    _ => CronExecutionStatus::Failed,
                },
                    started_at: row.get(3)?,
                    completed_at: row.get(4)?,
                    output: row.get(5)?,
                    error: row.get(6)?,
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AppError::Database(format!("Failed to map executions: {}", e)))?;

        debug!("Retrieved {} executions for job {}", executions.len(), job_id);
        Ok(executions)
    }
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
}