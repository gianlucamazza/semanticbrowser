/// OpenAI Streaming Example
///
/// This example demonstrates real-time token streaming from the OpenAI API.
/// It shows how to use the stream_chat_completion method to receive tokens
/// as they are generated, enabling real-time user feedback.
///
/// Usage:
///   OPENAI_API_KEY=sk-... cargo run --features llm-openai --example streaming_example
use semantic_browser::llm::*;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎬 OpenAI Streaming Example");
    println!("===========================");
    println!();

    // Load API key
    #[allow(clippy::disallowed_methods)]
    let api_key = env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY must be set. Get one from https://platform.openai.com/api-keys");

    println!("✅ Loaded OpenAI API key");

    // Create provider
    let provider = OpenAIProvider::new(api_key);
    println!("✅ Created OpenAI provider");
    println!();

    // Health check
    println!("🔍 Performing health check...");
    match provider.health_check().await {
        Ok(_) => println!("✅ OpenAI API is accessible"),
        Err(e) => {
            eprintln!("❌ Health check failed: {}", e);
            return Err(e.into());
        }
    }
    println!();

    // Configure LLM
    let config = LLMConfig {
        model: "gpt-3.5-turbo".to_string(),
        temperature: 0.7,
        max_tokens: Some(200),
        ..Default::default()
    };

    // Example 1: Basic streaming
    println!("📌 Example 1: Basic Streaming");
    println!("{}", "-".repeat(40));
    stream_example(&provider, &config, "Explain quantum computing in 3 sentences.").await?;
    println!();

    // Example 2: Streaming with longer response
    println!("📌 Example 2: Streaming Longer Response");
    println!("{}", "-".repeat(40));
    stream_example(&provider, &config, "List 5 benefits of learning Rust programming language.")
        .await?;
    println!();

    // Example 3: Real-time feedback simulation
    println!("📌 Example 3: Real-time Token Counter");
    println!("{}", "-".repeat(40));
    let messages = vec![Message::user("Write a short poem about the sea.".to_string())];

    match provider.stream_chat_completion(messages, &config).await {
        Ok(mut rx) => {
            let mut full_response = String::new();
            let mut token_count = 0;
            let mut char_count = 0;

            while let Some(token) = rx.recv().await {
                full_response.push_str(&token);
                token_count += 1;
                char_count += token.len();

                // Show real-time token counter every 5 tokens
                if token_count % 5 == 0 {
                    print!("\r🔄 Tokens: {}, Characters: {}   ", token_count, char_count);
                }
            }

            println!("\r✅ Stream complete!                      ");
            println!();
            println!("📊 Statistics:");
            println!("  - Total tokens: {}", token_count);
            println!("  - Total characters: {}", char_count);
            println!("  - Average token size: {:.2} chars", char_count as f64 / token_count as f64);
            println!();
            println!("📝 Response:\n{}", full_response);
        }
        Err(e) => {
            eprintln!("❌ Streaming failed: {}", e);
            return Err(e.into());
        }
    }

    println!();
    println!("🎉 Streaming example completed!");
    Ok(())
}

/// Helper function to demonstrate streaming
async fn stream_example(
    provider: &OpenAIProvider,
    config: &LLMConfig,
    prompt: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let messages = vec![Message::user(prompt.to_string())];

    println!("\n🤖 Prompt: {}", prompt);
    println!("💬 Response: ",);

    match provider.stream_chat_completion(messages, config).await {
        Ok(mut rx) => {
            let mut full_response = String::new();
            let mut token_count = 0;

            while let Some(token) = rx.recv().await {
                print!("{}", token);
                full_response.push_str(&token);
                token_count += 1;
                // Flush to show real-time output
                use std::io::Write;
                std::io::stdout().flush()?;
            }

            println!();
            println!("✅ Stream complete ({} tokens)", token_count);
        }
        Err(e) => {
            eprintln!("❌ Streaming failed: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
