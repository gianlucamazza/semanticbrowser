use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a match found when applying a user query to browsed web content.
/// This struct captures relevant excerpts and their context for semantic search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMatch {
    /// The text excerpt that contains the matching query tokens.
    /// This provides context around the matched content.
    pub excerpt: String,
    /// The HTML element type where the match was found (e.g., "h1", "p", "div").
    /// Helps identify the structural context of the match.
    pub element: String,
    /// Relevance score between 0.0 and 1.0 indicating how well the excerpt matches the query.
    /// Calculated based on token coverage and semantic similarity.
    pub score: f32,
}

/// Summary representation of Microdata items extracted from HTML microdata attributes.
/// Microdata provides structured data embedded in web pages using schema.org vocabulary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrodataSummary {
    /// The schema.org item type (e.g., "https://schema.org/Product", "Person").
    /// Indicates the type of structured data represented.
    pub item_type: String,
    /// Number of properties defined for this microdata item.
    /// Represents the richness of the structured data.
    pub properties: usize,
}

/// Comprehensive snapshot of semantic information extracted from a web page.
/// This struct aggregates metadata, structured data, and content analysis results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSnapshot {
    /// Page title extracted from HTML <title> tag or Open Graph metadata.
    pub title: Option<String>,
    /// Page description from meta description, Open Graph, or content summarization.
    pub description: Option<String>,
    /// Detected language of the page content (ISO 639-1 format, e.g., "en", "fr").
    pub language: Option<String>,
    /// Canonical URL as specified in the page's rel="canonical" link.
    pub canonical_url: Option<String>,
    /// Final resolved URL after redirects (the actual URL that was fetched).
    pub final_url: String,
    /// Keywords extracted from meta keywords, content analysis, or schema.org data.
    pub keywords: Vec<String>,
    /// Open Graph protocol metadata as key-value pairs.
    /// Contains social media sharing information like og:title, og:image, etc.
    pub open_graph: HashMap<String, String>,
    /// Twitter Card metadata for social media sharing on Twitter/X.
    pub twitter_card: HashMap<String, String>,
    /// Count of JSON-LD structured data objects found on the page.
    pub json_ld_count: usize,
    /// List of Microdata items extracted from the page.
    pub microdata: Vec<MicrodataSummary>,
    /// Short text preview of the page content for summarization purposes.
    /// Typically the first few sentences or paragraphs.
    pub text_preview: String,
    /// Total character count of extracted text content (UTF-8).
    pub text_length: usize,
    /// Query matches found when a search query was applied to the content.
    pub query_matches: Vec<QueryMatch>,
}

/// Result of a web page browsing operation.
/// Combines a human-readable summary with detailed semantic analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowseOutcome {
    /// Human-readable summary of the page content and key information.
    /// Generated through content analysis and summarization.
    pub summary: String,
    /// Detailed semantic snapshot containing all extracted metadata and structured data.
    pub snapshot: SemanticSnapshot,
}
