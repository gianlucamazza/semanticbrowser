//! Integration tests for browser automation module
//!
//! These tests require the browser-automation feature to be enabled:
//! cargo test --features browser-automation --test browser_test

#[cfg(feature = "browser-automation")]
mod browser_automation_tests {
    use semantic_browser::browser::{BrowserConfig, BrowserPool, NavigationOptions};
    use std::collections::HashMap;

    /// Test browser pool creation with default config
    #[tokio::test]
    async fn test_browser_pool_creation() {
        let config = BrowserConfig::default();
        let pool = BrowserPool::new(config).await;

        // Browser pool creation should succeed or fail gracefully
        // (may fail if Chromium not installed in CI environment)
        match pool {
            Ok(_) => {
                println!("Browser pool created successfully");
            }
            Err(e) => {
                println!("Browser pool creation failed (expected in CI): {}", e);
                // This is OK - Chromium may not be installed
            }
        }
    }

    /// Test browser pool creation from environment
    #[tokio::test]
    async fn test_browser_pool_from_env() {
        // Set test environment variables
        std::env::set_var("BROWSER_HEADLESS", "true");
        std::env::set_var("BLOCK_ADS", "true");
        std::env::set_var("BROWSER_POOL_SIZE", "3");

        let config = BrowserConfig::from_env();
        assert!(config.headless);
        assert!(config.block_ads);
        assert_eq!(config.pool_size, 3);
        assert!(config.user_data_dir.is_some());

        // Clean up
        std::env::remove_var("BROWSER_HEADLESS");
        std::env::remove_var("BLOCK_ADS");
        std::env::remove_var("BROWSER_POOL_SIZE");
    }

    /// Test navigation options creation
    #[test]
    fn test_navigation_options() {
        let mut options = NavigationOptions::default();
        assert!(options.cookies.is_empty());
        assert!(!options.take_screenshot);

        options.cookies.insert("session".to_string(), "abc123".to_string());
        options.take_screenshot = true;
        options.wait_for_selector = Some("body".to_string());

        assert_eq!(options.cookies.len(), 1);
        assert!(options.take_screenshot);
        assert_eq!(options.wait_for_selector, Some("body".to_string()));
    }

    /// Test browser config serialization
    #[test]
    fn test_browser_config_serialization() {
        let config = BrowserConfig {
            chromium_path: Some("/usr/bin/chromium".to_string()),
            headless: true,
            block_ads: true,
            block_images: false,
            timeout_secs: 45,
            pool_size: 3,
            user_data_dir: Some("/tmp/semantic-browser-test-profile".to_string()),
        };

        // Test serialization
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("chromium_path"));
        assert!(json.contains("headless"));

