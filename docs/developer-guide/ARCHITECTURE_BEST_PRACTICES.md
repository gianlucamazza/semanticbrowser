# Architecture & Logic Best Practices

> **Semantic Browser** - Comprehensive guide to architectural patterns, design principles, and implementation best practices used throughout the codebase.

## Table of Contents

1. [Overview](#overview)
2. [Architectural Principles](#architectural-principles)
3. [Design Patterns](#design-patterns)
4. [Concurrency & Async Programming](#concurrency--async-programming)
5. [Error Handling Strategy](#error-handling-strategy)
6. [Security Architecture](#security-architecture)
7. [Performance Optimization](#performance-optimization)
8. [Testing Philosophy](#testing-philosophy)
9. [Code Organization](#code-organization)
10. [Observability & Monitoring](#observability--monitoring)
11. [Best Practices Summary](#best-practices-summary)

---

## Overview

Semantic Browser is architected as a **multi-layer, feature-modular Rust application** designed for AI agents. The architecture embodies modern Rust best practices (2025), emphasizing:

- **Type safety** - Leveraging Rust's type system for correctness
- **Async-first** - Non-blocking I/O throughout the stack
- **Modularity** - Optional features via Cargo feature flags
- **Security** - Defense-in-depth with multiple security layers
- **Performance** - Zero-copy parsing, connection pooling, streaming
- **Testability** - Comprehensive test coverage with multiple strategies

---

## Architectural Principles

### 1. **Layered Architecture**

The application follows a clean layered architecture:

```
┌─────────────────────────────────────────────────────────┐
│                    API Layer (HTTP)                     │
│  - REST endpoints (Axum)                                │
│  - Authentication middleware                            │
│  - Rate limiting                                        │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│              Business Logic Layer                       │
│  - Agent orchestration                                  │
│  - LLM integration                                      │
│  - Browser automation                                   │
│  - Form filling logic                                   │
└────────────────┬────────────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────────────┐
│               Data & Integration Layer                  │
│  - Knowledge Graph (Oxigraph)                           │
│  - HTML Parser (html5ever)                              │
│  - ML Models (ONNX/Tract)                               │
│  - External APIs (reqwest)                              │
└─────────────────────────────────────────────────────────┘
```

**Benefits:**
- Clear separation of concerns
- Easy to test individual layers
- Can replace implementations without affecting other layers

**Implementation:**
- See `src/api.rs` for API layer
- See `src/llm/agent.rs` for business logic
- See `src/kg.rs`, `src/parser.rs` for data layer

### 2. **Feature-Based Modularity**

The codebase uses **Cargo feature flags** to enable/disable functionality:

```toml
[features]
default = []
pyo3-integration = ["pyo3"]
onnx-integration = ["tract-onnx", "tokenizers"]
browser-automation = ["chromiumoxide"]
redis-integration = ["redis"]
llm-openai = []
llm-anthropic = []
telemetry = ["opentelemetry", "tracing-opentelemetry"]
observability = ["prometheus"]
seccomp = ["seccompiler"]  # Linux only
```

**Benefits:**
- Minimal binary size when features aren't needed
- Faster compilation for development
- Clear dependency boundaries
- Can deploy different configurations for different use cases

**Example Usage:**
```bash
# Minimal build (no LLM, no browser)
cargo build --release --no-default-features

# Full-featured build
cargo build --release --all-features

# Only OpenAI LLM + Browser automation
cargo build --release --features "llm-openai,browser-automation"
```

### 3. **Dependency Inversion Principle**

High-level modules don't depend on low-level modules; both depend on abstractions.

**Example: LLM Provider Abstraction**

```rust
// src/llm/provider.rs
#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn chat_completion(&self, messages: Vec<Message>, config: &LLMConfig) 
        -> LLMResult<LLMResponse>;
    
    fn provider_name(&self) -> &str;
    async fn health_check(&self) -> LLMResult<bool>;
}

// Implementations:
// - src/llm/openai.rs: OpenAIProvider
// - src/llm/anthropic.rs: AnthropicProvider
// - src/llm/ollama.rs: OllamaProvider
```

**Agent orchestrator depends on trait, not concrete implementations:**

```rust
pub struct AgentOrchestrator {
    provider: Arc<dyn LLMProvider>,  // ← Trait object
    config: LLMConfig,
    tools: ToolRegistry,
}
```

**Benefits:**
- Easy to add new LLM providers
- Testable with mock implementations
- Runtime polymorphism for multi-provider support

---

## Design Patterns

### 1. **Trait-Based Polymorphism** ⭐

**Pattern:** Define behavior as traits, implement for multiple types.

**Location:** Throughout the codebase

**Examples:**

**LLM Provider Pattern:**
```rust
// src/llm/provider.rs
#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn chat_completion(&self, messages: Vec<Message>, config: &LLMConfig) 
        -> LLMResult<LLMResponse>;
    
    async fn stream_chat_completion(&self, messages: Vec<Message>, config: &LLMConfig) 
        -> LLMResult<tokio::sync::mpsc::Receiver<String>>;
    
    fn supports_vision(&self) -> bool { false }
    fn provider_name(&self) -> &str;
}
```

**Benefits:**
- Zero-cost abstraction (static dispatch when possible)
- Easy to extend with new implementations
- Compiler-enforced contracts

### 2. **Builder Pattern**

**Pattern:** Fluent API for constructing complex objects.

**Location:** Configuration structs

**Example:**

```rust
// src/llm/agent.rs
let task = AgentTask::new("Navigate to github.com")
    .with_context("Find trending repositories")
    .with_max_iterations(5);

// src/browser.rs
let options = NavigationOptions {
    take_screenshot: true,
    ..Default::default()
};
```

**Benefits:**
- Optional parameters without huge function signatures
- Immutable-by-default with controlled mutation
- Self-documenting API

### 3. **Repository Pattern**

**Pattern:** Abstract data access behind a clean interface.

**Location:** `src/kg.rs`

**Example:**

```rust
pub struct KnowledgeGraph {
    store: Store,  // Oxigraph backend (hidden)
}

impl KnowledgeGraph {
    pub fn insert(&mut self, s: &str, p: &str, o: &str) 
        -> Result<(), Box<dyn std::error::Error>> {
        // Internal implementation details hidden
    }
    
    pub fn query(&self, sparql: &str) 
        -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Can switch backends without affecting callers
    }
}
```

**Benefits:**
- Decouple business logic from storage implementation
- Can swap Oxigraph for another triple store
- Easier to test with in-memory implementation

### 4. **Strategy Pattern**

**Pattern:** Encapsulate algorithms and make them interchangeable.

**Location:** ML inference, LLM providers

**Example:**

```rust
// src/kg.rs
pub fn infer(&mut self) -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(model_path) = std::env::var("KG_INFERENCE_MODEL_PATH") {
        // Strategy 1: ML-based inference
        self.run_ml_inference(&model_path)?;
    } else {
        // Strategy 2: Rule-based inference
        self.infer_rules_based()?;
    }
}
```

**Benefits:**
- Runtime algorithm selection
- Easy to add new strategies
- Keeps code DRY

### 5. **Singleton Pattern (Rust Style)**

**Pattern:** Global state with lazy initialization.

**Location:** `src/auth.rs`

**Example:**

```rust
use std::sync::OnceLock;

static JWT_STATE: OnceLock<AuthState> = OnceLock::new();

impl JwtConfig {
    pub fn init() -> Result<(), String> {
        let secret = std::env::var("JWT_SECRET")?;
        let config = JwtConfig::new(secret)?;
        
        JWT_STATE.set(AuthState::Enabled(config))
            .map_err(|_| "Already initialized")?;
        
        Ok(())
    }
    
    pub fn get() -> Option<&'static JwtConfig> {
        match JWT_STATE.get()? {
            AuthState::Enabled(config) => Some(config),
            AuthState::Disabled => None,
        }
    }
}
```

**Benefits:**
- Thread-safe initialization
- No mutex overhead after initialization
- Zero-cost when feature disabled

### 6. **Middleware Chain Pattern**

**Pattern:** Composable request/response processing.

**Location:** `src/api.rs`

**Example:**

```rust
use tower_http::trace::TraceLayer;
use tower_http::cors::CorsLayer;

let app = Router::new()
    .route("/parse", post(parse_handler))
    .route("/query", post(query_handler))
    .layer(TraceLayer::new_for_http())  // Logging
    .layer(CorsLayer::permissive())     // CORS
    .layer(Extension(state));            // State injection
```

**Benefits:**
- Composable cross-cutting concerns
- Declarative middleware stack
- Easy to add/remove middleware

---

## Concurrency & Async Programming

### 1. **Async-First Design** ⭐⭐⭐

**Statistic:** **550+ occurrences** of `async fn`, `await`, `tokio::spawn`, `Arc<>`, `Mutex<>` found in codebase.

**Philosophy:** All I/O operations are async to maximize throughput.

**Example:**

```rust
// src/llm/openai.rs
#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn chat_completion(&self, messages: Vec<Message>, config: &LLMConfig) 
        -> LLMResult<LLMResponse> {
        
        let response = self.client
            .post(&self.endpoint)
            .json(&request_body)
            .send()        // ← async HTTP
            .await?;
        
        let data: OpenAIResponse = response.json().await?;  // ← async parse
        Ok(self.convert_response(data))
    }
}
```

**Benefits:**
- High concurrency without thread-per-request overhead
- Efficient use of system resources
- Scales to thousands of concurrent operations

### 2. **Shared State Management**

**Pattern:** `Arc<Mutex<T>>` for thread-safe shared state.

**Example:**

```rust
// src/api.rs
#[derive(Clone)]
pub struct AppState {
    pub kg: Arc<Mutex<KnowledgeGraph>>,
    pub rate_limits: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
}

// Handler acquires lock
async fn query_handler(
    State(state): State<AppState>,
    Json(request): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, StatusCode> {
    let kg = state.kg.lock().await;  // ← Async lock
    let results = kg.query(&request.query)?;
    Ok(Json(QueryResponse { results }))
}
```

**Best Practices:**
- ✅ Hold locks for minimal time
- ✅ Use `tokio::sync::Mutex` (not `std::sync::Mutex`) for async code
- ✅ Clone `Arc` when passing to spawned tasks
- ❌ Avoid holding locks across `.await` points

### 3. **Concurrent Task Spawning**

**Pattern:** `tokio::spawn` for background tasks.

**Example:**

```rust
// src/browser.rs
let (browser, mut handler) = Browser::launch(config).await?;

// Spawn handler in background
tokio::spawn(async move {
    while let Some(_) = handler.next().await {
        // Process browser events
    }
});

// Browser ready for use immediately
```

**Benefits:**
- Non-blocking background processing
- Graceful cancellation with drop
- Structured concurrency

### 4. **Connection Pooling**

**Pattern:** Reuse expensive resources (browsers, HTTP clients).

**Example:**

```rust
// src/browser.rs
pub struct BrowserPool {
    browser: Arc<Browser>,
    tab_manager: Arc<Mutex<TabManager>>,
    pool_size: usize,
}

impl BrowserPool {
    pub async fn navigate_and_extract(&self, url: &str, options: NavigationOptions) 
        -> Result<SemanticData, Box<dyn std::error::Error>> {
        
        // Acquire tab from pool
        let page = self.get_or_create_tab().await?;
        
        // Use tab
        page.goto(url).await?;
        let data = self.extract_semantic_data(&page).await?;
        
        // Tab returned to pool automatically
        Ok(data)
    }
}
```

**Benefits:**
- Amortize expensive setup costs
- Limit concurrent resource usage
- Improve throughput

### 5. **Streaming for Large Data**

**Pattern:** Process data incrementally instead of loading everything into memory.

**Example - Token Streaming:**

```rust
// src/llm/openai.rs
pub async fn stream_chat_completion(&self, messages: Vec<Message>, config: &LLMConfig) 
    -> LLMResult<tokio::sync::mpsc::Receiver<String>> {
    
    let (tx, rx) = tokio::sync::mpsc::channel(100);
    
    let response = self.client
        .post(&self.endpoint)
        .json(&request_body)
        .send()
        .await?;
    
    let mut stream = response.bytes_stream();
    
    tokio::spawn(async move {
        while let Some(chunk) = stream.next().await {
            // Parse SSE chunk
            // Extract token
            // Send via channel
            tx.send(token).await.ok();
        }
    });
    
    Ok(rx)
}
```

**Benefits:**
- Constant memory usage regardless of response size
- Start processing before full response received
- Better user experience (progressive display)

---

## Error Handling Strategy

### 1. **Type-Safe Errors with `thiserror`** ⭐

**Pattern:** Define custom error enums with `thiserror` for ergonomic error handling.

**Example:**

```rust
// src/llm/provider.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LLMError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("API error: {0}")]
    Api(String),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

pub type LLMResult<T> = Result<T, LLMError>;
```

**Benefits:**
- Structured error types with context
- Automatic `From` implementations for error conversion
- Beautiful error messages with `Display`
- Can match on specific error variants

### 2. **Error Propagation with `?`**

**Pattern:** Use `?` operator for concise error handling.

**Example:**

```rust
pub async fn execute_task(&self, task: AgentTask) -> LLMResult<AgentResponse> {
    let messages = self.build_messages(&task)?;  // ← Propagate error
    let response = self.provider.chat_completion(messages, &self.config).await?;
    let result = self.parse_response(response)?;
    Ok(result)
}
```

**Benefits:**
- Concise error handling
- Early return on error
- Error context preserved through the stack

### 3. **Defensive Programming**

**Pattern:** Validate inputs, handle edge cases, never panic in library code.

**Example:**

```rust
// src/security.rs
pub fn validate_html_input(html: &str) -> Result<(), &'static str> {
    // Size limit
    if html.len() > 10_000_000 {
        return Err("HTML too large");
    }
    
    // Dangerous content
    let html_lower = html.to_lowercase();
    if html_lower.contains("<script") || html_lower.contains("javascript:") {
        return Err("Potentially malicious HTML detected");
    }
    
    Ok(())
}
```

**Best Practices:**
- ✅ Validate all external inputs
- ✅ Return `Result` instead of panicking
- ✅ Use `expect()` with descriptive messages for unrecoverable errors
- ✅ Document failure modes in function comments
- ❌ Never `unwrap()` in library code (only in examples/tests)

### 4. **Error Context with `map_err`**

**Pattern:** Add context to errors as they propagate.

**Example:**

```rust
pub async fn run_ml_inference(&mut self, model_path: &str) 
    -> Result<(), Box<dyn std::error::Error>> {
    
    let predictor = LinkPredictor::load(model_path)
        .map_err(|e| format!("Failed to load ML model from {}: {}", model_path, e))?;
    
    predictor.predict(entity, relation)
        .map_err(|e| format!("Inference failed for ({}, {}): {}", entity, relation, e))?;
    
    Ok(())
}
```

**Benefits:**
- Error messages include full context
- Easier debugging
- Better error reporting to users

---

## Security Architecture

### 1. **Defense in Depth** ⭐

Multiple layers of security:

1. **Input Validation** - All external inputs validated
2. **Authentication** - JWT tokens for API access
3. **Authorization** - Role-based access control (RBAC)
4. **Rate Limiting** - Per-IP request limits
5. **Sandboxing** - Seccomp syscall filtering (Linux)
6. **Audit Logging** - Security events logged

### 2. **JWT Authentication**

**Location:** `src/auth.rs`

**Implementation:**

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,       // Username
    pub exp: i64,          // Expiration timestamp
    pub iat: i64,          // Issued at
    pub role: Option<String>,  // RBAC role
}

// Axum extractor for automatic token validation
#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = Response;
    
    async fn from_request_parts(parts: &mut Parts, _state: &S) 
        -> Result<Self, Self::Rejection> {
        
        // Extract token from Authorization header
        let token = parts.headers.get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or_else(|| unauthorized())?;
        
        // Validate token
        let config = JwtConfig::get().ok_or_else(|| unauthorized())?;
        let claims = config.validate_token(token)?;
        
        // Check revocation (if Redis enabled)
        #[cfg(feature = "redis-integration")]
        if let Some(store) = TokenRevocationStore::get() {
            if store.is_revoked(token).await? {
                return Err(unauthorized());
            }
        }
        
        Ok(claims)
    }
}
```

**Features:**
- Stateless authentication
- Token expiration
- Optional token revocation with Redis
- Automatic extraction in handlers

**Example Usage:**

```rust
async fn protected_handler(
    claims: Claims,  // ← Automatic authentication
    State(state): State<AppState>,
) -> Result<Json<Response>, StatusCode> {
    // Only called if token is valid
    tracing::info!("Request from user: {}", claims.sub);
    Ok(Json(response))
}
```

### 3. **Input Validation**

**Location:** `src/security.rs`

**HTML Validation:**
```rust
pub fn validate_html_input(html: &str) -> Result<(), &'static str> {
    if html.len() > 10_000_000 {
        return Err("HTML too large");
    }
    
    let html_lower = html.to_lowercase();
    if html_lower.contains("<script") || html_lower.contains("javascript:") {
        return Err("Malicious HTML detected");
    }
    
    Ok(())
}
```

**SPARQL Validation:**
```rust
pub fn validate_sparql_query(query: &str) -> Result<(), &'static str> {
    if query.len() > 10_000 {
        return Err("Query too long");
    }
    
    // Whitelist approach
    let allowed = ["SELECT", "INSERT", "DELETE", "CONSTRUCT", "ASK", "DESCRIBE"];
    let is_valid = allowed.iter().any(|op| query.trim().to_uppercase().starts_with(op));
    
    if !is_valid {
        return Err("Unsupported SPARQL operation");
    }
    
    // Block dangerous operations
    if query.to_uppercase().contains("DROP") {
        return Err("Dangerous operation blocked");
    }
    
    Ok(())
}
```

### 4. **Rate Limiting**

**Location:** `src/api.rs`

**Implementation:**

```rust
pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = extract_real_ip(&request.headers(), &addr);
    
    let mut limits = state.rate_limits.lock().await;
    let (count, window_start) = limits.entry(ip.clone())
        .or_insert((0, Instant::now()));
    
    // Reset window if expired
    if window_start.elapsed() > Duration::from_secs(60) {
        *count = 0;
        *window_start = Instant::now();
    }
    
    // Check limit
    if *count >= 100 {  // 100 requests per minute
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    
    *count += 1;
    Ok(next.run(request).await)
}
```

### 5. **Seccomp Sandboxing (Linux)**

**Location:** `src/security.rs`

**Purpose:** Restrict syscalls to minimal set needed for parsing.

```rust
#[cfg(all(target_os = "linux", feature = "seccomp"))]
pub fn sandbox_parsing<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    use seccompiler::{apply_filter, BpfProgram};
    
    // Whitelist safe syscalls
    let allowed_syscalls = vec![
        libc::SYS_brk,      // Memory
        libc::SYS_mmap,
        libc::SYS_read,     // I/O
        libc::SYS_write,
        libc::SYS_futex,    // Synchronization
    ];
    
    let filter = build_seccomp_filter(allowed_syscalls);
    apply_filter(&filter).expect("Failed to apply seccomp filter");
    
    f()  // Execute with restricted syscalls
}
```

**Benefits:**
- Limits attack surface
- Prevents exploitation even if parsing code has vulnerabilities
- Industry best practice for untrusted input processing

### 6. **Security Event Logging**

All security-relevant events are logged:

```rust
use tracing::{info, warn, error};

// Authentication
info!("User {} authenticated with role {:?}", claims.sub, claims.role);

// Authorization failures
warn!("Unauthorized access attempt from IP: {}", ip);

// Input validation failures
error!("Malicious input detected: {}", validation_error);

// Rate limit exceeded
warn!("Rate limit exceeded for IP: {}", ip);
```

---

## Performance Optimization

### 1. **Zero-Copy Parsing**

**Location:** `src/parser.rs`

**Technique:** Use `scraper` crate which builds on `html5ever` with zero-copy string handling.

```rust
pub fn parse_html(html: &str) -> ParseResult {
    let document = Html::parse_document(html);  // ← Zero-copy parse
    
    // Selector uses references to original document
    let selector = Selector::parse("script[type='application/ld+json']").unwrap();
    
    for element in document.select(&selector) {
        let text = element.text().collect::<String>();  // Only copy when needed
        // ...
    }
}
```

### 2. **Connection Pooling**

**HTTP Client Pooling:**
```rust
// src/api_client.rs
pub struct ApiClient {
    client: Client,  // ← Reuses connections internally
    config: ApiClientConfig,
}

impl ApiClient {
    pub fn new(base_url: &str) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .pool_max_idle_per_host(10)  // ← Connection pool
            .build()
            .expect("Failed to build client");
        
        Self { client, config }
    }
}
```

**Browser Tab Pooling:**
```rust
// src/browser.rs
pub struct BrowserPool {
    browser: Arc<Browser>,
    tabs: Arc<Mutex<Vec<Arc<Page>>>>,
    max_tabs: usize,
}
```

### 3. **Lazy Initialization**

**Pattern:** Defer expensive operations until needed.

```rust
use std::sync::OnceLock;

static JWT_CONFIG: OnceLock<JwtConfig> = OnceLock::new();

pub fn get_config() -> &'static JwtConfig {
    JWT_CONFIG.get_or_init(|| {
        // Expensive initialization only happens once
        JwtConfig::from_env().expect("JWT_SECRET not set")
    })
}
```

### 4. **Efficient Data Structures**

**RDF Triple Storage:**
```rust
// src/kg.rs uses Oxigraph which provides:
// - Indexed storage for fast queries
// - Memory-mapped files for large graphs
// - Efficient SPARQL query execution
```

**String Interning for Entities:**
```rust
// ML models use entity ID mapping to avoid string comparisons
pub struct LinkPredictor {
    entity_to_id: HashMap<String, usize>,  // String → ID once
    id_to_entity: Vec<String>,
    // Model works with IDs (usize comparison)
}
```

### 5. **Streaming APIs**

All large data operations support streaming:

- **LLM token streaming** - Process tokens as they arrive
- **File downloads** - Stream to disk without loading into memory
- **HTTP responses** - `bytes_stream()` for large responses

### 6. **Benchmarking**

**Location:** `benches/parsing_benchmark.rs`

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_html_parsing(c: &mut Criterion) {
    let html = std::fs::read_to_string("test_data/large_page.html").unwrap();
    
    c.bench_function("parse_html", |b| {
        b.iter(|| {
            semantic_browser::parser::parse_html(black_box(&html))
        });
    });
}

criterion_group!(benches, benchmark_html_parsing);
criterion_main!(benches);
```

**CI Integration:** Benchmarks run on every push to track performance regressions.

---

## Testing Philosophy

### 1. **Comprehensive Test Coverage**

**Statistics:**
- **125+ tests** across 10 test files
- Unit tests, integration tests, property-based tests, fuzz tests, stress tests

**Test Types:**

| Type | Files | Purpose |
|------|-------|---------|
| Unit Tests | `src/**/*.rs` | Test individual functions |
| Integration Tests | `tests/integration_test*.rs` | Test component interaction |
| Property-Based | `tests/proptest_tests.rs` | Test invariants |
| Fuzz Tests | `tests/fuzz_tests.rs` | Find edge cases |
| Stress Tests | `tests/stress_tests.rs` | Performance under load |
| Browser Tests | `tests/browser_test.rs` | End-to-end browser automation |

### 2. **Property-Based Testing**

**Location:** `tests/proptest_tests.rs`

**Example:**

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn html_parsing_never_panics(html in ".*") {
        // Property: parsing any string should never panic
        let result = semantic_browser::parser::parse_html(&html);
        // If this completes without panic, test passes
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn kg_insert_is_idempotent(s in "[a-z]+", p in "[a-z]+", o in "[a-z]+") {
        // Property: inserting same triple twice should be idempotent
        let mut kg = KnowledgeGraph::new();
        kg.insert(&s, &p, &o).ok();
        kg.insert(&s, &p, &o).ok();
        
        let count = kg.count_triples();
        assert_eq!(count, 1);  // Only one triple, not two
    }
}
```

### 3. **Feature Matrix Testing**

**CI Strategy:** Test all feature combinations to ensure they work independently and together.

```yaml
# .github/workflows/ci.yml
strategy:
  matrix:
    features:
      - "--no-default-features"
      - "--features browser-automation"
      - "--features llm-openai"
      - "--features llm-anthropic"
      - "--features onnx-integration"
      - "--all-features"
```

### 4. **Mock Implementations for Testing**

**Pattern:** Use trait objects to inject test doubles.

```rust
// Test with mock LLM provider
struct MockLLMProvider {
    responses: Vec<String>,
}

#[async_trait]
impl LLMProvider for MockLLMProvider {
    async fn chat_completion(&self, _: Vec<Message>, _: &LLMConfig) 
        -> LLMResult<LLMResponse> {
        Ok(LLMResponse {
            content: self.responses[0].clone(),
            // ...
        })
    }
    
    fn provider_name(&self) -> &str { "mock" }
    async fn health_check(&self) -> LLMResult<bool> { Ok(true) }
}

#[tokio::test]
async fn test_agent_with_mock_llm() {
    let provider = Arc::new(MockLLMProvider {
        responses: vec!["FINISH: Task complete".to_string()],
    });
    
    let agent = AgentOrchestrator::new(provider, LLMConfig::default(), tools);
    let result = agent.execute(task).await.unwrap();
    
    assert!(result.success);
}
```

### 5. **Integration Tests Structure**

```
tests/
├── integration_tests.rs       # Component integration
├── core_workflow.rs           # End-to-end workflows
├── browser_test.rs            # Browser automation
├── llm_openai.rs              # OpenAI provider
├── llm_anthropic.rs           # Anthropic provider
├── kg_integration_test.rs     # Knowledge graph operations
├── proptest_tests.rs          # Property-based tests
├── fuzz_tests.rs              # Fuzz testing
└── stress_tests.rs            # Load/stress testing
```

**Example Integration Test:**

```rust
// tests/integration_test.rs
#[tokio::test]
async fn test_full_workflow() {
    // 1. Parse HTML
    let html = r#"<html>...</html>"#;
    let parsed = semantic_browser::parser::parse_html(html).unwrap();
    
    // 2. Extract entities
    let entities = semantic_browser::annotator::extract_entities(&parsed.text).unwrap();
    
    // 3. Insert into KG
    let mut kg = KnowledgeGraph::new();
    for entity in entities {
        kg.insert(&entity.text, "rdf:type", &entity.entity_type).unwrap();
    }
    
    // 4. Query KG
    let results = kg.query("SELECT ?s WHERE { ?s rdf:type 'PERSON' }").unwrap();
    
    assert!(!results.is_empty());
}
```

---

## Code Organization

### 1. **Module Structure**

```
src/
├── lib.rs                      # Public API surface
├── main.rs                     # Binary entry point
├── api.rs                      # HTTP API (Axum)
├── api_client.rs               # HTTP client
├── auth.rs                     # JWT authentication
├── auth_manager.rs             # Auth management
├── security.rs                 # Security utilities
├── parser.rs                   # HTML parsing
├── annotator.rs                # NER
├── kg.rs                       # Knowledge Graph
├── kg_integration.rs           # KG integration helpers
├── browser.rs                  # Browser automation
├── form_analyzer.rs            # Form analysis
├── form_interaction.rs         # Form filling
├── smart_form_filler.rs        # AI-powered form filling
├── external.rs                 # External integrations
├── models.rs                   # Data models
├── observability.rs            # Telemetry
├── ml/                         # Machine learning
│   ├── embeddings.rs           # Entity embeddings
│   ├── inference.rs            # Link prediction
│   └── ner.rs                  # Named entity recognition
├── llm/                        # LLM integration
│   ├── provider.rs             # Provider trait
│   ├── openai.rs               # OpenAI implementation
│   ├── anthropic.rs            # Anthropic implementation
│   ├── ollama.rs               # Ollama implementation
│   ├── agent.rs                # Agent orchestration
│   ├── browser_executor.rs    # Browser tools for agents
│   ├── tools.rs                # Tool registry
│   └── workflow.rs             # Workflow management
└── bin/
    └── semantic_browser_mcp.rs # MCP server binary
```

### 2. **Public API Design**

**Principle:** Hide implementation details, expose minimal public surface.

```rust
// src/lib.rs
pub mod annotator;
pub mod api;
pub mod api_client;
pub mod auth;
pub mod browser;
pub mod external;
pub mod form_analyzer;
pub mod form_interaction;
pub mod kg;
pub mod kg_integration;
pub mod llm;
pub mod ml;
pub mod models;
pub mod observability;
pub mod parser;
pub mod security;
pub mod smart_form_filler;

// Internal modules not exposed
mod internal_helpers;  // Not public
```

### 3. **Feature-Gated Modules**

```rust
// src/ml/mod.rs
#[cfg(feature = "onnx-integration")]
pub mod embeddings;

#[cfg(feature = "onnx-integration")]
pub mod inference;

#[cfg(feature = "onnx-integration")]
pub mod ner;
```

**Benefits:**
- Modules only compiled when feature enabled
- Clear feature boundaries
- Faster compilation for minimal builds

### 4. **Documentation Standards**

Every public item has documentation:

```rust
/// Agent Orchestrator
///
/// Orchestrates LLM-based autonomous agents that can use tools to accomplish tasks.
/// Implements a ReAct-style (Reasoning + Acting) loop.
///
/// # Example
///
/// ```rust
/// use semantic_browser::llm::{AgentOrchestrator, AgentTask};
///
/// let agent = AgentOrchestrator::new(provider, config, tools);
/// let task = AgentTask::new("Find trending repositories on GitHub");
/// let result = agent.execute(task).await?;
/// ```
///
/// # Architecture
///
/// The agent follows a think-act-observe loop:
/// 1. THOUGHT: Analyze situation and plan next action
/// 2. ACTION: Execute tool or return result
/// 3. OBSERVATION: Process tool output
/// 4. Repeat until task complete
pub struct AgentOrchestrator {
    // ...
}
```

---

## Observability & Monitoring

### 1. **Structured Logging with `tracing`**

**Location:** Throughout codebase

**Levels:**
- `error!` - Critical errors requiring immediate attention
- `warn!` - Recoverable issues
- `info!` - Important state changes
- `debug!` - Detailed debugging information
- `trace!` - Very verbose tracing

**Example:**

```rust
use tracing::{info, warn, debug, instrument};

#[instrument(skip(self))]  // Auto-log function entry/exit
pub async fn execute_task(&self, task: AgentTask) -> LLMResult<AgentResponse> {
    info!("Starting task: {}", task.goal);
    
    for iteration in 0..task.max_iterations {
        debug!("Iteration {}/{}", iteration + 1, task.max_iterations);
        
        match self.step().await {
            Ok(response) => {
                info!("Task completed successfully after {} iterations", iteration + 1);
                return Ok(response);
            }
            Err(e) => {
                warn!("Step failed: {}, retrying...", e);
            }
        }
    }
    
    Err(LLMError::Api("Max iterations reached".to_string()))
}
```

**Configuration:**

```rust
// src/main.rs
fn init_logging() {
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();
}

// Usage: RUST_LOG=info cargo run
//        RUST_LOG=semantic_browser=debug cargo run
```

### 2. **OpenTelemetry Integration**

**Feature:** `telemetry`

**Location:** `src/observability.rs`

**Setup:**

```rust
#[cfg(feature = "telemetry")]
pub fn init_telemetry() -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry::global;
    use opentelemetry_otlp::WithExportConfig;
    use tracing_opentelemetry::OpenTelemetryLayer;
    
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")?);
    
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .install_batch(opentelemetry_sdk::runtime::Tokio)?;
    
    let otel_layer = OpenTelemetryLayer::new(tracer);
    
    tracing_subscriber::registry()
        .with(otel_layer)
        .init();
    
    Ok(())
}
```

**Benefits:**
- Distributed tracing across services
- Integration with Jaeger, Zipkin, etc.
- Performance profiling
- Service dependency mapping

### 3. **Prometheus Metrics**

**Feature:** `observability`

**Example Metrics:**

```rust
use prometheus::{IntCounter, IntGauge, Histogram, register_int_counter, register_histogram};
use lazy_static::lazy_static;

lazy_static! {
    static ref HTTP_REQUESTS: IntCounter = register_int_counter!(
        "http_requests_total",
        "Total HTTP requests"
    ).unwrap();
    
    static ref ACTIVE_CONNECTIONS: IntGauge = register_int_gauge!(
        "active_connections",
        "Currently active connections"
    ).unwrap();
    
    static ref REQUEST_DURATION: Histogram = register_histogram!(
        "http_request_duration_seconds",
        "HTTP request duration in seconds"
    ).unwrap();
}

// Usage in handlers
async fn handler() -> Response {
    HTTP_REQUESTS.inc();
    ACTIVE_CONNECTIONS.inc();
    
    let timer = REQUEST_DURATION.start_timer();
    let response = process_request().await;
    timer.observe_duration();
    
    ACTIVE_CONNECTIONS.dec();
    response
}
```

### 4. **Health Check Endpoints**

```rust
// src/api.rs
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
        uptime: get_uptime(),
    })
}

