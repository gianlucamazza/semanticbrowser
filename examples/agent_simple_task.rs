/// Simple Agent Task Example
///
/// Demonstrates the LLM-based agent orchestrator using Ollama.
///
/// Prerequisites:
/// 1. Install Ollama: https://ollama.ai
/// 2. Pull a model: `ollama pull llama3:70b` (or llama3:8b for faster inference)
/// 3. Start Ollama: `ollama serve` (if not running as a service)
///
/// Run with:
/// ```bash
/// cargo run --example agent_simple_task
/// ```
use semantic_browser::llm::{
    AgentOrchestrator, AgentTask, LLMConfig, LLMProvider, OllamaConfig, OllamaProvider,
    ToolRegistry,
};
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup tracing
    let subscriber = FmtSubscriber::builder().with_max_level(Level::INFO).finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("🤖 Semantic Browser - Agent Example");
    info!("====================================\n");

    // 1. Create Ollama provider
    let ollama_config = OllamaConfig::default();
    let provider = Arc::new(OllamaProvider::new(ollama_config));

    // Check if Ollama is available
    info!("Checking Ollama connection...");
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

    // 2. Configure LLM
    let llm_config = LLMConfig {
        model: "llama3:8b".to_string(), // Use 8b for faster inference, 70b for better quality
        temperature: 0.7,
        max_tokens: Some(2048),
        ..Default::default()
    };

    // 3. Create tool registry with browser automation tools
    let tools = ToolRegistry::with_browser_tools();
    info!("Loaded {} tools", tools.get_tools_json().len());

    // 4. Create agent orchestrator
    let agent = AgentOrchestrator::new(provider, llm_config, tools);

    // 5. Define tasks
    let tasks = vec![
        AgentTask::new("Navigate to github.com and find the trending repositories")
            .with_max_iterations(5),
        AgentTask::new("Fill out a contact form with name 'John Doe' and email 'john@example.com'")
            .with_context("The form has fields: name, email, message")
            .with_max_iterations(3),
        AgentTask::new("Search for 'Rust async programming' on a search engine")
            .with_max_iterations(3),
    ];

    // 6. Execute tasks
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
    }

    info!("🎉 All tasks completed!\n");

    Ok(())
}
