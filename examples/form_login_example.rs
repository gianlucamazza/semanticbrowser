//! Example: Basic Browser Navigation and Data Extraction
//!
//! Demonstrates basic browser automation and semantic data extraction.
//! For form interaction examples, see the browser-automation feature examples.

use semantic_browser::browser::{BrowserConfig, BrowserPool, NavigationOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize logging
    tracing_subscriber::fmt().with_env_filter("info,semantic_browser=debug").init();

    println!("🚀 Browser Navigation Example");
    println!("=============================\n");

    // 1. Create browser pool
    let config = BrowserConfig::default();
    let pool = BrowserPool::new(config)
        .await
        .map_err(|e| format!("Failed to create browser pool: {}", e))?;

    // 2. Navigate to a test page and extract semantic data
    println!("📄 Navigating to test page...");
    let options = NavigationOptions { take_screenshot: true, ..Default::default() };

    let semantic_data = pool
        .navigate_and_extract("https://the-internet.herokuapp.com/", options)
        .await
        .map_err(|e| format!("Failed to navigate to test page: {}", e))?;

    println!("📄 Page loaded successfully!");
    println!("  Title: {:?}", semantic_data.title);
    println!("  Final URL: {}", semantic_data.final_url);
    if let Some(meta_desc) = &semantic_data.meta_description {
        println!("  Meta Description: {}", meta_desc);
    }
    println!("  Text Content: {} chars", semantic_data.text_content.len());
    println!("  JSON-LD Items: {}", semantic_data.json_ld.len());
    println!("  Microdata Items: {}", semantic_data.microdata.len());

    // 3. Navigate to another page
    println!("\n📄 Navigating to another page...");
    let about_data = pool
        .navigate_and_extract(
            "https://the-internet.herokuapp.com/abtest",
            NavigationOptions::default(),
        )
        .await
        .map_err(|e| format!("Failed to navigate to about page: {}", e))?;

    println!("📄 About page loaded!");
    println!("  Title: {:?}", about_data.title);
    println!("  Final URL: {}", about_data.final_url);

    // Cleanup
    pool.shutdown().await.map_err(|e| format!("Failed to shutdown browser pool: {}", e))?;

    println!("\n✅ Example completed successfully!");
    println!("\n💡 Tip: Enable the 'browser-automation' feature for advanced form interaction:");
    println!("   cargo run --example form_login_example --features browser-automation");

    Ok(())
}
