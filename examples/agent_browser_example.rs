use semantic_browser::llm::*;
use std::{env, sync::Arc};

#[cfg(feature = "browser-automation")]
use semantic_browser::llm::browser_executor::BrowserExecutor;

#[cfg(feature = "browser-automation")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🤖 Browser Agent Example");
    println!("========================");

    // Load API key - using OpenAI for this example
    #[allow(clippy::disallowed_methods)]
    let api_key = env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY must be set. Get one from https://platform.openai.com/api-keys");

    println!("✅ Loaded OpenAI API key");

    // Create LLM provider
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

    // Create browser executor
    println!("🌐 Setting up browser automation...");
    let (browser_instance, _handler) =
        chromiumoxide::Browser::launch(chromiumoxide::BrowserConfig::builder().build()?)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    let page = browser_instance
        .new_page("about:blank")
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    let browser = BrowserExecutor::new(std::sync::Arc::new(page)).await?;
    println!("✅ Browser automation ready");

    // Create agent configuration
    let config = LLMConfig {
        model: "gpt-3.5-turbo".to_string(),
        temperature: 0.7,
        max_tokens: Some(1000),
        ..Default::default()
    };

    let tools = ToolRegistry::with_browser_tools();
    let agent = AgentOrchestrator::new(provider, config, tools).with_browser(Arc::new(browser));

    println!("✅ Created agent orchestrator with browser integration");

    // Execute a browser automation task
    let task = AgentTask::new(
        "Navigate to http://httpbin.org/html and extract the main heading text, then navigate to http://httpbin.org/json and extract the 'url' field from the JSON response."
    ).with_max_iterations(10);

    println!("🚀 Executing browser automation task: {}", task.goal);
    println!("⏳ This may take a moment as it involves real browser automation...");

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

    println!("🎉 Browser automation example completed!");
    Ok(())
}

#[cfg(not(feature = "browser-automation"))]
fn main() {
    println!("This example requires the 'browser-automation' feature.");
    println!("Run with: cargo run --features browser-automation --example agent_browser_example");
}
