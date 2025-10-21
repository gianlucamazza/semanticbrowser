//! Browser automation module using chromiumoxide
//!
//! Provides headless browser automation with full JavaScript support,
//! cookie/session management, screenshot capture, and semantic data extraction.
//!
//! Best practices 2025:
//! - Use chromiumoxide for pure Rust headless browsing
//! - Pool management for concurrent navigation
//! - Resource blocking for minimalist browsing (ads, trackers)
//! - Direct integration with Knowledge Graph
//! - Async-first design with Tokio

#[cfg(feature = "browser-automation")]
use chromiumoxide::browser::{Browser, BrowserConfig as ChromiumBrowserConfig};
#[cfg(feature = "browser-automation")]
use chromiumoxide::Page;
#[cfg(feature = "browser-automation")]
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(feature = "browser-automation")]
use std::sync::Arc;
#[cfg(feature = "browser-automation")]
use tokio::sync::Mutex;

/// Browser configuration for minimalist navigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    /// Path to Chromium/Chrome executable (optional, auto-detect if None)
    pub chromium_path: Option<String>,
    /// Run in headless mode
    pub headless: bool,
    /// Block ads and trackers
    pub block_ads: bool,
    /// Block images (for text-only extraction)
    pub block_images: bool,
    /// Default navigation timeout in seconds
    pub timeout_secs: u64,
    /// Maximum number of concurrent tabs
    pub pool_size: usize,
    /// Directory for Chromium user data (profile)
    pub user_data_dir: Option<String>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

impl BrowserConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        #[allow(clippy::disallowed_methods)]
        let chromium_path = std::env::var("CHROMIUM_PATH").ok();
        #[allow(clippy::disallowed_methods)]
        let headless = std::env::var("BROWSER_HEADLESS")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);
        #[allow(clippy::disallowed_methods)]
        let block_ads = std::env::var("BLOCK_ADS")
            .unwrap_or_else(|_| "true".to_string())
            .parse()
            .unwrap_or(true);
        #[allow(clippy::disallowed_methods)]
        let block_images = std::env::var("BLOCK_IMAGES")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);
        #[allow(clippy::disallowed_methods)]
        let timeout_secs = std::env::var("BROWSER_TIMEOUT_SECS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);
        #[allow(clippy::disallowed_methods)]
        let pool_size = std::env::var("BROWSER_POOL_SIZE")
            .unwrap_or_else(|_| "2".to_string())
            .parse()
            .unwrap_or(2);
        #[allow(clippy::disallowed_methods)]
        let user_data_dir = std::env::var("CHROMIUMOXIDE_USER_DATA_DIR")
            .ok()
            .and_then(|value| {
                let trimmed = value.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            })
            .or_else(|| {
                let mut path = std::env::temp_dir();
                path.push("semantic-browser");
                path.push("chromiumoxide");
                path.push(format!(
                    "profile-{}-{}",
                    std::process::id(),
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
                        .as_millis()
                ));
                Some(path.to_string_lossy().to_string())
            });

        Self {
            chromium_path,
            headless,
            block_ads,
            block_images,
            timeout_secs,
            pool_size,
            user_data_dir,
        }
    }
}

/// Navigation options for individual requests
#[derive(Debug, Clone)]
pub struct NavigationOptions {
    /// Wait for specific selector before considering page loaded
    pub wait_for_selector: Option<String>,
    /// Custom cookies to set before navigation
    pub cookies: HashMap<String, String>,
    /// Take screenshot after navigation
    pub take_screenshot: bool,
    /// Custom JavaScript to execute after page load
    pub execute_js: Option<String>,
    /// Maximum number of retry attempts on failure (default: 3)
    pub max_retries: u32,
}

impl Default for NavigationOptions {
    fn default() -> Self {
        Self {
            wait_for_selector: None,
            cookies: HashMap::new(),
            take_screenshot: false,
            execute_js: None,
            max_retries: 3,
        }
    }
}

