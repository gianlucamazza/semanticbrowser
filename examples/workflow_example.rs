//! Example demonstrating the Web Workflow Engine
//!
//! This example shows how to create and execute multi-step web automation workflows
//! with conditional branching, loops, and error handling.

use semantic_browser::llm::provider::{FunctionCall, ToolCall};
use semantic_browser::llm::tools::ToolRegistry;
use semantic_browser::llm::workflow::{Condition, WebWorkflow, WorkflowExecutor, WorkflowStep};

#[cfg(feature = "browser-automation")]
use futures::StreamExt;
#[cfg(feature = "browser-automation")]
use semantic_browser::llm::browser_executor::BrowserExecutor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 Web Workflow Engine Example");
    println!("================================");

    // Create a simple workflow that demonstrates various step types
    let workflow = create_demo_workflow();

    println!("📋 Workflow: {}", workflow.name);
    println!("📝 Description: {}", workflow.description);
    println!("🔢 Steps: {}", workflow.steps.len());

    // Create workflow executor
    let tool_registry = ToolRegistry::with_browser_tools();
    let mut executor = WorkflowExecutor::new(tool_registry);

    // Add browser support if available
    #[cfg(feature = "browser-automation")]
    {
        println!("🌐 Initializing browser...");
        let (browser, mut handler) = chromiumoxide::Browser::launch(Default::default())
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        // Spawn browser handler
        tokio::spawn(async move {
            while let Some(_event) = handler.next().await {
                // Handle browser events if needed
            }
        });

        let page = browser
            .new_page("about:blank")
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        let browser_executor = BrowserExecutor::new(std::sync::Arc::new(page)).await?;
        executor = executor.with_browser(std::sync::Arc::new(browser_executor));
        println!("✅ Browser integration enabled");
    }

    // Execute the workflow
    println!("▶️  Executing workflow...");
    match executor.execute_workflow(&workflow).await {
        Ok(state) => {
            println!("✅ Workflow completed!");
            println!("📊 Status: {:?}", state.status);
            println!("⏱️  Duration: {:?}", state.last_update - state.start_time);
            println!("📈 Steps executed: {}", state.step_results.len());

            // Show step results
            println!("\n📋 Step Results:");
            for (i, result) in state.step_results.iter().enumerate() {
                println!(
                    "  {}. {} - {} ({:.2}s)",
                    i + 1,
                    result.step_name,
                    if result.success { "✅" } else { "❌" },
                    result.execution_time_ms as f64 / 1000.0
                );
                if !result.success {
                    if let Some(error) = &result.error {
                        println!("     Error: {}", error);
                    }
                }
            }

            // Show final variables
            if !state.variables.is_empty() {
                println!("\n🔧 Final Variables:");
                for (key, value) in &state.variables {
                    println!("  {} = {}", key, value);
                }
            }
        }
        Err(e) => {
            println!("❌ Workflow failed: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}

/// Create a demonstration workflow
fn create_demo_workflow() -> WebWorkflow {
    WebWorkflow::builder("Demo Web Automation Workflow")
        .description("A comprehensive example showing various workflow step types")
        .variable("base_url", serde_json::json!("https://httpbin.org"))
        .variable("test_data", serde_json::json!(["item1", "item2", "item3"]))
        .tool_call(
            "navigate_to_home",
            ToolCall {
                id: "nav1".to_string(),
                tool_type: "function".to_string(),
                function: FunctionCall {
                    name: "navigate_to".to_string(),
                    arguments: r#"{"url": "https://httpbin.org"}"#.to_string(),
                },
            },
        )
        .set_variable("page_loaded", "current_url", serde_json::json!("https://httpbin.org"))
        .conditional_branch(
            "check_navigation",
            Condition::Exists { variable: "page_loaded".to_string() },
            vec![WorkflowStep::SetVariable {
                name: "success_flag".to_string(),
                variable: "navigation_success".to_string(),
                value: serde_json::json!(true),
            }],
            vec![WorkflowStep::SetVariable {
                name: "error_flag".to_string(),
                variable: "navigation_success".to_string(),
                value: serde_json::json!(false),
            }],
        )
        .wait("brief_pause", 1000)
        .tool_call(
            "get_page_title",
            ToolCall {
                id: "title1".to_string(),
                tool_type: "function".to_string(),
                function: FunctionCall {
                    name: "get_page_title".to_string(),
                    arguments: "{}".to_string(),
                },
            },
        )
        .step(WorkflowStep::Loop {
            name: "process_test_data".to_string(),
            variable: "item".to_string(),
            items: vec![
                serde_json::json!("alpha"),
                serde_json::json!("beta"),
                serde_json::json!("gamma"),
            ],
            loop_steps: vec![WorkflowStep::SetVariable {
                name: "processed_item".to_string(),
                variable: "current_item".to_string(),
                value: serde_json::json!("item_processed"),
            }],
            max_iterations: Some(10),
        })
        .build()
}