        // Test deserialization
        let deserialized: BrowserConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.chromium_path, config.chromium_path);
        assert_eq!(deserialized.timeout_secs, config.timeout_secs);
        assert_eq!(deserialized.user_data_dir, config.user_data_dir);
    }

    /// Integration test: Navigate to example.com
    /// This test only runs if Chromium is available
    #[tokio::test]
    #[ignore] // Ignore by default (requires Chromium)
    async fn test_navigate_example_com() {
        let config = BrowserConfig::default();
        let pool = match BrowserPool::new(config).await {
            Ok(pool) => pool,
            Err(e) => {
                println!("Skipping test: Chromium not available: {}", e);
                return;
            }
        };

        let options = NavigationOptions::default();
        let result = pool.navigate_and_extract("http://example.com", options).await;

        match result {
            Ok(semantic_data) => {
                println!("Successfully navigated to example.com");
                println!("Title: {:?}", semantic_data.title);
                println!("Text content length: {}", semantic_data.text_content.len());
                assert!(semantic_data.final_url.contains("example.com"));
            }
            Err(e) => {
                println!("Navigation failed: {}", e);
                // Don't fail the test - network issues etc.
            }
        }

        let _ = pool.shutdown().await;
    }

    /// Integration test: Extract semantic data
    #[tokio::test]
    #[ignore] // Ignore by default (requires Chromium and network)
    async fn test_extract_semantic_data() {
        let config = BrowserConfig {
            headless: true,
            block_ads: true,
            block_images: true, // Block images for faster test
            timeout_secs: 20,
            ..Default::default()
        };

        let pool = match BrowserPool::new(config).await {
            Ok(pool) => pool,
            Err(e) => {
                println!("Skipping test: Chromium not available: {}", e);
                return;
            }
        };

        // Test with a site that has structured data
        let options = NavigationOptions::default();
        let result = pool.navigate_and_extract("https://schema.org", options).await;

        match result {
            Ok(semantic_data) => {
                println!("Title: {:?}", semantic_data.title);
                println!("JSON-LD objects: {}", semantic_data.json_ld.len());
                println!("Microdata items: {}", semantic_data.microdata.len());

                // Schema.org should have some structured data
                assert!(!semantic_data.text_content.is_empty(), "Should extract text content");
            }
            Err(e) => {
                println!("Navigation failed: {}", e);
            }
        }

        let _ = pool.shutdown().await;
    }

    /// Integration test: Cookie management
    #[tokio::test]
    #[ignore] // Ignore by default (requires Chromium)
    async fn test_cookie_management() {
        let config = BrowserConfig::default();
        let pool = match BrowserPool::new(config).await {
            Ok(pool) => pool,
            Err(e) => {
                println!("Skipping test: {}", e);
                return;
            }
        };

        let mut cookies = HashMap::new();
        cookies.insert("test_cookie".to_string(), "test_value".to_string());

        let options = NavigationOptions { cookies, ..Default::default() };

        let result = pool.navigate_and_extract("http://example.com", options).await;
        match result {
            Ok(_) => println!("Cookie test passed"),
            Err(e) => println!("Cookie test failed: {}", e),
        }

        let _ = pool.shutdown().await;
    }

    /// Integration test: Screenshot capture
    #[tokio::test]
    #[ignore] // Ignore by default (requires Chromium)
    async fn test_screenshot_capture() {
        let config = BrowserConfig::default();
        let pool = match BrowserPool::new(config).await {
            Ok(pool) => pool,
            Err(e) => {
                println!("Skipping test: {}", e);
                return;
            }
        };

        let result = pool.take_screenshot("http://example.com").await;

        match result {
            Ok(screenshot_bytes) => {
                println!("Screenshot captured: {} bytes", screenshot_bytes.len());
                assert!(!screenshot_bytes.is_empty(), "Screenshot should not be empty");
            }
            Err(e) => {
                println!("Screenshot failed: {}", e);
            }
        }

        let _ = pool.shutdown().await;
    }

    /// Integration test: Custom JavaScript execution
    #[tokio::test]
    #[ignore] // Ignore by default (requires Chromium)
    async fn test_execute_javascript() {
        let config = BrowserConfig::default();
        let pool = match BrowserPool::new(config).await {
            Ok(pool) => pool,
            Err(e) => {
                println!("Skipping test: {}", e);
                return;
            }
        };

        let js_code = "document.title";
        let result = pool.execute_js("http://example.com", js_code).await;

        match result {
            Ok(value) => {
                println!("JavaScript result: {:?}", value);
            }
            Err(e) => {
                println!("JavaScript execution failed: {}", e);
            }
        }

        let _ = pool.shutdown().await;
    }

    /// Test external module integration
    #[tokio::test]
    async fn test_external_browse_with_best_available() {
        // This should work even without browser-automation feature (falls back to HTTP)
        let result = semantic_browser::external::browse_with_best_available(
            "http://example.com",
            "test query",
        )
        .await;

        // Should succeed with either chromium or HTTP fallback
        match result {
            Ok(outcome) => {
                println!("Browse summary: {}", outcome.summary);
                assert!(outcome.summary.contains("example.com"));
                assert!(outcome.snapshot.final_url.contains("example.com"));
            }
            Err(e) => {
                println!("Browse failed: {}", e);
                // Network errors are OK in tests
            }
        }
    }
}

