use crate::llm::provider::{
    ContentBlock, ImageSource, LLMConfig, LLMError, LLMProvider, LLMResponse, LLMResult, Message,
    MessageContent, Role, TokenUsage, ToolCall,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct OpenAIProvider {
    pub client: Client,
    pub api_key: String,
    pub base_url: String,
}

#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    max_tokens: Option<usize>,
    temperature: Option<f32>,
    tools: Option<Vec<OpenAITool>>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum OpenAIMessage {
    Text {
        role: String,
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<OpenAIToolCall>>,
    },
    Vision {
        role: String,
        content: Vec<OpenAIContentBlock>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tool_calls: Option<Vec<OpenAIToolCall>>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum OpenAIContentBlock {
    Text {
        r#type: String,
        text: String,
    },
    Image {
        r#type: String,
        image_url: OpenAIImageSource,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIImageSource {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

#[derive(Debug, Serialize)]
pub struct OpenAITool {
    #[serde(rename = "type")]
    pub r#type: String, // "function"
    pub function: OpenAIFunction,
}

#[derive(Debug, Serialize)]
pub struct OpenAIFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIToolCall {
    id: String,
    #[serde(rename = "type")]
    r#type: String,
    function: OpenAIToolCallFunction,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAIToolCallFunction {
    name: String,
    arguments: String,
}

/// OpenAI streaming event structure (SSE)
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenAIStreamEvent {
    object: String,
    choices: Option<Vec<OpenAIStreamChoice>>,
    created: Option<u64>,
    model: Option<String>,
}

/// Choice in streaming response
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenAIStreamChoice {
    index: usize,
    delta: OpenAIStreamDelta,
    finish_reason: Option<String>,
}

/// Delta content in streaming choice
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct OpenAIStreamDelta {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    role: Option<String>,
}

impl OpenAIProvider {
    pub fn new(api_key: String) -> Self {
        Self { client: Client::new(), api_key, base_url: "https://api.openai.com/v1".to_string() }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn convert_tools(&self, tools: &[serde_json::Value]) -> Result<Vec<OpenAITool>, LLMError> {
        let openai_tools = tools
            .iter()
            .filter_map(|tool| {
                let function = tool.get("function")?;
                Some(OpenAITool {
                    r#type: "function".to_string(),
                    function: OpenAIFunction {
                        name: function.get("name")?.as_str()?.to_string(),
                        description: function.get("description")?.as_str()?.to_string(),
                        parameters: function.get("parameters")?.clone(),
                    },
                })
            })
            .collect();

        Ok(openai_tools)
    }

    /// Convert our Message format to OpenAI's message format
    fn convert_message(&self, msg: Message) -> OpenAIMessage {
        let role = match msg.role {
            Role::System => "system".to_string(),
            Role::User => "user".to_string(),
            Role::Assistant => "assistant".to_string(),
            Role::Tool => "assistant".to_string(), // OpenAI doesn't have tool role, map to assistant
        };

        match msg.content {
            MessageContent::Text(text) => {
                OpenAIMessage::Text { role, content: text, tool_calls: None }
            }
            MessageContent::Blocks(blocks) => {
                let content = blocks
                    .into_iter()
                    .map(|block| match block {
                        ContentBlock::Text(text) => {
                            OpenAIContentBlock::Text { r#type: "text".to_string(), text }
                        }
                        ContentBlock::Image(image) => {
                            let url = match image.image_url {
                                ImageSource::Url(url) => url,
                                ImageSource::Base64 { media_type, data } => {
                                    format!("data:{};base64,{}", media_type, data)
                                }
                            };
                            OpenAIContentBlock::Image {
                                r#type: "image_url".to_string(),
                                image_url: OpenAIImageSource {
                                    url,
                                    detail: Some("auto".to_string()), // Let OpenAI choose detail level
                                },
                            }
                        }
                    })
                    .collect();

                OpenAIMessage::Vision { role, content, tool_calls: None }
            }
        }
    }

    /// Build a chat completion request (tools optional, streaming flag explicit).
    fn build_request(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
        tools: Option<Vec<OpenAITool>>,
        stream: bool,
    ) -> OpenAIRequest {
        OpenAIRequest {
            model: config.model.clone(),
            messages: messages.into_iter().map(|msg| self.convert_message(msg)).collect(),
            max_tokens: config.max_tokens,
            temperature: Some(config.temperature),
            tools,
            stream,
        }
    }

    /// POST a request to `/chat/completions` and deserialize the (non-streaming) response.
    async fn post_chat(&self, request: &OpenAIRequest) -> LLMResult<OpenAIResponse> {
        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(LLMError::Network)?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LLMError::Api(format!("OpenAI API error: {}", error_text)));
        }

        response
            .json()
            .await
            .map_err(|e| LLMError::InvalidResponse(format!("Failed to parse response: {}", e)))
    }

    /// Map the first choice of an OpenAI response into our generic `LLMResponse`.
    fn into_llm_response(response: OpenAIResponse) -> LLMResult<LLMResponse> {
        let Some(choice) = response.choices.first() else {
            return Err(LLMError::Api("No response choices".to_string()));
        };

        let usage = response
            .usage
            .as_ref()
            .map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            })
            .unwrap_or_default();

        Ok(LLMResponse {
            content: choice.message.extract_content(),
            tool_calls: choice.message.extract_tool_calls(),
            finish_reason: choice.finish_reason.clone().unwrap_or_else(|| "stop".to_string()),
            usage,
        })
    }
}

impl OpenAIMessage {
    /// Extract textual content (joins text blocks for vision messages).
    fn extract_content(&self) -> String {
        match self {
            OpenAIMessage::Text { content, .. } => content.clone(),
            OpenAIMessage::Vision { content: blocks, .. } => blocks
                .iter()
                .filter_map(|block| match block {
                    OpenAIContentBlock::Text { text, .. } => Some(text.clone()),
                    OpenAIContentBlock::Image { .. } => None,
                })
                .collect::<Vec<_>>()
                .join(" "),
        }
    }

    /// Map any tool calls on this message to our generic `ToolCall` type.
    fn extract_tool_calls(&self) -> Option<Vec<ToolCall>> {
        let (OpenAIMessage::Text { tool_calls, .. } | OpenAIMessage::Vision { tool_calls, .. }) =
            self;
        tool_calls.as_ref().map(|calls| {
            calls
                .iter()
                .map(|tc| ToolCall {
                    id: tc.id.clone(),
                    tool_type: tc.r#type.clone(),
                    function: crate::llm::provider::FunctionCall {
                        name: tc.function.name.clone(),
                        arguments: tc.function.arguments.clone(),
                    },
                })
                .collect()
        })
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse> {
        let request = self.build_request(messages, config, None, false);
        let response = self.post_chat(&request).await?;
        Self::into_llm_response(response)
    }

    async fn chat_completion_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<serde_json::Value>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse> {
        let tools = self.convert_tools(&tools)?;
        let request = self.build_request(messages, config, Some(tools), false);
        let response = self.post_chat(&request).await?;
        Self::into_llm_response(response)
    }

    async fn stream_chat_completion(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
    ) -> LLMResult<tokio::sync::mpsc::Receiver<String>> {
        let openai_messages =
            messages.into_iter().map(|msg| self.convert_message(msg)).collect::<Vec<_>>();

        let request = OpenAIRequest {
            model: config.model.clone(),
            messages: openai_messages,
            max_tokens: config.max_tokens,
            temperature: Some(config.temperature),
            tools: None,
            stream: true,
        };

        let (tx, rx) = tokio::sync::mpsc::channel::<String>(100);

        let client = self.client.clone();
        let api_key = self.api_key.clone();
        let base_url = self.base_url.clone();

        tokio::spawn(async move {
            let response = match client
                .post(format!("{}/chat/completions", base_url))
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&request)
                .send()
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    tracing::error!("OpenAI streaming request failed: {}", e);
                    return;
                }
            };

            if !response.status().is_success() {
                let error_text = response.text().await.unwrap_or_default();
                tracing::error!("OpenAI streaming error response: {}", error_text);
                return;
            }

            crate::llm::http::pump_sse(response, tx, |data| {
                match serde_json::from_str::<OpenAIStreamEvent>(data) {
                    Ok(event) => {
                        let tokens = event
                            .choices
                            .into_iter()
                            .flatten()
                            .filter_map(|choice| choice.delta.content)
                            .collect();
                        crate::llm::http::SseAction::Emit(tokens)
                    }
                    Err(e) => {
                        tracing::debug!("Failed to parse SSE event: {}. Data: {}", e, data);
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
        // For OpenAI, vision models use the same API as regular chat completion
        // The difference is in the message format (handled by convert_message)
        self.chat_completion(messages, config).await
    }

    async fn vision_chat_completion_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<serde_json::Value>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse> {
        // For OpenAI, vision models with tools use the same API as regular chat completion with tools
        self.chat_completion_with_tools(messages, tools, config).await
    }

    async fn stream_vision_chat_completion(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
    ) -> LLMResult<tokio::sync::mpsc::Receiver<String>> {
        // For OpenAI, vision streaming uses the same API as regular streaming
        self.stream_chat_completion(messages, config).await
    }

    fn provider_name(&self) -> &str {
        "openai"
    }

    fn supports_vision(&self) -> bool {
        true
    }

    async fn health_check(&self) -> LLMResult<bool> {
        let response = self
            .client
            .get(format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await;

        match response {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}
