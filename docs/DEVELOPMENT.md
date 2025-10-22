# Development Guide

This guide covers the architecture, design patterns, and development practices used in the Semantic Browser project.

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Module Structure](#module-structure)
3. [Key Design Patterns](#key-design-patterns)
4. [Building & Running](#building--running)
5. [Testing Strategy](#testing-strategy)
6. [Common Tasks](#common-tasks)
7. [Troubleshooting](#troubleshooting)

---

## Architecture Overview

### High-Level Design

The Semantic Browser is built around a multi-layered architecture:

```
┌─────────────────────────────────────┐
│   Application Layer                 │
│   (Examples, CLI, API endpoints)    │
├─────────────────────────────────────┤
│   Agent Orchestration Layer         │
│   (LLM Agents, Workflow Engine)     │
├─────────────────────────────────────┤
│   Tool & Browser Layer              │
│   (Browser Automation, Tools)       │
├─────────────────────────────────────┤
│   Knowledge & ML Layer              │
│   (Knowledge Graph, Embeddings)     │
├─────────────────────────────────────┤
│   Provider & API Layer              │
│   (LLM Providers, HTTP Clients)     │
├─────────────────────────────────────┤
│   Core Utilities                    │
│   (Error Handling, Config, Auth)    │
└─────────────────────────────────────┘
```

### Core Principles

1. **Multi-Provider Architecture**: Support multiple LLM providers (OpenAI, Anthropic, Ollama)
2. **Trait-Based Extensibility**: Use Rust traits for extensible components
3. **Async-First Design**: Built on tokio for high-performance async operations
4. **Type Safety**: Leverage Rust's type system for error handling
5. **Separation of Concerns**: Clear module boundaries and responsibilities

---

## Module Structure

### Root Modules (src/)

```
src/
├── lib.rs                  # Public API exports
├── annotator.rs            # HTML annotation engine
├── api.rs                  # HTTP API endpoints
├── api_client.rs           # External API client
├── auth.rs                 # Authentication/Authorization
├── browser.rs              # Browser pool management
├── external.rs             # External service integrations
├── form_analyzer.rs        # Form field analysis
├── form_interaction.rs     # Form interaction logic
├── kg.rs                   # Knowledge Graph
├── kg_integration.rs       # KG integration utilities
├── llm/                    # LLM provider abstraction
│   ├── provider.rs         # Trait definition
│   ├── openai.rs           # OpenAI implementation
│   ├── anthropic.rs        # Anthropic implementation
│   ├── ollama.rs           # Ollama implementation
│   ├── agent.rs            # Agent orchestration
│   ├── tools.rs            # Tool definitions
│   ├── workflow.rs         # Workflow engine
│   └── browser_executor.rs # Browser task execution
├── ml/                     # Machine Learning
│   ├── embeddings.rs       # Embedding generation
│   ├── inference.rs        # Model inference
│   └── link_prediction.rs  # Link prediction
├── models.rs               # Data models
├── observability.rs        # Metrics & tracing
├── parser.rs               # HTML/DOM parsing
├── security.rs             # Security utilities
└── smart_form_filler.rs    # Intelligent form filling
```

### Module Responsibilities

#### llm/ - LLM Integration

Provides unified interface to different LLM providers:
- **provider.rs**: Core LLMProvider trait
- **openai.rs**: OpenAI API implementation
- **anthropic.rs**: Anthropic Claude implementation
- **ollama.rs**: Local Ollama instance support
- **agent.rs**: ReAct agent orchestration
- **tools.rs**: Tool registry and definitions
- **workflow.rs**: Multi-step workflow execution
- **browser_executor.rs**: Browser-based task execution

#### browser/ - Browser Automation

Manages headless browser instances:
- Connection pooling
- Page lifecycle management
- Navigation tracking
- Resource handling

#### kg/ - Knowledge Graph

RDF triple store for semantic understanding:
- SPARQL query support
- Triple insertion/retrieval
- Entity relationship management

#### ml/ - Machine Learning

ML model integration:
- Embedding generation
- Named entity recognition
- Link prediction for knowledge graphs

---

## Key Design Patterns

### 1. Provider Pattern (LLM)

```rust
#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse>;

    async fn chat_completion_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<serde_json::Value>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse>;

    fn provider_name(&self) -> &str;
    async fn health_check(&self) -> LLMResult<bool>;
}
```

Implementations: OpenAIProvider, AnthropicProvider, OllamaProvider

### 2. Tool Registry Pattern

```rust
pub struct ToolRegistry {
    tools: HashMap<String, Tool>,
}

impl ToolRegistry {
    pub fn register_tool(&mut self, name: String, tool: Tool) { }
    pub fn get_tool(&self, name: &str) -> Option<&Tool> { }
    pub fn list_tools(&self) -> Vec<&str> { }
}
```

### 3. Error Handling Pattern

```rust
#[derive(Error, Debug)]
pub enum LLMError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("API error: {0}")]
    Api(String),

    // ... more variants
}

pub type LLMResult<T> = Result<T, LLMError>;
```

### 4. Configuration Pattern

Use config structs for flexibility:

```rust
pub struct OllamaConfig {
    pub base_url: String,
    pub timeout: Duration,
}

impl Default for OllamaConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:11434".to_string(),
            timeout: Duration::from_secs(120),
        }
    }
}
```

### 5. Builder Pattern

For complex object construction:

```rust
let config = BrowserConfig::builder()
    .headless(true)
    .timeout(Duration::from_secs(30))
    .build()?;
```

---

## Building & Running

### Build Commands

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# With specific features
cargo build --features llm-openai,browser-automation

# All features
cargo build --all-features

# Clean build
cargo clean && cargo build
```

### Running Examples

```bash
# Simple agent example
cargo run --example agent_simple_task

# With OpenAI
OPENAI_API_KEY=sk-... cargo run --features llm-openai --example agent_openai_example

# Browser automation
cargo run --features browser-automation --example agent_browser_example

# Workflow engine
cargo run --features browser-automation --example workflow_example
```

### Development Server

```bash
# Run with default settings
cargo run

# With log output
RUST_LOG=debug cargo run

# With specific log levels
RUST_LOG=semantic_browser=debug,tokio=info cargo run
```

---

## Testing Strategy

### Test Types

1. **Unit Tests** (60%)
   - Fast, isolated tests
   - Located in each module's `tests.rs`
   - Test single functions/components

2. **Integration Tests** (30%)
   - Test component interactions
   - Located in `tests/` directory
   - Test complete workflows

3. **E2E Tests** (10%)
   - Real-world scenarios
   - Test full system
   - May require external services

### Running Tests

```bash
# All tests
cargo test --all-features

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration_tests

# Specific module
cargo test --lib module_name

# With output
cargo test -- --nocapture

# Single test
cargo test test_name -- --exact

# Ignored tests
cargo test -- --ignored
```

### Test Coverage

```bash
# Check coverage with tarpaulin
cargo tarpaulin --all-features --timeout 300

# Generate HTML report
cargo tarpaulin --all-features --out Html
```

### Test Naming Convention

```rust
#[test]
fn test_function_happy_path() { }           // Success case

#[test]
fn test_function_error_handling() { }       // Error case

#[test]
fn test_function_edge_case_empty() { }      // Edge case

#[test]
#[should_panic]
fn test_function_panics_on_invalid() { }   // Panic case
```

---

## Common Tasks

### Adding a New LLM Provider

1. Create new file `src/llm/newprovider.rs`
2. Implement `LLMProvider` trait
3. Add to `src/llm/mod.rs` exports
4. Add feature flag in `Cargo.toml`
5. Add tests in `tests/llm_newprovider.rs`
6. Document in README.md

### Adding a New Tool

1. Create tool definition in `src/llm/tools.rs`
2. Register in `ToolRegistry`
3. Implement tool handler
4. Add tests
5. Update documentation

### Adding a New Module

1. Create `src/newmodule.rs` or `src/newmodule/mod.rs`
2. Export in `src/lib.rs`
3. Add public documentation
4. Create unit tests
5. Update module documentation

### Updating Dependencies

```bash
# Check for updates
cargo outdated

# Update specific dependency
cargo update -p dependency_name

# Update all
cargo update

# Run security audit
cargo audit
```

---

## Troubleshooting

### Common Issues

#### "Failed to compile pyo3"
```bash
# Solution: Update Rust
rustup update

# Or specify Python version
export PYTHON=python3.9
```

#### "Timeout waiting for page"
```bash
# Solution: Increase timeout in configuration
let config = BrowserConfig::builder()
    .timeout(Duration::from_secs(60))
    .build()?;
```

#### "chromiumoxide launch failed"
```bash
# Solution: Install Chromium
# macOS:
brew install chromium

# Ubuntu:
sudo apt-get install chromium-browser

# Fedora:
sudo dnf install chromium
```

#### "OPENAI_API_KEY not set"
```bash
# Solution: Set environment variable
export OPENAI_API_KEY=sk-your-key-here

# Or create .env file
echo "OPENAI_API_KEY=sk-..." > .env
```

### Debugging

#### Enable detailed logging
```bash
RUST_LOG=debug cargo run
```

#### Use debugger
```bash
# With lldb (macOS)
rust-lldb cargo test

# With gdb (Linux)
rust-gdb cargo test
```

#### Print debug info
```rust
println!("{:?}", variable);  // Debug format
dbg!(&variable);             // Macro helper
eprintln!("{:?}", error);    // stderr
```

---

## Performance Tips

### Compilation Speed

```bash
# Use incremental compilation
export CARGO_BUILD_INCREMENTAL=1

# Use minimal features while developing
cargo build --no-default-features

# Use mold linker (Linux)
RUSTFLAGS="-C link-arg=-fuse-ld=mold" cargo build
```

### Runtime Performance

- Use `--release` builds for benchmarks
- Profile with `cargo flamegraph`
- Check allocations with `heaptrack`
- Monitor async runtime with `tokio-console`

---

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for:
- Development setup
- Code style requirements
- Commit message format
- Pull request process

---

**Last Updated**: 2025-10-22
**Maintainer**: Technical Team
**Status**: Active Development