// Tests that run even without browser-automation feature
#[cfg(not(feature = "browser-automation"))]
mod fallback_tests {
    use semantic_browser::browser::{BrowserConfig, BrowserPool};

    #[tokio::test]
    async fn test_browser_pool_without_feature() {
        let config = BrowserConfig::default();
        let result = BrowserPool::new(config).await;

        // Should fail with helpful error message
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("browser-automation feature not enabled"));
    }

    #[tokio::test]
    async fn test_fallback_to_http() {
        // Without browser-automation, should fall back to HTTP
        let result =
            semantic_browser::external::browse_with_best_available("http://example.com", "test")
                .await;

        // Should use HTTP fallback
        match result {
            Ok(outcome) => {
                println!("HTTP fallback summary: {}", outcome.summary);
            }
            Err(e) => println!("Network error (expected in some environments): {}", e),
        }
    }
}

// NEW TESTS FOR SPRINT 2 FEATURES
#[cfg(feature = "browser-automation")]
mod sprint2_tests {
    use semantic_browser::browser::{BrowserConfig, BrowserPool, NavigationOptions};

    /// Test wait_for_selector with HTML that loads element dynamically
    #[tokio::test]
    #[ignore] // Requires Chromium
    async fn test_wait_for_selector_timeout() {
        let config = BrowserConfig::default();
        let pool = match BrowserPool::new(config).await {
            Ok(pool) => pool,
            Err(e) => {
                println!("Skipping test: Chromium not available: {}", e);
                return;
            }
        };

        // Create options with selector that will never appear
        let options = NavigationOptions {
            wait_for_selector: Some("#nonexistent-element-12345".to_string()),
            ..Default::default()
        };

        // This should timeout
        let result = pool.navigate_and_extract("https://example.com", options).await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Timeout") || err_msg.contains("timeout"));

        let _ = pool.shutdown().await;
    }

    /// Test wait_for_selector with existing element
    #[tokio::test]
    #[ignore] // Requires Chromium and network
    async fn test_wait_for_selector_success() {
        let config = BrowserConfig::default();
        let pool = match BrowserPool::new(config).await {
            Ok(pool) => pool,
            Err(e) => {
                println!("Skipping test: Chromium not available: {}", e);
                return;
            }
        };

        // Wait for body element which should exist
        let options =
            NavigationOptions { wait_for_selector: Some("body".to_string()), ..Default::default() };

        let result = pool.navigate_and_extract("https://example.com", options).await;
        assert!(result.is_ok());

        let _ = pool.shutdown().await;
    }

    /// Test resource blocking configuration
    #[tokio::test]
    #[ignore] // Requires Chromium
    async fn test_resource_blocking_ads() {
        let config = BrowserConfig {
            headless: true,
            block_ads: true,
            block_images: false,
            ..Default::default()
        };

        let pool = match BrowserPool::new(config).await {
            Ok(pool) => pool,
            Err(e) => {
                println!("Skipping test: Chromium not available: {}", e);
                return;
            }
        };

        // Navigate with ad blocking enabled
        let options = NavigationOptions::default();
        let result = pool.navigate_and_extract("https://example.com", options).await;

        // Should succeed even with blocking enabled
        assert!(result.is_ok());

        let _ = pool.shutdown().await;
    }

    /// Test resource blocking with images
    #[tokio::test]
    #[ignore] // Requires Chromium
    async fn test_resource_blocking_images() {
        let config = BrowserConfig {
            headless: true,
            block_ads: false,
            block_images: true,
            ..Default::default()
        };

        let pool = match BrowserPool::new(config).await {
            Ok(pool) => pool,
            Err(e) => {
                println!("Skipping test: Chromium not available: {}", e);
                return;
            }
        };

        let options = NavigationOptions::default();
        let result = pool.navigate_and_extract("https://example.com", options).await;

        // Should succeed with image blocking
        assert!(result.is_ok());

        let _ = pool.shutdown().await;
    }

