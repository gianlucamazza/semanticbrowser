//! Example: Form Login and Data Extraction
//!
//! Demonstrates how to use FormFiller to automate login and extract data from authenticated pages.

use semantic_browser::browser::{BrowserConfig, BrowserPool, NavigationOptions};
use semantic_browser::form_interaction::{FormData, FormFiller};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_env_filter("info,semantic_browser=debug").init();

    println!("🚀 Form Login Example");
    println!("=====================\n");

    // 1. Create browser pool
    let config = BrowserConfig::default();
    let pool = BrowserPool::new(config).await?;

    // 2. Navigate to login page
    println!("📄 Navigating to login page...");
    let options = NavigationOptions::default();
    let page = pool.get_page().await?;

    // Example: Login to a test site
    page.goto("https://the-internet.herokuapp.com/login").await?;
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // 3. Create form filler
    let filler = FormFiller::new(Arc::new(page.clone()));

    // 4. Fill login form
    println!("✍️  Filling login form...");
    let login_data =
        FormData::new().text("#username", "tomsmith").text("#password", "SuperSecretPassword!");

    filler.fill_form(&login_data).await?;

    // 5. Submit form
    println!("📤 Submitting form...");
    filler.submit_form("button[type='submit']").await?;

    println!("✅ Login successful!");

    // 6. Extract data from authenticated page
    println!("\n📊 Extracting data from authenticated page...");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let semantic_data = pool
        .navigate_and_extract(
            "https://the-internet.herokuapp.com/secure",
            NavigationOptions::default(),
        )
        .await?;

    println!("\n📄 Extracted Data:");
    println!("  Title: {:?}", semantic_data.title);
    println!("  Text Content: {} chars", semantic_data.text_content.len());
    println!("  Final URL: {}", semantic_data.final_url);

    // Cleanup
    pool.shutdown().await?;

    println!("\n✅ Example completed successfully!");
    Ok(())
}
