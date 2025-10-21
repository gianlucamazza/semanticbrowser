# Testing Guide - Semantic Browser

Complete guide for testing the Semantic Browser project.

## Test Structure

```
tests/
├── integration_test.rs         # End-to-end integration tests
│
src/
├── parser.rs                   # Unit tests: test_parse_html_basic
├── annotator.rs                # Unit tests: test_regex_entities
├── kg.rs                       # Unit tests: test_kg_insert_and_list, test_kg_query
├── api.rs                      # Unit tests: test_rate_limit_logic, test_check_auth
├── security.rs                 # Unit tests: test_sandbox_wrapper, test_html_validation
└── external.rs                 # Unit tests: test_browse_mock
│
benches/
└── parsing_benchmark.rs        # Performance benchmarks
```

## Running Tests

### Local Testing (Cargo)

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_parse_html_basic

# Run integration tests only
cargo test --test integration_test

# Run with specific log level
RUST_LOG=debug cargo test

# Run benchmarks
cargo bench
```

### Docker Testing (Recommended)

```bash
# Complete test suite
./scripts/docker-test.sh

# Quick unit tests only
./scripts/docker-test.sh --unit-only

# Integration tests with server
./scripts/docker-test.sh --integration-only

# Linting checks
./scripts/docker-test.sh --lint-only

# Include benchmarks
./scripts/docker-test.sh --with-bench

# Clean up after tests
./scripts/docker-test.sh --clean
```

## Test Coverage

### Unit Tests (11 tests)

**Parser Module** (src/parser.rs)
- ✅ `test_parse_html_basic`: Basic HTML parsing with title and JSON-LD

**Annotator Module** (src/annotator.rs)
- ✅ `test_regex_entities`: Entity extraction from text

**Knowledge Graph Module** (src/kg.rs)
- ✅ `test_kg_insert_and_list`: Triple insertion and listing
- ✅ `test_kg_query`: SPARQL query execution

**API Module** (src/api.rs)
- ✅ `test_rate_limit_logic`: Rate limiting functionality
- ✅ `test_check_auth`: Authentication checking
- ✅ `test_extract_ip`: IP extraction from headers

**Security Module** (src/security.rs)
- ✅ `test_sandbox_wrapper`: Sandbox function wrapper
- ✅ `test_html_validation`: HTML input validation
- ✅ `test_sparql_validation`: SPARQL query validation

**External Module** (src/external.rs)
- ✅ `test_browse_mock`: URL browsing functionality

### Integration Tests (6 tests)

**tests/integration_test.rs**
- ✅ `test_html_parsing_integration`: Full HTML parsing workflow
- ✅ `test_entity_extraction_integration`: Entity extraction workflow
- ✅ `test_knowledge_graph_integration`: KG operations
- ✅ `test_sparql_update_integration`: SPARQL INSERT operations
- ✅ `test_security_validation`: Security checks
- ✅ `test_kg_inference`: ML inference workflow

### Benchmarks (3 benchmarks)

**benches/parsing_benchmark.rs**
- ⚡ `bench_parse_html`: HTML parsing performance
- ⚡ `bench_kg_insert`: KG triple insertion performance
- ⚡ `bench_kg_query`: SPARQL query performance

## Test Scenarios

### 1. HTML Parsing

```rust
#[test]
fn test_parse_html_basic() {
    let html = r#"
    <html>
        <head>
            <title>Test Page</title>
            <script type="application/ld+json">
            {"@type": "Person", "name": "John"}
            </script>
        </head>
        <body>
            <div itemscope itemtype="http://schema.org/Product">
                <span itemprop="name">Widget</span>
            </div>
        </body>
    </html>
    "#;

    let result = parse_html(html).unwrap();
    assert_eq!(result.title, Some("Test Page".to_string()));
    assert_eq!(result.json_ld.len(), 1);
    assert!(result.microdata.len() > 0);
}
```

### 2. Knowledge Graph Operations

```rust
#[test]
fn test_knowledge_graph_integration() {
    let mut kg = KnowledgeGraph::new();

    // Insert triple
    kg.insert(
        "http://example.org/person1",
        "http://xmlns.com/foaf/0.1/name",
        "http://example.org/alice"
    ).unwrap();

    // List triples
    let triples = kg.list_triples();
    assert_eq!(triples.len(), 1);

    // Query
    let results = kg.query("SELECT * WHERE { ?s ?p ?o }").unwrap();
    assert!(!results.is_empty());
}
```

### 3. Security Validation

```rust
#[test]
fn test_security_validation() {
    // Valid HTML
    assert!(validate_html_input("<html><body>Test</body></html>").is_ok());

    // Invalid: too large
    let huge_html = "a".repeat(20_000_000);
    assert!(validate_html_input(&huge_html).is_err());

    // Invalid: malicious content
    let malicious = r#"<html><script>alert('xss')</script></html>"#;
    assert!(validate_html_input(malicious).is_err());

    // SPARQL validation
    assert!(validate_sparql_query("SELECT * WHERE { ?s ?p ?o }").is_ok());
    assert!(validate_sparql_query("DROP ALL").is_err());
}
```

## Integration Test Workflow

### Docker-based Integration Testing

1. **Build test images**:
   ```bash
   docker-compose -f docker-compose.test.yml build
   ```

2. **Start test server**:
   ```bash
   docker-compose -f docker-compose.test.yml up -d test_server
   ```

3. **Wait for health**:
   ```bash
   docker-compose -f docker-compose.test.yml ps test_server
   ```

4. **Run tests**:
   ```bash
   docker-compose -f docker-compose.test.yml run --rm integration_test
   ```

5. **Cleanup**:
   ```bash
   docker-compose -f docker-compose.test.yml down
   ```

### Manual Integration Testing

1. **Start server**:
   ```bash
   cargo run &
   SERVER_PID=$!
   ```

2. **Wait for startup**:
   ```bash
   sleep 2
   ```

3. **Run curl tests**:
   ```bash
   # Parse HTML
   curl -X POST http://localhost:3000/parse \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer secret" \
     -d '{"html": "<html><title>Test</title></html>"}'

   # Query KG
   curl -X POST http://localhost:3000/query \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer secret" \
     -d '{"query": "SELECT * WHERE { ?s ?p ?o }"}'
   ```

4. **Cleanup**:
   ```bash
   kill $SERVER_PID
   ```

## Continuous Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml
jobs:
  test:
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --lib
      - run: cargo test --test integration_test

  lint:
    steps:
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
```

