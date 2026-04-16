use crate::db::models::{FileOperation, FileOperationType};
use crate::db::Database;
use crate::utils::error::Result;
use std::fs;
use std::path::Path;
use tauri::State;
use tracing::{debug, info};
use uuid::Uuid;

/// List files in a directory
///
/// Returns a JSON string containing file information
#[tauri::command]
pub fn list_files(
    path: String,
    session_id: String,
    recursive: bool,
    db: State<'_, Database>,
) -> Result<String> {
    info!("Listing files in: {} (recursive: {})", path, recursive);

    let files = if recursive {
        list_files_recursive(&path)?
    } else {
        list_files_flat(&path)?
    };

    // Record operation
    let operation = FileOperation::new(
        Uuid::parse_str(&session_id).unwrap(),
        FileOperationType::List,
        path.clone(),
    );
    save_file_operation(&db, &operation)?;

    debug!("Found {} files", files.len());
    Ok(serde_json::to_string(&files).unwrap())
}

/// Read a file's contents
#[tauri::command]
pub fn read_file(path: String, session_id: String, db: State<'_, Database>) -> Result<String> {
    info!("Reading file: {}", path);

    let content = fs::read_to_string(&path)
        .map_err(|e| crate::utils::error::AppError::File(format!("Failed to read file: {e}")))?;

    // Record operation
    let operation = FileOperation::new(
        Uuid::parse_str(&session_id).unwrap(),
        FileOperationType::Read,
        path.clone(),
    );
    save_file_operation(&db, &operation)?;

    debug!("Read {} bytes from {}", content.len(), path);
    Ok(content)
}

/// Write content to a file
#[tauri::command]
pub fn write_file(
    path: String,
    content: String,
    session_id: String,
    db: State<'_, Database>,
) -> Result<()> {
    info!("Writing to file: {}", path);

    fs::write(&path, &content)
        .map_err(|e| crate::utils::error::AppError::File(format!("Failed to write file: {e}")))?;

    // Record operation
    let operation = FileOperation::new(
        Uuid::parse_str(&session_id).unwrap(),
        FileOperationType::Write,
        path.clone(),
    );
    save_file_operation(&db, &operation)?;

    debug!("Wrote {} bytes to {}", content.len(), path);
    Ok(())
}

/// Get file metadata
#[tauri::command]
pub fn get_file_metadata(path: String) -> Result<FileMetadata> {
    let metadata = fs::metadata(&path)
        .map_err(|e| crate::utils::error::AppError::File(format!("Failed to get metadata: {e}")))?;

    Ok(FileMetadata {
        path: path.clone(),
        size: metadata.len(),
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
        is_readonly: metadata.permissions().readonly(),
        modified: metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64),
    })
}

/// Delete a file
#[tauri::command]
pub fn delete_file(path: String, session_id: String, db: State<'_, Database>) -> Result<()> {
    info!("Deleting file: {}", path);

    fs::remove_file(&path)
        .map_err(|e| crate::utils::error::AppError::File(format!("Failed to delete file: {e}")))?;

    // Record operation
    let operation = FileOperation::new(
        Uuid::parse_str(&session_id).unwrap(),
        FileOperationType::Delete,
        path.clone(),
    );
    save_file_operation(&db, &operation)?;

    debug!("Deleted file: {}", path);
    Ok(())
}

// Data structures

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct FileMetadata {
    pub path: String,
    pub size: u64,
    pub is_file: bool,
    pub is_dir: bool,
    pub is_readonly: bool,
    pub modified: Option<i64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub is_file: bool,
    pub is_dir: bool,
    pub modified: Option<i64>,
}

// Helper functions

fn list_files_flat(path: &str) -> Result<Vec<FileInfo>> {
    let dir_path = Path::new(path);
    let mut files = Vec::new();

    for entry in fs::read_dir(dir_path).map_err(|e| {
        crate::utils::error::AppError::File(format!("Failed to read directory: {e}"))
    })? {
        let entry = entry.map_err(|e| {
            crate::utils::error::AppError::File(format!("Failed to read entry: {e}"))
        })?;
        let metadata = entry.metadata().ok();

        let file_info = FileInfo {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry.path().to_string_lossy().to_string(),
            size: metadata.as_ref().map_or(0, |m| m.len()),
            is_file: metadata.as_ref().map_or(false, |m| m.is_file()),
            is_dir: metadata.as_ref().map_or(false, |m| m.is_dir()),
            modified: metadata
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64),
        };
        files.push(file_info);
    }

    Ok(files)
}

fn list_files_recursive(path: &str) -> Result<Vec<FileInfo>> {
    let mut files = Vec::new();
    list_files_recursive_helper(path, &mut files)?;
    Ok(files)
}

fn list_files_recursive_helper(path: &str, files: &mut Vec<FileInfo>) -> Result<()> {
    let dir_path = Path::new(path);

    for entry in fs::read_dir(dir_path).map_err(|e| {
        crate::utils::error::AppError::File(format!("Failed to read directory: {e}"))
    })? {
        let entry = entry.map_err(|e| {
            crate::utils::error::AppError::File(format!("Failed to read entry: {e}"))
        })?;
        let metadata = entry.metadata().ok();
        let entry_path = entry.path();

        let file_info = FileInfo {
            name: entry.file_name().to_string_lossy().to_string(),
            path: entry_path.to_string_lossy().to_string(),
            size: metadata.as_ref().map_or(0, |m| m.len()),
            is_file: metadata.as_ref().map_or(false, |m| m.is_file()),
            is_dir: metadata.as_ref().map_or(false, |m| m.is_dir()),
            modified: metadata
                .clone()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64),
        };
        files.push(file_info.clone());

        if file_info.is_dir {
            list_files_recursive_helper(&entry_path.to_string_lossy(), files)?;
        }
    }

    Ok(())
}

fn save_file_operation(db: &Database, operation: &FileOperation) -> Result<()> {
    let conn_arc = db.conn();
    let conn = conn_arc.blocking_lock();
    conn.execute(
        r#"
        INSERT INTO file_operations (id, session_id, operation, path, success, error)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        rusqlite::params![
            &operation.id.to_string(),
            &operation.session_id.to_string(),
            &operation.operation.as_str(),
            &operation.path,
            operation.success as i32,
            &operation.error.as_ref().map_or("", |s| s.as_str()),
        ],
    )
    .map_err(|e: rusqlite::Error| crate::utils::error::AppError::Database(e.to_string()))?;
    Ok(())
}
