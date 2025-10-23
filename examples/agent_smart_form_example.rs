//! Smart Form Integration Example
//!
//! This example demonstrates the intelligent form filling and analysis capabilities
//! of the LLM agents. It shows how agents can:
//! - Analyze forms on web pages
//! - Auto-fill entire forms using semantic matching
//! - Submit forms intelligently
//! - Extract form field information

use futures::StreamExt;
use semantic_browser::llm::browser_executor::BrowserExecutor;
use semantic_browser::llm::*;
use std::env;
use std::sync::Arc;

#[cfg(feature = "browser-automation")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🤖 Smart Form Integration Example");
    println!("=================================");

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
    let (browser_instance, mut handler) =
        chromiumoxide::Browser::launch(chromiumoxide::BrowserConfig::builder().build().unwrap())
            .await?;
    #[allow(clippy::redundant_pattern_matching)]
    tokio::spawn(async move { while let Some(_) = handler.next().await {} });
    let page = browser_instance.new_page("about:blank").await?;
    let browser = BrowserExecutor::new(Arc::new(page)).await?;
    println!("✅ Browser automation ready");

    // Create agent configuration
    let config = LLMConfig {
        model: "gpt-4".to_string(),
        temperature: 0.1,
        max_tokens: Some(1000),
        ..Default::default()
    };

    let tools = ToolRegistry::with_browser_tools();
    let agent = AgentOrchestrator::new(provider, config, tools).with_browser(Arc::new(browser));

    println!("✅ Created agent orchestrator with smart form integration");

    // Navigate to a test page with forms (using a simple HTML page for demo)
    let task = AgentTask::new(
        r#"Navigate to this test form page: data:text/html,<html><body><h1>Smart Form Demo</h1><form action='/submit' method='post'><label for='username'>Username:</label><input type='text' id='username' name='username' required><br><label for='email'>Email:</label><input type='email' id='email' name='email' required><br><label for='password'>Password:</label><input type='password' id='password' name='password' required><br><button type='submit'>Register</button></form></body></html>"#
    ).with_max_iterations(5);

    println!("🚀 Setting up test form page...");
    match agent.execute(task).await {
        Ok(response) => {
            println!("✅ Navigation completed!");
            println!("📝 Result: {}", response.result);
        }
        Err(e) => {
            eprintln!("❌ Navigation failed: {}", e);
            return Err(e.into());
        }
    }

    // Step 1: Analyze forms on the page
    println!("\n📋 Step 1: Analyzing forms on the page...");
    let analysis_task =
        AgentTask::new("Analyze all forms on the current page and tell me what you find.")
            .with_max_iterations(5);

    match agent.execute(analysis_task).await {
        Ok(response) => {
            println!("✅ Form Analysis completed!");
            println!("📝 Result: {}", response.result);
        }
        Err(e) => {
            eprintln!("❌ Form analysis failed: {}", e);
            return Err(e.into());
        }
    }

    // Step 2: Get detailed form field information
    println!("\n📝 Step 2: Getting detailed form field information...");
    let fields_task =
        AgentTask::new("List all form fields with their types, labels, and other metadata.")
            .with_max_iterations(5);

    match agent.execute(fields_task).await {
        Ok(response) => {
            println!("✅ Form fields retrieved!");
            println!("📝 Result: {}", response.result);
        }
        Err(e) => {
            eprintln!("❌ Getting form fields failed: {}", e);
            return Err(e.into());
        }
    }

    // Step 3: Auto-fill the registration form
    println!("\n✍️  Step 3: Auto-filling the registration form...");
    let fill_task = AgentTask::new(
        r#"Fill out the registration form with these details:
        - Username: johndoe123
        - Email: john.doe@example.com
        - Password: securePass123!

        Use the auto_fill_form tool to fill all fields at once."#,
    )
    .with_max_iterations(5);

    match agent.execute(fill_task).await {
        Ok(response) => {
            println!("✅ Auto-fill completed!");
            println!("📝 Result: {}", response.result);
        }
        Err(e) => {
            eprintln!("❌ Auto-fill failed: {}", e);
            return Err(e.into());
        }
    }

    // Step 4: Submit the form
    println!("\n🚀 Step 4: Submitting the form...");
    let submit_task = AgentTask::new("Submit the registration form.").with_max_iterations(5);

    match agent.execute(submit_task).await {
        Ok(response) => {
            println!("✅ Form submission completed!");
            println!("📝 Result: {}", response.result);
        }
        Err(e) => {
            eprintln!("❌ Form submission failed: {}", e);
            return Err(e.into());
        }
    }

    // Step 5: Demonstrate individual field filling
    println!("\n🎯 Step 5: Demonstrating individual field filling...");

    // Navigate to a new form for individual filling demo
    let navigate_task = AgentTask::new(
        r#"Navigate to this contact form page: data:text/html,<html><body><h1>Contact Form</h1><form><label for='name'>Full Name:</label><input type='text' id='name' name='name'><br><label for='message'>Message:</label><textarea id='message' name='message'></textarea><br><input type='submit' value='Send'></form></body></html>"#
    ).with_max_iterations(5);

    match agent.execute(navigate_task).await {
        Ok(_) => println!("✅ Navigated to contact form"),
        Err(e) => {
            eprintln!("❌ Navigation failed: {}", e);
            return Err(e.into());
        }
    }

    let individual_task = AgentTask::new(
        r#"Fill the contact form with:
        - Name: Jane Smith
        - Message: Hello, this is a test message from the smart form filler!

        Use individual fill_form_field calls for each field."#,
    )
    .with_max_iterations(5);

    match agent.execute(individual_task).await {
        Ok(response) => {
            println!("✅ Individual field filling completed!");
            println!("📝 Result: {}", response.result);
        }
        Err(e) => {
            eprintln!("❌ Individual field filling failed: {}", e);
            return Err(e.into());
        }
    }

    println!("\n✅ Smart Form Integration Demo Complete!");
    println!("=========================================");
    println!("The agent successfully:");
    println!("- Analyzed forms and extracted metadata");
    println!("- Auto-filled entire forms with semantic matching");
    println!("- Submitted forms using detected submit buttons");
    println!("- Handled both bulk and individual field filling");

    Ok(())
}

#[cfg(not(feature = "browser-automation"))]
fn main() {
    println!("This example requires the 'browser-automation' feature.");
    println!(
        "Run with: cargo run --example agent_smart_form_example --features browser-automation"
    );
}
