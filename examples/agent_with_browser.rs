#[cfg(feature = "browser-automation")]
use chromiumoxide::browser::Browser;
use futures_util::stream::StreamExt;
#[cfg(feature = "browser-automation")]
use semantic_browser::llm::BrowserExecutor;
/// Agent with Real Browser Integration Example
///
/// Demonstrates the LLM agent using a real browser to perform web automation.
///
/// Prerequisites:
/// 1. Chrome/Chromium installed
/// 2. Ollama running with a model
/// 3. cargo run --features browser-automation --example agent_with_browser
///
/// This example shows:
/// - Agent navigating to real URLs
/// - Agent filling real forms
/// - Agent extracting real data from pages
use semantic_browser::llm::{
    AgentOrchestrator, AgentTask, LLMConfig, LLMProvider, OllamaConfig, OllamaProvider,
    ToolRegistry,
};
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Setup tracing
    let subscriber = FmtSubscriber::builder().with_max_level(Level::INFO).finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🤖 Semantic Browser - Agent with Real Browser");
    info!("=============================================\n");

    #[cfg(not(feature = "browser-automation"))]
    {
        eprintln!("❌ This example requires the 'browser-automation' feature!");
        eprintln!("\nRun with:");
        eprintln!("  cargo run --features browser-automation --example agent_with_browser");
        return Ok(());
    }

    #[cfg(feature = "browser-automation")]
    {
        // 1. Launch browser
        info!("🌐 Launching browser...");
        let (browser, mut handler) = Browser::launch(
            chromiumoxide::BrowserConfig::builder()
                .build()
                .map_err(|e| format!("Failed to build browser config: {}", e))?,
        )
        .await?;
        let page: Arc<chromiumoxide::Page> = Arc::new(browser.new_page("about:blank").await?);
        info!("✅ Browser launched");

        // Spawn handler task
        #[allow(clippy::redundant_pattern_matching)]
        tokio::spawn(async move {
            while let Some(_) = handler.next().await {
                // Process handler events
            }
        });

        // 2. Create browser executor
        info!("🔧 Creating browser executor...");
        let browser_exec = Arc::new(BrowserExecutor::new(page.clone()).await?);
        info!("✅ Browser executor ready");

        // 3. Create Ollama provider
        let ollama_config = OllamaConfig::default();
        let provider = Arc::new(OllamaProvider::new(ollama_config));

        // Check Ollama
        info!("🔍 Checking Ollama connection...");
        match LLMProvider::health_check(provider.as_ref()).await {
            Ok(true) => info!("✅ Ollama is running"),
            Ok(false) | Err(_) => {
                eprintln!("❌ Ollama is not running!");
                eprintln!("\nPlease start Ollama:");
                eprintln!("  1. Install: https://ollama.ai");
                eprintln!("  2. Pull model: ollama pull llama3:8b");
                eprintln!("  3. Start: ollama serve");
                return Ok(());
            }
        }

        // 4. Configure LLM
        let llm_config = LLMConfig {
            model: "llama3:8b".to_string(),
            temperature: 0.7,
            max_tokens: Some(2048),
            ..Default::default()
        };

        // 5. Create tool registry
        let tools = ToolRegistry::with_browser_tools();
        info!("🛠️  Loaded {} tools", tools.get_tools_json().len());

        // 6. Create agent WITH browser
        let agent = AgentOrchestrator::new(provider, llm_config, tools).with_browser(browser_exec);

        info!("🤖 Agent ready with REAL browser integration!\n");

        // 7. Define real-world tasks
        let tasks = [
            AgentTask::new("Navigate to example.com and get the page title").with_max_iterations(3),
            AgentTask::new("Go to httpbin.org/forms/post and extract the form field names")
                .with_max_iterations(5),
        ];

        // 8. Execute tasks
        for (idx, task) in tasks.iter().enumerate() {
            info!("\n📋 Task {}: {}", idx + 1, task.goal);
            info!("─────────────────────────────────────────────");

            match agent.execute(task.clone()).await {
                Ok(response) => {
                    if response.success {
                        info!("✅ Success! (iterations: {})", response.iterations);
                        info!("Result: {}", response.result);
                    } else {
                        info!("❌ Failed after {} iterations", response.iterations);
                        if let Some(error) = response.error {
                            info!("Error: {}", error);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Error executing task: {}", e);
                }
            }

            info!("");

            // Small delay between tasks
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }

        info!("🎉 All tasks completed with REAL browser!\n");

        // Keep browser open for a moment to see results
        info!("💤 Keeping browser open for 5 seconds...");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }

    Ok(())
}
