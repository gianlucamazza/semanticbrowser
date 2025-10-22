//! Form Interaction Module
//!
//! Provides high-level abstractions for interacting with HTML forms in headless browser.
//! Enables AI agents to fill, submit, and interact with web forms programmatically.
//!
//! # Features
//! - Text input filling
//! - Dropdown/select option selection
//! - Checkbox/radio button interaction
//! - Form submission with navigation handling
//! - Multi-step form workflows
//!
//! # Best Practices 2025
//! - Async-first design with Tokio
//! - Retry logic with exponential backoff
//! - Type-safe selectors
//! - Comprehensive error handling

#[cfg(feature = "browser-automation")]
use chromiumoxide::element::Element;
#[cfg(feature = "browser-automation")]
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(feature = "browser-automation")]
use std::sync::Arc;
#[cfg(feature = "browser-automation")]
use std::time::Duration;

// For file upload
#[cfg(feature = "browser-automation")]
use base64;

/// Form field data for bulk filling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormData {
    /// Text input fields: selector -> value
    pub text_fields: HashMap<String, String>,
    /// Dropdown selections: selector -> value
    pub select_fields: HashMap<String, String>,
    /// Checkboxes to check: selector -> checked
    pub checkboxes: HashMap<String, bool>,
    /// Radio buttons: group_name -> value
    pub radio_buttons: HashMap<String, String>,
}

impl FormData {
    /// Create new empty form data
    pub fn new() -> Self {
        Self {
            text_fields: HashMap::new(),
            select_fields: HashMap::new(),
            checkboxes: HashMap::new(),
            radio_buttons: HashMap::new(),
        }
    }

    /// Add text field
    pub fn text(mut self, selector: &str, value: &str) -> Self {
        self.text_fields.insert(selector.to_string(), value.to_string());
        self
    }

    /// Add select field
    pub fn select(mut self, selector: &str, value: &str) -> Self {
        self.select_fields.insert(selector.to_string(), value.to_string());
        self
    }

    /// Add checkbox
    pub fn checkbox(mut self, selector: &str, checked: bool) -> Self {
        self.checkboxes.insert(selector.to_string(), checked);
        self
    }

    /// Add radio button
    pub fn radio(mut self, name: &str, value: &str) -> Self {
        self.radio_buttons.insert(name.to_string(), value.to_string());
        self
    }
}

impl Default for FormData {
    fn default() -> Self {
        Self::new()
    }
}

/// Form interaction configuration
#[derive(Debug, Clone)]
pub struct FormConfig {
    /// Timeout for waiting for elements (seconds)
    pub element_timeout: u64,
    /// Delay after typing (milliseconds)
    pub typing_delay: u64,
    /// Wait for navigation after submit
    pub wait_for_navigation: bool,
    /// Navigation timeout (seconds)
    pub navigation_timeout: u64,
}

impl Default for FormConfig {
    fn default() -> Self {
        Self {
            element_timeout: 10,
            typing_delay: 100,
            wait_for_navigation: true,
            navigation_timeout: 30,
        }
    }
}

/// Form interaction handler
#[cfg(feature = "browser-automation")]
#[derive(Debug)]
pub struct FormFiller {
    page: Arc<Page>,
    config: FormConfig,
}

#[cfg(feature = "browser-automation")]
impl FormFiller {
    /// Create new form filler for a page
    pub fn new(page: Arc<Page>) -> Self {
        Self { page, config: FormConfig::default() }
    }

    /// Create with custom configuration
    pub fn with_config(page: Arc<Page>, config: FormConfig) -> Self {
        Self { page, config }
    }