async fn readiness_handler(State(state): State<AppState>) -> StatusCode {
    // Check dependencies
    if state.kg.lock().await.is_healthy() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}
```

---

## Best Practices Summary

### ✅ DO

1. **Use `async/await` for all I/O operations**
   - Maximize concurrency
   - Non-blocking by default

2. **Define behavior as traits**
   - `#[async_trait]` for async trait methods
   - Easy to extend and test

3. **Use `thiserror` for custom errors**
   - Type-safe error handling
   - Rich error context

4. **Validate all external inputs**
   - Fail fast with meaningful errors
   - Never trust user input

5. **Use feature flags for optional functionality**
   - Minimal dependencies by default
   - Clear opt-in for features

6. **Document public APIs thoroughly**
   - Examples in doc comments
   - Architecture explanations

7. **Write comprehensive tests**
   - Unit, integration, property-based
   - Test all feature combinations

8. **Use structured logging**
   - `tracing` with semantic fields
   - Configurable log levels

9. **Design for testability**
   - Dependency injection via traits
   - Mock implementations for tests

10. **Performance matters**
    - Zero-copy when possible
    - Connection pooling
    - Streaming for large data

### ❌ DON'T

1. **Don't `unwrap()` in library code**
   - Use `?` or `expect()` with clear message
   - Return `Result` to caller

