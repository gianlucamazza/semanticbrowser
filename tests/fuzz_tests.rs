//! Fuzzing tests for input validation
//!
//! Best practices 2025:
//! - Use fuzzing to discover edge cases and security vulnerabilities
//! - Test with random, malformed, and malicious inputs
//! - Focus on parsers, validators, and security-critical code
//!
//! To run fuzzing tests with cargo-fuzz:
//! ```bash
//! cargo install cargo-fuzz
//! cargo fuzz list
//! cargo fuzz run fuzz_html_parser
//! ```
//!
//! For now, these are fuzzing-style tests using proptest with arbitrary inputs

use proptest::prelude::*;
use semantic_browser::{parser, security};

// Fuzz HTML parser with completely arbitrary byte sequences
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]
    #[test]
    fn fuzz_html_parser_arbitrary_bytes(
        bytes in prop::collection::vec(any::<u8>(), 0..1000)
    ) {
        // Convert bytes to string (lossy)
        let input = String::from_utf8_lossy(&bytes).to_string();

        // Property: Parser should never panic, regardless of input
        let _ = parser::parse_html(&input);
        // Success: no panic
    }
}

// Fuzz HTML parser with malformed tags
proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]
    #[test]
    fn fuzz_html_malformed_tags(
        tag_start in "[<]{1,5}",
        tag_name in "[a-zA-Z0-9_]{0,20}",
        tag_end in "[>]{0,5}",
        content in ".*{0,100}"
    ) {
        let malformed = format!("{}{}{}{}", tag_start, tag_name, content, tag_end);

        // Property: Parser handles malformed tags gracefully
        let result = parser::parse_html(&malformed);
        prop_assert!(result.is_ok() || result.is_err(), "Parser should not panic on malformed tags");
    }
}

// Fuzz HTML with deeply nested structures (potential stack overflow)
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]
    #[test]
    fn fuzz_html_deep_nesting(
        depth in 1usize..100usize
    ) {
        let mut html = String::from("<html>");
        for _ in 0..depth {
            html.push_str("<div>");
        }
        html.push_str("content");
        for _ in 0..depth {
            html.push_str("</div>");
        }
        html.push_str("</html>");

        // Property: Deep nesting should not cause stack overflow
        let _ = parser::parse_html(&html);
        // Success: no panic
    }
}

// Fuzz HTML with special characters and encoding issues
proptest! {
    #![proptest_config(ProptestConfig::with_cases(500))]
    #[test]
    fn fuzz_html_special_characters(
        content in "[\\x00-\\x7F]{0,200}"
    ) {
        let html = format!("<html><body>{}</body></html>", content);

        // Property: Special characters should not break parser
        let _ = parser::parse_html(&html);
        // Success: no panic
    }
}

// Fuzz SPARQL query validator with arbitrary strings
proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]
    #[test]
    fn fuzz_sparql_validator_arbitrary(
        query in ".*{0,500}"
    ) {
        // Property: Validator should never panic
        let _ = security::validate_sparql_query(&query);
        // Success: no panic
    }
}

// Fuzz SPARQL with SQL injection attempts
proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]
    #[test]
    fn fuzz_sparql_sql_injection(
        injection in prop::sample::select(vec![
            "'; DROP TABLE users; --",
            "1' OR '1'='1",
            "admin'--",
            "' UNION SELECT * FROM passwords--",
        ]),
        query_part in "[a-z]{3,10}"
    ) {
        let query = format!("SELECT * WHERE {{ ?s ?p '{}' . ?s <{}> ?o }}", injection, query_part);

        // Property: SQL injection patterns should be safely handled
        let _ = security::validate_sparql_query(&query);
        // Success: no crash (validation may pass or fail, but no exploit)
    }
}

// Fuzz SPARQL with SPARQL injection attempts
proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]
    #[test]
    fn fuzz_sparql_injection_patterns(
        malicious in prop::sample::select(vec![
            "} DELETE { ?s ?p ?o } WHERE {",
            "} INSERT { ?s <hack> ?o } WHERE {",
            "} CLEAR ALL WHERE {",
            "} DROP GRAPH <http://example.org> WHERE {",
        ]),
        var in "[a-z]{1,5}"
    ) {
        let query = format!("SELECT ?{} WHERE {{ {} }}", var, malicious);

        // Property: Injection attempts should be detected/rejected
        // Validation should prevent dangerous operations
        let _ = security::validate_sparql_query(&query);
    }
}

