//! Cron job management commands
//! Provides Tauri commands for managing scheduled tasks

use crate::cron::{CronScheduler, CreateCronJobRequest, UpdateCronJobRequest, CronJob};
use crate::utils::error::Result;
use tauri::State;
use std::sync::Arc;

/// Create a new cron job
#[tauri::command]
pub async fn create_cron_job(
    request: CreateCronJobRequest,
    scheduler: State<'_, Arc<CronScheduler>>,
) -> Result<CronJob> {
    scheduler.create_job(request).await
}

/// Get all cron jobs
#[tauri::command]
pub async fn get_all_cron_jobs(
    scheduler: State<'_, Arc<CronScheduler>>,
) -> Result<Vec<CronJob>> {
    scheduler.get_all_jobs().await
}

/// Get a specific cron job
#[tauri::command]
pub async fn get_cron_job(
    id: String,
    scheduler: State<'_, Arc<CronScheduler>>,
) -> Result<Option<CronJob>> {
    scheduler.get_job(&id).await
}

/// Update a cron job
#[tauri::command]
pub async fn update_cron_job(
    request: UpdateCronJobRequest,
    scheduler: State<'_, Arc<CronScheduler>>,
) -> Result<CronJob> {
    scheduler.update_job(request).await
}

/// Delete a cron job
#[tauri::command]
pub async fn delete_cron_job(
    id: String,
    scheduler: State<'_, Arc<CronScheduler>>,
) -> Result<()> {
    scheduler.delete_job(&id).await
}

/// Get execution history for a job
#[tauri::command]
pub async fn get_cron_executions(
    job_id: String,
    limit: Option<u32>,
    scheduler: State<'_, Arc<CronScheduler>>,
) -> Result<Vec<crate::cron::CronExecution>> {
    scheduler.get_executions(&job_id, limit).await
}

/// Start the cron scheduler
#[tauri::command]
pub async fn start_cron_scheduler(
    scheduler: State<'_, Arc<CronScheduler>>,
) -> Result<()> {
    scheduler.start().await;
    Ok(())
}

/// Stop the cron scheduler
#[tauri::command]
pub async fn stop_cron_scheduler(
    scheduler: State<'_, Arc<CronScheduler>>,
) -> Result<()> {
    scheduler.stop().await;
    Ok(())
}

/// Run a cron job immediately (quick run)
#[tauri::command]
pub async fn run_cron_job(
    id: String,
    scheduler: State<'_, Arc<CronScheduler>>,
) -> Result<String> {
    let job = scheduler.get_job(&id).await?
        .ok_or_else(|| crate::utils::error::AppError::Validation(format!("Job not found: {}", id)))?;

    // Create execution record
    let db = scheduler.db.lock().await;
    let execution = crate::cron::CronExecution::new(&id).started();

    db.execute(
        r#"
        INSERT INTO cron_executions (id, job_id, status, started_at, completed_at, output, error)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
        rusqlite::params![
            &execution.id,
            &execution.job_id,
            "running",
            &execution.started_at,
            None::<&str>,
            None::<&str>,
            None::<&str>,
        ],
    ).map_err(|e| crate::utils::error::AppError::Database(format!("Failed to create execution: {}", e)))?;
    drop(db);

    // Execute the job
    let result = crate::cron::CronScheduler::execute_job(&job).await;

    // Update execution record
    let (status, output, error) = match result {
        Ok(output) => ("success", Some(output), None::<String>),
        Err(e) => ("failed", None::<String>, Some(e.to_string())),
    };

    let db = scheduler.db.lock().await;
    db.execute(
        r#"
        UPDATE cron_executions
        SET status = ?1, completed_at = ?2, output = ?3, error = ?4
        WHERE id = ?5
        "#,
        rusqlite::params![status, &chrono::Utc::now().to_rfc3339(), output, error, &execution.id],
    ).map_err(|e| crate::utils::error::AppError::Database(format!("Failed to update execution: {}", e)))?;

    result
}