    /// Test retry logic with invalid URL (should fail after retries)
    #[tokio::test]
    #[ignore] // Requires Chromium
    async fn test_retry_logic_failure() {
        let config = BrowserConfig::default();
        let pool = match BrowserPool::new(config).await {
            Ok(pool) => pool,
            Err(e) => {
                println!("Skipping test: Chromium not available: {}", e);
                return;
            }
        };

        let options = NavigationOptions { max_retries: 2, ..Default::default() };

        let start = std::time::Instant::now();
        let result = pool
            .navigate_and_extract("http://invalid-domain-that-does-not-exist-12345.com", options)
            .await;

        let elapsed = start.elapsed();

        // Should fail
        assert!(result.is_err());

        // Should have taken at least 1 second (1s + 2s delays for 2 retries)
        // But less than 10 seconds (to avoid excessive test time)
        assert!(elapsed.as_secs() >= 1);
        assert!(elapsed.as_secs() < 10);

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to navigate") || err_msg.contains("after"));

        let _ = pool.shutdown().await;
    }

    /// Test retry logic with max_retries = 0 (no retries)
    #[tokio::test]
    #[ignore] // Requires Chromium
    async fn test_retry_logic_no_retries() {
        let config = BrowserConfig::default();
        let pool = match BrowserPool::new(config).await {
            Ok(pool) => pool,
            Err(e) => {
                println!("Skipping test: Chromium not available: {}", e);
                return;
            }
        };

        let options = NavigationOptions { max_retries: 0, ..Default::default() };

        let start = std::time::Instant::now();
        let result = pool.navigate_and_extract("http://invalid-domain-12345.com", options).await;

        let elapsed = start.elapsed();

        // Should fail quickly (no retry delays)
        assert!(result.is_err());
        assert!(elapsed.as_secs() < 5);

        let _ = pool.shutdown().await;
    }

    /// Test health check on healthy browser
    #[tokio::test]
    #[ignore] // Requires Chromium
    async fn test_health_check_healthy() {
        let config = BrowserConfig::default();
        let pool = match BrowserPool::new(config).await {
            Ok(pool) => pool,
            Err(e) => {
                println!("Skipping test: Chromium not available: {}", e);
                return;
            }
        };

        // Start browser by navigating
        let options = NavigationOptions::default();
        let _ = pool.navigate_and_extract("https://example.com", options).await;

        // Health check should pass
        let is_healthy = pool.health_check().await;
        assert!(is_healthy, "Browser should be healthy after successful navigation");

        let _ = pool.shutdown().await;
    }

    /// Test health check on uninitialized browser
    #[tokio::test]
    async fn test_health_check_uninitialized() {
        let config = BrowserConfig::default();
        let pool = BrowserPool::new(config).await.unwrap();

        // Health check on uninitialized browser should fail
        let is_healthy = pool.health_check().await;
        assert!(!is_healthy, "Uninitialized browser should fail health check");
    }

    /// Test restart_if_unhealthy
    #[tokio::test]
    #[ignore] // Requires Chromium
    async fn test_restart_if_unhealthy() {
        let config = BrowserConfig::default();
        let pool = match BrowserPool::new(config).await {
            Ok(pool) => pool,
            Err(e) => {
                println!("Skipping test: Chromium not available: {}", e);
                return;
            }
        };

        // Restart should succeed (will start browser if needed)
        let result = pool.restart_if_unhealthy().await;
        assert!(result.is_ok());

        // After restart, health check should pass
        let is_healthy = pool.health_check().await;
        assert!(is_healthy, "Browser should be healthy after restart");

        let _ = pool.shutdown().await;
    }

    /// Test navigation options with max_retries
    #[test]
    fn test_navigation_options_with_retries() {
        let mut options = NavigationOptions::default();
        assert_eq!(options.max_retries, 3, "Default should be 3 retries");

        options.max_retries = 5;
        assert_eq!(options.max_retries, 5);

        options.max_retries = 0;
        assert_eq!(options.max_retries, 0, "Should allow 0 retries");
    }
}
