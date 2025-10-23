//! Vision Models Example
//!
//! This example demonstrates how to use vision models (GPT-4V, Claude 3 Vision)
//! to analyze images and web content with the Semantic Browser.
//!
//! Run with:
//! ```bash
//! # For OpenAI GPT-4V
//! OPENAI_API_KEY=your_key cargo run --example vision_example -- --provider openai
//!
//! # For Anthropic Claude 3 Vision
//! ANTHROPIC_API_KEY=your_key cargo run --example vision_example -- --provider anthropic
//!
//! # With a local image file
//! cargo run --example vision_example -- --provider openai --image /path/to/image.jpg
//!
//! # With a web screenshot
//! cargo run --example vision_example -- --provider openai --url https://example.com
//! ```

use base64::Engine;
use semantic_browser::llm::provider::{
    ContentBlock, ImageContent, ImageSource, LLMConfig, LLMProvider, Message,
};
use std::env;
use std::path::Path;

struct VisionExample {
    provider: Box<dyn LLMProvider>,
}

impl VisionExample {
    async fn new(provider_type: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let provider: Box<dyn LLMProvider> = match provider_type {
            "openai" => {
                #[cfg(feature = "llm-openai")]
                {
                    #[allow(clippy::disallowed_methods)]
                    let api_key = env::var("OPENAI_API_KEY")
                        .map_err(|_| "OPENAI_API_KEY environment variable not set")?;
                    Box::new(semantic_browser::llm::OpenAIProvider::new(api_key))
                }
                #[cfg(not(feature = "llm-openai"))]
                {
                    return Err(
                        "OpenAI feature not enabled. Compile with --features llm-openai".into()
                    );
                }
            }
            "anthropic" => {
                #[cfg(feature = "llm-anthropic")]
                {
                    #[allow(clippy::disallowed_methods)]
                    let api_key = env::var("ANTHROPIC_API_KEY")
                        .map_err(|_| "ANTHROPIC_API_KEY environment variable not set")?;
                    Box::new(semantic_browser::llm::AnthropicProvider::new(api_key))
                }
                #[cfg(not(feature = "llm-anthropic"))]
                {
                    return Err(
                        "Anthropic feature not enabled. Compile with --features llm-anthropic"
                            .into(),
                    );
                }
            }
            _ => return Err(format!("Unsupported provider: {}", provider_type).into()),
        };

        // Check if provider supports vision
        #[allow(unreachable_code)]
        if !provider.supports_vision() {
            return Err(format!("Provider {} does not support vision", provider_type).into());
        }

        Ok(Self { provider })
    }

    /// Analyze an image from a URL
    async fn analyze_image_url(
        &self,
        image_url: &str,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        println!("🔍 Analyzing image from URL: {}", image_url);

        let message = Message::user_with_image(prompt.to_string(), image_url.to_string());

        let config = LLMConfig {
            model: self.get_vision_model(),
            temperature: 0.7,
            max_tokens: Some(500),
            ..Default::default()
        };

        let response = self.provider.vision_chat_completion(vec![message], &config).await?;
        Ok(response.content)
    }

    /// Analyze a local image file (base64 encoded)
    async fn analyze_local_image(
        &self,
        image_path: &str,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        println!("📁 Analyzing local image: {}", image_path);

        // Read and encode image as base64
        let image_data = std::fs::read(image_path)?;
        let base64_data = base64::engine::general_purpose::STANDARD.encode(&image_data);

        // Detect MIME type from file extension
        let mime_type = match Path::new(image_path).extension().and_then(|ext| ext.to_str()) {
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("gif") => "image/gif",
            Some("webp") => "image/webp",
            _ => "image/jpeg", // default fallback
        };

        let message = Message::user_vision(vec![
            ContentBlock::Text(prompt.to_string()),
            ContentBlock::Image(ImageContent {
                image_url: ImageSource::Base64 {
                    media_type: mime_type.to_string(),
                    data: base64_data,
                },
            }),
        ]);

        let config = LLMConfig {
            model: self.get_vision_model(),
            temperature: 0.7,
            max_tokens: Some(500),
            ..Default::default()
        };

        let response = self.provider.vision_chat_completion(vec![message], &config).await?;
        Ok(response.content)
    }

