use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Skill arguments passed to a skill
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillArgs {
    /// Arguments as key-value pairs
    pub args: HashMap<String, WasmArgument>,
}

impl SkillArgs {
    /// Create new skill arguments
    pub fn new() -> Self {
        Self {
            args: HashMap::new(),
        }
    }

    /// Load skill arguments from environment variables
    pub fn from_env() -> Self {
        let mut args = Self::new();

        // Try to load from CEOCLAW_ARGS environment variable
        if let Ok(args_json) = std::env::var("CEOCLAW_ARGS") {
            if let Ok(serde_json::Value::from_str(&args_json)) {
                // Parse JSON arguments
                if let Ok(parsed_args) = serde_json::from_str::<HashMap<String, WasmArgument>>(&args_json) {
                    args.args = parsed_args;
                }
            }
        }

        args
    }

    /// Add an argument
    pub fn add(&mut self, key: String, value: WasmArgument) {
        self.args.insert(key, value);
    }

    /// Get a string argument
    pub fn get_string(&self, key: &str) -> Result<String, SkillError> {
        self.args.get(key)
            .and_then(|arg| arg.as_string())
            .ok_or_else(|| SkillError::missing_argument(key))
    }

    /// Get a string argument with a default value
    pub fn get_string_or_default(&self, key: &str, default: String) -> String {
        self.get_string(key).unwrap_or(default)
    }

    /// Get a number argument
    pub fn get_number(&self, key: &str) -> Result<f64, SkillError> {
        self.args.get(key)
            .and_then(|arg| arg.as_number())
            .ok_or_else(|| SkillError::missing_argument(key))
    }

    /// Get a number argument with a default value
    pub fn get_number_or_default(&self, key: &str, default: f64) -> f64 {
        self.get_number(key).unwrap_or(default)
    }

    /// Get a boolean argument
    pub fn get_bool(&self, key: &str) -> Result<bool, SkillError> {
        self.args.get(key)
            .and_then(|arg| arg.as_bool())
            .ok_or_else(|| SkillError::missing_argument(key))
    }

    /// Get a boolean argument with a default value
    pub fn get_bool_or_default(&self, key: &str, default: bool) -> bool {
        self.get_bool(key).unwrap_or(default)
    }

    /// Get an array argument
    pub fn get_array(&self, key: &str) -> Result<Vec<WasmArgument>, SkillError> {
        self.args.get(key)
            .and_then(|arg| arg.as_array())
            .ok_or_else(|| SkillError::missing_argument(key))
    }

    /// Get an object argument
    pub fn get_object(&self, key: &str) -> Result<HashMap<String, WasmArgument>, SkillError> {
        self.args.get(key)
            .and_then(|arg| arg.as_object())
            .ok_or_else(|| SkillError::missing_argument(key))
    }

    /// Check if an argument exists
    pub fn has(&self, key: &str) -> bool {
        self.args.contains_key(key)
    }

    /// Get all argument keys
    pub fn keys(&self) -> Vec<&String> {
        self.args.keys().collect()
    }
}

impl Default for SkillArgs {
    fn default() -> Self {
        Self::new()
    }
}

/// WASM argument type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WasmArgument {
    #[serde(rename = "string")]
    String(String),
    #[serde(rename = "number")]
    Number(f64),
    #[serde(rename = "boolean")]
    Boolean(bool),
    #[serde(rename = "array")]
    Array(Vec<WasmArgument>),
    #[serde(rename = "object")]
    Object(HashMap<String, WasmArgument>),
    #[serde(rename = "null")]
    Null,
}

impl WasmArgument {
    /// Create a string argument
    pub fn string(value: String) -> Self {
        Self::String(value)
    }

    /// Create a number argument
    pub fn number(value: f64) -> Self {
        Self::Number(value)
    }

    /// Create a boolean argument
    pub fn boolean(value: bool) -> Self {
        Self::Boolean(value)
    }

    /// Create an array argument
    pub fn array(values: Vec<WasmArgument>) -> Self {
        Self::Array(values)
    }

    /// Create an object argument
    pub fn object(values: HashMap<String, WasmArgument>) -> Self {
        Self::Object(values)
    }

    /// Create a null argument
    pub fn null() -> Self {
        Self::Null
    }

