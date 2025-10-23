//! Multi-tab Browser Orchestration Example
//!
//! This example demonstrates how to use the multi-tab browser orchestration
//! features to manage multiple browser tabs concurrently.
//!
//! Run with:
//! ```bash
//! cargo run --example multi_tab_example --features browser-automation
//! ```

use semantic_browser::browser::{BrowserConfig, BrowserPool};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 Multi-tab Browser Orchestration Example");
    println!("==========================================\n");

    // Create browser configuration
    let config = BrowserConfig {
        headless: true,
        block_ads: true,
        block_images: false,
        timeout_secs: 30,
        pool_size: 5, // Allow up to 5 tabs
        ..Default::default()
    };

    // Create browser pool
    let browser_pool = BrowserPool::new(config).await?;
    println!("✅ Browser pool initialized");

    // Example 1: Basic tab management
    println!("\n📑 Example 1: Basic Tab Management");
    println!("-----------------------------------");

    // Create some tabs
    let tab1 = browser_pool.create_tab(Some("search".to_string())).await?;
    let tab2 = browser_pool.create_tab(Some("docs".to_string())).await?;
    let tab3 = browser_pool.create_tab(Some("news".to_string())).await?;

    println!("Created tabs: {}, {}, {}", tab1, tab2, tab3);

    // List all tabs
    let tabs = browser_pool.list_tabs().await?;
    println!("Active tabs: {:?}", tabs);

    // Switch between tabs
    browser_pool.switch_tab(&tab2).await?;
    println!("Switched to tab: {}", tab2);

    // Example 2: Navigate different sites in different tabs
    println!("\n🌐 Example 2: Multi-site Navigation");
    println!("-----------------------------------");

    // Note: In a real implementation, we would use the navigate_and_extract_on_tab method
    // For now, we'll demonstrate the API structure
    println!("Tab '{}' ready for navigation", tab1);
    println!("Tab '{}' ready for navigation", tab2);
    println!("Tab '{}' ready for navigation", tab3);

    // Example 3: Execute actions on all tabs
    println!("\n⚡ Example 3: Concurrent Tab Operations");
    println!("--------------------------------------");

    // Define an action to execute on each tab
    let _get_tab_info = |page: std::sync::Arc<chromiumoxide::Page>| {
        // In a real implementation, this would extract information from the page
        // For demo purposes, we'll just return a placeholder
        format!("Tab info for page: {:?}", page)
    };

    // Execute on all tabs (this would work once we implement the full navigation)
    // let results = browser_pool.execute_on_all_tabs(get_tab_info).await?;
    // println!("Results from all tabs: {:?}", results);

    println!("Concurrent operations would be executed here");

    // Example 4: Tab lifecycle management
    println!("\n🔄 Example 4: Tab Lifecycle");
    println!("---------------------------");

    // Close a tab
    browser_pool.close_tab(&tab2).await?;
    println!("Closed tab: {}", tab2);

    // Check remaining tabs
    let remaining_tabs = browser_pool.list_tabs().await?;
    println!("Remaining tabs: {:?}", remaining_tabs);

    // Example 5: Resource management
    println!("\n🧹 Example 5: Resource Cleanup");
    println!("------------------------------");

    // Close remaining tabs
    for tab_id in &remaining_tabs {
        browser_pool.close_tab(tab_id).await?;
        println!("Closed tab: {}", tab_id);
    }

    // Shutdown browser pool
    browser_pool.shutdown().await?;
    println!("✅ Browser pool shut down cleanly");

    println!("\n🎉 Multi-tab orchestration demo completed!");
    println!("\nKey features demonstrated:");
    println!("  ✅ Tab creation and management");
    println!("  ✅ Tab switching");
    println!("  ✅ Concurrent operations framework");
    println!("  ✅ Resource cleanup");

    Ok(())
}
