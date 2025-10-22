// External tools integration module

use crate::models::{BrowseOutcome, MicrodataSummary, QueryMatch, SemanticSnapshot};
use futures::future::BoxFuture;
use serde_json;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::process::Stdio;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

#[cfg(feature = "browser-automation")]
use std::sync::Arc;
#[cfg(feature = "browser-automation")]
use tokio::sync::OnceCell;

// Global browser pool instance (initialized once)
#[cfg(feature = "browser-automation")]
static BROWSER_POOL: OnceCell<Arc<crate::browser::BrowserPool>> = OnceCell::const_new();

/// Initialize browser pool from environment configuration
#[cfg(feature = "browser-automation")]
async fn get_browser_pool(
) -> Result<Arc<crate::browser::BrowserPool>, Box<dyn std::error::Error + Send + Sync>> {
    BROWSER_POOL
        .get_or_try_init(|| async {
            let config = crate::browser::BrowserConfig::from_env();
            let pool = crate::browser::BrowserPool::new(config).await?;
            Ok(Arc::new(pool))
        })
        .await
        .map(Arc::clone)
}

/// Browse URL with chromiumoxide headless browser (primary method)
///
/// This is the primary browsing method when browser-automation feature is enabled.
/// Provides full JavaScript support, cookie management, and semantic data extraction.
#[cfg(feature = "browser-automation")]
pub async fn browse_with_chromium(
    url: &str,
    query: &str,
) -> Result<BrowseOutcome, Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Browsing with chromiumoxide: {} (query: {})", url, query);

    let pool = get_browser_pool().await?;
    let options = crate::browser::NavigationOptions::default();

    let semantic_data = pool.navigate_and_extract(url, options).await?;

    let microdata: Vec<MicrodataSummary> = semantic_data
        .microdata
        .iter()
        .map(|item| MicrodataSummary {
            item_type: item.item_type.clone(),
            properties: item.properties.len(),
        })
        .collect();

    let snapshot = SemanticSnapshot {
        title: semantic_data.title.clone(),
        description: semantic_data.meta_description.clone(),
        language: semantic_data.language.clone(),
        canonical_url: semantic_data.canonical_url.clone(),
        final_url: semantic_data.final_url.clone(),
        keywords: semantic_data.meta_keywords.clone(),
        open_graph: semantic_data.open_graph.clone(),
        twitter_card: semantic_data.twitter_card.clone(),
        json_ld_count: semantic_data.json_ld.len(),
        microdata,
        text_preview: build_text_preview(&semantic_data.text_content),
        text_length: semantic_data.text_content.len(),
        query_matches: build_query_matches_from_text(&semantic_data.text_content, query),
    };

    let summary = build_summary(url, query, &snapshot);

    Ok(BrowseOutcome { summary, snapshot })
}

/// Browse URL with chromiumoxide and return full semantic data
///
/// This method returns the complete SemanticData structure for direct integration
/// with Knowledge Graph and NER processing.
#[cfg(feature = "browser-automation")]
pub async fn browse_with_chromium_full(
    url: &str,
    options: crate::browser::NavigationOptions,
) -> Result<crate::browser::SemanticData, Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Browsing with chromiumoxide (full data): {}", url);

    let pool = get_browser_pool().await?;
    pool.navigate_and_extract(url, options).await
}

