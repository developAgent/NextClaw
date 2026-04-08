use std::io::{self, Write};

/// Log levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    /// Get the log level as a string
    pub fn as_str(&self) -> &str {
        match self {
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

/// Log a message
pub fn log(level: LogLevel, message: &str) {
    let timestamp = chrono::Utc::now().to_rfc3339();
    let log_line = format!("[{}] [{}] {}", timestamp, level.as_str(), message);

    // Write to stdout
    let _ = writeln!(io::stdout(), "{}", log_line);
}

/// Log a debug message
pub fn log_debug(message: &str) {
    log(LogLevel::Debug, message);
}

/// Log an info message
pub fn log_info(message: &str) {
    log(LogLevel::Info, message);
}

/// Log a warning message
pub fn log_warn(message: &str) {
    log(LogLevel::Warn, message);
}

/// Log an error message
pub fn log_error(message: &str) {
    log(LogLevel::Error, message);
}

/// Log a formatted message
#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)*) => {
        $crate::utils::log($level, &format!($($arg)*));
    };
}

/// Log a formatted debug message
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::utils::log_debug(&format!($($arg)*));
    };
}

/// Log a formatted info message
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::utils::log_info(&format!($($arg)*));
    };
}

/// Log a formatted warning message
#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::utils::log_warn(&format!($($arg)*));
    };
}

/// Log a formatted error message
#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::utils::log_error(&format!($($arg)*));
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_levels() {
        assert_eq!(LogLevel::Debug.as_str(), "DEBUG");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Warn.as_str(), "WARN");
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
    }
}