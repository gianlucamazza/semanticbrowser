#[cfg(feature = "llm-anthropic")]
mod tests {
    use semantic_browser::llm::*;

    #[tokio::test]
    async fn test_anthropic_provider_creation() {
        let api_key = "test-key".to_string();
        let provider = AnthropicProvider::new(api_key.clone());
        assert_eq!(provider.api_key, api_key);
        assert_eq!(provider.base_url, "https://api.anthropic.com/v1");
    }

    #[tokio::test]
    async fn test_anthropic_provider_with_custom_url() {
        let api_key = "test-key".to_string();
        let custom_url = "https://custom.anthropic.com/v1".to_string();
        let provider = AnthropicProvider::new(api_key).with_base_url(custom_url.clone());
        assert_eq!(provider.base_url, custom_url);
    }

    #[tokio::test]
    async fn test_anthropic_tools_conversion() {
        let provider = AnthropicProvider::new("test-key".to_string());

        let tools: Vec<serde_json::Value> = serde_json::from_str(
            r#"[
            {
                "function": {
                    "name": "navigate_to",
                    "description": "Navigate to a URL",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "url": {
                                "type": "string",
                                "description": "The URL to navigate to"
                            }
                        },
                        "required": ["url"]
                    }
                }
            }
        ]"#,
        )
        .unwrap();

        let anthropic_tools = provider.convert_tools(&tools).unwrap();
        assert_eq!(anthropic_tools.len(), 1);
        assert_eq!(anthropic_tools[0].name, "navigate_to");
        assert_eq!(anthropic_tools[0].description, "Navigate to a URL");
    }

    // Integration tests (require API key)
    // #[cfg(feature = "integration-tests")]
    mod integration_tests {
        use super::*;

        // #[tokio::test]
        // async fn test_anthropic_health_check() {
        //     let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap();
        //     let provider = AnthropicProvider::new(api_key);

        //     let result = provider.health_check().await;
        //     assert!(result.is_ok());
        // }

        // #[tokio::test]
        // async fn test_anthropic_chat_completion() {
        //     let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap();
        //     let provider = AnthropicProvider::new(api_key);

        //     let messages = vec![Message::user("Hello, how are you?".to_string())];

        //     let config = LLMConfig {
        //         model: "claude-3-haiku-20240307".to_string(),
        //         temperature: 0.7,
        //         max_tokens: Some(100),
        //         ..Default::default()
        //     };

        //     let result = provider.chat_completion(messages, &config).await;
        //     assert!(result.is_ok());
        //     let response = result.unwrap();
        //     assert!(!response.content.is_empty());
        //     println!("Response: {:?}", response);
        // }
    }
}