    /// Try to get as string
    pub fn as_string(&self) -> Option<String> {
        match self {
            Self::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    /// Try to get as number
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Try to get as boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get as array
    pub fn as_array(&self) -> Option<Vec<WasmArgument>> {
        match self {
            Self::Array(a) => Some(a.clone()),
            _ => None,
        }
    }

    /// Try to get as object
    pub fn as_object(&self) -> Option<HashMap<String, WasmArgument>> {
        match self {
            Self::Object(o) => Some(o.clone()),
            _ => None,
        }
    }

    /// Get the argument type
    pub fn get_type(&self) -> WasmArgumentType {
        match self {
            Self::String(_) => WasmArgumentType::String,
            Self::Number(_) => WasmArgumentType::Number,
            Self::Boolean(_) => WasmArgumentType::Boolean,
            Self::Array(_) => WasmArgumentType::Array,
            Self::Object(_) => WasmArgumentType::Object,
            Self::Null => WasmArgumentType::Null,
        }
    }
}

/// WASM argument type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmArgumentType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Null,
}

/// Skill result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillResponse {
    /// Success flag
    pub success: bool,
    /// Response data
    pub data: Option<WasmArgument>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Error code (if failed)
    pub error_code: Option<u32>,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl SkillResponse {
    /// Create a successful response
    pub fn success<T: Into<WasmArgument>>(data: T) -> Self {
        Self {
            success: true,
            data: Some(data.into()),
            error: None,
            error_code: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a failed response
    pub fn error<T: Into<String>>(error: T) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
            error_code: Some(ErrorCode::UNKNOWN as u32),
            metadata: HashMap::new(),
        }
    }

    /// Create a failed response with error code
    pub fn error_with_code<T: Into<String>>(error: T, code: ErrorCode) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error.into()),
            error_code: Some(code as u32),
            metadata: HashMap::new(),
        }
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Check if the response is successful
    pub fn is_success(&self) -> bool {
        self.success
    }
}

impl<T: Into<WasmArgument>> From<T> for SkillResponse {
    fn from(data: T) -> Self {
        Self::success(data)
    }
}

/// Skill error
#[derive(Debug, Error)]
pub enum SkillError {
    #[error("Missing required argument: {0}")]
    MissingArgument(String),

    #[error("Invalid argument type for: {0}")]
    InvalidArgumentType(String),

    #[error("Invalid argument value for: {0}: {1}")]
    InvalidArgumentValue(String, String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(String),
}

impl SkillError {
    /// Create a missing argument error
    pub fn missing_argument(name: &str) -> Self {
        Self::MissingArgument(name.to_string())
    }

    /// Create an invalid argument type error
    pub fn invalid_argument_type(name: &str) -> Self {
        Self::InvalidArgumentType(name.to_string())
    }

    /// Create an invalid argument value error
    pub fn invalid_argument_value(name: &str, reason: &str) -> Self {
        Self::InvalidArgumentValue(name.to_string(), reason.to_string())
    }

    /// Get the error code
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::MissingArgument(_) => ErrorCode::MISSING_ARGUMENT,
            Self::InvalidArgumentType(_) => ErrorCode::INVALID_ARGUMENT_TYPE,
            Self::InvalidArgumentValue(_, _) => ErrorCode::INVALID_ARGUMENT_VALUE,
            Self::ExecutionFailed(_) => ErrorCode::EXECUTION_FAILED,
            Self::PermissionDenied(_) => ErrorCode::PERMISSION_DENIED,
            Self::Io(_) => ErrorCode::IO_ERROR,
            Self::Serialization(_) => ErrorCode::SERIALIZATION_ERROR,
            Self::Other(_) => ErrorCode::UNKNOWN,
        }
    }
}

impl From<SkillError> for SkillResponse {
    fn from(error: SkillError) -> Self {
        Self::error_with_code(error.to_string(), error.code())
    }
}

/// Error codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ErrorCode {
    UNKNOWN = 0,
    MISSING_ARGUMENT = 1,
    INVALID_ARGUMENT_TYPE = 2,
    INVALID_ARGUMENT_VALUE = 3,
    EXECUTION_FAILED = 4,
    PERMISSION_DENIED = 5,
    IO_ERROR = 6,
    SERIALIZATION_ERROR = 7,
    TIMEOUT = 8,
    OUT_OF_MEMORY = 9,
}

impl ErrorCode {
    /// Get the error code as u32
    pub fn as_u32(self) -> u32 {
        self as u32
    }

    /// Get the error code name
    pub fn name(&self) -> &str {
        match self {
            Self::UNKNOWN => "UNKNOWN",
            Self::MISSING_ARGUMENT => "MISSING_ARGUMENT",
            Self::INVALID_ARGUMENT_TYPE => "INVALID_ARGUMENT_TYPE",
            Self::INVALID_ARGUMENT_VALUE => "INVALID_ARGUMENT_VALUE",
            Self::EXECUTION_FAILED => "EXECUTION_FAILED",
            Self::PERMISSION_DENIED => "PERMISSION_DENIED",
            Self::IO_ERROR => "IO_ERROR",
            Self::SERIALIZATION_ERROR => "SERIALIZATION_ERROR",
            Self::TIMEOUT => "TIMEOUT",
            Self::OUT_OF_MEMORY => "OUT_OF_MEMORY",
        }
    }
}

/// Skill result type
pub type SkillResult = Result<SkillResponse, SkillError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_args() {
        let mut args = SkillArgs::new();
        args.add("test".to_string(), WasmArgument::string("value".to_string()));

        assert_eq!(args.get_string("test").unwrap(), "value");
    }

    #[test]
    fn test_wasm_argument() {
        let arg = WasmArgument::string("test".to_string());
        assert_eq!(arg.as_string(), Some("test".to_string()));
        assert_eq!(arg.get_type(), WasmArgumentType::String);
    }

    #[test]
    fn test_skill_response() {
        let response = SkillResponse::success("Hello, World!");
        assert!(response.is_success());
        assert_eq!(response.data, Some(WasmArgument::string("Hello, World!".to_string())));
    }
}