/// Browse URL and insert extracted data into Knowledge Graph (2025 best practice)
///
/// This is the recommended method for AI agents that want to:
/// - Browse web pages
/// - Extract semantic metadata (Phase 1: meta tags, Open Graph, Twitter Cards)
/// - Populate Knowledge Graph with RDF triples
/// - Enable SPARQL queries over extracted data
///
/// # Parameters
/// - `url`: Target URL to browse
/// - `options`: Navigation options (cookies, screenshots, etc.)
/// - `kg`: Mutable reference to Knowledge Graph
///
/// # Returns
/// Tuple of (SemanticData, triples_count)
///
/// # Example
/// ```ignore
/// use semantic_browser::kg::KnowledgeGraph;
/// use semantic_browser::browser::NavigationOptions;
/// use semantic_browser::external::browse_and_insert_kg;
///
/// let mut kg = KnowledgeGraph::new();
/// let options = NavigationOptions::default();
/// let (data, count) = browse_and_insert_kg(
///     "https://example.com",
///     options,
///     &mut kg
/// ).await?;
///
/// println!("Inserted {} triples into KG", count);
///
/// // Now query with SPARQL
/// let results = kg.query("SELECT ?p ?o WHERE { <https://example.com> ?p ?o }")?;
/// ```
#[cfg(feature = "browser-automation")]
pub async fn browse_and_insert_kg(
    url: &str,
    options: crate::browser::NavigationOptions,
    kg: &mut crate::kg::KnowledgeGraph,
) -> Result<(crate::browser::SemanticData, usize), Box<dyn std::error::Error + Send + Sync>> {
    tracing::info!("Browsing {} and inserting into KG", url);

    // 1. Browse and extract semantic data
    let semantic_data = browse_with_chromium_full(url, options).await?;

    // 2. Insert into Knowledge Graph
    let count = crate::kg_integration::insert_semantic_data_to_kg(&semantic_data, kg, url)
        .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> {
            Box::new(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        })?;

    tracing::info!("Successfully inserted {} triples into KG for {}", count, url);

    Ok((semantic_data, count))
}

/// Call browser-use to browse a URL and extract semantic data (HTTP fallback)
///
/// This is now the fallback method when chromiumoxide is not available or fails.
/// Uses simple HTTP GET with reqwest - no JavaScript support.
pub async fn browse_with_browser_use(
    url: &str,
    query: &str,
) -> Result<BrowseOutcome, Box<dyn std::error::Error + Send + Sync>> {
    // For now, fetch HTML and parse semantically
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    let final_url = response.url().to_string();
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
    let mut json_ld_count = 0usize;
    for element in document.select(&json_ld_selector) {
        if let Some(text) = element.text().next() {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(text) {
                if !value.is_null() {
                    json_ld_count += 1;
                }
            }
        }
    }

    // Extract microdata
    let itemscope_selector = scraper::Selector::parse("[itemscope]")
        .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e.to_string()))?;
    let itemprop_selector = scraper::Selector::parse("[itemprop]")
        .map_err(|e| Box::<dyn std::error::Error + Send + Sync>::from(e.to_string()))?;
    let mut microdata = Vec::new();
    for element in document.select(&itemscope_selector) {
        let item_type = element.value().attr("itemtype").unwrap_or("").to_string();
        let mut property_count = 0usize;
        for prop in element.select(&itemprop_selector) {
            let prop_value = prop.text().collect::<String>();
            if !prop_value.trim().is_empty() {
                property_count += 1;
            }
        }
        microdata.push(MicrodataSummary { item_type, properties: property_count });
    }

    let description = extract_meta_description(&document);
    let keywords = extract_meta_keywords(&document);
    let language = extract_language(&document);
    let canonical_url = extract_canonical_url(&document);
    let open_graph = extract_open_graph(&document);
    let twitter_card = extract_twitter_card(&document);
    let text_content = extract_text_content(&document);
    let query_matches = build_query_matches_from_document(&document, query);

    let snapshot = SemanticSnapshot {
        title,
        description,
        language,
        canonical_url,
        final_url,
        keywords,
        open_graph,
        twitter_card,
        json_ld_count,
        microdata,
        text_preview: build_text_preview(&text_content),
        text_length: text_content.len(),
        query_matches,
    };

    let summary = build_summary(url, query, &snapshot);

    Ok(BrowseOutcome { summary, snapshot })
}

/// Smart browse: Try chromiumoxide first, fallback to HTTP
///
/// Best practice 2025: Use Rust-native headless browser with fallback
/// This is the recommended public API for browsing.
pub async fn browse_with_best_available(
    url: &str,
    query: &str,
) -> Result<BrowseOutcome, Box<dyn std::error::Error + Send + Sync>> {
    // Try chromiumoxide first (if feature enabled)
    #[cfg(feature = "browser-automation")]
    {
        match browse_with_chromium(url, query).await {
            Ok(result) => {
                tracing::info!("Successfully browsed with chromiumoxide");
                return Ok(result);
            }
            Err(e) => {
                tracing::warn!("Chromiumoxide failed: {}, falling back to HTTP", e);
            }
        }
    }

    // Fallback to simple HTTP
    tracing::info!("Using HTTP fallback for {}", url);
    browse_with_browser_use(url, query).await
}

