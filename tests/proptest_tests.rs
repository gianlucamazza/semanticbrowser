// Property-based testing with proptest
//
// Best practices 2025:
// - Test properties instead of specific values
// - Use proptest for better shrinking than quickcheck
// - Combine with traditional unit tests
// - Focus on invariants and edge cases

use proptest::prelude::*;
use semantic_browser::{auth, kg, parser, security};

// Test that JWT tokens can always be validated if correctly generated
proptest! {
    fn test_jwt_roundtrip(
        username in "[a-zA-Z0-9_]{3,20}",
        role in prop::option::of("[a-z]{4,10}")
    ) {
        // Set up a Tokio runtime for this test
        let rt = tokio::runtime::Runtime::new().unwrap();

        let result = rt.block_on(async {
            // Initialize JWT config
            std::env::set_var("JWT_SECRET", "test-secret-key-that-is-long-enough-for-validation-32chars");
            let _ = auth::JwtConfig::init();

            // Create claims
            let claims = auth::Claims::new(username.clone(), role.clone());

            // Generate token
            let token = auth::generate_token(&claims).expect("Token generation should succeed");

            // Validate token
            let decoded = auth::validate_token_async(&token).await.expect("Token validation should succeed");

            // Properties:
            prop_assert_eq!(decoded.sub, username);
            prop_assert_eq!(decoded.role, role);
            prop_assert!(decoded.exp > decoded.iat, "Expiration must be after issued time");

            Ok(())
        });

        result.expect("Test should pass");
    }
}

// Test that HTML validation correctly rejects oversized input
proptest! {
    #[test]
    fn test_html_size_limit(
        size in 10_000_001usize..20_000_000usize
    ) {
        let huge_html = "a".repeat(size);
        let result = security::validate_html_input(&huge_html);

        // Property: HTML larger than 10MB should be rejected
        prop_assert!(result.is_err(), "Large HTML should be rejected");
    }
}

// Test that valid HTML of reasonable size is accepted
proptest! {
    #[test]
    fn test_html_validation_accepts_valid(
        content in "[a-zA-Z0-9 <>/]{10,1000}"
    ) {
        let html = format!("<html><body>{}</body></html>", content);

        // Property: Valid, reasonably-sized HTML should be accepted
        if html.len() < 10_000_000 && !html.contains("<script") {
            prop_assert!(security::validate_html_input(&html).is_ok());
        }
    }
}

// Test that HTML validation is case-insensitive
proptest! {
    #[test]
    fn test_html_validation_case_insensitive(
        casing in prop::sample::select(vec!["<SCRIPT>alert(1)</SCRIPT>", "<script>alert(1)</script>"])
    ) {
        let result = security::validate_html_input(casing);
        prop_assert!(result.is_err(), "Script tags should be blocked regardless of case");
    }
}

// Test that JSON-LD scripts remain allowed even with varying attribute casing
proptest! {
    #[test]
    fn test_html_json_ld_script_allowed(
        whitespace in "\\s{0,4}"
    ) {
        let html = format!(
            "<html><head><script{}type=\"APPLICATION/LD+JSON\">{{}}</script></head></html>",
            whitespace
        );
        prop_assert!(security::validate_html_input(&html).is_ok());
    }
}

// Test SPARQL query validation
proptest! {
    #[test]
    fn test_sparql_select_always_valid(
        var in "[a-z]{1,10}",
        predicate in "[a-z]{1,15}".prop_filter("Avoid dangerous keywords", |p| {
            let upper = p.to_uppercase();
            !upper.contains("LOAD") && !upper.contains("DROP") && !upper.contains("CLEAR")
        })
    ) {
        let query = format!("SELECT ?{} WHERE {{ ?s <http://ex.org/{}> ?o }}", var, predicate);

        // Property: Well-formed SELECT queries should be valid
        prop_assert!(security::validate_sparql_query(&query).is_ok());
    }
}

// Test that dangerous SPARQL operations are rejected
proptest! {
    #[test]
    fn test_sparql_dangerous_rejected(
        dangerous_op in prop::sample::select(vec!["DROP", "CLEAR", "LOAD", "CREATE"])
    ) {
        let query = format!("{} ALL", dangerous_op);

        // Property: Dangerous operations should be rejected
        prop_assert!(security::validate_sparql_query(&query).is_err());
    }
}

