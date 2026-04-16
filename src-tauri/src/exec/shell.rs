use shlex;
use std::path::PathBuf;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use crate::exec::security::SecurityValidator;
use crate::utils::error::{AppError, Result};
use tracing::{debug, error, info, warn};

/// Command execution result
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration_ms: u64,
    pub timed_out: bool,
}

impl ExecutionResult {
    pub fn success(&self) -> bool {
        self.exit_code.map_or(false, |code| code == 0) && !self.timed_out
    }
}

/// Shell executor
pub struct ShellExecutor {
    validator: SecurityValidator,
    timeout: Duration,
}

impl ShellExecutor {
    /// Create a new shell executor
    #[must_use]
    pub fn new(
        whitelist: Vec<String>,
        blacklist: Vec<String>,
        sandbox_path: Option<PathBuf>,
        timeout_secs: u64,
    ) -> Self {
        let validator = SecurityValidator::new(whitelist, blacklist, sandbox_path);
        let timeout = Duration::from_secs(timeout_secs);

        debug!("Shell executor created with timeout: {:?}", timeout);
        Self { validator, timeout }
    }

    /// Execute a command string
    ///
    /// # Errors
    ///
    /// Returns an error if validation fails or execution fails
    pub async fn execute(&self, command: &str) -> Result<ExecutionResult> {
        // Validate command first
        self.validator.validate_command(command)?;

        let requires_confirmation = self.validator.requires_confirmation(command);
        if requires_confirmation {
            info!("Command requires confirmation: {}", command);
            return Err(AppError::Security(
                "Command requires explicit user confirmation before execution".to_string(),
            ));
        }

        // Sanitize command
        let sanitized = self.validator.sanitize_command(command);
        debug!("Executing command: {}", sanitized);

        let start_time = std::time::Instant::now();

        let result = match self.execute_internal(&sanitized).await {
            Ok(result) => result,
            Err(e) => {
                error!("Command execution failed: {}", e);
                return Err(e);
            }
        };

        let duration = start_time.elapsed();
        info!(
            "Command completed in {}ms with exit code: {:?}",
            duration.as_millis(),
            result.exit_code
        );

        Ok(ExecutionResult {
            exit_code: result.exit_code,
            stdout: result.stdout,
            stderr: result.stderr,
            duration_ms: duration.as_millis() as u64,
            timed_out: result.timed_out,
        })
    }

    /// Internal command execution
    async fn execute_internal(&self, command: &str) -> Result<ExecutionResult> {
        let mut cmd = Self::parse_command(command)?;

        let output_future = async {
            let output = cmd.output().await.map_err(|e| {
                error!("Command execution error: {}", e);
                AppError::Execution(format!("Failed to execute command: {e}"))
            })?;

            Ok::<ExecutionResultInner, AppError>(ExecutionResultInner {
                exit_code: output.status.code(),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                timed_out: false,
            })
        };

        match timeout(self.timeout, output_future).await {
            Ok(Ok(result)) => Ok(ExecutionResult {
                exit_code: result.exit_code,
                stdout: result.stdout,
                stderr: result.stderr,
                duration_ms: 0,
                timed_out: false,
            }),
            Ok(Err(e)) => Err(e),
            Err(_) => {
                warn!("Command timed out: {}", command);
                Ok(ExecutionResult {
                    exit_code: None,
                    stdout: String::new(),
                    stderr: format!("Command timed out after {:?}", self.timeout),
                    duration_ms: 0,
                    timed_out: true,
                })
            }
        }
    }

    /// Parse command string into Command
    fn parse_command(command: &str) -> Result<Command> {
        let parts: Vec<String> = shlex::split(command)
            .ok_or_else(|| AppError::Execution("Failed to parse command".to_string()))?;

        if parts.is_empty() {
            return Err(AppError::Execution("Empty command".to_string()));
        }

        let program = &parts[0];
        let args = &parts[1..];

        debug!("Parsed command: {} with args: {:?}", program, args);

        let mut cmd = Command::new(program);
        cmd.args(args);
        Ok(cmd)
    }

    /// Check if command requires confirmation
    pub fn requires_confirmation(&self, command: &str) -> bool {
        self.validator.requires_confirmation(command)
    }
}

#[derive(Debug)]
struct ExecutionResultInner {
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    timed_out: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_execute_simple_command() {
        let executor = ShellExecutor::new(vec![], vec![], None, 10);
        let result = executor.execute("echo hello").await.unwrap();
        assert!(result.success());
        assert!(result.stdout.contains("hello"));
    }

    #[tokio::test]
    async fn test_execute_list_command() {
        let executor = ShellExecutor::new(vec![], vec![], None, 10);
        let result = executor.execute("ls").await.unwrap();
        assert!(result.success());
    }

    #[tokio::test]
    async fn test_command_timeout() {
        let executor = ShellExecutor::new(vec![], vec![], None, 1);
        let result = executor.execute("sleep 10").await.unwrap();
        assert!(result.timed_out);
    }

    #[tokio::test]
    async fn test_requires_confirmation_blocks_execution() {
        let executor = ShellExecutor::new(vec![], vec![], None, 10);
        let result = executor.execute("rm -rf /tmp/test-dir").await;
        assert!(matches!(result, Err(AppError::Security(_))));
    }

    #[test]
    fn test_blacklist_blocks_command() {
        let executor = ShellExecutor::new(vec![], vec![r"rm -rf".to_string()], None, 10);

        // This should be async but we can test validation directly
        let validator = executor.validator.validate_command("rm -rf /tmp");
        assert!(validator.is_err());
    }
}
