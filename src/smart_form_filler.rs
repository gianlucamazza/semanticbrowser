//! Smart Form Filler Module
//!
//! Intelligent form filling with automatic field discovery.
//! No hardcoded selectors required - uses semantic analysis.
#[cfg(feature = "browser-automation")]
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(feature = "browser-automation")]
use std::sync::Arc;
#[cfg(feature = "browser-automation")]
use std::time::Duration;

/// Result of smart field filling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldFillResult {
    /// Selector that was used
    pub selector_used: String,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
    /// Alternative selectors that could be tried
    pub alternatives: Vec<String>,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Report of automatic form filling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoFillReport {
    /// Fields that were successfully filled
    pub filled: Vec<String>,
    /// Fields that were not found
    pub not_found: Vec<String>,
    /// Fields that failed to fill
    pub failed: HashMap<String, String>,
    /// Overall success rate
    pub success_rate: f32,
}

impl AutoFillReport {
    pub fn new() -> Self {
        Self {
            filled: Vec::new(),
            not_found: Vec::new(),
            failed: HashMap::new(),
            success_rate: 0.0,
        }
    }

    pub fn calculate_success_rate(&mut self) {
        let total = self.filled.len() + self.not_found.len() + self.failed.len();
        if total > 0 {
            self.success_rate = self.filled.len() as f32 / total as f32;
        }
    }
}

impl Default for AutoFillReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Smart form filler with auto-discovery
#[cfg(feature = "browser-automation")]
pub struct SmartFormFiller {
    page: Arc<Page>,
    forms: Vec<FormDescription>,
    confidence_threshold: f32,
}

#[cfg(feature = "browser-automation")]
impl SmartFormFiller {
    /// Create new smart form filler
    pub async fn new(page: Arc<Page>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut filler = Self { page, forms: Vec::new(), confidence_threshold: 0.3 };
        filler.analyze_page().await?;
        Ok(filler)
    }

    /// Set confidence threshold for matching (0.0-1.0)
    pub fn with_confidence_threshold(mut self, threshold: f32) -> Self {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Analyze current page to discover forms
    pub async fn analyze_page(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Get page HTML
        let html = self.page.content().await?;

        // Analyze forms
        self.forms = FormAnalyzer::analyze_html(&html);

        tracing::info!("Discovered {} forms on page", self.forms.len());
        Ok(())
    }

    /// Fill field by semantic hint (auto-discovers the selector)
    pub async fn fill_field_smart(
        &self,
        hint: &str,
        value: &str,
    ) -> Result<FieldFillResult, Box<dyn std::error::Error + Send + Sync>> {
        tracing::debug!("Smart filling field with hint '{}' value '{}'", hint, value);

        // Find best matching field across all forms
        let mut best_field: Option<&FieldDescription> = None;
        let mut best_score = 0.0f32;

        for form in &self.forms {
            for field in &form.fields {
                let score = field.similarity_score(hint);
                if score > best_score {
                    best_score = score;
                    best_field = Some(field);
                }
            }
        }

        // Check if we found a good match
        if best_score < self.confidence_threshold {
            return Ok(FieldFillResult {
                selector_used: String::new(),
                confidence: best_score,
                alternatives: best_field.map(|f| f.alternatives.clone()).unwrap_or_default(),
                success: false,
                error: Some(format!(
                    "No field found with confidence > {} for hint '{}'",
                    self.confidence_threshold, hint
                )),
            });
        }

        let field = best_field.unwrap();

        // Try to fill the field
        match self.fill_element(&field.selector, value).await {
            Ok(_) => {
                tracing::info!(
                    "Successfully filled field '{}' (confidence: {:.2})",
                    hint,
                    best_score
                );
                Ok(FieldFillResult {
                    selector_used: field.selector.clone(),
                    confidence: best_score,
                    alternatives: field.alternatives.clone(),
                    success: true,
                    error: None,
                })
            }
            Err(e) => {
                tracing::warn!("Failed to fill field '{}': {}", hint, e);
                Ok(FieldFillResult {
                    selector_used: field.selector.clone(),
                    confidence: best_score,
                    alternatives: field.alternatives.clone(),
                    success: false,
                    error: Some(e.to_string()),
                })
            }
        }
    }

    /// Fill field by label text
    pub async fn fill_field_by_label(
        &self,
        label_text: &str,
        value: &str,
    ) -> Result<FieldFillResult, Box<dyn std::error::Error + Send + Sync>> {
        tracing::debug!("Filling field by label '{}'", label_text);

        let label_lower = label_text.to_lowercase();

        // Find field with matching label
        for form in &self.forms {
            for field in &form.fields {
                if let Some(ref label) = field.label {
                    if label.as_str().to_lowercase().contains(&label_lower) {
                        return match self.fill_element(&field.selector, value).await {
                            Ok(_) => Ok(FieldFillResult {
                                selector_used: field.selector.clone(),
                                confidence: 1.0,
                                alternatives: field.alternatives.clone(),
                                success: true,
                                error: None,
                            }),
                            Err(e) => Ok(FieldFillResult {
                                selector_used: field.selector.clone(),
                                confidence: 1.0,
                                alternatives: field.alternatives.clone(),
                                success: false,
                                error: Some(e.to_string()),
                            }),
                        };
                    }
                }
            }
        }

        Ok(FieldFillResult {
            selector_used: String::new(),
            confidence: 0.0,
            alternatives: vec![],
            success: false,
            error: Some(format!("No field found with label '{}'", label_text)),
        })
    }

    /// Fill field by placeholder text
    pub async fn fill_field_by_placeholder(
        &self,
        placeholder_text: &str,
        value: &str,
    ) -> Result<FieldFillResult, Box<dyn std::error::Error + Send + Sync>> {
        tracing::debug!("Filling field by placeholder '{}'", placeholder_text);

        let placeholder_lower = placeholder_text.to_lowercase();

        for form in &self.forms {
            for field in &form.fields {
                if let Some(ref placeholder) = field.placeholder {
                    if placeholder.as_str().to_lowercase().contains(&placeholder_lower) {
                        return match self.fill_element(&field.selector, value).await {
                            Ok(_) => Ok(FieldFillResult {
                                selector_used: field.selector.clone(),
                                confidence: 0.9,
                                alternatives: field.alternatives.clone(),
                                success: true,
                                error: None,
                            }),
                            Err(e) => Ok(FieldFillResult {
                                selector_used: field.selector.clone(),
                                confidence: 0.9,
                                alternatives: field.alternatives.clone(),
                                success: false,
                                error: Some(e.to_string()),
                            }),
                        };
                    }
                }
            }
        }

        Ok(FieldFillResult {
            selector_used: String::new(),
            confidence: 0.0,
            alternatives: vec![],
            success: false,
            error: Some(format!("No field found with placeholder '{}'", placeholder_text)),
        })
    }

    /// Fill field by type
    pub async fn fill_field_by_type(
        &self,
        field_type: FieldType,
        value: &str,
    ) -> Result<FieldFillResult, Box<dyn std::error::Error + Send + Sync>> {
        tracing::debug!("Filling field by type {:?}", field_type);

        // Find first field of matching type
        for form in &self.forms {
            for field in &form.fields {
                if field.field_type == field_type {
                    return match self.fill_element(&field.selector, value).await {
                        Ok(_) => Ok(FieldFillResult {
                            selector_used: field.selector.clone(),
                            confidence: 0.8,
                            alternatives: field.alternatives.clone(),
                            success: true,
                            error: None,
                        }),
                        Err(e) => Ok(FieldFillResult {
                            selector_used: field.selector.clone(),
                            confidence: 0.8,
                            alternatives: field.alternatives.clone(),
                            success: false,
                            error: Some(e.to_string()),
                        }),
                    };
                }
            }
        }

        Ok(FieldFillResult {
            selector_used: String::new(),
            confidence: 0.0,
            alternatives: vec![],
            success: false,
            error: Some(format!("No field found with type {:?}", field_type)),
        })
    }

    /// Automatically fill entire form
    pub async fn auto_fill_form(
        &self,
        data: HashMap<String, String>,
    ) -> Result<AutoFillReport, Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Auto-filling form with {} fields", data.len());

        let mut report = AutoFillReport::new();

        for (hint, value) in &data {
            let result = self.fill_field_smart(hint, value).await?;

            if result.success {
                report.filled.push(hint.clone());
            } else if result.confidence < self.confidence_threshold {
                report.not_found.push(hint.clone());
            } else {
                report.failed.insert(hint.clone(), result.error.unwrap_or_default());
            }
        }

        report.calculate_success_rate();
        tracing::info!(
            "Auto-fill complete: {}/{} fields filled (success rate: {:.1}%)",
            report.filled.len(),
            data.len(),
            report.success_rate * 100.0
        );

        Ok(report)
    }