/// Semantic data extracted from page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticData {
    /// Page title
    pub title: Option<String>,
    /// JSON-LD structured data
    pub json_ld: Vec<serde_json::Value>,
    /// Microdata items
    pub microdata: Vec<crate::parser::MicrodataItem>,
    /// Plain text content (for NER)
    pub text_content: String,
    /// Screenshot bytes (if requested)
    pub screenshot: Option<Vec<u8>>,
    /// Final URL after redirects
    pub final_url: String,

    // Phase 1: Enhanced Meta Tags Extraction (2025 best practices)
    /// Meta description tag
    pub meta_description: Option<String>,
    /// Meta keywords (comma-separated -> vec)
    pub meta_keywords: Vec<String>,
    /// Page language (from html lang attribute)
    pub language: Option<String>,
    /// Canonical URL (rel="canonical")
    pub canonical_url: Option<String>,
    /// Open Graph meta tags (og:title, og:image, etc.)
    pub open_graph: HashMap<String, String>,
    /// Twitter Card meta tags (twitter:card, twitter:site, etc.)
    pub twitter_card: HashMap<String, String>,
}

/// Browser pool for managing concurrent browser instances
#[cfg(feature = "browser-automation")]
#[derive(Debug)]
pub struct BrowserPool {
    config: BrowserConfig,
    browser: Arc<Mutex<Option<Browser>>>,
}

#[cfg(feature = "browser-automation")]
impl BrowserPool {
    /// Create a new browser pool with configuration
    pub async fn new(
        config: BrowserConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Initializing browser pool with config: {:?}", config);
        Ok(Self { config, browser: Arc::new(Mutex::new(None)) })
    }

