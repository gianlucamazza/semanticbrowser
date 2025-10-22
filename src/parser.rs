use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

/// Container for semantic data extracted from HTML content.
/// This struct holds the results of parsing HTML for structured data including
/// page titles, microdata items, and JSON-LD objects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticData {
    /// The page title extracted from the HTML <title> tag.
    /// None if no title tag is found.
    pub title: Option<String>,
    /// List of microdata items found in the HTML.
    /// Microdata provides structured data using schema.org vocabulary.
    pub microdata: Vec<MicrodataItem>,
    /// List of JSON-LD objects extracted from script tags.
    /// JSON-LD is a structured data format commonly used for SEO and rich snippets.
    pub json_ld: Vec<serde_json::Value>,
}

/// Represents a single microdata item extracted from HTML.
/// Microdata items are defined using itemscope and itemprop attributes,
/// following the schema.org vocabulary for structured data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrodataItem {
    /// The item type URL (e.g., "https://schema.org/Person").
    /// Specifies the type of entity being described.
    pub item_type: String,
    /// Map of property names to their values.
    /// Each property can have multiple values if multiple elements share the same itemprop.
    pub properties: std::collections::HashMap<String, Vec<String>>,
}

/// Parses HTML content and extracts semantic elements including titles, microdata, and JSON-LD.
/// This function performs security validation on the input HTML before parsing.
/// It uses the scraper crate for HTML parsing and CSS selector matching.
///
/// # Arguments
/// * `html` - The HTML content to parse as a string
///
/// # Returns
/// A Result containing SemanticData with extracted information, or an error if parsing fails.
///
/// # Security
/// Input HTML is validated through the security module before processing.
/// All parsing operations are logged for observability.
///
/// # Examples
/// ```
/// let html = r#"<html><head><title>Test</title></head><body></body></html>"#;
/// let data = parse_html(html).unwrap();
/// assert_eq!(data.title, Some("Test".to_string()));
/// ```
pub fn parse_html(html: &str) -> Result<SemanticData, Box<dyn std::error::Error + Send + Sync>> {
    crate::security::validate_html_input(html)?;
    crate::security::log_action("parse_html", &format!("length: {}", html.len()));
    let document = Html::parse_document(html);

    // Extract title
    let title_selector = Selector::parse("title")
        .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e.to_string()))?;
    let title = document.select(&title_selector).next().map(|t| t.text().collect::<String>());

    // Extract JSON-LD
    let json_ld = extract_json_ld(&document)?;

    // Extract microdata (simplified)
    let microdata = extract_microdata(&document)?;

    Ok(SemanticData { title, microdata, json_ld })
}

/// Extracts JSON-LD (JSON for Linking Data) structured data from HTML script tags.
/// JSON-LD is a method of encoding linked data using JSON, commonly used for
/// SEO rich snippets, schema.org markup, and semantic web applications.
///
/// This function searches for all script tags with type="application/ld+json"
/// and attempts to parse their content as JSON.
///
/// # Arguments
/// * `document` - Parsed HTML document from the scraper crate
///
/// # Returns
/// A vector of parsed JSON values, or an error if parsing fails.
///
/// # Note
/// Invalid JSON in script tags will cause the entire extraction to fail.
/// In production, consider more robust error handling for individual scripts.
fn extract_json_ld(
    document: &Html,
) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error + Send + Sync>> {
    let selector = Selector::parse("script[type=\"application/ld+json\"]")
        .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e.to_string()))?;
    let mut json_ld = Vec::new();

    for element in document.select(&selector) {
        if let Some(text) = element.text().next() {
            let value: serde_json::Value = serde_json::from_str(text)?;
            json_ld.push(value);
        }
    }

    Ok(json_ld)
}

/// Extracts microdata items from HTML using the microdata specification.
/// Microdata allows embedding structured data in HTML using attributes like
/// itemscope, itemtype, and itemprop. This follows the schema.org vocabulary.
///
/// This is a simplified implementation that:
/// - Finds all elements with itemscope attribute
/// - Extracts the itemtype
/// - Collects all itemprop values within the itemscope scope
///
/// # Arguments
/// * `document` - Parsed HTML document from the scraper crate
///
/// # Returns
/// A vector of MicrodataItem structs representing the extracted structured data.
///
/// # Limitations
/// - Does not handle nested microdata scopes properly
/// - Simple text extraction for property values (no URL or date parsing)
/// - Does not validate schema.org compliance
///
/// # Note
/// This is a basic implementation. For production use, consider a more robust
/// microdata parser that handles the full specification.
fn extract_microdata(
    document: &Html,
) -> Result<Vec<MicrodataItem>, Box<dyn std::error::Error + Send + Sync>> {
    // Simplified: look for itemscope
    let selector = Selector::parse("[itemscope]")
        .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e.to_string()))?;
    let mut items = Vec::new();

    for element in document.select(&selector) {
        let item_type = element.value().attr("itemtype").unwrap_or("").to_string();
        let mut properties = std::collections::HashMap::new();

        // Find itemprop within
        let prop_selector = Selector::parse("[itemprop]")
            .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e.to_string()))?;
        for prop in element.select(&prop_selector) {
            let prop_name = prop.value().attr("itemprop").unwrap_or("").to_string();
            let prop_value = prop.text().collect::<String>();
            properties.entry(prop_name).or_insert(Vec::new()).push(prop_value);
        }

        items.push(MicrodataItem { item_type, properties });
    }

    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_html_basic() {
        let html = r#"
        <html>
        <head><title>Test Page</title></head>
        <body>
        <h1>Hello World</h1>
        <script type="application/ld+json">{"@type": "Person", "name": "John"}</script>
        </body>
        </html>
        "#;
        let result = parse_html(html).unwrap();
        assert_eq!(result.title, Some("Test Page".to_string()));
        assert_eq!(result.json_ld.len(), 1);
    }
}