/// Call browser-use Python library using PyO3 (if available) or subprocess fallback
///
/// DEPRECATED: Use browse_with_chromium or browse_with_best_available instead.
/// This method is kept for backward compatibility.
pub async fn browse_with_python_browser_use(
    url: &str,
    query: &str,
) -> Result<BrowseOutcome, Box<dyn std::error::Error + Send + Sync>> {
    #[allow(unused_mut)]
    let mut py_summary: Option<String> = None;
    // Try PyO3 integration first
    #[cfg(feature = "pyo3-integration")]
    {
        use pyo3::prelude::*;
        use pyo3::types::PyDict;

        #[allow(deprecated)]
        match Python::with_gil(|py| {
            // Try to import browser-use
            let result: PyResult<String> = (|| {
                let browser_use = py.import("browser_use")?;
                let kwargs = PyDict::new(py);
                kwargs.set_item("url", url)?;
                kwargs.set_item("query", query)?;

                let result = browser_use.call_method("browse", (), Some(&kwargs))?;
                result.extract::<String>()
            })();

            match result {
                Ok(data) => Ok::<String, Box<dyn std::error::Error>>(data),
                Err(e) => {
                    tracing::warn!("PyO3 browser-use failed: {}, falling back to HTTP", e);
                    Err(format!("PyO3 error: {}", e).into())
                }
            }
        }) {
            Ok(data) => {
                tracing::debug!("browser-use via PyO3 succeeded");
                py_summary = Some(data);
            }
            Err(err) => {
                tracing::debug!("PyO3 browser-use unavailable: {}", err);
            }
        }
    }

    // If PyO3 path or subprocess returns a summary string, reuse HTTP parser to build snapshot
    if let Some(summary) = py_summary {
        match browse_with_browser_use(url, query).await {
            Ok(mut outcome) => {
                outcome.summary = summary;
                return Ok(outcome);
            }
            Err(err) => {
                tracing::warn!("HTTP parsing failed after PyO3 success: {}", err);
                return Err(err);
            }
        }
    }

    // Fallback to subprocess execution for summary, then HTTP parsing
    match run_browser_use_subprocess(url, query).await {
        Ok(summary) => {
            let mut outcome = browse_with_browser_use(url, query).await?;
            outcome.summary = summary;
            Ok(outcome)
        }
        Err(err) => Err(err),
    }
}

/// State for LangGraph workflow
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowState {
    pub input: String,
    pub current_step: String,
    pub results: HashMap<String, String>,
    pub error: Option<String>,
}

/// Node function type (async)
type NodeFn = Box<
    dyn Fn(
            &WorkflowState,
        ) -> BoxFuture<Result<WorkflowState, Box<dyn std::error::Error + Send + Sync>>>
        + Send
        + Sync,
>;

/// Conditional edge function type
type ConditionalFn = Box<dyn Fn(&WorkflowState) -> String + Send + Sync>;

/// StateGraph for workflow orchestration
pub struct StateGraph {
    nodes: HashMap<String, NodeFn>,
    edges: HashMap<String, String>,
    conditional_edges: HashMap<String, ConditionalFn>,
    entry_point: String,
}

