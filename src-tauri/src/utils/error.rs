use thiserror::Error;

/// Application error types
#[derive(Debug, Error)]
pub enum AppError {
    /// Authentication error (invalid API key, etc.)
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// AI/LLM error
    #[error("AI error: {0}")]
    Ai(String),

    /// Command execution error
    #[error("Command execution error: {0}")]
    Execution(String),

    /// Security error
    #[error("Security error: {0}")]
    Security(String),

    /// File operation error
    #[error("File operation error: {0}")]
    File(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type alias for application errors
pub type Result<T> = std::result::Result<T, AppError>;

// Implement conversion from rusqlite::Error
impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Database(err.to_string())
    }
}

// Implement conversion from anyhow::Error
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

// Implement conversion to Tauri's InvokeError
impl From<AppError> for tauri::ipc::InvokeError {
    fn from(error: AppError) -> Self {
        tauri::ipc::InvokeError::from(error.to_string())
    }
}

impl AppError {
    /// Check if this is a recoverable error
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(self, Self::RateLimit(_) | Self::Validation(_))
    }

    /// Get error category for logging/metrics
    #[must_use]
    pub fn category(&self) -> &'static str {
        match self {
            Self::Authentication(_) => "authentication",
            Self::RateLimit(_) => "rate_limit",
            Self::Validation(_) => "validation",
            Self::Ai(_) => "ai",
            Self::Execution(_) => "execution",
            Self::Security(_) => "security",
            Self::File(_) => "file",
            Self::Database(_) => "database",
            Self::Config(_) => "config",
            Self::Io(_) => "io",
            Self::Internal(_) => "internal",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categories() {
        assert_eq!(
            AppError::Authentication("test".to_string()).category(),
            "authentication"
        );
        assert_eq!(
            AppError::RateLimit("test".to_string()).category(),
            "rate_limit"
        );
        assert_eq!(
            AppError::Security("test".to_string()).category(),
            "security"
        );
    }

    #[test]
    fn test_recoverable_errors() {
        assert!(AppError::RateLimit("test".to_string()).is_recoverable());
        assert!(AppError::Validation("test".to_string()).is_recoverable());
        assert!(!AppError::Security("test".to_string()).is_recoverable());
        assert!(!AppError::Authentication("test".to_string()).is_recoverable());
    }
}
