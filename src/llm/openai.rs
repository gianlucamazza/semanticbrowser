use crate::llm::provider::{
    LLMConfig, LLMError, LLMProvider, LLMResponse, LLMResult, Message, Role, TokenUsage, ToolCall,
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
struct OpenAIMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAIToolCall>>,
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
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse> {
        let openai_messages = messages
            .into_iter()
            .map(|msg| OpenAIMessage {
                role: match msg.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Tool => "assistant".to_string(), // OpenAI doesn't have tool role, map to assistant
                },
                content: msg.content,
                tool_calls: None,
            })
            .collect::<Vec<_>>();

        let request = OpenAIRequest {
            model: config.model.clone(),
            messages: openai_messages,
            max_tokens: config.max_tokens,
            temperature: Some(config.temperature),
            tools: None,
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(LLMError::Network)?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LLMError::Api(format!("OpenAI API error: {}", error_text)));
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| LLMError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        if let Some(choice) = openai_response.choices.first() {
            let usage = openai_response
                .usage
                .as_ref()
                .map(|u| TokenUsage {
                    prompt_tokens: u.prompt_tokens,
                    completion_tokens: u.completion_tokens,
                    total_tokens: u.total_tokens,
                })
                .unwrap_or_default();

            Ok(LLMResponse {
                content: choice.message.content.clone(),
                tool_calls: None,
                finish_reason: choice.finish_reason.clone().unwrap_or("stop".to_string()),
                usage,
            })
        } else {
            Err(LLMError::Api("No response choices".to_string()))
        }
    }

    async fn chat_completion_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<serde_json::Value>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse> {
        let openai_messages = messages
            .into_iter()
            .map(|msg| OpenAIMessage {
                role: match msg.role {
                    Role::System => "system".to_string(),
                    Role::User => "user".to_string(),
                    Role::Assistant => "assistant".to_string(),
                    Role::Tool => "assistant".to_string(), // OpenAI doesn't have tool role, map to assistant
                },
                content: msg.content,
                tool_calls: None,
            })
            .collect::<Vec<_>>();

        let tools = self.convert_tools(&tools)?;

        let request = OpenAIRequest {
            model: config.model.clone(),
            messages: openai_messages,
            max_tokens: config.max_tokens,
            temperature: Some(config.temperature),
            tools: Some(tools),
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(LLMError::Network)?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LLMError::Api(format!("OpenAI API error: {}", error_text)));
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| LLMError::InvalidResponse(format!("Failed to parse response: {}", e)))?;

        if let Some(choice) = openai_response.choices.first() {
            let usage = openai_response
                .usage
                .as_ref()
                .map(|u| TokenUsage {
                    prompt_tokens: u.prompt_tokens,
                    completion_tokens: u.completion_tokens,
                    total_tokens: u.total_tokens,
                })
                .unwrap_or_default();

            // Handle tool calls
            let tool_calls = choice.message.tool_calls.as_ref().map(|openai_tool_calls| {
                openai_tool_calls
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
            });

            Ok(LLMResponse {
                content: choice.message.content.clone(),
                tool_calls,
                finish_reason: choice.finish_reason.clone().unwrap_or("stop".to_string()),
                usage,
            })
        } else {
            Err(LLMError::Api("No response choices".to_string()))
        }
    }

    async fn stream_chat_completion(
        &self,
        _messages: Vec<Message>,
        _config: &LLMConfig,
    ) -> LLMResult<tokio::sync::mpsc::Receiver<String>> {
        // TODO: Implement streaming
        Err(LLMError::Api("Streaming not implemented yet".to_string()))
    }

    fn provider_name(&self) -> &str {
        "openai"
    }

    async fn health_check(&self) -> LLMResult<bool> {
        let response = self
            .client
            .get(format!("{}/models", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .map_err(LLMError::Network)?;

        if response.status().is_success() {
            Ok(true)
        } else {
            Err(LLMError::Api("Health check failed".to_string()))
        }
    }
}