impl StateGraph {
    pub fn new(entry_point: String) -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            conditional_edges: HashMap::new(),
            entry_point,
        }
    }

    pub fn add_node<F>(mut self, name: &str, node_fn: F) -> Self
    where
        F: Fn(
                &WorkflowState,
            )
                -> BoxFuture<Result<WorkflowState, Box<dyn std::error::Error + Send + Sync>>>
            + Send
            + Sync
            + 'static,
    {
        self.nodes.insert(name.to_string(), Box::new(node_fn));
        self
    }

    pub fn add_edge(mut self, from: &str, to: &str) -> Self {
        self.edges.insert(from.to_string(), to.to_string());
        self
    }

    pub fn add_conditional_edge<F>(mut self, from: &str, condition_fn: F) -> Self
    where
        F: Fn(&WorkflowState) -> String + Send + Sync + 'static,
    {
        self.conditional_edges.insert(from.to_string(), Box::new(condition_fn));
        self
    }

    pub async fn execute(
        &self,
        initial_state: WorkflowState,
    ) -> Result<WorkflowState, Box<dyn std::error::Error + Send + Sync>> {
        let mut state = initial_state;
        let mut current_node = self.entry_point.clone();

        loop {
            tracing::debug!("Executing node: {}", current_node);

            let node_fn = self
                .nodes
                .get(&current_node)
                .ok_or_else(|| format!("Node '{}' not found", current_node))?;

            state = node_fn(&state).await?;
            state.current_step = current_node.clone();

            // Check for conditional edge first
            if let Some(condition_fn) = self.conditional_edges.get(&current_node) {
                current_node = condition_fn(&state);
                if current_node.is_empty() {
                    break; // End workflow
                }
            } else if let Some(next_node) = self.edges.get(&current_node) {
                current_node = next_node.clone();
            } else {
                break; // No more edges, end workflow
            }
        }

        Ok(state)
    }
}

/// Call LangGraph for agent workflow using custom StateGraph implementation
/// StateGraph framework with real node execution (browse_url, extract, kg_query), conditional edges, and error recovery.
pub async fn run_langgraph_workflow(
    graph_def: &str,
    input: &str,
    kg: std::sync::Arc<tokio::sync::Mutex<crate::kg::KnowledgeGraph>>,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    // Parse graph definition (simplified JSON)
    let graph_config: serde_json::Value =
        serde_json::from_str(graph_def).map_err(|e| format!("Invalid graph definition: {}", e))?;

    let entry_point = graph_config["entry_point"].as_str().unwrap_or("start").to_string();

    let mut graph = StateGraph::new(entry_point);

    // Clone kg for use in closures
    let kg_for_query = kg.clone();

    // Add predefined nodes for common workflow
    graph = graph
        .add_node("browse", |state| {
            Box::pin(async move {
                let url = state.results.get("url").unwrap_or(&state.input).clone();

                // Real browse implementation using browse_with_best_available
                match browse_with_best_available(&url, "").await {
                    Ok(outcome) => {
                        let mut new_state = state.clone();
                        new_state.results.insert("browse_result".to_string(), outcome.summary);
                        new_state.results.insert(
                            "semantic_data".to_string(),
                            serde_json::to_string(&outcome.snapshot)?,
                        );
                        Ok(new_state)
                    }
                    Err(e) => {
                        let mut new_state = state.clone();
                        new_state.error = Some(format!("Browse failed: {}", e));
                        new_state
                            .results
                            .insert("browse_result".to_string(), format!("Error: {}", e));
                        Ok(new_state)
                    }
                }
            })
        })
        .add_node("extract", |state| {
            Box::pin(async move {
                let semantic_data_str =
                    state.results.get("semantic_data").unwrap_or(&"{}".to_string()).clone();

                // Parse semantic data and extract entities
                match serde_json::from_str::<crate::models::SemanticSnapshot>(&semantic_data_str) {
                    Ok(snapshot) => {
                        // Extract entities using annotator
                        let html_content = snapshot
                            .microdata
                            .iter()
                            .map(|item| format!("{:?}", item))
                            .collect::<Vec<_>>()
                            .join(" ");

                        let entities =
                            crate::annotator::annotate_html(&html_content).unwrap_or_default();

                        let mut new_state = state.clone();
                        new_state.results.insert(
                            "extract_result".to_string(),
                            format!("Extracted {} entities: {:?}", entities.len(), entities),
                        );
                        new_state
                            .results
                            .insert("entities".to_string(), serde_json::to_string(&entities)?);
                        Ok(new_state)
                    }
                    Err(e) => {
                        let mut new_state = state.clone();
                        new_state.error = Some(format!("Extract failed: {}", e));
                        new_state
                            .results
                            .insert("extract_result".to_string(), format!("Error: {}", e));
                        Ok(new_state)
                    }
                }
            })
        })
        .add_node("query", move |state| {
            let kg_clone = kg_for_query.clone();
            Box::pin(async move {
                let query = state.results.get("query").unwrap_or(&state.input).clone();

                // Real KG query implementation
                match kg_clone.lock().await.query(&query) {
                    Ok(results) => {
                        let mut new_state = state.clone();
                        new_state.results.insert(
                            "query_result".to_string(),
                            format!("Query returned {} results: {:?}", results.len(), results),
                        );
                        Ok(new_state)
                    }
                    Err(e) => {
                        let mut new_state = state.clone();
                        new_state.error = Some(format!("Query failed: {}", e));
                        new_state
                            .results
                            .insert("query_result".to_string(), format!("Error: {}", e));
                        Ok(new_state)
                    }
                }
            })
        });

    // Add edges from config or default
    if let Some(edges) = graph_config["edges"].as_object() {
        for (from, to) in edges {
            if let Some(to_str) = to.as_str() {
                graph = graph.add_edge(from, to_str);
            }
        }
    } else {
        // Default workflow: browse -> extract -> query
        graph = graph
            .add_edge("start", "browse")
            .add_edge("browse", "extract")
            .add_edge("extract", "query");
    }

    // Add conditional edges
    if let Some(conditional_edges) = graph_config["conditional_edges"].as_object() {
        for (from, condition_def) in conditional_edges {
            if let Some(condition_type) = condition_def["type"].as_str() {
                match condition_type {
                    "has_data" => {
                        graph = graph.add_conditional_edge(from, |state| {
                            if state.results.contains_key("extract_result") {
                                "query".to_string()
                            } else {
                                "end".to_string()
                            }
                        });
                    }
                    _ => {
                        graph = graph.add_conditional_edge(from, |_| "end".to_string());
                    }
                }
            }
        }
    }

    // Add edges
    if let Some(edges) = graph_config["edges"].as_object() {
        for (from, to) in edges {
            if let Some(to_str) = to.as_str() {
                graph = graph.add_edge(from, to_str);
            }
        }
    }

    // Add conditional edges
    if let Some(conditional_edges) = graph_config["conditional_edges"].as_object() {
        for (from, condition_def) in conditional_edges {
            if let Some(condition_type) = condition_def["type"].as_str() {
                match condition_type {
                    "has_data" => {
                        graph = graph.add_conditional_edge(from, move |state| {
                            if state.results.contains_key("extract_result") {
                                "query".to_string()
                            } else {
                                "end".to_string()
                            }
                        });
                    }
                    _ => {
                        graph = graph.add_conditional_edge(from, move |_| "end".to_string());
                    }
                }
            }
        }
    }

    // Execute workflow
    let initial_state = WorkflowState {
        input: input.to_string(),
        current_step: "start".to_string(),
        results: HashMap::new(),
        error: None,
    };

    let final_state = graph.execute(initial_state).await?;

    // Format result
    let mut result_lines =
        vec![format!("Workflow completed. Final step: {}", final_state.current_step)];

    for (key, value) in &final_state.results {
        result_lines.push(format!("{}: {}", key, value));
    }

    if let Some(error) = &final_state.error {
        result_lines.push(format!("Error: {}", error));
    }

    Ok(result_lines.join("\n"))
}

