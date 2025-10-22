//! Example: Smart Form Filling
//!
//! Demonstrates intelligent form filling without hardcoded selectors.
//! The SmartFormFiller automatically discovers field selectors.

use semantic_browser::browser::{BrowserConfig, BrowserPool};
use semantic_browser::form_analyzer::FieldType;
use semantic_browser::smart_form_filler::SmartFormFiller;
use std::collections::HashMap;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_env_filter("info,semantic_browser=debug").init();

    println!("🤖 Smart Form Filling Example");
    println!("==============================\n");

    // 1. Create browser pool
    let config = BrowserConfig::default();
    let pool = BrowserPool::new(config).await?;

    // 2. Navigate to login page
    println!("📄 Navigating to login page...");
    let page = pool.get_page().await?;
    page.goto("https://the-internet.herokuapp.com/login").await?;
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // 3. Create smart form filler (auto-discovers form structure)
    println!("🔍 Analyzing page structure...");
    let filler = SmartFormFiller::new(Arc::new(page.clone())).await?;

    // 4. Show discovered forms
    println!("\n📋 Discovered Forms:");
    for (idx, form) in filler.get_forms().iter().enumerate() {
        println!("  Form #{}: {:?}", idx + 1, form.purpose);
        println!("    Fields: {}", form.fields.len());
        for field in &form.fields {
            println!(
                "      - {:?}: {} (confidence: {:.2})",
                field.field_type,
                field.label.as_ref().unwrap_or(&"No label".to_string()),
                field.confidence
            );
        }
    }

    // 5. Smart fill using semantic hints (no hardcoded selectors!)
    println!("\n✍️  Filling form with smart hints...");

    // Method 1: Smart fill by hint
    println!("  • Filling 'username' field...");
    let result = filler.fill_field_smart("username", "tomsmith").await?;
    if result.success {
        println!("    ✅ Success! Used selector: {}", result.selector_used);
        println!("    Confidence: {:.2}", result.confidence);
    } else {
        println!("    ❌ Failed: {:?}", result.error);
    }

    // Method 2: Fill by field type
    println!("  • Filling password field by type...");
    let result = filler.fill_field_by_type(FieldType::Password, "SuperSecretPassword!").await?;
    if result.success {
        println!("    ✅ Success! Used selector: {}", result.selector_used);
    }

    // 6. Auto-fill entire form at once
    println!("\n🚀 Auto-filling entire form...");
    let mut form_data = HashMap::new();
    form_data.insert("username".to_string(), "tomsmith".to_string());
    form_data.insert("password".to_string(), "SuperSecretPassword!".to_string());

    let report = filler.auto_fill_form(form_data).await?;
    println!("\n📊 Auto-fill Report:");
    println!("  ✅ Filled: {:?}", report.filled);
    println!("  ❌ Not found: {:?}", report.not_found);
    println!("  ⚠️  Failed: {:?}", report.failed);
    println!("  Success rate: {:.1}%", report.success_rate * 100.0);

    // 7. Submit form
    println!("\n📤 Submitting form...");
    if let Some(form) = filler.get_form(0) {
        if let Some(ref submit_selector) = form.submit_button {
            let element = page.find_element(submit_selector).await?;
            element.click().await?;
            println!("✅ Form submitted!");

            tokio::time::sleep(std::time::Duration::from_secs(2)).await;

            if let Ok(Some(url)) = page.url().await {
                println!("📍 Redirected to: {}", url);
            }
        }
    }

    // Cleanup
    pool.shutdown().await?;

    println!("\n🎉 Example completed successfully!");
    println!("\nKey Features Demonstrated:");
    println!("  ✅ Auto-discovery of form fields");
    println!("  ✅ Semantic field matching (no hardcoded selectors)");
    println!("  ✅ Multiple filling strategies (by hint, type, label)");
    println!("  ✅ Confidence scoring");
    println!("  ✅ Bulk auto-fill with reporting");

    Ok(())
}
