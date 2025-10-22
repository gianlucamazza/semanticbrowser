use crate::form_analyzer::FormAnalyzer;
use crate::llm::provider::{LLMError, LLMResult, ToolCall};
use crate::smart_form_filler::{AutoFillReport, SmartFormFiller};
use std::collections::HashMap;

#[cfg(feature = "browser-automation")]
use chromiumoxide::Page;
#[cfg(feature = "browser-automation")]
use std::sync::Arc;

/// Browser executor for LLM agents
///
/// Provides high-level browser operations that can be called by AI agents.
/// Integrates with SmartFormFiller for intelligent form filling.
#[cfg(feature = "browser-automation")]
pub struct BrowserExecutor {
    page: Arc<Page>,
    filler: Option<SmartFormFiller>,
}

#[cfg(feature = "browser-automation")]
impl BrowserExecutor {
    /// Create new browser executor
    pub async fn new(page: Arc<Page>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let filler = SmartFormFiller::new(page.clone()).await.ok();

        Ok(Self { page, filler })
    }

    /// Navigate to URL
    pub async fn navigate(
        &self,
        url: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Navigating to: {}", url);

        self.page.goto(url).await?;
        self.page.wait_for_navigation().await?;

        // Re-analyze page for forms after navigation
        if self.filler.is_some() {
            // Note: filler is immutable, would need refactoring for re-analysis
            tracing::debug!("Page loaded successfully");
        }

        Ok(format!("Successfully navigated to: {}", url))
    }

    /// Fill form fields intelligently
    pub async fn fill_form(
        &self,
        form_data: &HashMap<String, String>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let filler = self.filler.as_ref().ok_or("SmartFormFiller not available")?;

        let mut report = AutoFillReport::new();

        for (hint, value) in form_data {
            tracing::debug!("Filling field '{}' with value '{}'", hint, value);

            match filler.fill_field_smart(hint, value).await {
                Ok(result) => {
                    if result.success {
                        report.filled.push(hint.to_string());
                    } else {
                        report.failed.insert(
                            hint.to_string(),
                            result.error.unwrap_or_else(|| "Unknown error".to_string()),
                        );
                    }
                }
                Err(e) => {
                    report.failed.insert(hint.to_string(), e.to_string());
                }
            }
        }

        report.calculate_success_rate();

        Ok(format!(
            "Form filling complete: {} succeeded, {} failed (success rate: {:.1}%)",
            report.filled.len(),
            report.failed.len(),
            report.success_rate * 100.0
        ))
    }

    /// Click element by selector
    pub async fn click(
        &self,
        selector: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Clicking element: {}", selector);

        // Wait for element to be visible
        self.page.find_element(selector).await?.click().await?;

        Ok(format!("Clicked element: {}", selector))
    }

    /// Get page content (HTML)
    pub async fn get_content(
        &self,
        format: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        match format {
            "html" => {
                let html = self.page.content().await?;
                Ok(html)
            }
            "text" => {
                // Extract text content only
                let html = self.page.content().await?;
                // Simple text extraction (could be improved)
                let text = html.to_string();
                Ok(text)
            }
            _ => Err(format!("Unknown format: {}", format).into()),
        }
    }

    /// Extract structured data from page
    pub async fn extract_data(
        &self,
        selectors: &HashMap<String, String>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = HashMap::new();

        for (key, selector) in selectors {
            match self.page.find_element(selector).await {
                Ok(element) => {
                    if let Ok(Some(text)) = element.inner_text().await {
                        results.insert(key.clone(), text);
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to extract '{}' with selector '{}': {}",
                        key,
                        selector,
                        e
                    );
                }
            }
        }

        Ok(serde_json::to_string_pretty(&results)?)
    }

    /// Take screenshot
    pub async fn screenshot(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Taking screenshot");

        let screenshot =
            self.page.screenshot(chromiumoxide::page::ScreenshotParams::builder().build()).await?;

        Ok(screenshot)
    }

    /// Wait for element to appear
    pub async fn wait_for_element(
        &self,
        selector: &str,
        timeout_ms: u64,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        tracing::debug!("Waiting for element: {}", selector);

        let timeout = std::time::Duration::from_millis(timeout_ms);

        tokio::time::timeout(timeout, async {
            loop {
                if self.page.find_element(selector).await.is_ok() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        })
        .await?;

        Ok(format!("Element '{}' appeared", selector))
    }

    /// Check if element exists on the page
    pub async fn element_exists(&self, selector: &str) -> bool {
        self.page.find_element(selector).await.is_ok()
    }

    /// Get current URL
    pub async fn current_url(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.page.url().await?.unwrap_or_default())
    }