// -----------------------------------------------------------------------------
// Helper functions for metadata extraction and query-aware summaries
// -----------------------------------------------------------------------------

fn build_summary(original_url: &str, query: &str, snapshot: &SemanticSnapshot) -> String {
    let mut lines = Vec::new();
    lines.push(format!("Browsed {} with query '{}'", original_url, query));

    if let Some(title) = &snapshot.title {
        lines.push(format!("Title: {}", title));
    }

    if let Some(desc) = &snapshot.description {
        lines.push(format!("Description: {}", desc));
    }

    if !snapshot.keywords.is_empty() {
        lines.push(format!("Keywords: {}", snapshot.keywords.join(", ")));
    }

    if let Some(lang) = &snapshot.language {
        lines.push(format!("Language: {}", lang));
    }

    if let Some(canonical) = &snapshot.canonical_url {
        lines.push(format!("Canonical URL: {}", canonical));
    }

    if !snapshot.open_graph.is_empty() {
        lines.push(format!("Open Graph tags: {}", snapshot.open_graph.len()));
        let mut items: Vec<_> = snapshot.open_graph.iter().collect();
        items.sort_by(|a, b| a.0.cmp(b.0));
        for (key, value) in items {
            lines.push(format!("  og:{}: {}", key, value));
        }
    }

    if !snapshot.twitter_card.is_empty() {
        lines.push(format!("Twitter Card tags: {}", snapshot.twitter_card.len()));
        let mut items: Vec<_> = snapshot.twitter_card.iter().collect();
        items.sort_by(|a, b| a.0.cmp(b.0));
        for (key, value) in items {
            lines.push(format!("  twitter:{}: {}", key, value));
        }
    }

    lines.push(format!("JSON-LD objects: {}", snapshot.json_ld_count));
    lines.push(format!("Microdata items: {}", snapshot.microdata.len()));
    for item in &snapshot.microdata {
        lines.push(format!("- {}: {} properties", item.item_type, item.properties));
    }

    lines.push(format!("Text content length: {} chars", snapshot.text_length));
    lines.push(format!("Final URL: {}", snapshot.final_url));

    if !snapshot.query_matches.is_empty() {
        lines.push("Query matches:".to_string());
        for m in snapshot.query_matches.iter().take(5) {
            lines.push(format!("- [{} | {:.2}] {}", m.element, m.score, m.excerpt));
        }
    }

    lines.join("\n")
}

