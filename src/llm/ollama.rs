use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::provider::{
    LLMConfig, LLMError, LLMProvider, LLMResponse, LLMResult, Message, TokenUsage,
};

/// Ollama API endpoint configuration
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub timeout: Duration,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self { base_url: "http://localhost:11434".to_string(), timeout: Duration::from_secs(120) }
    }
}

/// Ollama LLM Provider
///
/// Provides integration with local Ollama instance for running LLMs locally.
/// Supports Llama 3, Mistral, Mixtral, and other models.
pub struct OllamaProvider {
    client: Client,
    config: OllamaConfig,
}

impl OllamaProvider {
    pub fn new(config: OllamaConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    /// Convert tools for Ollama API (Ollama uses tools as-is)
    fn convert_tools(&self, tools: &[serde_json::Value]) -> LLMResult<Vec<serde_json::Value>> {
        Ok(tools.to_vec())
    }
}

impl Default for OllamaProvider {
    fn default() -> Self {
        Self::new(OllamaConfig::default())
    }
}

impl OllamaProvider {
    async fn call_api(&self, request: &OllamaChatRequest) -> LLMResult<OllamaChatResponse> {
        let url = format!("{}/api/chat", self.config.base_url);

        let response = self.client.post(&url).json(request).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(LLMError::Api(format!("Ollama API error: {}", error_text)));
        }

        let ollama_response: OllamaChatResponse = response.json().await?;
        Ok(ollama_response)
    }
}

#[async_trait]
impl LLMProvider for OllamaProvider {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse> {
        let request = OllamaChatRequest {
            model: config.model.clone(),
            messages,
            stream: false,
            options: Some(OllamaOptions {
                temperature: Some(config.temperature),
                top_p: config.top_p,
                num_predict: config.max_tokens,
                stop: config.stop.clone(),
            }),
            tools: None,
        };

        let response = self.call_api(&request).await?;

        Ok(LLMResponse {
            content: response.message.content.to_string(),
            tool_calls: response.message.tool_calls,
            finish_reason: response.done_reason.unwrap_or_default(),
            usage: TokenUsage {
                prompt_tokens: response.prompt_eval_count.unwrap_or(0),
                completion_tokens: response.eval_count.unwrap_or(0),
                total_tokens: response.prompt_eval_count.unwrap_or(0)
                    + response.eval_count.unwrap_or(0),
            },
        })
    }

    async fn chat_completion_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<serde_json::Value>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse> {
        let tools = self.convert_tools(&tools)?;

        let request = OllamaChatRequest {
            model: config.model.clone(),
            messages,
            stream: false,
            options: Some(OllamaOptions {
                temperature: Some(config.temperature),
                top_p: config.top_p,
                num_predict: config.max_tokens,
                stop: config.stop.clone(),
            }),
            tools: Some(tools),
        };

        let response = self.call_api(&request).await?;

        Ok(LLMResponse {
            content: response.message.content.to_string(),
            tool_calls: response.message.tool_calls,
            finish_reason: response.done_reason.unwrap_or_default(),
            usage: TokenUsage {
                prompt_tokens: response.prompt_eval_count.unwrap_or(0),
                completion_tokens: response.eval_count.unwrap_or(0),
                total_tokens: response.prompt_eval_count.unwrap_or(0)
                    + response.eval_count.unwrap_or(0),
            },
        })
    }

    fn provider_name(&self) -> &str {
        "ollama"
    }

    async fn health_check(&self) -> LLMResult<bool> {
        let url = format!("{}/api/tags", self.config.base_url);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

// Ollama API types
#[derive(Debug, Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    num_predict: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct OllamaChatResponse {
    message: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    done_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_eval_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    eval_count: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ollama_health_check() {
        let provider = OllamaProvider::default();
        // This test will pass if Ollama is running locally
        let _ = provider.health_check().await;
    }

    #[tokio::test]
    #[ignore] // Requires Ollama running with llama3:70b
    async fn test_simple_chat() {
        let provider = OllamaProvider::default();
        let config = LLMConfig { model: "llama3:70b".to_string(), ..Default::default() };

        let messages =
            vec![Message::system("You are a helpful assistant."), Message::user("What is 2+2?")];

        let response = provider.chat_completion(messages, &config).await.unwrap();
        assert!(!response.content.is_empty());
        println!("Response: {}", response.content);
    }
}
