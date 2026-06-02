use crate::llm::provider::{
    ContentBlock, ImageSource, LLMConfig, LLMError, LLMProvider, LLMResponse, LLMResult, Message,
    MessageContent, Role, TokenUsage, ToolCall,
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
    content: serde_json::Value, // Can be string or array of content blocks
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

/// Anthropic streaming event structures
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AnthropicStreamEvent {
    #[serde(rename = "type")]
    event_type: String,
    content_block: Option<serde_json::Value>,
    delta: Option<AnthropicDelta>,
    index: Option<usize>,
    usage: Option<AnthropicStreamUsage>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AnthropicDelta {
    #[serde(rename = "type")]
    delta_type: String,
    text: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct AnthropicStreamUsage {
    output_tokens: Option<usize>,
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
                    let text_content = match &message.content {
                        MessageContent::Text(text) => text.clone(),
                        MessageContent::Blocks(_) => {
                            // System messages should be text-only, extract text from blocks
                            message.content.to_string()
                        }
                    };
                    if let Some(content) = system_message {
                        system_message = Some(format!("{}\n{}", content, text_content));
                    } else {
                        system_message = Some(text_content);
                    }
                }
                Role::User => {
                    let content = self.convert_message_content(&message.content);
                    anthropic_messages.push(AnthropicMessage { role: "user".to_string(), content });
                }
                Role::Assistant => {
                    let content = self.convert_message_content(&message.content);
                    anthropic_messages
                        .push(AnthropicMessage { role: "assistant".to_string(), content });
                }
                Role::Tool => {
                    let content = self.convert_message_content(&message.content);
                    anthropic_messages.push(AnthropicMessage { role: "user".to_string(), content });
                }
            }
        }

        (system_message, anthropic_messages)
    }

    fn convert_message_content(&self, content: &MessageContent) -> serde_json::Value {
        match content {
            MessageContent::Text(text) => serde_json::Value::String(text.clone()),
            MessageContent::Blocks(blocks) => {
                let anthropic_blocks: Vec<serde_json::Value> = blocks
                    .iter()
                    .map(|block| match block {
                        ContentBlock::Text(text) => serde_json::json!({
                            "type": "text",
                            "text": text
                        }),
                        ContentBlock::Image(image) => {
                            let (source_type, data) = match &image.image_url {
                                ImageSource::Url(url) => ("url".to_string(), url.clone()),
                                ImageSource::Base64 { media_type, data } => {
                                    ("base64".to_string(), format!("data:{};base64,{}", media_type, data))
                                }
                            };
                            serde_json::json!({
                                "type": "image",
                                "source": {
                                    "type": source_type,
                                    "media_type": match &image.image_url {
                                        ImageSource::Base64 { media_type, .. } => Some(media_type.clone()),
                                        _ => None,
                                    },
                                    "data": data
                                }
                            })
                        }
                    })
                    .collect();

                serde_json::Value::Array(anthropic_blocks)
            }
        }
    }

    /// Build a `/messages` request (tools optional, streaming flag explicit).
    fn build_request(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
        tools: Option<Vec<AnthropicTool>>,
        stream: bool,
    ) -> AnthropicRequest {
        let (system, anthropic_messages) = self.convert_messages(messages);
        AnthropicRequest {
            model: config.model.clone(),
            max_tokens: config.max_tokens.unwrap_or(4096),
            messages: anthropic_messages,
            system,
            tools,
            stream,
        }
    }

    /// POST a request to `/messages` and deserialize the (non-streaming) response.
    async fn post_messages(&self, request: &AnthropicRequest) -> LLMResult<AnthropicResponse> {
        let response = self
            .client
            .post(format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(LLMError::Network)?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LLMError::Api(format!("Anthropic API error: {}", error_text)));
        }

        response
            .json()
            .await
            .map_err(|e| LLMError::InvalidResponse(format!("Failed to parse response: {}", e)))
    }
}

impl AnthropicResponse {
    /// Concatenate the text content blocks (tool_use blocks are handled separately).
    fn extract_content(&self) -> String {
        let mut content = String::new();
        for block in &self.content {
            match block.content_type.as_str() {
                "text" => {
                    if let Some(text) = &block.text {
                        content.push_str(text);
                    }
                }
                "tool_use" => {} // surfaced via extract_tool_calls
                unknown_type => tracing::warn!("Unknown Anthropic content type: {}", unknown_type),
            }
        }
        content
    }

