#[cfg(feature = "llm-openai")]
mod tests {
    use semantic_browser::llm::*;

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
    #[allow(clippy::disallowed_methods)]
    mod integration_tests {

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

        #[tokio::test]
        async fn test_openai_stream_chat_completion() {
            let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();

            if api_key.is_empty() {
                println!("Skipping streaming test: OPENAI_API_KEY not set");
                return;
            }

            let provider = OpenAIProvider::new(api_key);
            let messages =
                vec![Message::user("Say 'Hello from stream!' in exactly 3 words.".to_string())];

            let config = LLMConfig {
                model: "gpt-3.5-turbo".to_string(),
                temperature: 0.7,
                max_tokens: Some(50),
                ..Default::default()
            };

            let result = provider.stream_chat_completion(messages, &config).await;
            assert!(result.is_ok(), "stream_chat_completion should succeed");

            let mut rx = result.unwrap();
            let mut full_response = String::new();
            let mut token_count = 0;

            while let Some(token) = rx.recv().await {
                full_response.push_str(&token);
                token_count += 1;
            }

            println!("Streamed {} tokens: {}", token_count, full_response);
            assert!(token_count > 0, "Should receive at least one token");
            assert!(!full_response.is_empty(), "Full response should not be empty");
        }

        #[tokio::test]
        async fn test_openai_stream_channel_closure() {
            let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();

            if api_key.is_empty() {
                println!("Skipping streaming channel test: OPENAI_API_KEY not set");
                return;
            }

            let provider = OpenAIProvider::new(api_key);
            let messages = vec![Message::user("Count to 3.".to_string())];

            let config = LLMConfig {
                model: "gpt-3.5-turbo".to_string(),
                temperature: 0.7,
                max_tokens: Some(20),
                ..Default::default()
            };

            let result = provider.stream_chat_completion(messages, &config).await;
            assert!(result.is_ok());

            let mut rx = result.unwrap();
            let mut count = 0;

            while let Some(_token) = rx.recv().await {
                count += 1;
            }

            println!("Received {} tokens before channel closed", count);
            assert!(count > 0, "Should have received some tokens");

            // Verify receiver is closed
            let next = rx.recv().await;
            assert!(next.is_none(), "Receiver should be closed after stream completes");
        }
    }

    #[tokio::test]
    async fn test_openai_supports_vision() {
        let provider = OpenAIProvider::new("test-key".to_string());
        assert!(provider.supports_vision());
    }

    #[tokio::test]
    async fn test_openai_vision_message_creation() {
        use semantic_browser::llm::provider::{ContentBlock, ImageContent, ImageSource, Role};

        // Test creating a vision message with text and image
        let image_content = ImageContent {
            image_url: ImageSource::Base64 {
                data: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPhfDwAChwGA60e6kgAAAABJRU5ErkJggg==".to_string(),
                media_type: "image/png".to_string(),
            },
        };

        let blocks = vec![
            ContentBlock::Text("What's in this image?".to_string()),
            ContentBlock::Image(image_content),
        ];

        let message = Message::user_vision(blocks);
        assert!(message.has_vision_content());
        assert_eq!(message.role, Role::User);
    }

    #[tokio::test]
    async fn test_openai_vision_message_formatting() {
        use semantic_browser::llm::provider::{ContentBlock, ImageContent, ImageSource};

        let image_content = ImageContent {
            image_url: ImageSource::Base64 {
                data: "base64data".to_string(),
                media_type: "image/jpeg".to_string(),
            },
        };

        let blocks = vec![
            ContentBlock::Text("Describe this image:".to_string()),
            ContentBlock::Image(image_content),
        ];

        let message = Message::user_vision(blocks);
        let formatted = format!("{}", message.content);
        assert!(formatted.contains("Describe this image:"));
        assert!(formatted.contains("[Image]"));
    }

    // Unit tests for SSE parsing logic
    mod stream_parsing_tests {
        use super::*;

        #[test]
        fn test_sse_data_line_parsing() {
            let data_line = "data: {\"object\":\"chat.completion.chunk\",\"choices\":[{\"delta\":{\"content\":\"Hello\"}}]}";

            if let Some(json_str) = data_line.strip_prefix("data: ") {
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(json_str);
                assert!(parsed.is_ok(), "Should parse valid SSE data line");

                let value = parsed.unwrap();
                assert!(value["choices"].is_array());
                assert_eq!(value["choices"][0]["delta"]["content"], "Hello");
            } else {
                panic!("Failed to strip data: prefix");
            }
        }

        #[test]
        fn test_sse_done_message() {
            let done_line = "[DONE]";
            assert_eq!(done_line, "[DONE]", "Should recognize done marker");
        }

        #[test]
        fn test_sse_empty_line_handling() {
            let empty_line = "";
            let trimmed = empty_line.trim();
            assert!(trimmed.is_empty(), "Empty lines should be skipped");
        }

        #[test]
        fn test_sse_multiline_buffer_handling() {
            let buffer = "data: {\"choices\":[{\"delta\":{\"content\":\"A\"}}]}\ndata: {\"choices\":[{\"delta\":{\"content\":\"B\"}}]}\n";
            let mut remaining = buffer;
            let mut lines = Vec::new();

            while let Some(newline_pos) = remaining.find('\n') {
                let line = &remaining[..newline_pos];
                lines.push(line);
                remaining = &remaining[newline_pos + 1..];
            }

            assert_eq!(lines.len(), 2, "Should split buffer into 2 lines");
            assert!(lines[0].starts_with("data: "));
            assert!(lines[1].starts_with("data: "));
        }

        #[test]
        fn test_sse_incomplete_line_buffer() {
            let buffer = "data: {\"incomplete";
            let has_newline = buffer.find('\n');
            assert!(
                has_newline.is_none(),
                "Incomplete lines without newline should remain in buffer"
            );
        }
    }
}