    /// Get page title
    pub async fn page_title(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let title = self.page.get_title().await?.unwrap_or_default();
        Ok(title)
    }

    /// Analyze forms on the current page
    pub async fn analyze_forms(
        &self,
        form_index: Option<usize>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let html = self.page.content().await?;
        let forms = FormAnalyzer::analyze_html(&html);

        if forms.is_empty() {
            return Ok("No forms found on the current page".to_string());
        }

        if let Some(index) = form_index {
            if index >= forms.len() {
                return Err(format!(
                    "Form index {} out of range ({} forms found)",
                    index,
                    forms.len()
                )
                .into());
            }
            let form = &forms[index];
            Ok(serde_json::to_string_pretty(form)?)
        } else {
            // Return summary of all forms
            let summary: Vec<serde_json::Value> = forms
                .iter()
                .enumerate()
                .map(|(i, form)| {
                    serde_json::json!({
                        "index": i,
                        "purpose": format!("{:?}", form.purpose),
                        "field_count": form.fields.len(),
                        "action": form.action,
                        "method": form.method,
                        "submit_button": form.submit_button
                    })
                })
                .collect();
            Ok(serde_json::to_string_pretty(&summary)?)
        }
    }

    /// Get all form fields with metadata
    pub async fn get_form_fields(
        &self,
        form_index: Option<usize>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let html = self.page.content().await?;
        let forms = FormAnalyzer::analyze_html(&html);

        if forms.is_empty() {
            return Ok("No forms found on the current page".to_string());
        }

        let target_forms = if let Some(index) = form_index {
            if index >= forms.len() {
                return Err(format!(
                    "Form index {} out of range ({} forms found)",
                    index,
                    forms.len()
                )
                .into());
            }
            vec![&forms[index]]
        } else {
            forms.iter().collect()
        };

        let mut result = Vec::new();
        for (form_idx, form) in target_forms.iter().enumerate() {
            for field in &form.fields {
                result.push(serde_json::json!({
                    "form_index": if form_index.is_some() { form_index.unwrap() } else { form_idx },
                    "selector": field.selector,
                    "field_type": format!("{:?}", field.field_type),
                    "label": field.label,
                    "placeholder": field.placeholder,
                    "name": field.name,
                    "id": field.id,
                    "required": field.required,
                    "confidence": field.confidence
                }));
            }
        }

        Ok(serde_json::to_string_pretty(&result)?)
    }

    /// Auto-fill entire form
    pub async fn auto_fill_entire_form(
        &self,
        form_data: &HashMap<String, String>,
        _form_index: Option<usize>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let filler = self.filler.as_ref().ok_or("SmartFormFiller not available")?;

        let report = filler.auto_fill_form(form_data.clone()).await?;

        Ok(format!(
            "Auto-fill complete: {}/{} fields filled successfully (success rate: {:.1}%)\nFilled: {}\nNot found: {}\nFailed: {}",
            report.filled.len(),
            form_data.len(),
            report.success_rate * 100.0,
            report.filled.join(", "),
            report.not_found.join(", "),
            report.failed.iter().map(|(k, v)| format!("{}: {}", k, v)).collect::<Vec<_>>().join("; ")
        ))
    }

    /// Submit form intelligently
    pub async fn submit_form(
        &self,
        form_selector: Option<&str>,
        form_index: Option<usize>,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // If explicit selector provided, use it
        if let Some(selector) = form_selector {
            return self.click(selector).await;
        }

        // Otherwise, try to find submit button from form analysis
        let html = self.page.content().await?;
        let forms = FormAnalyzer::analyze_html(&html);

        if forms.is_empty() {
            return Err("No forms found on the current page".into());
        }

        let target_form = if let Some(index) = form_index {
            if index >= forms.len() {
                return Err(format!(
                    "Form index {} out of range ({} forms found)",
                    index,
                    forms.len()
                )
                .into());
            }
            &forms[index]
        } else {
            &forms[0] // Use first form
        };

        if let Some(submit_selector) = &target_form.submit_button {
            tracing::info!("Submitting form using detected submit button: {}", submit_selector);
            self.click(submit_selector).await
        } else {
            // Try to find any submit button in the form
            let form_selector = &target_form.selector;
            let js = format!(
                r#"
                const form = document.querySelector('{}');
                if (!form) throw new Error('Form not found');

                // Try input[type="submit"]
                let submit = form.querySelector('input[type="submit"]');
                if (submit) {{
                    submit.click();
                    return 'Clicked input[type="submit"]';
                }}

                // Try button[type="submit"]
                submit = form.querySelector('button[type="submit"]');
                if (submit) {{
                    submit.click();
                    return 'Clicked button[type="submit"]';
                }}

                // Try any button
                submit = form.querySelector('button');
                if (submit) {{
                    submit.click();
                    return 'Clicked button';
                }}

                throw new Error('No submit button found in form');
                "#,
                form_selector.replace('\'', "\\'")
            );

            self.page.evaluate(js).await?;
            Ok("Form submitted using detected submit button".to_string())
        }
    }

