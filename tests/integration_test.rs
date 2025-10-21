use semantic_browser::{annotator, kg, parser, security};

#[test]
fn test_html_parsing_integration() {
    let html = r#"
        <html>
        <head>
            <title>Test Page</title>
            <script type="application/ld+json">
            {
                "@type": "Person",
                "name": "John Doe",
                "jobTitle": "Software Engineer"
            }
            </script>
        </head>
        <body>
            <div itemscope itemtype="http://schema.org/Product">
                <span itemprop="name">Widget</span>
                <span itemprop="price">$19.99</span>
            </div>
        </body>
        </html>
    "#;

    // Test parsing
    let result = parser::parse_html(html);
    assert!(result.is_ok());
    let data = result.unwrap();

    // Verify title extraction
    assert_eq!(data.title, Some("Test Page".to_string()));

    // Verify JSON-LD extraction
    assert_eq!(data.json_ld.len(), 1);
    assert_eq!(data.json_ld[0]["@type"], "Person");

    // Verify microdata extraction
    assert!(data.microdata.len() > 0);
    let product = &data.microdata[0];
    assert!(product.item_type.contains("Product"));
}

#[test]
fn test_entity_extraction_integration() {
    let html = r#"
        <html>
        <body>
            <p>John Smith works at Microsoft in New York City.</p>
        </body>
        </html>
    "#;

    let result = annotator::annotate_html(html);
    assert!(result.is_ok());
    let entities = result.unwrap();

    // Should extract capitalized words/phrases
    assert!(entities.len() > 0);
}

#[test]
fn test_knowledge_graph_integration() {
    let mut kg = kg::KnowledgeGraph::new();

    // Test insert
    let result = kg.insert(
        "http://example.org/person1",
        "http://xmlns.com/foaf/0.1/name",
        "http://example.org/alice",
    );
    assert!(result.is_ok());

    // Test list triples
    let triples = kg.list_triples();
    assert_eq!(triples.len(), 1);
    assert!(triples[0].contains("person1"));

    // Test SPARQL query
    let query_result = kg.query("SELECT * WHERE { ?s ?p ?o }");
    assert!(query_result.is_ok());
    let results = query_result.unwrap();
    assert!(!results.is_empty());
}

#[test]
fn test_sparql_update_integration() {
    let mut kg = kg::KnowledgeGraph::new();

    // Test INSERT
    let insert_query = r#"
        INSERT DATA {
            <http://example.org/person1> <http://xmlns.com/foaf/0.1/name> "Alice" .
            <http://example.org/person1> <http://xmlns.com/foaf/0.1/age> "30" .
        }
    "#;

    let result = kg.update(insert_query);
    assert!(result.is_ok());

    // Verify data was inserted
    let query_result = kg.query("SELECT * WHERE { ?s ?p ?o }");
    assert!(query_result.is_ok());
    let results = query_result.unwrap();
    assert_eq!(results.len(), 2);
}

#[test]
fn test_security_validation() {
    // Test HTML validation
    let valid_html = "<html><body>Test</body></html>";
    assert!(security::validate_html_input(valid_html).is_ok());

    // Test oversized HTML rejection
    let huge_html = "a".repeat(20_000_000);
    assert!(security::validate_html_input(&huge_html).is_err());

    // Test malicious HTML rejection
    let malicious_html = r#"<html><body><script>alert('xss')</script></body></html>"#;
    assert!(security::validate_html_input(malicious_html).is_err());

    // Test SPARQL validation
    assert!(security::validate_sparql_query("SELECT * WHERE { ?s ?p ?o }").is_ok());
    assert!(security::validate_sparql_query(
        "INSERT DATA { <http://example.org/s> <http://example.org/p> <http://example.org/o> }"
    )
    .is_ok());
    assert!(security::validate_sparql_query("DELETE WHERE { ?s ?p ?o }").is_ok());
    assert!(security::validate_sparql_query("CONSTRUCT { ?s ?p ?o } WHERE { ?s ?p ?o }").is_ok());

    // Test dangerous query rejection
    assert!(security::validate_sparql_query("DROP ALL").is_err());
    assert!(security::validate_sparql_query("CLEAR GRAPH <http://example.org>").is_err());
}

#[test]
fn test_kg_inference() {
    let mut kg = kg::KnowledgeGraph::new();

    // Add some data
    kg.insert(
        "http://example.org/person1",
        "http://www.w3.org/2000/01/rdf-schema#subClassOf",
        "http://example.org/entity",
    )
    .unwrap();

    // Run inference
    let result = kg.infer();
    assert!(result.is_ok());
}
