//! Knowledge Graph Integration for Semantic Browser
//!
//! This module provides integration between extracted SemanticData
//! and the Knowledge Graph, converting web metadata into RDF triples
//! following W3C Semantic Web standards.
//!
//! Best practices 2025:
//! - Use standard W3C namespace URIs (og:, twitter:, schema:, dcterms:)
//! - Language-tagged literals for i18n support
//! - Canonical URL for deduplication
//! - Structured Open Graph and Twitter Card mapping

use crate::kg::KnowledgeGraph;
use crate::models::{MicrodataSummary, SemanticSnapshot};

#[cfg(feature = "browser-automation")]
use crate::browser::SemanticData;

/// Insert a SemanticSnapshot into the Knowledge Graph (feature-agnostic).
pub fn insert_snapshot_to_kg(
    snapshot: &SemanticSnapshot,
    kg: &mut KnowledgeGraph,
    base_url: &str,
    query: Option<&str>,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut count = 0usize;

    if let Some(title) = &snapshot.title {
        if let Some(lang) = &snapshot.language {
            kg.insert_language_literal(
                base_url,
                &KnowledgeGraph::expand_namespace("dcterms:title"),
                title,
                lang,
            )?;
        } else {
            kg.insert_literal(base_url, &KnowledgeGraph::expand_namespace("dcterms:title"), title)?;
        }
        count += 1;
    }

    if let Some(desc) = &snapshot.description {
        if let Some(lang) = &snapshot.language {
            kg.insert_language_literal(
                base_url,
                &KnowledgeGraph::expand_namespace("dcterms:description"),
                desc,
                lang,
            )?;
        } else {
            kg.insert_literal(
                base_url,
                &KnowledgeGraph::expand_namespace("dcterms:description"),
                desc,
            )?;
        }
        count += 1;
    }

    for keyword in &snapshot.keywords {
        kg.insert_literal(base_url, &KnowledgeGraph::expand_namespace("schema:keywords"), keyword)?;
        count += 1;
    }

    if let Some(lang) = &snapshot.language {
        kg.insert_literal(base_url, &KnowledgeGraph::expand_namespace("dcterms:language"), lang)?;
        count += 1;
    }

    if let Some(canonical) = &snapshot.canonical_url {
        kg.insert_literal(
            base_url,
            &KnowledgeGraph::expand_namespace("dcterms:identifier"),
            canonical,
        )?;
        count += 1;

        kg.insert(base_url, &KnowledgeGraph::expand_namespace("dcterms:isVersionOf"), canonical)?;
        count += 1;
    }

    if snapshot.final_url != base_url {
        kg.insert_literal(
            base_url,
            &KnowledgeGraph::expand_namespace("dcterms:hasVersion"),
            &snapshot.final_url,
        )?;
        count += 1;
    }

    for (key, value) in &snapshot.open_graph {
        let predicate = KnowledgeGraph::expand_namespace(&format!("og:{}", key));
        if (value.starts_with("http://") || value.starts_with("https://"))
            && kg.insert(base_url, &predicate, value).is_ok()
        {
            count += 1;
            continue;
        }
        kg.insert_literal(base_url, &predicate, value)?;
        count += 1;
    }

    for (key, value) in &snapshot.twitter_card {
        let predicate = KnowledgeGraph::expand_namespace(&format!("twitter:{}", key));
        if (value.starts_with("http://") || value.starts_with("https://"))
            && kg.insert(base_url, &predicate, value).is_ok()
        {
            count += 1;
            continue;
        }
        kg.insert_literal(base_url, &predicate, value)?;
        count += 1;
    }

    if snapshot.json_ld_count > 0 {
        kg.insert_typed_literal(
            base_url,
            &KnowledgeGraph::expand_namespace("schema:numberOfItems"),
            &snapshot.json_ld_count.to_string(),
            &KnowledgeGraph::expand_namespace("xsd:integer"),
        )?;
        count += 1;
    }

    if !snapshot.microdata.is_empty() {
        kg.insert_typed_literal(
            base_url,
            &KnowledgeGraph::expand_namespace("schema:numberOfItems"),
            &snapshot.microdata.len().to_string(),
            &KnowledgeGraph::expand_namespace("xsd:integer"),
        )?;
        count += 1;
    }

    for MicrodataSummary { item_type, properties } in &snapshot.microdata {
        if !item_type.is_empty() {
            kg.insert_literal(
                base_url,
                &KnowledgeGraph::expand_namespace("schema:itemType"),
                item_type,
            )?;
            count += 1;
        }
        kg.insert_typed_literal(
            base_url,
            &KnowledgeGraph::expand_namespace("schema:numberOfProperties"),
            &properties.to_string(),
            &KnowledgeGraph::expand_namespace("xsd:integer"),
        )?;
        count += 1;
    }

    if !snapshot.text_preview.is_empty() {
        kg.insert_literal(
            base_url,
            &KnowledgeGraph::expand_namespace("schema:abstract"),
            &snapshot.text_preview,
        )?;
        count += 1;
    }

    for match_item in &snapshot.query_matches {
        let mention = format!(
            "{} (element={}, score={:.2})",
            match_item.excerpt, match_item.element, match_item.score
        );
        kg.insert_literal(
            base_url,
            &KnowledgeGraph::expand_namespace("schema:mentions"),
            &mention,
        )?;
        count += 1;
    }

    if let Some(q) = query {
        kg.insert_literal(base_url, &KnowledgeGraph::expand_namespace("schema:query"), q)?;
        count += 1;
    }

    kg.insert(
        base_url,
        &KnowledgeGraph::expand_namespace("rdf:type"),
        &KnowledgeGraph::expand_namespace("schema:WebPage"),
    )?;
    count += 1;

    Ok(count)
}

