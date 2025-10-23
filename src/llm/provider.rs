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

/// Content block for messages (supports text and images for vision models)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentBlock {
    /// Text content
    Text(String),
    /// Image content
    Image(ImageContent),
}

/// Image content for vision models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageContent {
    /// Image source (URL or base64 data)
    pub image_url: ImageSource,
}

/// Image source types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ImageSource {
    /// Image URL
    Url(String),
    /// Base64 encoded image data
    Base64 {
        /// MIME type (e.g., "image/jpeg", "image/png")
        #[serde(rename = "type")]
        media_type: String,
        /// Base64 encoded data
        data: String,
    },
}

/// A message in the LLM conversation (supports both text-only and vision content)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    /// Content can be either a simple string (backward compatibility) or content blocks (vision)
    #[serde(flatten)]
    pub content: MessageContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Message content (backward compatible with string, extensible for vision)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    /// Simple text content (backward compatibility)
    Text(String),
    /// Content blocks (for vision models)
    Blocks(Vec<ContentBlock>),
}

impl std::fmt::Display for MessageContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageContent::Text(text) => write!(f, "{}", text),
            MessageContent::Blocks(blocks) => {
                for (i, block) in blocks.iter().enumerate() {
                    if i > 0 {
                        writeln!(f)?;
                    }
                    match block {
                        ContentBlock::Text(text) => write!(f, "{}", text)?,
                        ContentBlock::Image(_) => write!(f, "[Image]")?,
                    }
                }
                Ok(())
            }
        }
    }
}

impl Message {
    /// Create a system message with text content (backward compatible)
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: MessageContent::Text(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Create a user message with text content (backward compatible)
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: MessageContent::Text(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Create an assistant message with text content (backward compatible)
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: MessageContent::Text(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Create a tool response message with text content
    pub fn tool_response(content: impl Into<String>, tool_call_id: String) -> Self {
        Self {
            role: Role::Tool,
            content: MessageContent::Text(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id),
        }
    }

    /// Create a user message with vision content (text + images)
    pub fn user_vision(content_blocks: Vec<ContentBlock>) -> Self {
        Self {
            role: Role::User,
            content: MessageContent::Blocks(content_blocks),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Create a user message with text and a single image
    pub fn user_with_image(text: impl Into<String>, image_url: impl Into<String>) -> Self {
        let blocks = vec![
            ContentBlock::Text(text.into()),
            ContentBlock::Image(ImageContent { image_url: ImageSource::Url(image_url.into()) }),
        ];
        Self::user_vision(blocks)
    }

    /// Create a user message with a single image
    pub fn user_image(image_url: impl Into<String>) -> Self {
        let blocks = vec![ContentBlock::Image(ImageContent {
            image_url: ImageSource::Url(image_url.into()),
        })];
        Self::user_vision(blocks)
    }

    /// Create a user message with base64 encoded image
    pub fn user_image_base64(media_type: impl Into<String>, data: impl Into<String>) -> Self {
        let blocks = vec![ContentBlock::Image(ImageContent {
            image_url: ImageSource::Base64 { media_type: media_type.into(), data: data.into() },
        })];
        Self::user_vision(blocks)
    }

    /// Check if message contains vision content
    pub fn has_vision_content(&self) -> bool {
        matches!(self.content, MessageContent::Blocks(_))
    }

    /// Get text content if message is text-only (for backward compatibility)
    pub fn text_content(&self) -> Option<&str> {
        match &self.content {
            MessageContent::Text(text) => Some(text),
            MessageContent::Blocks(_) => None,
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

    /// Send a vision chat completion request (supports images)
    async fn vision_chat_completion(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse> {
        // Default implementation falls back to regular chat completion
        // Providers that support vision should override this
        self.chat_completion(messages, config).await
    }

    /// Send a vision chat completion request with tools/functions
    async fn vision_chat_completion_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<serde_json::Value>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse> {
        // Default implementation falls back to regular chat completion with tools
        // Providers that support vision should override this
        self.chat_completion_with_tools(messages, tools, config).await
    }

    /// Stream a chat completion (optional, can return error if not supported)
    async fn stream_chat_completion(
        &self,
        _messages: Vec<Message>,
        _config: &LLMConfig,
    ) -> LLMResult<tokio::sync::mpsc::Receiver<String>> {
        Err(LLMError::Config("Streaming not supported".to_string()))
    }

    /// Stream a vision chat completion (optional, can return error if not supported)
    async fn stream_vision_chat_completion(
        &self,
        _messages: Vec<Message>,
        _config: &LLMConfig,
    ) -> LLMResult<tokio::sync::mpsc::Receiver<String>> {
        Err(LLMError::Config("Vision streaming not supported".to_string()))
    }

    /// Get the provider name
    fn provider_name(&self) -> &str;

    /// Check if the provider is available
    async fn health_check(&self) -> LLMResult<bool>;

    /// Check if provider supports vision models
    fn supports_vision(&self) -> bool {
        false
    }
}