    /// Execute a tool call from LLM
    pub async fn execute_tool(&self, tool_call: &ToolCall) -> LLMResult<String> {
        match tool_call.function.name.as_str() {
            "navigate_to" => self.execute_navigate_to(tool_call).await,
            "click_element" => self.execute_click_element(tool_call).await,
            "fill_form_field" => self.execute_fill_form_field(tool_call).await,
            "extract_text" => self.execute_extract_text(tool_call).await,
            "get_page_content" => self.execute_get_page_content(tool_call).await,
            "wait_for_element" => self.execute_wait_for_element(tool_call).await,
            "get_current_url" => self.execute_get_current_url(tool_call).await,
            "get_page_title" => self.execute_get_page_title(tool_call).await,
            "analyze_form" => self.execute_analyze_form(tool_call).await,
            "auto_fill_form" => self.execute_auto_fill_form(tool_call).await,
            "submit_form" => self.execute_submit_form(tool_call).await,
            "get_form_fields" => self.execute_get_form_fields(tool_call).await,
            _ => Err(LLMError::Api(format!("Unknown browser tool: {}", tool_call.function.name))),
        }
    }

    async fn execute_navigate_to(&self, tool_call: &ToolCall) -> LLMResult<String> {
        let args: serde_json::Value =
            serde_json::from_str(&tool_call.function.arguments).map_err(|e| {
                LLMError::InvalidResponse(format!("Invalid navigate_to arguments: {}", e))
            })?;

        let url = args
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LLMError::InvalidResponse("Missing 'url' parameter".to_string()))?;