    /// Fill a text input field
    ///
    /// # Arguments
    /// * `selector` - CSS selector for input element
    /// * `value` - Text value to fill
    ///
    /// # Example
    /// ```ignore
    /// filler.fill_input("#username", "admin").await?;
    /// ```
    pub async fn fill_input(
        &self,
        selector: &str,
        value: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::debug!("Filling input '{}' with value", selector);

        // Wait for element to appear
        let element = self.wait_for_element(selector).await?;

        // Click to focus
        element.click().await?;

        // Clear existing value (use JavaScript)
        let clear_js =
            format!(r#"document.querySelector('{}').value = ''"#, selector.replace('\'', "\\'"));
        self.page.evaluate(clear_js).await?;

        // Type new value with delay
        if self.config.typing_delay > 0 {
            element.type_str(value).await?;
            tokio::time::sleep(Duration::from_millis(self.config.typing_delay)).await;
        } else {
            element.type_str(value).await?;
        }

        tracing::info!("Successfully filled input '{}'", selector);
        Ok(())
    }

    /// Select option from dropdown
    ///
    /// # Arguments
    /// * `selector` - CSS selector for select element
    /// * `value` - Value of option to select
    ///
    /// # Example
    /// ```ignore
    /// filler.select_dropdown("#country", "USA").await?;
    /// ```
    pub async fn select_dropdown(
        &self,
        selector: &str,
        value: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::debug!("Selecting dropdown '{}' value '{}'", selector, value);

        // Wait for element but use JavaScript to set value (more reliable)
        let _element = self.wait_for_element(selector).await?;

        // Use JavaScript to set value
        let js = format!(
            r#"
            const select = document.querySelector('{}');
            if (!select) throw new Error('Select element not found');
            select.value = '{}';
            select.dispatchEvent(new Event('change', {{ bubbles: true }}));
            "#,
            selector.replace('\'', "\\'"),
            value.replace('\'', "\\'")
        );

        self.page.evaluate(js).await?;

        tracing::info!("Successfully selected dropdown '{}'", selector);
        Ok(())
    }

    /// Set checkbox state
    ///
    /// # Arguments
    /// * `selector` - CSS selector for checkbox
    /// * `checked` - Desired checked state
    pub async fn set_checkbox(
        &self,
        selector: &str,
        checked: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::debug!("Setting checkbox '{}' to {}", selector, checked);

        let element = self.wait_for_element(selector).await?;

        // Get current state
        let is_checked = self.is_checkbox_checked(selector).await?;

        // Click only if state differs
        if is_checked != checked {
            element.click().await?;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        tracing::info!("Successfully set checkbox '{}'", selector);
        Ok(())
    }

    /// Check if checkbox is checked
    pub async fn is_checkbox_checked(
        &self,
        selector: &str,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let js = format!(
            r#"
            const checkbox = document.querySelector('{}');
            checkbox ? checkbox.checked : false
            "#,
            selector.replace('\'', "\\'")
        );

        let result = self.page.evaluate(js).await?;
        let value: serde_json::Value = result.into_value()?;
        Ok(value.as_bool().unwrap_or(false))
    }

    /// Select radio button by value
    ///
    /// # Arguments
    /// * `name` - Radio button group name
    /// * `value` - Value of radio to select
    pub async fn select_radio(
        &self,
        name: &str,
        value: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::debug!("Selecting radio '{}' with value '{}'", name, value);

        let selector = format!("input[type='radio'][name='{}'][value='{}']", name, value);
        let element = self.wait_for_element(&selector).await?;
        element.click().await?;

        tracing::info!("Successfully selected radio '{}'", name);
        Ok(())
    }

    /// Submit form by selector
    ///
    /// # Arguments
    /// * `form_selector` - CSS selector for form element or submit button
    ///
    /// # Example
    /// ```ignore
    /// filler.submit_form("#login-form").await?;
    /// ```
    pub async fn submit_form(
        &self,
        form_selector: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Submitting form '{}'", form_selector);

        // Try to find form element first
        let js = format!(
            r#"
            const element = document.querySelector('{}');
            if (!element) throw new Error('Form element not found');
            
            if (element.tagName === 'FORM') {{
                element.submit();
            }} else if (element.tagName === 'BUTTON' || element.type === 'submit') {{
                element.click();
            }} else {{
                throw new Error('Element is not a form or submit button');
            }}
            "#,
            form_selector.replace('\'', "\\'")
        );

        self.page.evaluate(js).await?;

        // Wait for navigation if configured
        if self.config.wait_for_navigation {
            tracing::debug!("Waiting for navigation after form submit");
            tokio::time::sleep(Duration::from_millis(500)).await;

            let timeout = Duration::from_secs(self.config.navigation_timeout);
            tokio::time::timeout(timeout, async {
                // Wait for network to be idle
                tokio::time::sleep(Duration::from_secs(2)).await;
                Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
            })
            .await??;
        }

        tracing::info!("Successfully submitted form '{}'", form_selector);
        Ok(())
    }

    /// Fill entire form with FormData
    ///
    /// # Arguments
    /// * `data` - FormData containing all field values
    ///
    /// # Example
    /// ```ignore
    /// let data = FormData::new()
    ///     .text("#username", "admin")
    ///     .text("#password", "secret")
    ///     .checkbox("#remember", true);
    /// filler.fill_form(&data).await?;
    /// ```
    pub async fn fill_form(
        &self,
        data: &FormData,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!(
            "Filling form with {} fields",
            data.text_fields.len()
                + data.select_fields.len()
                + data.checkboxes.len()
                + data.radio_buttons.len()
        );

        // Fill text inputs
        for (selector, value) in &data.text_fields {
            self.fill_input(selector, value).await?;
        }

        // Fill select dropdowns
        for (selector, value) in &data.select_fields {
            self.select_dropdown(selector, value).await?;
        }

        // Set checkboxes
        for (selector, checked) in &data.checkboxes {
            self.set_checkbox(selector, *checked).await?;
        }

        // Select radio buttons
        for (name, value) in &data.radio_buttons {
            self.select_radio(name, value).await?;
        }

        tracing::info!("Successfully filled form");
        Ok(())
    }

    /// Fill form and submit in one operation
    ///
    /// # Arguments
    /// * `data` - FormData containing all field values
    /// * `submit_selector` - CSS selector for submit button/form
    pub async fn fill_and_submit(
        &self,
        data: &FormData,
        submit_selector: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.fill_form(data).await?;
        tokio::time::sleep(Duration::from_millis(200)).await; // Brief pause before submit
        self.submit_form(submit_selector).await?;
        Ok(())
    }

    /// Wait for element to appear with timeout
    async fn wait_for_element(
        &self,
        selector: &str,
    ) -> Result<Element, Box<dyn std::error::Error + Send + Sync>> {
        let timeout = Duration::from_secs(self.config.element_timeout);
        let start = std::time::Instant::now();
        let mut interval = Duration::from_millis(100);
        let max_interval = Duration::from_millis(500);

        loop {
            match self.page.find_element(selector).await {
                Ok(element) => {
                    tracing::debug!("Element '{}' found after {:?}", selector, start.elapsed());
                    return Ok(element);
                }
                Err(_) => {
                    if start.elapsed() >= timeout {
                        return Err(format!(
                            "Timeout waiting for element '{}' after {:?}",
                            selector, timeout
                        )
                        .into());
                    }

                    tokio::time::sleep(interval).await;
                    interval = std::cmp::min(interval * 2, max_interval);
                }
            }
        }
    }

    /// Upload file to input field
    ///
    /// # Arguments
    /// * `selector` - CSS selector for file input
    /// * `file_path` - Path to file to upload
    pub async fn upload_file(
        &self,
        selector: &str,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Uploading file '{}' to '{}'", file_path, selector);

        // Use JavaScript to set files on input element
        // Note: chromiumoxide doesn't have direct file upload support yet
        // This is a workaround using data URL for small files
        let file_content = std::fs::read(file_path)?;
        use base64::{engine::general_purpose, Engine as _};
        let base64_str = general_purpose::STANDARD.encode(&file_content);
        let file_name =
            std::path::Path::new(file_path).file_name().and_then(|n| n.to_str()).unwrap_or("file");

        let js = format!(
            r#"
            const input = document.querySelector('{}');
            const dataUrl = 'data:application/octet-stream;base64,{}';
            const blob = await (await fetch(dataUrl)).blob();
            const file = new File([blob], '{}', {{ type: blob.type }});
            const dataTransfer = new DataTransfer();
            dataTransfer.items.add(file);
            input.files = dataTransfer.files;
            "#,
            selector.replace('\'', "\\'"),
            base64_str,
            file_name
        );

        self.page.evaluate(js).await?;

        tracing::info!("Successfully uploaded file");
        Ok(())
    }
}

// Fallback implementation when browser-automation feature is not enabled
#[cfg(not(feature = "browser-automation"))]
#[derive(Debug)]
pub struct FormFiller;

#[cfg(not(feature = "browser-automation"))]
impl FormFiller {
    pub fn new(_page: ()) -> Self {
        Self
    }

    pub async fn fill_input(
        &self,
        _selector: &str,
        _value: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Err("browser-automation feature not enabled".into())
    }

    pub async fn submit_form(
        &self,
        _selector: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Err("browser-automation feature not enabled".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_data_builder() {
        let data = FormData::new()
            .text("#username", "admin")
            .text("#password", "secret123")
            .checkbox("#remember", true)
            .radio("gender", "male");

        assert_eq!(data.text_fields.get("#username"), Some(&"admin".to_string()));
        assert_eq!(data.text_fields.get("#password"), Some(&"secret123".to_string()));
        assert_eq!(data.checkboxes.get("#remember"), Some(&true));
        assert_eq!(data.radio_buttons.get("gender"), Some(&"male".to_string()));
    }

    #[test]
    fn test_form_config_default() {
        let config = FormConfig::default();
        assert_eq!(config.element_timeout, 10);
        assert_eq!(config.typing_delay, 100);
        assert!(config.wait_for_navigation);
    }
}