    /// Map `tool_use` content blocks to our generic `ToolCall` type.
    fn extract_tool_calls(&self) -> Option<Vec<ToolCall>> {
        let tool_calls: Vec<ToolCall> = self
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
            .collect();
        (!tool_calls.is_empty()).then_some(tool_calls)
    }

    fn token_usage(&self) -> TokenUsage {
        TokenUsage {
            prompt_tokens: self.usage.input_tokens,
            completion_tokens: self.usage.output_tokens,
            total_tokens: self.usage.input_tokens + self.usage.output_tokens,
        }
    }
}

#[async_trait]
impl LLMProvider for AnthropicProvider {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse> {
        let request = self.build_request(messages, config, None, false);
        let response = self.post_messages(&request).await?;

        Ok(LLMResponse {
            content: response.extract_content(),
            tool_calls: None,
            finish_reason: response.stop_reason.clone().unwrap_or_else(|| "stop".to_string()),
            usage: response.token_usage(),
        })
    }

    async fn chat_completion_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<serde_json::Value>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse> {
        let anthropic_tools = self.convert_tools(&tools)?;
        let request = self.build_request(messages, config, Some(anthropic_tools), false);
        let response = self.post_messages(&request).await?;

        Ok(LLMResponse {
            content: response.extract_content(),
            tool_calls: response.extract_tool_calls(),
            finish_reason: response.stop_reason.clone().unwrap_or_else(|| "stop".to_string()),
            usage: response.token_usage(),
        })
    }

    async fn stream_chat_completion(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
    ) -> LLMResult<tokio::sync::mpsc::Receiver<String>> {
        let (system, anthropic_messages) = self.convert_messages(messages);

        let request = AnthropicRequest {
            model: config.model.clone(),
            max_tokens: config.max_tokens.unwrap_or(4096),
            messages: anthropic_messages,
            system,
            tools: None,
            stream: true,
        };

        let (tx, rx) = tokio::sync::mpsc::channel::<String>(100);

        let client = self.client.clone();
        let api_key = self.api_key.clone();
        let base_url = self.base_url.clone();

        tokio::spawn(async move {
            let response = match client
                .post(format!("{}/messages", base_url))
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    tracing::error!("Anthropic streaming request failed: {}", e);
                    return;
                }
            };

            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_default();
                tracing::error!("Anthropic streaming error response: {}", error_text);
                return;
            }

            crate::llm::http::pump_sse(response, tx, |data| {
                match serde_json::from_str::<AnthropicStreamEvent>(data) {
                    Ok(event) => match event.event_type.as_str() {
                        "content_block_delta" => {
                            let tokens = event.delta.and_then(|d| d.text).into_iter().collect();
                            crate::llm::http::SseAction::Emit(tokens)
                        }
                        "message_stop" => crate::llm::http::SseAction::Stop,
                        other => {
                            tracing::debug!("Received event type: {}", other);
                            crate::llm::http::SseAction::Skip
                        }
                    },
                    Err(e) => {
                        tracing::debug!("Failed to parse Anthropic SSE event: {}. Data: {}", e, data);
                        crate::llm::http::SseAction::Skip
                    }
                }
            })
            .await;
        });

        Ok(rx)
    }

    async fn vision_chat_completion(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse> {
        // For Anthropic, vision models use the same API as regular chat completion
        // The difference is in the message format (handled by convert_messages)
        self.chat_completion(messages, config).await
    }

    async fn vision_chat_completion_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<serde_json::Value>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse> {
        // For Anthropic, vision models with tools use the same API as regular chat completion with tools
        self.chat_completion_with_tools(messages, tools, config).await
    }

    async fn stream_vision_chat_completion(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
    ) -> LLMResult<tokio::sync::mpsc::Receiver<String>> {
        // For Anthropic, vision streaming uses the same API as regular streaming
        self.stream_chat_completion(messages, config).await
    }

    fn provider_name(&self) -> &str {
        "anthropic"
    }

    fn supports_vision(&self) -> bool {
        true
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
            Err(_) => Ok(false),
        }
    }
}
