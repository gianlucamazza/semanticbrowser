#[cfg(feature = "llm-openai")]
mod implementation {
    use semantic_browser::llm::*;
    use std::{env, sync::Arc};

    #[tokio::main]
    pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
        println!("🤖 OpenAI Agent Example");
        println!("========================");

        // Load API key
        #[allow(clippy::disallowed_methods)]
        let api_key = env::var("OPENAI_API_KEY").expect(
            "OPENAI_API_KEY must be set. Get one from https://platform.openai.com/api-keys",
        );

        println!("✅ Loaded OpenAI API key");

        // Create provider
        let provider = Arc::new(OpenAIProvider::new(api_key));
        println!("✅ Created OpenAI provider");

        // Health check
        println!("🔍 Performing health check...");
        match provider.health_check().await {
            Ok(_) => println!("✅ OpenAI API is accessible"),
            Err(e) => {
                eprintln!("❌ Health check failed: {}", e);
                return Err(e.into());
            }
        }

        // Create agent configuration
        let config = LLMConfig {
            model: "gpt-3.5-turbo".to_string(), // Use GPT-3.5 for cost efficiency in examples
            temperature: 0.7,
            max_tokens: Some(1000),
            ..Default::default()
        };

        let tools = ToolRegistry::with_browser_tools();
        let agent = AgentOrchestrator::new(provider, config, tools);

        println!("✅ Created agent orchestrator");

        // Execute a simple task
        let task = AgentTask::new(
            "Navigate to https://httpbin.org and extract the current IP address from the JSON response",
        );

        println!("🚀 Executing task: {}", task.goal);
        println!("⏳ This may take a moment...");

        match agent.execute(task).await {
            Ok(response) => {
                println!("✅ Task completed successfully!");
                println!("📊 Iterations: {}", response.iterations);
                println!("📝 Result: {}", response.result);

                if let Some(error) = &response.error {
                    println!("⚠️  Warning: {}", error);
                }
            }
            Err(e) => {
                eprintln!("❌ Task failed: {}", e);
                return Err(e.into());
            }
        }

        println!("🎉 Example completed!");
        Ok(())
    }
}

#[cfg(feature = "llm-openai")]
use implementation::main;

#[cfg(not(feature = "llm-openai"))]
fn main() {
    eprintln!("❌ This example requires the 'llm-openai' feature.");
    eprintln!("Run with: cargo run --features llm-openai --example agent_openai_example");
    std::process::exit(1);
}