        self.navigate(url).await.map_err(|e| LLMError::Api(format!("Navigation failed: {}", e)))
    }

    async fn execute_click_element(&self, tool_call: &ToolCall) -> LLMResult<String> {
        let args: serde_json::Value =
            serde_json::from_str(&tool_call.function.arguments).map_err(|e| {
                LLMError::InvalidResponse(format!("Invalid click_element arguments: {}", e))
            })?;

        let selector = args
            .get("selector")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LLMError::InvalidResponse("Missing 'selector' parameter".to_string()))?;

        self.click(selector).await.map_err(|e| LLMError::Api(format!("Click failed: {}", e)))
    }

    async fn execute_fill_form_field(&self, tool_call: &ToolCall) -> LLMResult<String> {
        let args: serde_json::Value =
            serde_json::from_str(&tool_call.function.arguments).map_err(|e| {
                LLMError::InvalidResponse(format!("Invalid fill_form_field arguments: {}", e))
            })?;

        let field_name = args.get("field_name").and_then(|v| v.as_str()).ok_or_else(|| {
            LLMError::InvalidResponse("Missing 'field_name' parameter".to_string())
        })?;

        let value = args
            .get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LLMError::InvalidResponse("Missing 'value' parameter".to_string()))?;

        let form_data = HashMap::from([(field_name.to_string(), value.to_string())]);

        self.fill_form(&form_data)
            .await
            .map_err(|e| LLMError::Api(format!("Form filling failed: {}", e)))
    }

    async fn execute_extract_text(&self, tool_call: &ToolCall) -> LLMResult<String> {
        let args: serde_json::Value =
            serde_json::from_str(&tool_call.function.arguments).map_err(|e| {
                LLMError::InvalidResponse(format!("Invalid extract_text arguments: {}", e))
            })?;

        let selector = args
            .get("selector")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LLMError::InvalidResponse("Missing 'selector' parameter".to_string()))?;

        let selectors = HashMap::from([("text".to_string(), selector.to_string())]);

        self.extract_data(&selectors)
            .await
            .map_err(|e| LLMError::Api(format!("Text extraction failed: {}", e)))
    }

    async fn execute_get_page_content(&self, tool_call: &ToolCall) -> LLMResult<String> {
        let args: serde_json::Value =
            serde_json::from_str(&tool_call.function.arguments).map_err(|e| {
                LLMError::InvalidResponse(format!("Invalid get_page_content arguments: {}", e))
            })?;

        let format = args.get("format").and_then(|v| v.as_str()).unwrap_or("text");

        self.get_content(format)
            .await
            .map_err(|e| LLMError::Api(format!("Content extraction failed: {}", e)))
    }

    async fn execute_wait_for_element(&self, tool_call: &ToolCall) -> LLMResult<String> {
        let args: serde_json::Value =
            serde_json::from_str(&tool_call.function.arguments).map_err(|e| {
                LLMError::InvalidResponse(format!("Invalid wait_for_element arguments: {}", e))
            })?;

        let selector = args
            .get("selector")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LLMError::InvalidResponse("Missing 'selector' parameter".to_string()))?;

        let timeout_ms = args.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(5000);

        self.wait_for_element(selector, timeout_ms)
            .await
            .map_err(|e| LLMError::Api(format!("Wait for element failed: {}", e)))
    }

    async fn execute_get_current_url(&self, _tool_call: &ToolCall) -> LLMResult<String> {
        self.current_url()
            .await
            .map_err(|e| LLMError::Api(format!("Getting current URL failed: {}", e)))
    }

    async fn execute_get_page_title(&self, _tool_call: &ToolCall) -> LLMResult<String> {
        self.page_title()
            .await
            .map_err(|e| LLMError::Api(format!("Getting page title failed: {}", e)))
    }

    async fn execute_analyze_form(&self, tool_call: &ToolCall) -> LLMResult<String> {
        let args: serde_json::Value =
            serde_json::from_str(&tool_call.function.arguments).map_err(|e| {
                LLMError::InvalidResponse(format!("Invalid analyze_form arguments: {}", e))
            })?;

        let form_index = args.get("form_index").and_then(|v| v.as_u64()).map(|v| v as usize);

        self.analyze_forms(form_index)
            .await
            .map_err(|e| LLMError::Api(format!("Form analysis failed: {}", e)))
    }

    async fn execute_auto_fill_form(&self, tool_call: &ToolCall) -> LLMResult<String> {
        let args: serde_json::Value =
            serde_json::from_str(&tool_call.function.arguments).map_err(|e| {
                LLMError::InvalidResponse(format!("Invalid auto_fill_form arguments: {}", e))
            })?;

        let form_data_value = args.get("form_data").ok_or_else(|| {
            LLMError::InvalidResponse("Missing 'form_data' parameter".to_string())
        })?;

        let form_data: HashMap<String, String> = serde_json::from_value(form_data_value.clone())
            .map_err(|e| LLMError::InvalidResponse(format!("Invalid form_data format: {}", e)))?;

        let form_index = args.get("form_index").and_then(|v| v.as_u64()).map(|v| v as usize);

        self.auto_fill_entire_form(&form_data, form_index)
            .await
            .map_err(|e| LLMError::Api(format!("Auto-fill form failed: {}", e)))
    }

    async fn execute_submit_form(&self, tool_call: &ToolCall) -> LLMResult<String> {
        let args: serde_json::Value =
            serde_json::from_str(&tool_call.function.arguments).map_err(|e| {
                LLMError::InvalidResponse(format!("Invalid submit_form arguments: {}", e))
            })?;

        let form_selector = args.get("form_selector").and_then(|v| v.as_str());

        let form_index = args.get("form_index").and_then(|v| v.as_u64()).map(|v| v as usize);

        self.submit_form(form_selector, form_index)
            .await
            .map_err(|e| LLMError::Api(format!("Form submission failed: {}", e)))
    }

    async fn execute_get_form_fields(&self, tool_call: &ToolCall) -> LLMResult<String> {
        let args: serde_json::Value =
            serde_json::from_str(&tool_call.function.arguments).map_err(|e| {
                LLMError::InvalidResponse(format!("Invalid get_form_fields arguments: {}", e))
            })?;

        let form_index = args.get("form_index").and_then(|v| v.as_u64()).map(|v| v as usize);

        self.get_form_fields(form_index)
            .await
            .map_err(|e| LLMError::Api(format!("Getting form fields failed: {}", e)))
    }
}