2. **Don't block the async runtime**
   - No `std::sync::Mutex` in async code
   - Use `tokio::sync::Mutex` instead

3. **Don't hold locks across `.await` points**
   - Can cause deadlocks
   - Release before awaiting

4. **Don't hardcode configuration**
   - Use environment variables
   - Fail if required config missing

5. **Don't ignore errors**
   - Handle or propagate
   - Log unexpected errors

6. **Don't expose internal types in public API**
   - Use newtype pattern if needed
   - Keep public surface minimal

7. **Don't panic in response to bad input**
   - Return validation errors
   - Reserve panics for bugs

8. **Don't duplicate code**
   - Extract common patterns
   - Use macros for repetitive code

9. **Don't skip security validations**
   - Validate on every entry point
   - Defense in depth

10. **Don't optimize prematurely**
    - Profile first
    - Benchmark to prove improvement

---

## Conclusion

The Semantic Browser codebase demonstrates **mature Rust engineering practices** with:

- ✅ **Solid architecture** - Clear layers, modular design
- ✅ **Type safety** - Compile-time guarantees
- ✅ **Async-first** - High performance I/O
- ✅ **Secure by default** - Multiple security layers
- ✅ **Well-tested** - Comprehensive test suite
- ✅ **Observable** - Structured logging and metrics
- ✅ **Maintainable** - Clear code organization
- ✅ **Extensible** - Easy to add new features

These practices make the codebase:
- **Reliable** - Hard to introduce bugs
- **Performant** - Efficient resource usage
- **Secure** - Defense in depth
- **Maintainable** - Easy to understand and modify

**Recommended for:**
- Production deployments
- As reference for Rust best practices
- Teaching async Rust patterns
- Building AI agent systems

---

## Further Reading

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Async Rust Book](https://rust-lang.github.io/async-book/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)
- [Architecture Decision Records](../internal/IMPLEMENTATION_STATUS.md)
- [Security Best Practices](../reference/security.md)
- [Performance Tuning Guide](./performance-tuning.md)
- [Testing Guide](./testing.md)
- [Contributing Guidelines](./contributing.md)

---

**Document Version:** 1.0  
**Last Updated:** 2025-01-10  
**Maintained By:** Semantic Browser Team