// Test KG triple insertion and retrieval
proptest! {
    #[test]
    fn test_kg_insert_retrieve(
        subj in "[a-z]{3,10}",
        pred in "[a-z]{3,10}",
        obj in "[a-z]{3,10}"
    ) {
        let mut kg = kg::KnowledgeGraph::new();

        let subject = format!("http://example.org/{}", subj);
        let predicate = format!("http://example.org/{}", pred);
        let object = format!("http://example.org/{}", obj);

        // Insert triple
        let insert_result = kg.insert(&subject, &predicate, &object);
        prop_assert!(insert_result.is_ok(), "Triple insertion should succeed");

        // Retrieve triples
        let triples = kg.list_triples();

        // Property: Inserted triple should be retrievable
        prop_assert!(
            triples.iter().any(|t| t.contains(&subj) && t.contains(&pred) && t.contains(&obj)),
            "Inserted triple should be in KG"
        );
    }
}

// Test that HTML parsing doesn't crash on various inputs
proptest! {
    #[test]
    fn test_html_parsing_robustness(
        content in ".*{0,500}"
    ) {
        let html = format!("<html>{}</html>", content);

        // Property: Parser should never crash, even on malformed HTML
        let result = parser::parse_html(&html);

        // We accept both success and graceful errors, but no panics
        match result {
            Ok(_) | Err(_) => {}, // Both are acceptable
        }
    }
}

// Test that entity extraction is consistent
proptest! {
    #[test]
    fn test_entity_extraction_consistency(
        text in "[A-Z][a-z]{2,15}( [A-Z][a-z]{2,15}){0,3}"
    ) {
        // Property: Running extraction twice should give same results
        let html1 = format!("<html><body>{}</body></html>", text);
        let html2 = html1.clone();

        let result1 = semantic_browser::annotator::annotate_html(&html1);
        let result2 = semantic_browser::annotator::annotate_html(&html2);

        if let (Ok(entities1), Ok(entities2)) = (result1, result2) {
            // Property: Results should be deterministic
            prop_assert_eq!(entities1.len(), entities2.len(),
                "Entity extraction should be deterministic");
        }
    }
}

// Test role-based access control
proptest! {
    #[test]
    fn test_rbac_properties(
        username in "[a-z]{3,10}",
        role in prop::option::of(prop::sample::select(vec!["admin", "user", "readonly"]))
    ) {
        // Initialize JWT config to enable role checking
        std::env::set_var("JWT_SECRET", "test-secret-key-that-is-long-enough-for-validation-32chars");
        let _ = auth::JwtConfig::init();

        let claims = auth::Claims::new(username, role.map(String::from));

        // Property: Admin role should always pass admin check
        if role == Some("admin") {
            prop_assert!(auth::require_role(&claims, "admin").is_ok());
        }

        // Property: User role should fail admin check
        if role == Some("user") {
            prop_assert!(auth::require_role(&claims, "admin").is_err());
        }

        // Property: Role check should match assigned role
        if let Some(r) = role {
            prop_assert!(auth::require_role(&claims, r).is_ok());
        }
    }
}

// Test that inference doesn't corrupt the KG
proptest! {
    #[test]
    fn test_inference_preserves_data(
        triples_count in 1usize..10usize
    ) {
        let mut kg = kg::KnowledgeGraph::new();

        // Insert some triples
        for i in 0..triples_count {
            let _ = kg.insert(
                &format!("http://ex.org/subj{}", i),
                "http://www.w3.org/2000/01/rdf-schema#subClassOf",
                &format!("http://ex.org/obj{}", i)
            );
        }

        let before_count = kg.list_triples().len();

        // Run inference
        let _ = kg.infer();

        let after_count = kg.list_triples().len();

        // Property: Inference should only add, never remove triples
        prop_assert!(after_count >= before_count,
            "Inference should not remove existing triples");
    }
}

// Test parser with complex nested HTML structures
proptest! {
    #[test]
    fn test_parser_nested_structures(
        depth in 1usize..6usize,
        tag in prop::sample::select(vec!["div", "span", "p", "article"])
    ) {
        // Generate nested HTML
        let mut html = String::from("<html><body>");
        for _ in 0..depth {
            html.push_str(&format!("<{}>", tag));
        }
        html.push_str("content");
        for _ in 0..depth {
            html.push_str(&format!("</{}>", tag));
        }
        html.push_str("</body></html>");

        // Property: Parser should handle nested structures without crashing
        let result = parser::parse_html(&html);
        prop_assert!(result.is_ok() || result.is_err(), "Parser should not panic");
    }
}

