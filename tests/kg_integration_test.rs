//! End-to-End Tests for Knowledge Graph Integration (2025)
//!
//! These tests verify the complete pipeline:
//! Browse → Extract Semantic Data → Insert into KG → Query SPARQL

#[cfg(feature = "browser-automation")]
use semantic_browser::browser::{BrowserConfig, BrowserPool, NavigationOptions};
use semantic_browser::kg::KnowledgeGraph;
#[cfg(feature = "browser-automation")]
use semantic_browser::kg_integration::insert_semantic_data_to_kg;

/// Test complete pipeline: Browse → Extract → Insert KG → Query
#[tokio::test]
#[cfg(feature = "browser-automation")]
#[ignore] // Ignore by default (requires real browser)
async fn test_browse_extract_insert_query_pipeline() {
    // Setup
    let config = BrowserConfig::default();
    let pool = BrowserPool::new(config).await.expect("Failed to create browser pool");
    let mut kg = KnowledgeGraph::new();

    // Use a simple, stable test page (example.com)
    let url = "https://example.com";
    let options = NavigationOptions::default();

    // Step 1: Browse and extract semantic data
    let semantic_data =
        pool.navigate_and_extract(url, options).await.expect("Failed to browse URL");

    println!("Extracted semantic data from {}", url);
    println!("  Title: {:?}", semantic_data.title);
    println!("  Language: {:?}", semantic_data.language);
    println!("  Meta description: {:?}", semantic_data.meta_description);

    // Step 2: Insert into Knowledge Graph
    let count =
        insert_semantic_data_to_kg(&semantic_data, &mut kg, url).expect("Failed to insert into KG");

    println!("Inserted {} triples into Knowledge Graph", count);
    assert!(count > 0, "Should have inserted at least one triple");

    // Step 3: Query the Knowledge Graph with SPARQL
    let query = format!("SELECT ?p ?o WHERE {{ <{}> ?p ?o }}", url);
    let results = kg.query(&query).expect("Failed to query KG");

    println!("SPARQL query returned {} results", results.len());
    assert!(!results.is_empty(), "Should have query results");

    // Verify specific triples exist
    let triples = kg.list_triples();
    println!("Total triples in KG: {}", triples.len());

    // Should have rdf:type triple
    let has_type = triples.iter().any(|t| t.contains("rdf-syntax-ns#type"));
    assert!(has_type, "Should have rdf:type triple");

    pool.shutdown().await.expect("Failed to shutdown browser");
}

/// Test browse_and_insert_kg() function directly
#[tokio::test]
#[cfg(feature = "browser-automation")]
#[ignore] // Ignore by default (requires real browser)
async fn test_browse_and_insert_kg_function() {
    use semantic_browser::external::browse_and_insert_kg;

    let mut kg = KnowledgeGraph::new();
    let url = "https://example.com";
    let options = NavigationOptions::default();

    let (semantic_data, count) =
        browse_and_insert_kg(url, options, &mut kg).await.expect("Failed to browse and insert");

    println!("Browsed {} and inserted {} triples", url, count);
    assert!(count > 0, "Should have inserted triples");
    assert!(semantic_data.title.is_some(), "Should have extracted title");

    // Query KG to verify data was inserted
    let query = format!("SELECT ?p ?o WHERE {{ <{}> ?p ?o }}", url);
    let results = kg.query(&query).expect("Failed to query KG");
    assert!(!results.is_empty(), "Should have query results");
}

/// Test KG namespace expansion
#[test]
fn test_namespace_expansion() {
    assert_eq!(KnowledgeGraph::expand_namespace("og:title"), "http://ogp.me/ns#title");
    assert_eq!(
        KnowledgeGraph::expand_namespace("twitter:card"),
        "https://dev.twitter.com/cards/markup#card"
    );
    assert_eq!(KnowledgeGraph::expand_namespace("schema:name"), "https://schema.org/name");
    assert_eq!(
        KnowledgeGraph::expand_namespace("dcterms:description"),
        "http://purl.org/dc/terms/description"
    );
}

