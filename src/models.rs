use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Matches derived from applying the user query to the browsed content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMatch {
    /// Text excerpt that satisfied the query tokens.
    pub excerpt: String,
    /// HTML element context (e.g., heading, paragraph).
    pub element: String,
    /// Match score in range [0.0, 1.0] based on token coverage.
    pub score: f32,
}

/// Minimal summary of a Microdata item extracted from the page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrodataSummary {
    pub item_type: String,
    pub properties: usize,
}

/// Structured snapshot of semantic information extracted from a page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSnapshot {
    pub title: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub canonical_url: Option<String>,
    pub final_url: String,
    pub keywords: Vec<String>,
    pub open_graph: HashMap<String, String>,
    pub twitter_card: HashMap<String, String>,
    pub json_ld_count: usize,
    pub microdata: Vec<MicrodataSummary>,
    /// Minimal text preview for downstream summarisation.
    pub text_preview: String,
    /// Total number of UTF-8 characters in the extracted text content.
    pub text_length: usize,
    pub query_matches: Vec<QueryMatch>,
}

/// Rich browsing result containing both legacy summary and structured snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowseOutcome {
    pub summary: String,
    pub snapshot: SemanticSnapshot,
}
