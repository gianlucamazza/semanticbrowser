use crate::llm::provider::{
    LLMConfig, LLMError, LLMProvider, LLMResponse, LLMResult, Message, Role, TokenUsage, ToolCall,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AnthropicProvider {
    pub client: Client,
    pub api_key: String,
    pub base_url: String,
}

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: usize,
    messages: Vec<AnthropicMessage>,
    system: Option<String>,
    tools: Option<Vec<AnthropicTool>>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContentBlock>,
    usage: AnthropicUsage,
    stop_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnthropicContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: Option<String>,
    #[serde(rename = "tool_use")]
    tool_use: Option<AnthropicToolUse>,
}

#[derive(Debug, Deserialize)]
struct AnthropicToolUse {
    id: String,
    name: String,
    input: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: usize,
    output_tokens: usize,
}

#[derive(Debug, Serialize)]
pub struct AnthropicTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: "https://api.anthropic.com/v1".to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn convert_tools(
        &self,
        tools: &[serde_json::Value],
    ) -> Result<Vec<AnthropicTool>, LLMError> {
        let anthropic_tools = tools
            .iter()
            .filter_map(|tool| {
                let function = tool.get("function")?;
                Some(AnthropicTool {
                    name: function.get("name")?.as_str()?.to_string(),
                    description: function.get("description")?.as_str()?.to_string(),
                    input_schema: function.get("parameters")?.clone(),
                })
            })
            .collect();

        Ok(anthropic_tools)
    }

    fn convert_messages(&self, messages: Vec<Message>) -> (Option<String>, Vec<AnthropicMessage>) {
        let mut system_message = None;
        let mut anthropic_messages = Vec::new();

        for message in messages {
            match message.role {
                Role::System => {
                    if let Some(content) = system_message {
                        system_message = Some(format!("{}\n{}", content, message.content));
                    } else {
                        system_message = Some(message.content);
                    }
                }
                Role::User => {
                    anthropic_messages.push(AnthropicMessage {
                        role: "user".to_string(),
                        content: serde_json::Value::String(message.content),
                    });
                }
                Role::Assistant => {
                    anthropic_messages.push(AnthropicMessage {
                        role: "assistant".to_string(),
                        content: serde_json::Value::String(message.content),
                    });
                }
                Role::Tool => {
                    // Anthropic handles tool responses differently - skip for now
                }
            }
        }

        (system_message, anthropic_messages)
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse> {
        let (system, anthropic_messages) = self.convert_messages(messages);

        let request = AnthropicRequest {
            model: config.model.clone(),
            max_tokens: config.max_tokens.unwrap_or(4096),
            messages: anthropic_messages,
            system,
            tools: None,
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(LLMError::Network)?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LLMError::Api(format!("Anthropic API error: {}", error_text)));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| LLMError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        let mut content = String::new();
        for block in &anthropic_response.content {
            match block.content_type.as_str() {
                "text" => {
                    if let Some(text) = &block.text {
                        content.push_str(text);
                    }
                }
                "tool_use" => {
                    // Tool use content is handled separately in tool_calls
                }
                unknown_type => {
                    tracing::warn!("Unknown Anthropic content type: {}", unknown_type);
                }
            }
        }

        let usage = TokenUsage {
            prompt_tokens: anthropic_response.usage.input_tokens,
            completion_tokens: anthropic_response.usage.output_tokens,
            total_tokens: anthropic_response.usage.input_tokens
                + anthropic_response.usage.output_tokens,
        };

        Ok(LLMResponse {
            content,
            tool_calls: None,
            finish_reason: anthropic_response.stop_reason.unwrap_or("stop".to_string()),
            usage,
        })
    }

    async fn chat_completion_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<serde_json::Value>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse> {
        let (system, anthropic_messages) = self.convert_messages(messages);
        let anthropic_tools = self.convert_tools(&tools)?;

        let request = AnthropicRequest {
            model: config.model.clone(),
            max_tokens: config.max_tokens.unwrap_or(4096),
            messages: anthropic_messages,
            system,
            tools: Some(anthropic_tools),
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(LLMError::Network)?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LLMError::Api(format!("Anthropic API error: {}", error_text)));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| LLMError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        let mut content = String::new();
        for block in &anthropic_response.content {
            match block.content_type.as_str() {
                "text" => {
                    if let Some(text) = &block.text {
                        content.push_str(text);
                    }
                }
                "tool_use" => {
                    // Tool use content is handled separately in tool_calls
                }
                unknown_type => {
                    tracing::warn!("Unknown Anthropic content type: {}", unknown_type);
                }
            }
        }

        let tool_calls = anthropic_response
            .content
            .iter()
            .filter_map(|block| block.tool_use.as_ref())
            .map(|tool_use| ToolCall {
                id: tool_use.id.clone(),
                tool_type: "function".to_string(),
                function: crate::llm::provider::FunctionCall {
                    name: tool_use.name.clone(),
                    arguments: serde_json::to_string(&tool_use.input)
                        .unwrap_or_else(|_| "{}".to_string()),
                },
            })
            .collect::<Vec<_>>();

        let tool_calls = if tool_calls.is_empty() {
            None
        } else {
            Some(tool_calls)
        };

        let usage = TokenUsage {
            prompt_tokens: anthropic_response.usage.input_tokens,
            completion_tokens: anthropic_response.usage.output_tokens,
            total_tokens: anthropic_response.usage.input_tokens
                + anthropic_response.usage.output_tokens,
        };

        Ok(LLMResponse {
            content,
            tool_calls,
            finish_reason: anthropic_response.stop_reason.unwrap_or("stop".to_string()),
            usage,
        })
    }

    async fn stream_chat_completion(
        &self,
        _messages: Vec<Message>,
        _config: &LLMConfig,
    ) -> LLMResult<tokio::sync::mpsc::Receiver<String>> {
        Err(LLMError::Api("Streaming not implemented yet".to_string()))
    }

    fn provider_name(&self) -> &str {
        "anthropic"
    }

    async fn health_check(&self) -> LLMResult<bool> {
        let messages = vec![Message::user("Hello".to_string())];
        let config = LLMConfig {
            model: "claude-3-haiku-20240307".to_string(),
            temperature: 0.0,
            max_tokens: Some(10),
            ..Default::default()
        };

        match self.chat_completion(messages, &config).await {
            Ok(_) => Ok(true),
            Err(_) => Err(LLMError::Api("Health check failed".to_string())),
        }
    }
}