#[cfg(feature = "browser-automation")]
pub(crate) fn semantic_data_to_snapshot(data: &SemanticData) -> SemanticSnapshot {
    let microdata = data
        .microdata
        .iter()
        .map(|item| MicrodataSummary {
            item_type: item.item_type.clone(),
            properties: item.properties.len(),
        })
        .collect();

    SemanticSnapshot {
        title: data.title.clone(),
        description: data.meta_description.clone(),
        language: data.language.clone(),
        canonical_url: data.canonical_url.clone(),
        final_url: data.final_url.clone(),
        keywords: data.meta_keywords.clone(),
        open_graph: data.open_graph.clone(),
        twitter_card: data.twitter_card.clone(),
        json_ld_count: data.json_ld.len(),
        microdata,
        text_preview: build_preview(&data.text_content),
        text_length: data.text_content.len(),
        query_matches: Vec::new(),
    }
}

#[cfg(feature = "browser-automation")]
fn build_preview(text: &str) -> String {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut preview: String = normalized.chars().take(320).collect();
    if normalized.len() > preview.len() {
        preview.push_str("...");
    }
    preview
}

#[cfg(feature = "browser-automation")]
/// Insert SemanticData into Knowledge Graph as RDF triples
///
/// This function converts all fields from Phase 1 meta tags extraction
/// into W3C-compliant RDF triples in the Knowledge Graph.
///
/// # Parameters
/// - `data`: Extracted semantic data from browser navigation
/// - `kg`: Mutable reference to Knowledge Graph
/// - `base_url`: Base URL to use as subject for triples
///
/// # Returns
pub fn insert_semantic_data_to_kg(
    data: &SemanticData,
    kg: &mut KnowledgeGraph,
    base_url: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    tracing::info!("Inserting SemanticData into KG for URL: {}", base_url);
    let snapshot = semantic_data_to_snapshot(data);
    let inserted = insert_snapshot_to_kg(&snapshot, kg, base_url, None)?;
    tracing::info!("Successfully inserted {} triples into KG for {}", inserted, base_url);
    Ok(inserted)
}

#[cfg(test)]
mod tests {

    #[test]
    #[cfg(feature = "browser-automation")]
    fn test_insert_semantic_data_minimal() {
        use crate::browser::SemanticData;
        use crate::kg::KnowledgeGraph;
        use std::collections::HashMap;

        let data = SemanticData {
            title: Some("Test Page".to_string()),
            json_ld: vec![],
            microdata: vec![],
            text_content: String::new(),
            screenshot: None,
            final_url: "https://example.com".to_string(),
            meta_description: Some("Test description".to_string()),
            meta_keywords: vec!["test".to_string(), "example".to_string()],
            language: Some("en".to_string()),
            canonical_url: Some("https://example.com/canonical".to_string()),
            open_graph: HashMap::new(),
            twitter_card: HashMap::new(),
        };

        let mut kg = KnowledgeGraph::new();
        let count =
            super::insert_semantic_data_to_kg(&data, &mut kg, "https://example.com").unwrap();

        // Should insert: title, description, keywords, language, canonical (2), final_url, rdf:type
        assert!(count >= 7, "Expected at least 7 triples, got {}", count);

        let triples = kg.list_triples();
        assert!(!triples.is_empty());
    }

    #[test]
    #[cfg(feature = "browser-automation")]
    fn test_insert_semantic_data_with_og_twitter() {
        use crate::browser::SemanticData;
        use crate::kg::KnowledgeGraph;
        use std::collections::HashMap;

        let mut open_graph = HashMap::new();
        open_graph.insert("title".to_string(), "OG Title".to_string());
        open_graph.insert("image".to_string(), "https://example.com/image.jpg".to_string());
        open_graph.insert("type".to_string(), "article".to_string());

        let mut twitter_card = HashMap::new();
        twitter_card.insert("card".to_string(), "summary".to_string());
        twitter_card.insert("site".to_string(), "@example".to_string());

        let data = SemanticData {
            title: Some("Test".to_string()),
            json_ld: vec![],
            microdata: vec![],
            text_content: String::new(),
            screenshot: None,
            final_url: "https://example.com".to_string(),
            meta_description: None,
            meta_keywords: vec![],
            language: None,
            canonical_url: None,
            open_graph,
            twitter_card,
        };

        let mut kg = KnowledgeGraph::new();
        let count =
            super::insert_semantic_data_to_kg(&data, &mut kg, "https://example.com").unwrap();

        // Should insert: title, 3 OG tags, 2 Twitter tags, rdf:type = 7+
        assert!(count >= 6, "Expected at least 6 triples, got {}", count);

        let triples = kg.list_triples();
        let has_og = triples.iter().any(|t| t.contains("ogp.me"));
        let has_twitter = triples.iter().any(|t| t.contains("twitter.com"));
        assert!(has_og || has_twitter, "Should have OG or Twitter triples");
    }
}