fn build_text_preview(text: &str) -> String {
    let normalized = normalize_whitespace(text);
    let mut preview = normalized.chars().take(320).collect::<String>();
    if normalized.len() > preview.len() {
        preview.push_str("...");
    }
    preview
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn tokenize_query(query: &str) -> Vec<String> {
    query
        .split(|c: char| !c.is_alphanumeric())
        .filter_map(|token| {
            let lowered = token.trim().to_lowercase();
            if lowered.len() > 1 {
                Some(lowered)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(feature = "browser-automation")]
fn build_query_matches_from_text(text: &str, query: &str) -> Vec<QueryMatch> {
    let tokens = tokenize_query(query);
    if tokens.is_empty() {
        return Vec::new();
    }

    let mut matches = Vec::new();
    for sentence in text.split(|c: char| ".!?\n".contains(c)) {
        let snippet = normalize_whitespace(sentence);
        if snippet.is_empty() {
            continue;
        }

        if let Some(score) = match_score(&snippet, &tokens) {
            matches.push(QueryMatch {
                excerpt: truncate_excerpt(&snippet),
                element: "text".to_string(),
                score,
            });
        }
    }

    rank_and_limit_matches(matches)
}

fn build_query_matches_from_document(document: &scraper::Html, query: &str) -> Vec<QueryMatch> {
    let tokens = tokenize_query(query);
    if tokens.is_empty() {
        return Vec::new();
    }

    let mut matches = Vec::new();
    if let Ok(selector) = scraper::Selector::parse("h1,h2,h3,h4,h5,h6,p,li") {
        for element in document.select(&selector) {
            let snippet = normalize_whitespace(&element.text().collect::<Vec<_>>().join(" "));
            if snippet.is_empty() {
                continue;
            }

            if let Some(score) = match_score(&snippet, &tokens) {
                matches.push(QueryMatch {
                    excerpt: truncate_excerpt(&snippet),
                    element: element.value().name().to_string(),
                    score,
                });
            }
        }
    }

    rank_and_limit_matches(matches)
}

fn match_score(snippet: &str, tokens: &[String]) -> Option<f32> {
    if tokens.is_empty() {
        return None;
    }
    let snippet_lower = snippet.to_lowercase();
    let mut hits = 0usize;
    for token in tokens {
        if snippet_lower.contains(token) {
            hits += 1;
        }
    }
    if hits == 0 {
        None
    } else {
        Some(hits as f32 / tokens.len() as f32)
    }
}

fn rank_and_limit_matches(mut matches: Vec<QueryMatch>) -> Vec<QueryMatch> {
    matches.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.excerpt.len().cmp(&b.excerpt.len()))
    });
    matches.truncate(5);
    matches
}

fn truncate_excerpt(snippet: &str) -> String {
    let mut excerpt: String = snippet.chars().take(200).collect();
    if snippet.len() > excerpt.len() {
        excerpt.push_str("...");
    }
    excerpt
}

fn extract_meta_description(document: &scraper::Html) -> Option<String> {
    let meta_selector = scraper::Selector::parse("meta[name='description']").ok()?;
    document
        .select(&meta_selector)
        .next()
        .and_then(|el| el.value().attr("content"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn extract_meta_keywords(document: &scraper::Html) -> Vec<String> {
    if let Ok(selector) = scraper::Selector::parse("meta[name='keywords']") {
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

fn extract_language(document: &scraper::Html) -> Option<String> {
    let html_selector = scraper::Selector::parse("html").ok()?;
    document
        .select(&html_selector)
        .next()
        .and_then(|el| el.value().attr("lang"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn extract_canonical_url(document: &scraper::Html) -> Option<String> {
    let link_selector = scraper::Selector::parse("link[rel='canonical']").ok()?;
    document
        .select(&link_selector)
        .next()
        .and_then(|el| el.value().attr("href"))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn extract_open_graph(document: &scraper::Html) -> HashMap<String, String> {
    let mut og_data = HashMap::new();
    if let Ok(selector) = scraper::Selector::parse("meta[property^='og:']") {
        for el in document.select(&selector) {
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

fn extract_twitter_card(document: &scraper::Html) -> HashMap<String, String> {
    let mut twitter = HashMap::new();
    if let Ok(selector) = scraper::Selector::parse("meta[name^='twitter:']") {
        for el in document.select(&selector) {
            if let Some(name) = el.value().attr("name") {
                if let Some(content) = el.value().attr("content") {
                    let key = name.trim_start_matches("twitter:").to_string();
                    twitter.insert(key, content.trim().to_string());
                }
            }
        }
    }
    twitter
}

fn extract_text_content(document: &scraper::Html) -> String {
    let selectors = ["main", "article", "[role='main']", ".content", "#content"];
    for selector_str in &selectors {
        if let Ok(selector) = scraper::Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let text = normalize_whitespace(&element.text().collect::<Vec<_>>().join(" "));
                if !text.is_empty() {
                    return text;
                }
            }
        }
    }

    if let Ok(body_selector) = scraper::Selector::parse("body") {
        if let Some(body) = document.select(&body_selector).next() {
            return normalize_whitespace(&body.text().collect::<Vec<_>>().join(" "));
        }
    }

    String::new()
}

const PY_BROWSER_TIMEOUT: Duration = Duration::from_secs(45);

/// Spawn a Python subprocess to drive a headless browser via browser-use/Playwright.
async fn run_browser_use_subprocess(
    url: &str,
    query: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let url_json = serde_json::to_string(url)?;
    let query_json = serde_json::to_string(query)?;

    let python_code = format!(
        r#"
import json
import sys

url = {url}
query = {query}

def emit_error(message: str) -> None:
    sys.stderr.write(message + "\n")
    sys.stderr.flush()

try:
    from browser_use import browse
except Exception as exc:  # noqa: BLE001
    emit_error(f"browser_use import failed: {{exc}}")
    sys.exit(2)

try:
    result = browse(url=url, query=query)
except Exception as exc:  # noqa: BLE001
    emit_error(f"browser_use execution failed: {{exc}}")
    sys.exit(3)

if isinstance(result, (bytes, bytearray)):
    sys.stdout.write(result.decode("utf-8", errors="replace"))
elif isinstance(result, str):
    sys.stdout.write(result)
else:
    sys.stdout.write(json.dumps(result, ensure_ascii=False))
sys.stdout.flush()
"#,
        url = url_json,
        query = query_json
    );

    let mut command = Command::new("python3");
    command.arg("-u").arg("-c").arg(python_code);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());

    let output = match timeout(PY_BROWSER_TIMEOUT, command.output()).await {
        Ok(Ok(out)) => out,
        Ok(Err(e)) => {
            return Err(Box::<dyn std::error::Error + Send + Sync>::from(format!(
                "Failed to spawn python3: {e}"
            )))
        }
        Err(_) => {
            return Err(Box::<dyn std::error::Error + Send + Sync>::from(
                "Python browser subprocess timed out",
            ))
        }
    };

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
        if stdout.is_empty() {
            tracing::warn!("Python browser subprocess returned empty output for {}", url);
        }
        return Ok(stdout);
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    let code = output.status.code().unwrap_or(-1);
    Err(format!("Python browser subprocess failed (code {code}): {stderr}").into())
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