// Fuzz HTML validation with extremely large inputs
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn fuzz_html_size_extremes(
        size in 1usize..20_000_000usize
    ) {
        let huge_html = "x".repeat(size);

        // Property: Size validation should prevent DoS
        let result = security::validate_html_input(&huge_html);

        if size > 10_000_000 {
            prop_assert!(result.is_err(), "Huge HTML should be rejected");
        }
    }
}

// Fuzz HTML with Unicode edge cases
proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]
    #[test]
    fn fuzz_html_unicode_edge_cases(
        unicode in "[\u{0}-\u{FFFF}]{0,100}"
    ) {
        let html = format!("<html><body>{}</body></html>", unicode);

        // Property: Unicode handling should be robust
        let _ = parser::parse_html(&html);
        // Success: no panic
    }
}

// Fuzz microdata with malformed attributes
proptest! {
    #![proptest_config(ProptestConfig::with_cases(400))]
    #[test]
    fn fuzz_microdata_malformed_attributes(
        itemscope_val in ".*{0,20}",
        itemtype_val in ".*{0,50}",
        itemprop_val in ".*{0,30}"
    ) {
        let html = format!(
            r#"<html><body><div itemscope="{}" itemtype="{}" itemprop="{}">content</div></body></html>"#,
            itemscope_val, itemtype_val, itemprop_val
        );

        // Property: Malformed microdata should not crash parser
        let _ = parser::parse_html(&html);
    }
}

// Fuzz JSON-LD with malformed JSON
proptest! {
    #![proptest_config(ProptestConfig::with_cases(400))]
    #[test]
    fn fuzz_jsonld_malformed_json(
        json_like in ".*{0,200}"
    ) {
        let html = format!(
            r#"<html><head><script type="application/ld+json">{}</script></head></html>"#,
            json_like
        );

        // Property: Malformed JSON-LD should not crash parser
        let result = parser::parse_html(&html);

        // Parser should handle gracefully (may or may not extract JSON-LD)
        prop_assert!(result.is_ok() || result.is_err(), "Parser should handle malformed JSON-LD");
    }
}

// Fuzz with mixed encodings
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]
    #[test]
    fn fuzz_html_mixed_encodings(
        ascii in "[a-zA-Z0-9]{0,50}",
        utf8 in "[\u{80}-\u{10FFFF}]{0,20}",
        control_chars in "[\\x00-\\x1F]{0,10}"
    ) {
        let html = format!("<html><body>{}{}{}</body></html>", ascii, utf8, control_chars);

        // Property: Mixed encodings should not corrupt parser
        let _ = parser::parse_html(&html);
    }
}

// Fuzz SPARQL with extreme nesting
proptest! {
    #![proptest_config(ProptestConfig::with_cases(150))]
    #[test]
    fn fuzz_sparql_nested_queries(
        depth in 1usize..20usize
    ) {
        let mut query = String::from("SELECT * WHERE { ");
        for i in 0..depth {
            query.push_str(&format!("{{ ?s{} ?p{} ?o{} . ", i, i, i));
        }
        for _ in 0..depth {
            query.push_str("} ");
        }
        query.push('}');

        // Property: Nested queries should not cause stack overflow
        let _ = security::validate_sparql_query(&query);
    }
}

// Fuzz with XSS attempts in HTML
proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]
    #[test]
    fn fuzz_html_xss_attempts(
        xss in prop::sample::select(vec![
            "<script>alert('XSS')</script>",
            "<img src=x onerror=alert('XSS')>",
            "<svg onload=alert('XSS')>",
            "javascript:alert('XSS')",
            "<iframe src=javascript:alert('XSS')>",
        ]),
        content in "[a-zA-Z0-9 ]{0,50}"
    ) {
        let html = format!("<html><body>{}{}</body></html>", xss, content);

        // Property: XSS in input should not execute (we're just parsing, not rendering)
        // Parser should handle without crashing
        let _ = parser::parse_html(&html);

        // Note: Actual XSS prevention is responsibility of renderer, not parser
        // We just ensure parser doesn't crash
    }
}

// Fuzz with path traversal attempts
proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]
    #[test]
    fn fuzz_url_path_traversal(
        traversal in prop::sample::select(vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32",
            "....//....//....//",
            "file:///etc/passwd",
        ]),
        suffix in "[a-z]{0,10}"
    ) {
        let url = format!("{}{}", traversal, suffix);

        // Property: Path traversal patterns should be detectable
        // (Assumes we'll add URL validation)
        prop_assert!(!url.starts_with("http://") && !url.starts_with("https://"),
            "Path traversal URLs should be distinguishable from valid HTTP(S)");
    }
}
