# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added - LLM Agent Integration (Phase 1 & 2)

#### Core LLM Integration
- **LLM Provider Abstraction** (`src/llm/provider.rs`)
  - Generic `LLMProvider` trait for multi-provider support
  - `LLMRequest` and `LLMResponse` types
  - Error handling with `LLMError`

- **Ollama Provider** (`src/llm/providers/ollama.rs`)
  - Full integration with local Ollama LLMs
  - Support for llama3, mistral, codellama models
  - Streaming and non-streaming responses
  - Configuration via environment variables

- **OpenAI Provider** (stub in `src/llm/providers/openai.rs`)
  - Placeholder for GPT-4/GPT-3.5 integration
  - Ready for implementation

- **Anthropic Provider** (stub in `src/llm/providers/anthropic.rs`)
  - Placeholder for Claude integration
  - Ready for implementation

#### Tool System
- **Tool Registry** (`src/llm/tools/registry.rs`)
  - Dynamic tool registration and discovery
  - Tool metadata and parameter schemas
  - Tool execution with error handling

- **Browser Tools** (`src/llm/tools/browser_tools.rs`)
  - `navigate_to` - Navigate to URLs
  - `click_element` - Click on page elements
  - `extract_data` - Extract structured data
  - `fill_form` - Smart form filling
  - `take_screenshot` - Capture page screenshots

#### Agent Orchestration
- **AgentOrchestrator** (`src/llm/orchestrator.rs`)
  - ReAct pattern implementation (Reason + Act)
  - Iterative reasoning loop with max iterations
  - Tool calling and result integration
  - Conversation history management
  - Timeout and safety controls

#### Browser Integration
- **BrowserExecutor** (`src/llm/executor/browser_executor.rs`)
  - Real browser automation via chromiumoxide
  - Integration with SmartFormFiller
  - Dual mode: real vs mock execution
  - Screenshot capture and data extraction
  - Form filling with AI-powered field detection

#### Examples
- `examples/agent_simple_task.rs` - Basic agent with mock executor
- `examples/agent_with_browser.rs` - Agent with real browser
- `examples/agent_with_ml.rs` - Agent with ML/KG integration
- `examples/test_llm_connection.rs` - LLM provider testing

#### Documentation
- `QUICK_START_LLM.md` - 5-minute quick start guide
- `src/llm/README.md` - Complete LLM integration guide
- `docs/INTEGRATION_ANALYSIS.md` - Architecture analysis
- `docs/LLM_AGENT_ROADMAP.md` - Feature roadmap
- `docs/BROWSER_INTEGRATION_COMPLETE.md` - Browser integration summary
- `docs/PHASE1_SUMMARY.md` - Phase 1 implementation summary
- `docs/user-guide/environment-variables.md` - Complete env var reference

#### Configuration
- **Environment Variables**
  - `LLM_PROVIDER` - Provider selection (ollama/openai/anthropic)
  - `OLLAMA_API_URL` - Ollama endpoint
  - `OLLAMA_MODEL` - Model selection
  - `OPENAI_API_KEY` - OpenAI authentication
  - `OPENAI_MODEL` - GPT model selection
  - `ANTHROPIC_API_KEY` - Anthropic authentication
  - `ANTHROPIC_MODEL` - Claude model selection
  - `AGENT_MAX_ITERATIONS` - Safety limit
  - `AGENT_TIMEOUT_SECS` - Timeout control
  - `AGENT_DEBUG` - Debug logging

- Updated `.env.example` with LLM configuration
- Updated `.env` with development defaults

#### Testing
- Unit tests for tool registry
- Integration test placeholders
- Mock executor for testing without browser

### Changed

#### Module Structure
- Added `llm` module to `src/lib.rs`
- Organized providers in `src/llm/providers/`
- Created executor abstraction in `src/llm/executor/`

#### Documentation
- Updated `README.md` with LLM features
- Enhanced `docs/index.md` with agent documentation
- Reorganized documentation structure

#### Dependencies
- Added `reqwest` for HTTP client
- Added `serde_json` for JSON handling
- Added `tokio` async runtime features

### Fixed
- Compilation errors in browser executor
- Module visibility and exports
- Type mismatches in tool execution
- Async/await handling in orchestrator

## [0.1.0-alpha] - 2024-01-XX (Previous Release)

### Added
- JWT Authentication system
- HTML5 parsing with semantic extraction
- Knowledge Graph with SPARQL support
- Browser automation with chromiumoxide
- MCP Protocol server
- ONNX ML integration
- Prometheus metrics
- Docker containerization
- Security features (rate limiting, validation, sandboxing)
- Redis integration for token revocation
- Comprehensive test suite
- API documentation

### Security
- JWT-based authentication
- Role-based access control (RBAC)
- Input validation and sanitization
- Rate limiting per IP
- Audit logging
- Seccomp sandboxing (Linux)

## Versioning

This project uses [Semantic Versioning](https://semver.org/):

- **MAJOR** version for incompatible API changes
- **MINOR** version for new functionality in a backward compatible manner
- **PATCH** version for backward compatible bug fixes

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md` with release notes
3. Create git tag: `git tag -a v0.1.0 -m "Release v0.1.0"`
4. Push tag: `git push origin v0.1.0`
5. GitHub Actions will build and publish release

## Links

- [Repository](https://github.com/gianlucamazza/semanticbrowser)
- [Issues](https://github.com/gianlucamazza/semanticbrowser/issues)
- [Discussions](https://github.com/gianlucamazza/semanticbrowser/discussions)

---

**Note**: Dates in `[Unreleased]` section will be updated upon release.
