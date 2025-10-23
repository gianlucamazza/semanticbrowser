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

    #[tokio::test]
    async fn test_anthropic_supports_vision() {
        let provider = AnthropicProvider::new("test-key".to_string());
        assert!(provider.supports_vision());
    }

    #[tokio::test]
    async fn test_anthropic_vision_message_creation() {
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
    async fn test_anthropic_vision_message_formatting() {
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

    // Integration tests (require API key)
    #[allow(clippy::disallowed_methods)]
    mod integration_tests {
        use super::*;

        #[tokio::test]
        async fn test_anthropic_stream_chat_completion() {
            let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();

            if api_key.is_empty() {
                println!("Skipping streaming test: ANTHROPIC_API_KEY not set");
                return;
            }

            let provider = AnthropicProvider::new(api_key);
            let messages = vec![Message::user(
                "Say 'Hello from Claude stream!' in exactly 3 words.".to_string(),
            )];

            let config = LLMConfig {
                model: "claude-3-haiku-20240307".to_string(),
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
        async fn test_anthropic_stream_channel_closure() {
            let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();

            if api_key.is_empty() {
                println!("Skipping streaming channel test: ANTHROPIC_API_KEY not set");
                return;
            }

            let provider = AnthropicProvider::new(api_key);
            let messages = vec![Message::user("Count to 3.".to_string())];

            let config = LLMConfig {
                model: "claude-3-haiku-20240307".to_string(),
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

            let next = rx.recv().await;
            assert!(next.is_none(), "Receiver should be closed after stream completes");
        }
    }

    // Unit tests for SSE parsing logic
    mod stream_parsing_tests {

        #[test]
        fn test_anthropic_sse_content_block_delta() {
            let data_line = "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}";

            if let Some(json_str) = data_line.strip_prefix("data: ") {
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(json_str);
                assert!(parsed.is_ok(), "Should parse valid Anthropic SSE data line");

                let value = parsed.unwrap();
                assert_eq!(value["type"], "content_block_delta");
                assert_eq!(value["delta"]["text"], "Hello");
            } else {
                panic!("Failed to strip data: prefix");
            }
        }

        #[test]
        fn test_anthropic_sse_message_stop() {
            let data_line = "data: {\"type\":\"message_stop\"}";

            if let Some(json_str) = data_line.strip_prefix("data: ") {
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(json_str);
                assert!(parsed.is_ok(), "Should parse message_stop event");

                let value = parsed.unwrap();
                assert_eq!(value["type"], "message_stop");
            } else {
                panic!("Failed to strip data: prefix");
            }
        }

        #[test]
        fn test_anthropic_sse_event_types() {
            let event_types = vec![
                "content_block_start",
                "content_block_delta",
                "content_block_stop",
                "message_start",
                "message_stop",
                "message_delta",
            ];

            for event_type in event_types {
                let data = format!("{{\"type\":\"{}\"}}", event_type);
                let parsed: Result<serde_json::Value, _> = serde_json::from_str(&data);
                assert!(parsed.is_ok(), "Should parse event type: {}", event_type);
                assert_eq!(parsed.unwrap()["type"], event_type);
            }
        }

        #[test]
        fn test_anthropic_sse_multiline_buffer() {
            let buffer = "data: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"A\"}}\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"B\"}}\n";
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
        fn test_anthropic_sse_incomplete_line() {
            let buffer = "data: {\"type\":\"content_block";
            let has_newline = buffer.find('\n');
            assert!(
                has_newline.is_none(),
                "Incomplete lines without newline should remain in buffer"
            );
        }
    }
}
