use scraper::{Html, Selector};

/// Represents parsed semantic data from HTML
#[derive(Debug)]
pub struct SemanticData {
    pub title: Option<String>,
    pub microdata: Vec<MicrodataItem>,
    pub json_ld: Vec<serde_json::Value>,
}

/// Microdata item
#[derive(Debug)]
pub struct MicrodataItem {
    pub item_type: String,
    pub properties: std::collections::HashMap<String, Vec<String>>,
}

/// Parse HTML content and extract semantic elements
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

/// Extract JSON-LD scripts
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

/// Extract microdata (basic implementation)
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
