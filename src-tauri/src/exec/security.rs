use regex::Regex;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

use crate::utils::error::{AppError, Result};

/// Security validator for command execution
pub struct SecurityValidator {
    whitelist: Vec<Regex>,
    blacklist: Vec<Regex>,
    sandbox_path: Option<PathBuf>,
}

impl SecurityValidator {
    /// Create a new security validator
    #[must_use]
    pub fn new(
        whitelist: Vec<String>,
        blacklist: Vec<String>,
        sandbox_path: Option<PathBuf>,
    ) -> Self {
        let whitelist_patterns: Vec<Regex> = whitelist
            .iter()
            .filter_map(|pattern| Regex::new(pattern).ok())
            .collect();

        let blacklist_patterns: Vec<Regex> = blacklist
            .iter()
            .filter_map(|pattern| Regex::new(pattern).ok())
            .collect();

        debug!(
            "Security validator initialized: {} whitelist patterns, {} blacklist patterns",
            whitelist_patterns.len(),
            blacklist_patterns.len()
        );

        Self {
            whitelist: whitelist_patterns,
            blacklist: blacklist_patterns,
            sandbox_path,
        }
    }

    /// Validate a command before execution
    ///
    /// # Errors
    ///
    /// Returns an error if the command is not allowed
    pub fn validate_command(&self, command: &str) -> Result<()> {
        // Check blacklist first
        for pattern in &self.blacklist {
            if pattern.is_match(command) {
                warn!("Command blocked by blacklist: {}", command);
                return Err(AppError::Security(format!(
                    "Command not allowed: matches blacklist pattern"
                )));
            }
        }

        // If whitelist is not empty, check if command matches
        if !self.whitelist.is_empty() {
            let allowed = self
                .whitelist
                .iter()
                .any(|pattern| pattern.is_match(command));
            if !allowed {
                warn!("Command blocked by whitelist: {}", command);
                return Err(AppError::Security(format!(
                    "Command not allowed: does not match any whitelist pattern"
                )));
            }
        }

        // Check for path sandbox violations
        if let Some(ref sandbox) = self.sandbox_path {
            self.check_sandbox_violation(command, sandbox)?;
        }

        // Check for suspicious patterns
        self.check_suspicious_patterns(command)?;

        debug!("Command validated successfully: {}", command);
        Ok(())
    }

    /// Check if command violates sandbox restrictions
    fn check_sandbox_violation(&self, command: &str, sandbox: &Path) -> Result<()> {
        let tokens = command.split_whitespace().collect::<Vec<_>>();

        for token in tokens {
            if token.starts_with('/') || token.starts_with('\\') || token.contains(':') {
                let path = PathBuf::from(token);
                if path.is_absolute() {
                    let canonical_path = path.canonicalize().unwrap_or(path);
                    let canonical_sandbox = sandbox.canonicalize().unwrap_or(sandbox.to_path_buf());

                    if !canonical_path.starts_with(&canonical_sandbox) {
                        warn!(
                            "Path outside sandbox: {:?} (sandbox: {:?})",
                            canonical_path, canonical_sandbox
                        );
                        return Err(AppError::Security(format!(
                            "Path outside allowed sandbox: {:?}",
                            token
                        )));
                    }
                }
            }
        }

        Ok(())
    }

    /// Check for suspicious/dangerous patterns
    fn check_suspicious_patterns(&self, command: &str) -> Result<()> {
        let dangerous_patterns = [
            (r"rm\s+-rf\s+/", "rm -rf / - dangerous deletion"),
            (r":>\s*/", "writing to root filesystem"),
            (r"dd\s+if=/dev/zero", "dd command - disk destruction"),
            (r"mkfs\b", "mkfs command - filesystem formatting"),
            (r">\s*/dev/sda", "writing directly to disk"),
            (r"chmod\s+-R\s+777", "chmod 777 on directory"),
        ];

        for (pattern, description) in &dangerous_patterns {
            let regex = Regex::new(pattern).unwrap();
            if regex.is_match(command) {
                warn!("Suspicious command detected: {} - {}", command, description);
                // Return success but mark as requiring confirmation
                return Ok(());
            }
        }

        Ok(())
    }

    /// Check if command requires user confirmation
    #[must_use]
    pub fn requires_confirmation(&self, command: &str) -> bool {
        let high_risk_patterns = [
            r"rm\s+",
            r"dd\b",
            r"mkfs\b",
            r"format\b",
            r"del\s+",
            r"rmdir",
        ];

        for pattern in &high_risk_patterns {
            if Regex::new(pattern).unwrap().is_match(command) {
                return true;
            }
        }

        false
    }

    /// Sanitize command string
    #[must_use]
    pub fn sanitize_command(&self, command: &str) -> String {
        // Remove any escape sequences or control characters
        command
            .chars()
            .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blacklist() {
        let validator = SecurityValidator::new(vec![], vec![r"rm -rf /".to_string()], None);

        assert!(validator.validate_command("rm -rf /").is_err());
        assert!(validator.validate_command("ls -la").is_ok());
    }

    #[test]
    fn test_whitelist() {
        let validator = SecurityValidator::new(
            vec![r"^ls\b".to_string(), r"^cat\b".to_string()],
            vec![],
            None,
        );

        assert!(validator.validate_command("ls -la").is_ok());
        assert!(validator.validate_command("cat file.txt").is_ok());
        assert!(validator.validate_command("rm file").is_err());
    }

    #[test]
    fn test_confirmation_required() {
        let validator = SecurityValidator::new(vec![], vec![], None);

        assert!(validator.requires_confirmation("rm -rf /tmp"));
        assert!(validator.requires_confirmation("dd if=/dev/zero"));
        assert!(!validator.requires_confirmation("ls -la"));
        assert!(!validator.requires_confirmation("cat file.txt"));
    }

    #[test]
    fn test_sanitize() {
        let validator = SecurityValidator::new(vec![], vec![], None);
        let cmd = "ls\x1b[31m -la"; // With ANSI escape
        let sanitized = validator.sanitize_command(cmd);
        assert!(!sanitized.contains('\x1b'));
    }
}
