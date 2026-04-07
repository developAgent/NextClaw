use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// Parsed command from AI response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedCommand {
    pub command: String,
    pub description: String,
    pub requires_confirmation: bool,
}

/// Parse AI response to extract commands and other structured data
pub struct ResponseParser {
    command_pattern: Regex,
    code_block_pattern: Regex,
}

impl Default for ResponseParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ResponseParser {
    /// Create a new response parser
    #[must_use]
    pub fn new() -> Self {
        Self {
            command_pattern: Regex::new(r"```(?:bash|sh|shell|cmd|powershell)?\n?([^`]+)```")
                .expect("Invalid command pattern"),
            code_block_pattern: Regex::new(r"```(\w+)?\n?([^`]+)```")
                .expect("Invalid code block pattern"),
        }
    }

    /// Extract commands from AI response
    pub fn extract_commands(&self, response: &str) -> Vec<ParsedCommand> {
        let mut commands = Vec::new();

        for captures in self.command_pattern.captures_iter(response) {
            if let Some(command) = captures.get(1) {
                let cmd_str = command.as_str().trim();
                if !cmd_str.is_empty() {
                    let requires_confirmation = self.is_dangerous_command(cmd_str);
                    commands.push(ParsedCommand {
                        command: cmd_str.to_string(),
                        description: self.extract_command_description(response, cmd_str),
                        requires_confirmation,
                    });
                }
            }
        }

        debug!("Extracted {} commands from response", commands.len());
        commands
    }

    /// Extract code blocks from response
    pub fn extract_code_blocks(&self, response: &str) -> Vec<(Option<String>, String)> {
        let mut blocks = Vec::new();

        for captures in self.code_block_pattern.captures_iter(response) {
            let lang = captures.get(1).map(|m| m.as_str().to_string());
            let content = captures.get(2).map(|m| m.as_str().trim().to_string());
            if let Some(code) = content {
                blocks.push((lang, code));
            }
        }

        blocks
    }

    /// Clean response by removing code blocks if desired
    #[must_use]
    pub fn clean_response(&self, response: &str, remove_code: bool) -> String {
        if remove_code {
            self.code_block_pattern.replace_all(response, "").to_string()
        } else {
            response.to_string()
        }
    }

    /// Check if a command is potentially dangerous
    fn is_dangerous_command(&self, command: &str) -> bool {
        let dangerous_patterns = [
            r"rm\s+-rf\b",
            r"rm\s+-r\s+/",
            r"dd\s+if=",
            r"mkfs\b",
            r"format\b",
            r"del\s+/?s/q",
            r"rmdir\s+/s/q",
            r"shutdown\b",
            r"reboot\b",
            r":>\s*/*",
            r">\s*/*",
        ];

        for pattern in &dangerous_patterns {
            if Regex::new(pattern).unwrap().is_match(command) {
                return true;
            }
        }

        false
    }

    /// Extract description for a command from surrounding text
    fn extract_command_description(&self, response: &str, command: &str) -> String {
        let lines: Vec<&str> = response.lines().collect();
        for (i, line) in lines.iter().enumerate() {
            if line.contains(command) && i > 0 {
                return lines[i - 1].trim().to_string();
            }
        }
        "Execute command".to_string()
    }

    /// Parse structured JSON from response
    pub fn extract_json<T>(&self, response: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let json_pattern = Regex::new(r"```json\n?([^`]+)```").ok()?;
        if let Some(captures) = json_pattern.captures(response) {
            if let Some(json_str) = captures.get(1) {
                serde_json::from_str(json_str.as_str()).ok()
            } else {
                None
            }
        } else {
            // Try to find JSON without code blocks
            serde_json::from_str(response).ok()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_commands() {
        let parser = ResponseParser::new();
        let response = r#"Here's a command to list files:
```bash
ls -la
```
And another one:
```sh
pwd
```"#;

        let commands = parser.extract_commands(response);
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].command, "ls -la");
        assert_eq!(commands[1].command, "pwd");
    }

    #[test]
    fn test_dangerous_command_detection() {
        let parser = ResponseParser::new();
        assert!(parser.is_dangerous_command("rm -rf /"));
        assert!(parser.is_dangerous_command("dd if=/dev/zero of=/dev/sda"));
        assert!(!parser.is_dangerous_command("ls -la"));
        assert!(!parser.is_dangerous_command("echo hello"));
    }

    #[test]
    fn test_extract_code_blocks() {
        let parser = ResponseParser::new();
        let response = r#"Here's some Rust code:
```rust
fn main() {
    println!("Hello");
}
```
And some Python:
```python
print("World")
```"#;

        let blocks = parser.extract_code_blocks(response);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].0, Some("rust".to_string()));
        assert_eq!(blocks[1].0, Some("python".to_string()));
    }

    #[test]
    fn test_clean_response() {
        let parser = ResponseParser::new();
        let response = r#"Here's some text:
```bash
ls -la
```
And more text."#;

        let cleaned = parser.clean_response(response, true);
        assert!(!cleaned.contains("ls -la"));
        assert!(cleaned.contains("Here's some text"));
    }
}