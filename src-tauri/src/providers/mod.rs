//! AI Provider implementations
//! Support for various AI providers (OpenAI, Anthropic, etc.)

pub mod anthropic;
pub mod openai;

pub use anthropic::{
    AnthropicConfig, AnthropicMessage, AnthropicMessageContent, AnthropicMessageRole,
    AnthropicProvider, MessageCreateRequest,
};
pub use openai::{ChatCompletionRequest, ChatMessage, MessageRole, OpenAIConfig, OpenAIProvider};

/// Provider type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderType {
    OpenAI,
    Anthropic,
    Moonshot,
    Custom,
}

impl ProviderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProviderType::OpenAI => "openai",
            ProviderType::Anthropic => "anthropic",
            ProviderType::Moonshot => "moonshot",
            ProviderType::Custom => "custom",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "openai" => Some(ProviderType::OpenAI),
            "anthropic" => Some(ProviderType::Anthropic),
            "moonshot" => Some(ProviderType::Moonshot),
            "custom" => Some(ProviderType::Custom),
            _ => None,
        }
    }
}

impl std::fmt::Display for ProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_type_conversion() {
        assert_eq!(ProviderType::OpenAI.as_str(), "openai");
        assert_eq!(ProviderType::Anthropic.as_str(), "anthropic");
        assert_eq!(ProviderType::from_str("openai"), Some(ProviderType::OpenAI));
        assert_eq!(
            ProviderType::from_str("anthropic"),
            Some(ProviderType::Anthropic)
        );
        assert_eq!(ProviderType::from_str("invalid"), None);
    }

    #[test]
    fn test_provider_type_display() {
        assert_eq!(ProviderType::OpenAI.to_string(), "openai");
        assert_eq!(ProviderType::Anthropic.to_string(), "anthropic");
    }
}
