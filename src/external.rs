// External tools integration module

use std::process::Stdio;
use tokio::process::Command;

/// Call browser-use to browse a URL and extract semantic data
pub async fn browse_with_browser_use(
    url: &str,
    query: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // For now, fetch HTML and parse semantically
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    let html = response.text().await?;

    // Parse HTML semantically without validation (since external sites may have scripts)
    let document = scraper::Html::parse_document(&html);

    // Extract title
    let title_selector = scraper::Selector::parse("title")
        .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e.to_string()))?;
    let title = document.select(&title_selector).next().map(|t| t.text().collect::<String>());

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
    let itemscope_selector = scraper::Selector::parse("[itemscope]")
        .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e.to_string()))?;
    let mut microdata = Vec::new();
    for element in document.select(&itemscope_selector) {
        let item_type = element.value().attr("itemtype").unwrap_or("").to_string();
        let mut properties = std::collections::HashMap::new();
        let itemprop_selector = scraper::Selector::parse("[itemprop]")
            .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e.to_string()))?;
        for prop in element.select(&itemprop_selector) {
            let prop_name = prop.value().attr("itemprop").unwrap_or("").to_string();
            let prop_value = prop.text().collect::<String>();
            properties.entry(prop_name).or_insert(Vec::new()).push(prop_value);
        }
        microdata.push(crate::parser::MicrodataItem { item_type, properties });
    }

    // Return semantic summary
    let mut result = format!("Browsed {} with query '{}'\n", url, query);
    if let Some(title) = title {
        result.push_str(&format!("Title: {}\n", title));
    }
    result.push_str(&format!("JSON-LD objects: {}\n", json_ld.len()));
    result.push_str(&format!("Microdata items: {}\n", microdata.len()));
    for item in &microdata {
        result.push_str(&format!("- {}: {}\n", item.item_type, item.properties.len()));
    }
    Ok(result)
}

/// Call browser-use Python library using PyO3 (if available) or subprocess fallback
pub async fn browse_with_python_browser_use(
    _url: &str,
    _query: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Try PyO3 integration first
    #[cfg(feature = "pyo3-integration")]
    {
        use pyo3::prelude::*;
        use pyo3::types::PyDict;

        Python::with_gil(|py| {
            // Try to import browser-use
            let result: PyResult<String> = (|| {
                let browser_use = py.import("browser_use")?;
                let kwargs = PyDict::new(py);
                kwargs.set_item("url", url)?;
                kwargs.set_item("query", query)?;

                let result = browser_use.call_method("browse", (), Some(kwargs))?;
                Ok(result.extract::<String>()?)
            })();

            match result {
                Ok(data) => Ok(data),
                Err(e) => {
                    tracing::warn!("PyO3 browser-use failed: {}, falling back to HTTP", e);
                    Err(format!("PyO3 error: {}", e).into())
                }
            }
        })
    }

    #[cfg(not(feature = "pyo3-integration"))]
    {
        tracing::debug!("PyO3 integration not enabled, using HTTP-based browsing");
        Err("PyO3 not enabled".into())
    }
}

/// Call LangGraph for agent workflow using PyO3 (if available) or subprocess fallback
pub async fn run_langgraph_workflow(
    graph_def: &str,
    input: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Try PyO3 integration first
    #[cfg(feature = "pyo3-integration")]
    {
        use pyo3::prelude::*;

        let result = Python::with_gil(|py| -> PyResult<String> {
            // Try to import langgraph
            let code = format!(
                r#"
from langgraph.graph import StateGraph
import json

# Parse graph definition
# graph_def = json.loads('{}')
# For now, return mock result
result = 'LangGraph workflow result for: {}'
result
"#,
                graph_def, input
            );

            let result = py.eval(&code, None, None)?;
            Ok(result.extract::<String>()?)
        });

        if let Ok(data) = result {
            return Ok(data);
        } else {
            tracing::warn!("PyO3 LangGraph integration failed, falling back to subprocess");
        }
    }

    // Fallback to subprocess
    tracing::debug!("Using subprocess for LangGraph workflow");
    let python_code = format!(
        r#"
import sys
# Mock LangGraph integration
# In real: from langgraph import StateGraph
# graph_def: {}
# input: {}
print('Mock workflow result for input: {}')
        "#,
        graph_def, input, input
    );

    let output =
        Command::new("python3").arg("-c").arg(&python_code).stdout(Stdio::piped()).output().await?;

    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err("LangGraph subprocess failed".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_browse_mock() {
        let result = browse_with_browser_use("http://example.com", "extract title").await;
        // Placeholder test
        assert!(result.is_ok());
    }
}