    /// Fill element by selector (internal helper)
    async fn fill_element(
        &self,
        selector: &str,
        value: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Wait for element
        let element = self.wait_for_element(selector).await?;

        // Click to focus
        element.click().await?;

        // Clear existing value
        let clear_js =
            format!(r#"document.querySelector('{}').value = ''"#, selector.replace('\'', "\\'"));
        self.page.evaluate(clear_js).await?;

        // Type new value
        element.type_str(value).await?;

        // Small delay for JS to process
        tokio::time::sleep(Duration::from_millis(100)).await;

        Ok(())
    }

    /// Wait for element to appear
    async fn wait_for_element(
        &self,
        selector: &str,
    ) -> Result<chromiumoxide::element::Element, Box<dyn std::error::Error + Send + Sync>> {
        let timeout = Duration::from_secs(10);
        let start = std::time::Instant::now();

        loop {
            match self.page.find_element(selector).await {
                Ok(element) => return Ok(element),
                Err(_) => {
                    if start.elapsed() >= timeout {
                        return Err(format!(
                            "Element '{}' not found after {:?}",
                            selector, timeout
                        )
                        .into());
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Get all discovered forms
    pub fn get_forms(&self) -> &[FormDescription] {
        &self.forms
    }

    /// Get form by index
    pub fn get_form(&self, index: usize) -> Option<&FormDescription> {
        self.forms.get(index)
    }
}

// Fallback for non-browser features
#[cfg(not(feature = "browser-automation"))]
pub struct SmartFormFiller;

#[cfg(not(feature = "browser-automation"))]
impl SmartFormFiller {
    pub async fn new(_page: ()) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Err("browser-automation feature not enabled".into())
    }

    pub async fn fill_field_smart(
        &self,
        _hint: &str,
        _value: &str,
    ) -> Result<FieldFillResult, Box<dyn std::error::Error + Send + Sync>> {
        Err("browser-automation feature not enabled".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autofill_report() {
        let mut report = AutoFillReport::new();
        report.filled.push("username".to_string());
        report.filled.push("password".to_string());
        report.not_found.push("phone".to_string());

        report.calculate_success_rate();
        assert_eq!(report.success_rate, 2.0 / 3.0);
    }
}