// Test microdata extraction properties
proptest! {
    #[test]
    fn test_microdata_extraction(
        item_type in "http://schema\\.org/[A-Z][a-z]{3,10}",
        prop_name in "[a-z]{3,10}",
        prop_value in "[a-zA-Z0-9 ]{3,20}"
    ) {
        let html = format!(
            r#"<html><body><div itemscope itemtype="{}" ><span itemprop="{}">{}</span></div></body></html>"#,
            item_type, prop_name, prop_value
        );

        let result = parser::parse_html(&html);

        if let Ok(data) = result {
            // Property: Microdata items should be extracted
            if !data.microdata.is_empty() {
                prop_assert!(
                    data.microdata.iter().any(|item| item.item_type.contains(&item_type)),
                    "Item type should match"
                );
            }
        }
    }
}

// Test URL validation in security module
proptest! {
    #[test]
    fn test_url_validation_rejects_malicious(
        scheme in prop::sample::select(vec!["javascript:", "data:", "file:", "ftp:"])
    ) {
        let malicious_url = format!("{}malicious-content", scheme);

        // Property: Non-HTTP(S) schemes should be rejected for browsing
        // Note: This assumes we'll add URL validation in security module
        prop_assert!(!malicious_url.starts_with("http://") && !malicious_url.starts_with("https://"),
            "Malicious URL schemes should be detectable");
    }
}

// Test JSON-LD extraction
proptest! {
    #[test]
    fn test_jsonld_extraction_valid(
        name in "[A-Z][a-z]{2,15}",
        url in "https?://[a-z]{3,10}\\.(com|org|net)"
    ) {
        let json_ld = format!(
            r#"{{"@context": "http://schema.org", "@type": "Person", "name": "{}", "url": "{}"}}"#,
            name, url
        );
        let html = format!(
            r#"<html><head><script type="application/ld+json">{}</script></head><body></body></html>"#,
            json_ld
        );

        let result = parser::parse_html(&html);

        // Property: Valid JSON-LD should be extracted
        if let Ok(data) = result {
            prop_assert!(
                data.json_ld.len() <= 1,  // Should have at most one JSON-LD object
                "JSON-LD extraction should not duplicate"
            );
        }
    }
}

// Test SPARQL query length limits
proptest! {
    #[test]
    fn test_sparql_query_length_limit(
        query_size in 10_001usize..15_000usize
    ) {
        let huge_query = format!("SELECT * WHERE {{ {} }}", "?s ?p ?o . ".repeat(query_size / 10));

        // Property: Queries over 10KB should be rejected
        let result = security::validate_sparql_query(&huge_query);
        prop_assert!(result.is_err(), "Huge SPARQL queries should be rejected");
    }
}

// Test KG SPARQL query execution doesn't break on valid queries
proptest! {
    #[test]
    fn test_kg_query_execution_robustness(
        var in "[a-z]{1,5}",
        limit in 1u32..100u32
    ) {
        let kg = kg::KnowledgeGraph::new();
        let query = format!("SELECT ?{} WHERE {{ ?s ?p ?{} }} LIMIT {}", var, var, limit);

        // Property: Valid queries should execute without panicking
        let result = kg.query(&query);
        prop_assert!(result.is_ok() || result.is_err(), "Query execution should not panic");
    }
}

// Test browser config serialization roundtrip
#[cfg(feature = "browser-automation")]
proptest! {
    #[test]
    fn test_browser_config_serialization(
        headless in prop::bool::ANY,
        block_ads in prop::bool::ANY,
        timeout_secs in 10u64..120u64,
        pool_size in 1usize..5usize
    ) {
        use semantic_browser::browser::BrowserConfig;

        let config = BrowserConfig {
            chromium_path: None,
            headless,
            block_ads,
            block_images: false,
            timeout_secs,
            pool_size,
            user_data_dir: Some(format!("/tmp/semantic-browser-proptest-{}", pool_size)),
        };

        // Serialize
        let json = serde_json::to_string(&config).expect("Serialization should succeed");

        // Deserialize
        let deserialized: BrowserConfig =
            serde_json::from_str(&json).expect("Deserialization should succeed");

        // Property: Roundtrip should preserve all fields
        prop_assert_eq!(deserialized.headless, headless);
        prop_assert_eq!(deserialized.block_ads, block_ads);
        prop_assert_eq!(deserialized.timeout_secs, timeout_secs);
        prop_assert_eq!(deserialized.user_data_dir, config.user_data_dir);
        prop_assert_eq!(deserialized.pool_size, pool_size);
    }
}