/// Test literal insertion methods
#[test]
fn test_kg_literal_methods() {
    let mut kg = KnowledgeGraph::new();

    // Test simple literal
    kg.insert_literal("https://example.com", "dcterms:title", "Example Domain")
        .expect("Failed to insert literal");

    // Test language literal
    kg.insert_language_literal("https://example.com", "dcterms:description", "Description", "en")
        .expect("Failed to insert language literal");

    // Test typed literal
    kg.insert_typed_literal(
        "https://example.com",
        "schema:datePublished",
        "2025-01-15",
        "http://www.w3.org/2001/XMLSchema#date",
    )
    .expect("Failed to insert typed literal");

    let triples = kg.list_triples();
    assert_eq!(triples.len(), 3, "Should have 3 triples");
}

/// Test SPARQL query on literal values
#[test]
fn test_sparql_query_literals() {
    let mut kg = KnowledgeGraph::new();

    kg.insert_literal(
        "https://example.com",
        &KnowledgeGraph::expand_namespace("dcterms:title"),
        "Example Title",
    )
    .expect("Failed to insert");

    kg.insert_literal(
        "https://example.com",
        &KnowledgeGraph::expand_namespace("dcterms:description"),
        "Example Description",
    )
    .expect("Failed to insert");

    // Query all properties
    let query = "SELECT ?p ?o WHERE { <https://example.com> ?p ?o }";
    let results = kg.query(query).expect("Failed to query");

    assert!(results.len() >= 2, "Should have at least 2 results");

    // Verify results contain expected data
    let results_str = results.join(" ");
    assert!(
        results_str.contains("Example Title") || results_str.contains("Example Description"),
        "Results should contain literal values"
    );
}

/// Test metadata extraction count
#[test]
#[cfg(feature = "browser-automation")]
fn test_metadata_count_insertion() {
    use std::collections::HashMap;

    let data = semantic_browser::browser::SemanticData {
        title: Some("Test".to_string()),
        json_ld: vec![serde_json::json!({"@type": "Thing"})],
        microdata: vec![],
        text_content: String::new(),
        screenshot: None,
        final_url: "https://example.com".to_string(),
        meta_description: None,
        meta_keywords: vec![],
        language: None,
        canonical_url: None,
        open_graph: HashMap::new(),
        twitter_card: HashMap::new(),
    };

    let mut kg = KnowledgeGraph::new();
    let count = insert_semantic_data_to_kg(&data, &mut kg, "https://example.com")
        .expect("Failed to insert");

    // Should insert: title, JSON-LD count (numberOfItems), rdf:type
    assert!(count >= 3, "Should insert at least 3 triples for minimal data");
}

/// Test Open Graph and Twitter Card insertion
#[test]
#[cfg(feature = "browser-automation")]
fn test_og_twitter_insertion() {
    use std::collections::HashMap;

    let mut og = HashMap::new();
    og.insert("title".to_string(), "OG Title".to_string());
    og.insert("type".to_string(), "website".to_string());

    let mut twitter = HashMap::new();
    twitter.insert("card".to_string(), "summary".to_string());

    let data = semantic_browser::browser::SemanticData {
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
        open_graph: og,
        twitter_card: twitter,
    };

    let mut kg = KnowledgeGraph::new();
    let count = insert_semantic_data_to_kg(&data, &mut kg, "https://example.com")
        .expect("Failed to insert");

    // Should insert: title, 2 OG tags, 1 Twitter tag, rdf:type
    assert!(count >= 5, "Should insert at least 5 triples");

    // Verify OG and Twitter namespaces in triples
    let triples = kg.list_triples();
    let has_og = triples.iter().any(|t| t.contains("ogp.me"));
    let has_twitter = triples.iter().any(|t| t.contains("twitter.com"));

    assert!(has_og, "Should have Open Graph triples");
    assert!(has_twitter, "Should have Twitter Card triples");
}
