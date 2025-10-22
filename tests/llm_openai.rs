#[cfg(feature = "llm-openai")]
mod tests {
    use semantic_browser::llm::*;
    use serde_json;

    #[tokio::test]
    async fn test_openai_provider_creation() {
        let api_key = "test-key".to_string();
        let provider = OpenAIProvider::new(api_key.clone());
        assert_eq!(provider.api_key, api_key);
        assert_eq!(provider.base_url, "https://api.openai.com/v1");
    }

    #[tokio::test]
    async fn test_openai_provider_with_custom_url() {
        let api_key = "test-key".to_string();
        let custom_url = "https://custom.openai.com/v1".to_string();
        let provider = OpenAIProvider::new(api_key).with_base_url(custom_url.clone());
        assert_eq!(provider.base_url, custom_url);
    }

    #[tokio::test]
    async fn test_openai_tools_conversion() {
        let provider = OpenAIProvider::new("test-key".to_string());

        let tools_json: Vec<serde_json::Value> = serde_json::from_str(
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

        let tools = provider.convert_tools(&tools_json).unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].function.name, "navigate_to");
        assert_eq!(tools[0].function.description, "Navigate to a URL");
    }

    // Integration tests (require API key)
    // #[cfg(feature = "integration-tests")]
    mod integration_tests {
        use super::*;

        #[tokio::test]
        async fn test_openai_health_check() {
            let api_key = std::env::var("OPENAI_API_KEY").unwrap();
            let provider = OpenAIProvider::new(api_key);

            let result = provider.health_check().await;
            assert!(result.is_ok());
        }

        #[tokio::test]
        async fn test_openai_chat_completion() {
            let api_key = std::env::var("OPENAI_API_KEY").unwrap();
            let provider = OpenAIProvider::new(api_key);

            let messages = vec![Message::user("Hello, how are you?".to_string())];

            let config = LLMConfig {
                model: "gpt-3.5-turbo".to_string(),
                temperature: 0.7,
                max_tokens: Some(100),
                ..Default::default()
            };

            let result = provider.chat_completion(messages, &config).await;
            assert!(result.is_ok());
            let response = result.unwrap();
            assert!(!response.content.is_empty());
            println!("Response: {:?}", response);
        }

        #[tokio::test]
        async fn test_openai_chat_completion_with_tools() {
            let api_key = std::env::var("OPENAI_API_KEY").unwrap();
            let provider = OpenAIProvider::new(api_key);

            let messages = vec![Message::user("Navigate to example.com".to_string())];

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

            let config = LLMConfig {
                model: "gpt-3.5-turbo".to_string(),
                temperature: 0.7,
                max_tokens: Some(200),
                ..Default::default()
            };

            let result = provider.chat_completion_with_tools(messages, tools, &config).await;
            assert!(result.is_ok());
            let response = result.unwrap();
            println!("Response with tools: {:?}", response);
        }
    }
}
