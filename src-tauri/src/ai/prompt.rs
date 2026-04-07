use crate::ai::{Message, MessageRole};

/// System prompt for CEOClaw
pub const SYSTEM_PROMPT: &str = r#"You are CEOClaw, an AI assistant that helps users execute commands and automate tasks on their computer.

## Your Capabilities

You can:
- Execute system commands (via command execution tools)
- List and read files
- Write files with user confirmation
- Analyze file contents
- Help with development tasks

## Important Guidelines

1. **Safety First**: Never suggest dangerous commands without warnings (e.g., `rm -rf`, `dd`, `mkfs`)
2. **Clarity**: Explain what each command does before suggesting it
3. **Confirmation**: For destructive operations, always ask for confirmation
4. **Context**: Maintain context across the conversation to provide helpful follow-up suggestions
5. **Honesty**: If you're unsure about something, admit it rather than guessing

## Workflow

1. Understand the user's request
2. Plan the necessary steps
3. Execute commands or file operations
4. Report results clearly
5. Suggest follow-up actions if relevant

## Error Handling

When a command fails:
- Report the error clearly
- Suggest possible causes
- Offer alternative approaches when available"#;

/// Build a system prompt with custom context
pub fn build_system_prompt(context: Option<&str>) -> String {
    if let Some(ctx) = context {
        format!("{SYSTEM_PROMPT}\n\n## Current Context\n\n{ctx}")
    } else {
        SYSTEM_PROMPT.to_string()
    }
}

/// Format messages for the AI API
pub fn format_messages(messages: &[Message]) -> Vec<(String, String)> {
    messages
        .iter()
        .map(|msg| (msg.role.to_string(), msg.content.clone()))
        .collect()
}

/// Create a prompt for command execution
pub fn command_execution_prompt(command: &str, explanation: &str) -> String {
    format!(
        r#"I'm about to execute the following command:

```
{command}
```

## Explanation
{explanation}

## Status
Ready to execute. Please confirm or provide feedback."#
    )
}

/// Create a prompt for file analysis
pub fn file_analysis_prompt(filename: &str, content: &str) -> String {
    format!(
        r#"Please analyze the following file:

**File:** {filename}
**Size:** {} bytes

```
{content}
```

Provide a summary and any observations."#,
        content.len()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_system_prompt() {
        let prompt = build_system_prompt(None);
        assert!(prompt.contains("CEOClaw"));
        assert!(prompt.contains("Safety First"));
    }

    #[test]
    fn test_system_prompt_with_context() {
        let prompt = build_system_prompt(Some("User is working on a Rust project"));
        assert!(prompt.contains("Current Context"));
        assert!(prompt.contains("Rust project"));
    }

    #[test]
    fn test_format_messages() {
        let session_id = Uuid::new_v4();
        let messages = vec![
            Message::new(session_id, MessageRole::User, "Hello".to_string()),
            Message::new(session_id, MessageRole::Assistant, "Hi there!".to_string()),
        ];
        let formatted = format_messages(&messages);
        assert_eq!(formatted.len(), 2);
        assert_eq!(formatted[0], ("user".to_string(), "Hello".to_string()));
    }
}