### Local CI Simulation

```bash
# Simulate full CI pipeline
./scripts/docker-test.sh

# Individual steps
docker-compose -f docker-compose.test.yml run --rm lint_runner
docker-compose -f docker-compose.test.yml run --rm test_runner
docker-compose -f docker-compose.test.yml run --rm integration_test
docker-compose -f docker-compose.test.yml run --rm benchmark
```

## Test Data

### Example HTML

```html
<html>
  <head>
    <title>Product Page</title>
    <script type="application/ld+json">
    {
      "@context": "https://schema.org",
      "@type": "Product",
      "name": "Example Product",
      "offers": {
        "@type": "Offer",
        "price": "19.99",
        "priceCurrency": "USD"
      }
    }
    </script>
  </head>
  <body>
    <div itemscope itemtype="http://schema.org/Product">
      <span itemprop="name">Example Product</span>
      <span itemprop="price">$19.99</span>
    </div>
  </body>
</html>
```

### Example SPARQL

```sparql
# Insert data
INSERT DATA {
  <http://example.org/product1> <http://xmlns.com/foaf/0.1/name> "Widget" .
  <http://example.org/product1> <http://schema.org/price> "19.99" .
}

# Query data
SELECT ?product ?price
WHERE {
  ?product <http://schema.org/price> ?price .
}

# Construct new triples
CONSTRUCT {
  ?product <http://example.org/hasPrice> ?price .
}
WHERE {
  ?product <http://schema.org/price> ?price .
}
```

## Performance Testing

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench bench_parse_html

# Docker benchmark
./scripts/docker-test.sh --with-bench

# View results
cat target/criterion/*/report/index.html
```

### Expected Performance

| Operation | Target | Typical |
|-----------|--------|---------|
| HTML Parse (simple) | <1ms | ~0.5ms |
| KG Insert | <100µs | ~50µs |
| SPARQL Query | <1ms | ~0.8ms |
| API Request | <10ms | ~5ms |

## Debugging Tests

### Enable Logging

```bash
# Verbose output
RUST_LOG=debug cargo test -- --nocapture

# Specific module
RUST_LOG=semantic_browser::api=trace cargo test

# Docker with logs
RUST_LOG=debug ./scripts/docker-test.sh
```

### Debug Specific Test

```bash
# Run with backtrace
RUST_BACKTRACE=1 cargo test test_name

# Run in single thread (easier debugging)
cargo test -- --test-threads=1

# Debug in container
docker-compose -f docker-compose.test.yml run --rm test_runner \
  cargo test test_name -- --nocapture
```

## Test Coverage Analysis

### Using Tarpaulin

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html --output-dir coverage

# Open report
open coverage/index.html

# Docker coverage
docker-compose -f docker-compose.test.yml run --rm \
  --build-arg TARGET=coverage coverage
```

### Coverage Goals

- ✅ Unit Test Coverage: >80%
- ✅ Integration Test Coverage: >60%
- ✅ Critical Paths: 100%

## Best Practices

### Writing Tests

1. **Arrange-Act-Assert Pattern**
   ```rust
   #[test]
   fn test_something() {
       // Arrange
       let input = "test data";

       // Act
       let result = function_under_test(input);

       // Assert
       assert_eq!(result, expected);
   }
   ```

2. **Test Naming**
   ```rust
   // Good: Describes what is tested
   #[test]
   fn test_parse_html_extracts_title() { }

   // Bad: Generic name
   #[test]
   fn test1() { }
   ```

3. **Isolation**
   - Each test should be independent
   - Use fresh instances (KnowledgeGraph::new())
   - Clean up resources

4. **Edge Cases**
   - Test empty inputs
   - Test maximum values
   - Test error conditions

### Running Tests in CI

1. Always run linting first (fast feedback)
2. Run unit tests before integration tests
3. Cache dependencies for speed
4. Run tests in parallel when possible
5. Generate coverage reports

## Troubleshooting

### Common Issues

**Issue**: Tests pass locally but fail in CI
- **Solution**: Ensure same Rust version, check environment variables

**Issue**: Integration tests timeout
- **Solution**: Increase timeout, check server startup

**Issue**: Flaky tests
- **Solution**: Check for race conditions, timing issues

**Issue**: Docker tests slow
- **Solution**: Use BuildKit, cache volumes, prune old images

## Additional Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Criterion Benchmarking](https://github.com/bheisler/criterion.rs)
- [Docker Testing Best Practices](https://docs.docker.com/develop/dev-best-practices/)