    /// Get or create browser instance (returns reference, not owned)
    async fn ensure_browser_started(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut browser_lock = self.browser.lock().await;

        if browser_lock.is_some() {
            // Browser already initialized
            return Ok(());
        }

        // Create new browser
        tracing::info!("Creating new browser instance");
        let mut builder = ChromiumBrowserConfig::builder();

        if self.config.headless {
            builder = builder.with_head();
        }

        if let Some(ref path) = self.config.chromium_path {
            builder = builder.chrome_executable(path);
        }

        // Additional chromium args for minimalist browsing
        let mut args = vec![];
        if self.config.headless {
            args.push("--headless");
        }
        args.push("--disable-gpu");
        args.push("--no-sandbox"); // Required for Docker
        args.push("--disable-dev-shm-usage"); // Overcome limited resource problems

        if self.config.block_ads {
            // Block common ad/tracker domains via hosts
            args.push("--disable-background-networking");
        }

        for arg in args {
            builder = builder.arg(arg);
        }

        if let Some(ref dir) = self.config.user_data_dir {
            let dir_path = std::path::Path::new(dir);
            if let Err(e) = std::fs::create_dir_all(dir_path) {
                return Err(Box::new(std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to prepare Chromium user data dir {}: {}",
                        dir_path.display(),
                        e
                    ),
                )));
            }
            builder = builder.user_data_dir(dir_path);
        }

        let (browser, mut handler) = Browser::launch(builder.build()?).await?;

        // Spawn handler task
        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                tracing::trace!("Browser event: {:?}", event);
            }
        });

        *browser_lock = Some(browser);
        Ok(())
    }

    /// Navigate to URL and extract semantic data with retry logic
    ///
    /// Best practices 2025: Automatic retry with exponential backoff for resilience.
    pub async fn navigate_and_extract(
        &self,
        url: &str,
        options: NavigationOptions,
    ) -> Result<SemanticData, Box<dyn std::error::Error + Send + Sync>> {
        let max_retries = options.max_retries;
        let mut last_error = None;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                // Exponential backoff: 1s, 2s, 4s, ...
                let delay = std::time::Duration::from_secs(2u64.pow(attempt - 1));
                tracing::warn!("Retry attempt {} for {} after {:?} delay", attempt, url, delay);
                tokio::time::sleep(delay).await;
            }

            match self.navigate_and_extract_internal(url, &options).await {
                Ok(data) => {
                    if attempt > 0 {
                        tracing::info!(
                            "Successfully navigated to {} after {} retries",
                            url,
                            attempt
                        );
                    }
                    return Ok(data);
                }
                Err(e) => {
                    tracing::warn!("Navigation attempt {} failed for {}: {}", attempt + 1, url, e);
                    last_error = Some(e);
                }
            }
        }

        Err(format!(
            "Failed to navigate to {} after {} attempts: {}",
            url,
            max_retries + 1,
            last_error.unwrap()
        )
        .into())
    }

    /// Internal navigation logic (called by navigate_and_extract with retry)
    async fn navigate_and_extract_internal(
        &self,
        url: &str,
        options: &NavigationOptions,
    ) -> Result<SemanticData, Box<dyn std::error::Error + Send + Sync>> {
        // Ensure browser is started
        self.ensure_browser_started().await?;

        // Get browser reference
        let browser_lock = self.browser.lock().await;
        let browser = browser_lock.as_ref().ok_or("Browser not initialized")?;

        let page = browser.new_page("about:blank").await?;

        // Set cookies if provided
        for (name, value) in &options.cookies {
            self.set_cookie(&page, url, name, value).await?;
        }

        // Block resources if configured
        if self.config.block_ads || self.config.block_images {
            self.setup_resource_blocking(&page).await?;
        }

        // Navigate to URL
        tracing::info!("Navigating to: {}", url);
        let timeout = std::time::Duration::from_secs(self.config.timeout_secs);
        let navigation = page.goto(url).await?;

        // Wait for navigation with timeout
        tokio::time::timeout(timeout, navigation.wait_for_navigation()).await??;

        // Wait for selector if specified
        if let Some(selector) = &options.wait_for_selector {
            tracing::debug!("Waiting for selector: {}", selector);
            self.wait_for_element(&page, selector, timeout).await?;
        }

        // Execute custom JS if provided
        if let Some(js) = &options.execute_js {
            tracing::debug!("Executing custom JavaScript");
            page.evaluate(js.clone()).await?;
        }

        // Extract semantic data
        let semantic_data = self.extract_semantic_data(&page, url, options.take_screenshot).await?;

        tracing::info!("Successfully extracted semantic data from: {}", url);
        Ok(semantic_data)
    }

    /// Set a cookie for the page
    async fn set_cookie(
        &self,
        page: &Page,
        _url: &str,
        name: &str,
        value: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use chromiumoxide::cdp::browser_protocol::network::SetCookieParams;

        // SetCookieParams takes name and value directly, not CookieParam
        let params = SetCookieParams::new(name, value);

        page.execute(params).await?;
        Ok(())
    }

    /// Wait for element to appear on page
    ///
    /// Polls for element using `find_element` with configurable timeout and interval.
    /// Best practices 2025: robust waiting with exponential backoff for dynamic content.
    async fn wait_for_element(
        &self,
        page: &Page,
        selector: &str,
        timeout: std::time::Duration,
    ) -> Result<chromiumoxide::element::Element, Box<dyn std::error::Error + Send + Sync>> {
        let start = std::time::Instant::now();
        let mut interval = std::time::Duration::from_millis(100);
        let max_interval = std::time::Duration::from_millis(500);

        loop {
            // Try to find element
            match page.find_element(selector).await {
                Ok(element) => {
                    tracing::debug!("Element '{}' found after {:?}", selector, start.elapsed());
                    return Ok(element);
                }
                Err(_) => {
                    // Check timeout
                    if start.elapsed() >= timeout {
                        return Err(format!(
                            "Timeout waiting for selector '{}' after {:?}",
                            selector, timeout
                        )
                        .into());
                    }

                    // Exponential backoff (cap at max_interval)
                    tokio::time::sleep(interval).await;
                    interval = std::cmp::min(interval * 2, max_interval);
                }
            }
        }
    }

    /// Setup resource blocking for minimalist browsing
    ///
    /// Blocks ads, trackers, and images based on configuration.
    /// Best practices 2025: minimalist browsing for semantic extraction.
    async fn setup_resource_blocking(
        &self,
        page: &Page,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use chromiumoxide::cdp::browser_protocol::network::{EnableParams, SetBlockedUrLsParams};

        // Enable network domain (required for blocking)
        page.execute(EnableParams::default()).await?;

        // Build list of URL patterns to block
        let mut blocked_patterns: Vec<String> = Vec::new();

        if self.config.block_ads {
            // Block common ad and tracker domains
            let ad_patterns = [
                "*doubleclick.net*",
                "*googleadservices.com*",
                "*googlesyndication.com*",
                "*google-analytics.com*",
                "*googletagmanager.com*",
                "*facebook.com/tr/*",
                "*facebook.net*",
                "*adservice*",
                "*advertisement*",
                "*/ads/*",
                "*analytics*",
                "*tracking*",
                "*tracker*",
            ];
            blocked_patterns.extend(ad_patterns.iter().map(|s| s.to_string()));
            tracing::debug!("Blocking ads and trackers");
        }

        if self.config.block_images {
            // Block common image formats
            let image_patterns =
                ["*.jpg", "*.jpeg", "*.png", "*.gif", "*.webp", "*.bmp", "*.svg", "*.ico"];
            blocked_patterns.extend(image_patterns.iter().map(|s| s.to_string()));
            tracing::debug!("Blocking images");
        }

        // Apply blocking if patterns exist
        if !blocked_patterns.is_empty() {
            let params = SetBlockedUrLsParams::new(blocked_patterns.clone());
            page.execute(params).await?;
            tracing::info!("Resource blocking enabled: {} patterns", blocked_patterns.len());
        }

        Ok(())
    }

    /// Extract semantic data from page
    async fn extract_semantic_data(
        &self,
        page: &Page,
        original_url: &str,
        take_screenshot: bool,
    ) -> Result<SemanticData, Box<dyn std::error::Error + Send + Sync>> {
        // Get final URL after redirects
        let final_url = page.url().await?.unwrap_or_else(|| original_url.to_string());

        // Get page title
        let title = page.get_title().await?;

        // Get HTML content
        let html = page.content().await?;

        // Parse with scraper and extract all data synchronously (before any async calls)
        let (
            json_ld,
            microdata,
            text_content,
            meta_description,
            meta_keywords,
            language,
            canonical_url,
            open_graph,
            twitter_card,
        ) = {
            let document = scraper::Html::parse_document(&html);

            // Extract JSON-LD
            let json_ld_selector = scraper::Selector::parse("script[type=\"application/ld+json\"]")
                .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e.to_string()))?;
            let mut json_ld = Vec::new();
            for element in document.select(&json_ld_selector) {
                if let Some(text) = element.text().next() {
                    if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
                        json_ld.push(value);
                    }
                }
            }

            // Extract microdata
            let microdata = self.extract_microdata(&document)?;

            // Extract text content (for NER and minimalist view)
            let text_content = self.extract_text_content(&document)?;

            // Phase 1: Extract meta tags
            let meta_description = self.extract_meta_description(&document);
            let meta_keywords = self.extract_meta_keywords(&document);
            let language = self.extract_language(&document);
            let canonical_url = self.extract_canonical_url(&document);
            let open_graph = self.extract_open_graph(&document);
            let twitter_card = self.extract_twitter_card(&document);

            (
                json_ld,
                microdata,
                text_content,
                meta_description,
                meta_keywords,
                language,
                canonical_url,
                open_graph,
                twitter_card,
            )
        };
        // document is dropped here, before screenshot await

        // Take screenshot if requested
        let screenshot = if take_screenshot {
            Some(self.take_screenshot_internal(page).await?)
        } else {
            None
        };

        Ok(SemanticData {
            title,
            json_ld,
            microdata,
            text_content,
            screenshot,
            final_url,
            // Phase 1: Meta tags
            meta_description,
            meta_keywords,
            language,
            canonical_url,
            open_graph,
            twitter_card,
        })
    }

    /// Extract microdata from HTML document
    fn extract_microdata(
        &self,
        document: &scraper::Html,
    ) -> Result<Vec<crate::parser::MicrodataItem>, Box<dyn std::error::Error + Send + Sync>> {
        let itemscope_selector = scraper::Selector::parse("[itemscope]")
            .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e.to_string()))?;
        let itemprop_selector = scraper::Selector::parse("[itemprop]")
            .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e.to_string()))?;

        let mut microdata = Vec::new();
        for element in document.select(&itemscope_selector) {
            let item_type = element.value().attr("itemtype").unwrap_or("").to_string();
            let mut properties = HashMap::new();
            for prop in element.select(&itemprop_selector) {
                let prop_name = prop.value().attr("itemprop").unwrap_or("").to_string();
                let prop_value = prop.text().collect::<String>();
                properties.entry(prop_name).or_insert_with(Vec::new).push(prop_value);
            }
            microdata.push(crate::parser::MicrodataItem { item_type, properties });
        }

        Ok(microdata)
    }

    /// Extract clean text content (exclude nav, footer, ads)
    fn extract_text_content(
        &self,
        document: &scraper::Html,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Try to find main content area
        let content_selectors = ["main", "article", "[role=\"main\"]", ".content", "#content"];

        for selector_str in &content_selectors {
            if let Ok(selector) = scraper::Selector::parse(selector_str) {
                if let Some(element) = document.select(&selector).next() {
                    let text: String = element.text().collect::<Vec<_>>().join(" ");
                    // Clean up whitespace
                    let cleaned = text.split_whitespace().collect::<Vec<_>>().join(" ");
                    if !cleaned.is_empty() {
                        return Ok(cleaned);
                    }
                }
            }
        }

        // Fallback: get body text
        if let Ok(body_selector) = scraper::Selector::parse("body") {
            if let Some(body) = document.select(&body_selector).next() {
                let text: String = body.text().collect::<Vec<_>>().join(" ");
                let cleaned = text.split_whitespace().collect::<Vec<_>>().join(" ");
                return Ok(cleaned);
            }
        }

        Ok(String::new())
    }

    // Phase 1: Enhanced Meta Tags Extraction Functions (2025 best practices)

    /// Extract meta description from HTML
    fn extract_meta_description(&self, document: &scraper::Html) -> Option<String> {
        let meta_selector = scraper::Selector::parse("meta[name='description']").ok()?;
        document
            .select(&meta_selector)
            .next()
            .and_then(|el| el.value().attr("content"))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Extract meta keywords from HTML (comma-separated -> Vec)
    fn extract_meta_keywords(&self, document: &scraper::Html) -> Vec<String> {
        let meta_selector = scraper::Selector::parse("meta[name='keywords']").ok();
        if let Some(selector) = meta_selector {
            if let Some(el) = document.select(&selector).next() {
                if let Some(content) = el.value().attr("content") {
                    return content
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            }
        }
        Vec::new()
    }

    /// Extract page language from html lang attribute
    fn extract_language(&self, document: &scraper::Html) -> Option<String> {
        let html_selector = scraper::Selector::parse("html").ok()?;
        document
            .select(&html_selector)
            .next()
            .and_then(|el| el.value().attr("lang"))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Extract canonical URL from link rel="canonical"
    fn extract_canonical_url(&self, document: &scraper::Html) -> Option<String> {
        let link_selector = scraper::Selector::parse("link[rel='canonical']").ok()?;
        document
            .select(&link_selector)
            .next()
            .and_then(|el| el.value().attr("href"))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Extract Open Graph meta tags (og:*)
    fn extract_open_graph(&self, document: &scraper::Html) -> HashMap<String, String> {
        let mut og_data = HashMap::new();

        if let Ok(og_selector) = scraper::Selector::parse("meta[property^='og:']") {
            for el in document.select(&og_selector) {
                if let Some(property) = el.value().attr("property") {
                    if let Some(content) = el.value().attr("content") {
                        let key = property.trim_start_matches("og:").to_string();
                        og_data.insert(key, content.trim().to_string());
                    }
                }
            }
        }

        og_data
    }

    /// Extract Twitter Card meta tags (twitter:*)
    fn extract_twitter_card(&self, document: &scraper::Html) -> HashMap<String, String> {
        let mut twitter_data = HashMap::new();

        if let Ok(twitter_selector) = scraper::Selector::parse("meta[name^='twitter:']") {
            for el in document.select(&twitter_selector) {
                if let Some(name) = el.value().attr("name") {
                    if let Some(content) = el.value().attr("content") {
                        let key = name.trim_start_matches("twitter:").to_string();
                        twitter_data.insert(key, content.trim().to_string());
                    }
                }
            }
        }

        twitter_data
    }

    /// Take screenshot of current page
    async fn take_screenshot_internal(
        &self,
        page: &Page,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotParams;

        let screenshot = page.screenshot(CaptureScreenshotParams::default()).await?;
        Ok(screenshot)
    }

    /// Public method to take screenshot
    pub async fn take_screenshot(
        &self,
        url: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let options = NavigationOptions { take_screenshot: true, ..Default::default() };
        let data = self.navigate_and_extract(url, options).await?;
        data.screenshot.ok_or_else(|| "Screenshot not available".into())
    }

    /// Execute custom JavaScript on a page
    pub async fn execute_js(
        &self,
        url: &str,
        js_code: &str,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        // Ensure browser is started
        self.ensure_browser_started().await?;

        // Get browser reference
        let browser_lock = self.browser.lock().await;
        let browser = browser_lock.as_ref().ok_or("Browser not initialized")?;

        let page = browser.new_page(url).await?;

        let result = page.evaluate(js_code).await?;
        Ok(result.into_value()?)
    }

    /// Health check for browser pool
    ///
    /// Verifies browser is running and responsive.
    /// Best practices 2025: Kubernetes readiness/liveness probes.
    pub async fn health_check(&self) -> bool {
        // Check if browser is initialized
        let browser_lock = self.browser.lock().await;
        let browser = match browser_lock.as_ref() {
            Some(b) => b,
            None => {
                tracing::warn!("Health check failed: browser not initialized");
                return false;
            }
        };

        // Try to create a test page to verify browser is responsive
        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            browser.new_page("about:blank"),
        )
        .await
        {
            Ok(Ok(page)) => {
                // Successfully created page, browser is healthy
                // Close the test page
                let _ = page.close().await;
                tracing::debug!("Health check passed");
                true
            }
            Ok(Err(e)) => {
                tracing::warn!("Health check failed: error creating page: {}", e);
                false
            }
            Err(_) => {
                tracing::warn!("Health check failed: timeout creating page");
                false
            }
        }
    }

    /// Restart browser if unhealthy
    ///
    /// Best practices 2025: Auto-recovery for production resilience.
    pub async fn restart_if_unhealthy(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if !self.health_check().await {
            tracing::warn!("Browser unhealthy, attempting restart...");

            // Shutdown existing browser
            let _ = self.shutdown().await;

            // Clear the browser reference
            {
                let mut browser_lock = self.browser.lock().await;
                *browser_lock = None;
            }

            // Restart browser
            self.ensure_browser_started().await?;
            tracing::info!("Browser restarted successfully");
        }
        Ok(())
    }

    /// Shutdown the browser pool
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut browser_lock = self.browser.lock().await;
        if let Some(mut browser) = browser_lock.take() {
            tracing::info!("Shutting down browser");
            browser.close().await?;
        }
        Ok(())
    }
}

// Fallback implementation when browser-automation feature is not enabled
#[cfg(not(feature = "browser-automation"))]
#[derive(Debug)]
pub struct BrowserPool;

#[cfg(not(feature = "browser-automation"))]
impl BrowserPool {
    pub async fn new(
        _config: BrowserConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Err("browser-automation feature not enabled. Enable with: cargo build --features browser-automation".into())
    }

    pub async fn navigate_and_extract(
        &self,
        _url: &str,
        _options: NavigationOptions,
    ) -> Result<SemanticData, Box<dyn std::error::Error + Send + Sync>> {
        Err("browser-automation feature not enabled".into())
    }

    pub async fn take_screenshot(
        &self,
        _url: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        Err("browser-automation feature not enabled".into())
    }

    pub async fn execute_js(
        &self,
        _url: &str,
        _js_code: &str,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        Err("browser-automation feature not enabled".into())
    }

    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_config_default() {
        let config = BrowserConfig::default();
        assert!(config.headless);
        assert!(config.block_ads);
        assert!(!config.block_images);
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.pool_size, 2);
    }

    #[test]
    fn test_navigation_options_default() {
        let options = NavigationOptions::default();
        assert!(options.wait_for_selector.is_none());
        assert!(options.cookies.is_empty());
        assert!(!options.take_screenshot);
        assert!(options.execute_js.is_none());
    }

    #[tokio::test]
    #[cfg(feature = "browser-automation")]
    async fn test_browser_pool_creation() {
        let config = BrowserConfig::default();
        let result = BrowserPool::new(config).await;
        assert!(result.is_ok());
    }

    // Phase 1: Meta Tags Extraction Tests (2025 best practices)

    #[tokio::test]
    #[cfg(feature = "browser-automation")]
    async fn test_extract_meta_description() {
        let config = BrowserConfig::default();
        let pool = BrowserPool::new(config).await.unwrap();

        let html = r#"
            <html>
            <head>
                <meta name="description" content="This is a test description">
            </head>
            <body></body>
            </html>
        "#;
        let document = scraper::Html::parse_document(html);
        let result = pool.extract_meta_description(&document);

        assert_eq!(result, Some("This is a test description".to_string()));
    }

    #[tokio::test]
    #[cfg(feature = "browser-automation")]
    async fn test_extract_meta_description_missing() {
        let config = BrowserConfig::default();
        let pool = BrowserPool::new(config).await.unwrap();

        let html = "<html><head></head><body></body></html>";
        let document = scraper::Html::parse_document(html);
        let result = pool.extract_meta_description(&document);

        assert_eq!(result, None);
    }

    #[tokio::test]
    #[cfg(feature = "browser-automation")]
    async fn test_extract_meta_description_empty() {
        let config = BrowserConfig::default();
        let pool = BrowserPool::new(config).await.unwrap();

        let html = r#"<html><head><meta name="description" content=""></head><body></body></html>"#;
        let document = scraper::Html::parse_document(html);
        let result = pool.extract_meta_description(&document);

        assert_eq!(result, None);
    }

    #[tokio::test]
    #[cfg(feature = "browser-automation")]
    async fn test_extract_meta_keywords() {
        let config = BrowserConfig::default();
        let pool = BrowserPool::new(config).await.unwrap();

        let html = r#"
            <html>
            <head>
                <meta name="keywords" content="rust, html, semantic, web">
            </head>
            <body></body>
            </html>
        "#;
        let document = scraper::Html::parse_document(html);
        let result = pool.extract_meta_keywords(&document);

        assert_eq!(result, vec!["rust", "html", "semantic", "web"]);
    }

    #[tokio::test]
    #[cfg(feature = "browser-automation")]
    async fn test_extract_meta_keywords_empty() {
        let config = BrowserConfig::default();
        let pool = BrowserPool::new(config).await.unwrap();

        let html = "<html><head></head><body></body></html>";
        let document = scraper::Html::parse_document(html);
        let result = pool.extract_meta_keywords(&document);

        assert!(result.is_empty());
    }

    #[tokio::test]
    #[cfg(feature = "browser-automation")]
    async fn test_extract_language() {
        let config = BrowserConfig::default();
        let pool = BrowserPool::new(config).await.unwrap();

        let html = r#"<html lang="en-US"><head></head><body></body></html>"#;
        let document = scraper::Html::parse_document(html);
        let result = pool.extract_language(&document);

        assert_eq!(result, Some("en-US".to_string()));
    }

    #[tokio::test]
    #[cfg(feature = "browser-automation")]
    async fn test_extract_language_missing() {
        let config = BrowserConfig::default();
        let pool = BrowserPool::new(config).await.unwrap();

        let html = "<html><head></head><body></body></html>";
        let document = scraper::Html::parse_document(html);
        let result = pool.extract_language(&document);

        assert_eq!(result, None);
    }

    #[tokio::test]
    #[cfg(feature = "browser-automation")]
    async fn test_extract_canonical_url() {
        let config = BrowserConfig::default();
        let pool = BrowserPool::new(config).await.unwrap();

        let html = r#"
            <html>
            <head>
                <link rel="canonical" href="https://example.com/canonical-page">
            </head>
            <body></body>
            </html>
        "#;
        let document = scraper::Html::parse_document(html);
        let result = pool.extract_canonical_url(&document);

        assert_eq!(result, Some("https://example.com/canonical-page".to_string()));
    }

    #[tokio::test]
    #[cfg(feature = "browser-automation")]
    async fn test_extract_canonical_url_missing() {
        let config = BrowserConfig::default();
        let pool = BrowserPool::new(config).await.unwrap();

        let html = "<html><head></head><body></body></html>";
        let document = scraper::Html::parse_document(html);
        let result = pool.extract_canonical_url(&document);

        assert_eq!(result, None);
    }

    #[tokio::test]
    #[cfg(feature = "browser-automation")]
    async fn test_extract_open_graph() {
        let config = BrowserConfig::default();
        let pool = BrowserPool::new(config).await.unwrap();

        let html = r#"
            <html>
            <head>
                <meta property="og:title" content="Test Page">
                <meta property="og:description" content="A test page description">
                <meta property="og:image" content="https://example.com/image.jpg">
                <meta property="og:type" content="article">
            </head>
            <body></body>
            </html>
        "#;
        let document = scraper::Html::parse_document(html);
        let result = pool.extract_open_graph(&document);

        assert_eq!(result.len(), 4);
        assert_eq!(result.get("title"), Some(&"Test Page".to_string()));
        assert_eq!(result.get("description"), Some(&"A test page description".to_string()));
        assert_eq!(result.get("image"), Some(&"https://example.com/image.jpg".to_string()));
        assert_eq!(result.get("type"), Some(&"article".to_string()));
    }

    #[tokio::test]
    #[cfg(feature = "browser-automation")]
    async fn test_extract_open_graph_empty() {
        let config = BrowserConfig::default();
        let pool = BrowserPool::new(config).await.unwrap();

        let html = "<html><head></head><body></body></html>";
        let document = scraper::Html::parse_document(html);
        let result = pool.extract_open_graph(&document);

        assert!(result.is_empty());
    }

    #[tokio::test]
    #[cfg(feature = "browser-automation")]
    async fn test_extract_twitter_card() {
        let config = BrowserConfig::default();
        let pool = BrowserPool::new(config).await.unwrap();

        let html = r#"
            <html>
            <head>
                <meta name="twitter:card" content="summary_large_image">
                <meta name="twitter:site" content="@example">
                <meta name="twitter:title" content="Test Page">
                <meta name="twitter:description" content="A test page">
                <meta name="twitter:image" content="https://example.com/twitter-image.jpg">
            </head>
            <body></body>
            </html>
        "#;
        let document = scraper::Html::parse_document(html);
        let result = pool.extract_twitter_card(&document);

        assert_eq!(result.len(), 5);
        assert_eq!(result.get("card"), Some(&"summary_large_image".to_string()));
        assert_eq!(result.get("site"), Some(&"@example".to_string()));
        assert_eq!(result.get("title"), Some(&"Test Page".to_string()));
        assert_eq!(result.get("description"), Some(&"A test page".to_string()));
        assert_eq!(result.get("image"), Some(&"https://example.com/twitter-image.jpg".to_string()));
    }

    #[tokio::test]
    #[cfg(feature = "browser-automation")]
    async fn test_extract_twitter_card_empty() {
        let config = BrowserConfig::default();
        let pool = BrowserPool::new(config).await.unwrap();

        let html = "<html><head></head><body></body></html>";
        let document = scraper::Html::parse_document(html);
        let result = pool.extract_twitter_card(&document);

        assert!(result.is_empty());
    }

    #[tokio::test]
    #[cfg(feature = "browser-automation")]
    async fn test_extract_meta_keywords_with_spaces() {
        let config = BrowserConfig::default();
        let pool = BrowserPool::new(config).await.unwrap();

        let html = r#"
            <html>
            <head>
                <meta name="keywords" content="  rust  ,  html  ,  semantic  ">
            </head>
            <body></body>
            </html>
        "#;
        let document = scraper::Html::parse_document(html);
        let result = pool.extract_meta_keywords(&document);

        // Should trim whitespace from each keyword
        assert_eq!(result, vec!["rust", "html", "semantic"]);
    }
}
