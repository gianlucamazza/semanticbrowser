use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Role of a message in the conversation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

/// A message in the LLM conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    pub fn tool_response(content: impl Into<String>, tool_call_id: String) -> Self {
        Self {
            role: Role::Tool,
            content: content.into(),
            name: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id),
        }
    }
}

/// Function call parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String, // JSON string
}

/// Tool call made by the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionCall,
}

/// LLM Provider Response
#[derive(Debug, Clone)]
pub struct LLMResponse {
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub finish_reason: String,
    pub usage: TokenUsage,
}

/// Token usage statistics
#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

/// Configuration for LLM requests
#[derive(Debug, Clone)]
pub struct LLMConfig {
    pub model: String,
    pub temperature: f32,
    pub max_tokens: Option<usize>,
    pub top_p: Option<f32>,
    pub stop: Option<Vec<String>>,
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            model: "llama3:70b".to_string(),
            temperature: 0.7,
            max_tokens: Some(2048),
            top_p: Some(0.9),
            stop: None,
        }
    }
}

/// Error types for LLM operations
#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("API error: {0}")]
    Api(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

pub type LLMResult<T> = Result<T, LLMError>;

/// Trait for LLM providers (OpenAI, Anthropic, Ollama, etc.)
#[async_trait]
pub trait LLMProvider: Send + Sync {
    /// Send a chat completion request
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse>;

    /// Send a chat completion request with tools/functions
    async fn chat_completion_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<serde_json::Value>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse>;

    /// Stream a chat completion (optional, can return error if not supported)
    async fn stream_chat_completion(
        &self,
        _messages: Vec<Message>,
        _config: &LLMConfig,
    ) -> LLMResult<tokio::sync::mpsc::Receiver<String>> {
        Err(LLMError::Config("Streaming not supported".to_string()))
    }

    /// Get the provider name
    fn provider_name(&self) -> &str;

    /// Check if the provider is available
    async fn health_check(&self) -> LLMResult<bool>;
}