    /// Analyze multiple images in one request
    async fn analyze_multiple_images(
        &self,
        image_urls: Vec<String>,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        println!("📸 Analyzing {} images", image_urls.len());

        let mut content_blocks = vec![ContentBlock::Text(prompt.to_string())];

        for url in image_urls {
            content_blocks
                .push(ContentBlock::Image(ImageContent { image_url: ImageSource::Url(url) }));
        }

        let message = Message::user_vision(content_blocks);

        let config = LLMConfig {
            model: self.get_vision_model(),
            temperature: 0.7,
            max_tokens: Some(1000),
            ..Default::default()
        };

        let response = self.provider.vision_chat_completion(vec![message], &config).await?;
        Ok(response.content)
    }

    /// Get the appropriate vision model for the provider
    fn get_vision_model(&self) -> String {
        match self.provider.provider_name() {
            "openai" => "gpt-4o".to_string(), // GPT-4o supports vision
            "anthropic" => "claude-3-opus-20240229".to_string(), // Claude 3 Opus supports vision
            _ => "gpt-4o".to_string(),        // fallback
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        eprintln!(
            "Usage: {} --provider <openai|anthropic> [--image <path>] [--url <url>]",
            args[0]
        );
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} --provider openai", args[0]);
        eprintln!("  {} --provider anthropic --image /path/to/image.jpg", args[0]);
        eprintln!("  {} --provider openai --url https://example.com/image.jpg", args[0]);
        std::process::exit(1);
    }

    let mut provider_type = None;
    let mut image_path = None;
    let mut image_url = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--provider" => {
                if i + 1 < args.len() {
                    provider_type = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --provider requires a value");
                    std::process::exit(1);
                }
            }
            "--image" => {
                if i + 1 < args.len() {
                    image_path = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --image requires a path");
                    std::process::exit(1);
                }
            }
            "--url" => {
                if i + 1 < args.len() {
                    image_url = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --url requires a URL");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                std::process::exit(1);
            }
        }
    }

    let provider_type = provider_type.ok_or("Provider type is required (--provider)")?;
    let example = VisionExample::new(&provider_type).await?;

    println!("🤖 Using {} vision models", provider_type.to_uppercase());
    println!("📋 Model: {}", example.get_vision_model());
    println!();

    // Example 1: Analyze a famous image
    if let Some(url) = image_url {
        let prompt =
            "Describe this image in detail. What do you see? What colors and objects are present?";
        let result = example.analyze_image_url(&url, prompt).await?;
        println!("🖼️  Image Analysis Result:");
        println!("{}", result);
    }
    // Example 2: Analyze local image
    else if let Some(path) = image_path {
        if !Path::new(&path).exists() {
            eprintln!("Error: Image file does not exist: {}", path);
            std::process::exit(1);
        }

        let prompt = "What do you see in this image? Please provide a detailed description.";
        let result = example.analyze_local_image(&path, prompt).await?;
        println!("📁 Local Image Analysis Result:");
        println!("{}", result);
    }
    // Example 3: Default demo with sample images
    else {
        println!("🌟 Running vision model demo with sample images...");

        // Example with a well-known image URL (this would need to be a real accessible URL)
        let sample_urls = vec![
            "https://images.unsplash.com/photo-1441974231531-c6227db76b6e?w=400".to_string(), // Sample nature image
        ];

        let prompt = "This is a landscape photograph. Describe the scene, the lighting, colors, and mood. What time of day does it appear to be?";
        let result = example.analyze_multiple_images(sample_urls, prompt).await?;
        println!("🖼️  Sample Image Analysis:");
        println!("{}", result);
    }

    println!();
    println!("✅ Vision analysis completed successfully!");
    Ok(())
}
