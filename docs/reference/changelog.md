# Changelog

All notable changes to the Semantic Browser project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3] - 2025-01-23

### Fixed - Documentation Validation & Corrections

#### Documentation Corrections
- **Authentication Examples**: Updated all example scripts to use dynamic JWT token generation
  - `docs/user-guide/examples/parse_html.sh` now generates tokens correctly
  - `docs/user-guide/examples/query_kg.sh` now includes proper auth flow
  - Added error handling and prerequisite checks
- **Environment Variables**: Added missing ML configuration variables to `.env.example`
  - NER tokenizer and labels configuration
  - KG embedding model configuration (entity/relation nodes)
  - OpenTelemetry service name
  - All ML inference parameters documented
- **Browser Automation Limitations**: Clarified chromiumoxide 0.7 limitations
  - `BLOCK_ADS` only uses `--disable-background-networking` flag
  - `BLOCK_IMAGES` not yet implemented in chromiumoxide 0.7
  - Added workarounds and production recommendations
- **API Documentation**: Enhanced JWT authentication section
  - Added security prerequisites and warnings
  - Included complete token generation workflow
  - Documented token expiration and rotation
- **Streaming Guide**: Added vision model limitations and examples
  - Clarified that vision models do NOT support streaming
  - Added OpenAI and Anthropic vision examples
  - Explained technical reasons for no vision streaming

#### New Documentation
- **ML Configuration Guide** (`docs/reference/ml-configuration.md`)
  - Complete NER configuration guide
  - Knowledge Graph ML inference setup
  - Model format requirements and examples
  - Troubleshooting section
  - Best practices for production use

### Changed
- Updated `STREAMING_GUIDE.md` to reflect Anthropic streaming as completed
- Improved error messages in example scripts
- Enhanced inline documentation for configuration options

---

## [0.1.2] - 2025-01-22

### Added - Vision & Multi-Tab Features

#### LLM & Vision Integration
- **Multi-Provider LLM Support** (`src/llm/`)
  - OpenAI GPT models with vision capabilities (GPT-4V, GPT-4o)
  - Anthropic Claude models with vision support (Claude 3)
  - Ollama local model integration
  - Unified provider interface with async traits
  - Content blocks supporting text and images
  - Streaming responses for real-time interaction
  - Tool/function calling for agent workflows
- **Vision Model Integration** (`src/llm/provider.rs`)
  - `ContentBlock` enum for text and image content
  - `ImageSource` enum supporting URLs and base64 data
  - `MessageContent` for backward-compatible message handling
  - Vision-enabled chat completion methods
  - Base64 image encoding support
- **Vision Example** (`examples/vision_example.rs`)
  - Demonstrates image analysis with OpenAI and Anthropic
  - Base64 image encoding and API usage
  - Error handling for vision operations

#### Browser Automation Enhancements
- **Multi-Tab Orchestration** (`src/browser.rs`)
  - `TabManager` struct for tab lifecycle management
  - `BrowserPool` extensions with multi-tab API
  - Concurrent tab operations with resource limits
  - Tab creation, switching, and cleanup methods
  - Resource pooling for efficient browser usage
- **Multi-Tab Example** (`examples/multi_tab_example.rs`)
  - Demonstrates concurrent tab workflows
  - Tab lifecycle management and cleanup
  - Resource limit testing and error handling

#### Testing & Quality
- **Comprehensive Test Suite**
  - Unit tests for vision message creation and formatting
  - Multi-tab functionality tests with resource limits
  - Integration tests for LLM providers
  - Browser automation test coverage
- **Documentation Updates**
  - Updated architecture diagram with LLM and browser components
  - Enhanced API documentation for new features
  - Comprehensive changelog with feature details

#### Dependencies
- Added `base64 0.22` for image encoding in vision models

## [0.1.0] - 2025-01-15

### Added - 2025 Best Practices Implementation

#### Authentication & Security
- **JWT Authentication System** (`src/auth.rs`)
  - Configurable JWT-based authentication replacing hardcoded tokens
  - `AuthenticatedUser` extractor for Axum routes
  - Token generation with customizable expiration
  - Role-based access control (RBAC) support
  - Environment-based secret configuration via `JWT_SECRET`
  - `/auth/token` endpoint for token generation
  - **Token Revocation with Redis** (`src/auth.rs`)
    - Redis-backed token invalidation for immediate revocation
    - `TokenRevocationStore` with async Redis operations
    - `/auth/revoke` endpoint for admin token revocation
    - Automatic cleanup of expired revoked tokens
    - Optional `redis-integration` feature flag
  - Comprehensive test coverage for auth module

#### Machine Learning & NER
- **ONNX Model Integration** (`src/annotator.rs`)
  - tract-onnx support for production NER models
  - Optimized model loading with `.into_optimized()`
  - Support for BERT, DistilBERT, and other NER architectures
  - Automatic fallback to regex-based extraction
  - Optional `onnx-integration` feature flag
  - Detailed documentation for production tokenizer integration

#### Performance & Benchmarking
- **Comprehensive Benchmark Suite** (`benches/parsing_benchmark.rs`)
  - HTML parsing benchmarks (small/large documents)
  - Knowledge Graph operations (insert/query/inference)
  - JWT token generation/validation benchmarks
  - LangGraph workflow performance testing
  - Browser automation operation benchmarks
  - ML inference performance measurement
  - Criterion.rs integration for statistical analysis

#### Security Hardening
- **Seccomp Sandboxing** (`src/security.rs`)
  - Linux seccomp-bpf filtering for syscall restriction
  - Whitelist approach for minimal attack surface
  - Blocks dangerous syscalls (exec, socket, ptrace, etc.)
  - Graceful fallback on error
  - Optional `seccomp` feature flag (Linux only)
  - Applied to HTML parsing operations

#### Observability & Monitoring
- **Prometheus Metrics System** (`src/observability/metrics.rs`)
  - RED (Rate/Errors/Duration) metrics for all operations
  - HTTP request metrics with endpoint and method breakdown
  - Knowledge Graph operation metrics (insert/query/inference)
  - Browser automation success/failure tracking
  - ML inference performance and accuracy metrics
  - `/metrics` endpoint for Prometheus scraping
  - Optional `observability` feature flag
  - Integration with existing API middleware
- **Distributed Tracing Support**
  - OpenTelemetry integration for request tracing
  - Configurable tracing backends (Jaeger, Zipkin, Honeycomb)
  - Automatic instrumentation of key operations
- **Structured Logging**
  - Configurable log levels per module
  - JSON logging support for production deployments

#### LLM & Vision Integration
- **Multi-Provider LLM Support** (`src/llm/`)
  - OpenAI GPT models with vision capabilities (GPT-4V, GPT-4o)
  - Anthropic Claude models with vision support (Claude 3)
  - Ollama local model integration
  - Unified provider interface with async traits
  - Content blocks supporting text and images
  - Streaming responses for real-time interaction
  - Tool/function calling for agent workflows
- **Vision Model Integration** (`src/llm/provider.rs`)
  - `ContentBlock` enum for text and image content
  - `ImageSource` enum supporting URLs and base64 data
  - `MessageContent` for backward-compatible message handling
  - Vision-enabled chat completion methods
  - Base64 image encoding support
- **Vision Example** (`examples/vision_example.rs`)
  - Demonstrates image analysis with OpenAI and Anthropic
  - Base64 image encoding and API usage
  - Error handling for vision operations

#### Browser Automation Enhancements
- **Multi-Tab Orchestration** (`src/browser.rs`)
  - `TabManager` struct for tab lifecycle management
  - `BrowserPool` extensions with multi-tab API
  - Concurrent tab operations with resource limits
  - Tab creation, switching, and cleanup methods
  - Resource pooling for efficient browser usage
- **Multi-Tab Example** (`examples/multi_tab_example.rs`)
  - Demonstrates concurrent tab workflows
  - Tab lifecycle management and cleanup
  - Resource limit testing and error handling

#### Dependencies
- Added `jsonwebtoken 9.3` for JWT support
- Added `chrono 0.4` for timestamp management
- Added `redis 0.25` (optional) for token revocation
- Added `prometheus 0.13` (optional) for metrics collection
- Added `lazy_static 1.4` for metrics registry
- Added `tract-onnx 0.21` (optional) for ONNX inference
- Added `seccompiler 0.5` (optional, Linux only) for sandboxing
- Added `tower-http 0.6` for middleware utilities
- Added `libc 0.2` for syscall constants
- Added `async-trait 0.1` for async trait support
- Added `criterion 0.5` for performance benchmarking
- Added `base64 0.22` for image encoding in vision models

### Changed

#### API Improvements
- Updated all API endpoints to use JWT authentication
- Removed hardcoded "Bearer secret" authentication
- Enhanced security logging for auth events
- Better error messages for authentication failures

#### Configuration
- JWT secret now configured via `JWT_SECRET` environment variable
- NER model path via `NER_MODEL_PATH` environment variable
- Added warning for development secrets (< 32 characters)

#### Code Quality
- Updated PyO3 to 0.27 (latest 2025 version)
- Fixed `py.eval()` usage for PyO3 0.27 compatibility
- Improved error handling across modules
- Enhanced tracing and logging throughout

### Fixed
- Removed unused imports in `external.rs`
- Fixed StatusCode import in `api.rs`
- Corrected tensor conversion for tract-onnx
- Platform-specific dependency configuration for seccompiler

### Security
- **BREAKING**: Authentication now requires valid JWT tokens
- Minimum JWT secret length enforced (32 characters)
- Syscall filtering reduces attack surface on Linux
- Input validation enhanced with security logging

### Documentation
- Added comprehensive inline documentation
- Best practices 2025 annotations throughout codebase
- Detailed implementation notes for production use
- Security considerations documented

## Migration Guide

### Upgrading from Previous Versions

#### Authentication Changes
**Before:**
```bash
curl -H "Authorization: Bearer secret" http://localhost:3000/parse
```

**After:**
```bash
# 1. Generate a token
curl -X POST http://localhost:3000/auth/token \
  -H "Content-Type: application/json" \
  -d '{"username": "your-user", "role": "admin"}'

# 2. Use the token
curl -H "Authorization: Bearer <your-jwt-token>" http://localhost:3000/parse
```

#### Environment Variables
Add to your `.env` file:
```bash
# Required - minimum 32 characters
JWT_SECRET=your-super-secret-jwt-key-here-make-it-long-and-random

# Optional
NER_MODEL_PATH=/path/to/your/ner-model.onnx
KG_PERSIST_PATH=/path/to/knowledge-graph-storage
```

#### Feature Flags
```bash
# Enable ONNX support
cargo build --features onnx-integration

# Enable seccomp (Linux only)
cargo build --features seccomp

# All features
cargo build --all-features
```

## [0.1.0] - Initial Release

### Added
- HTML5 parsing with semantic extraction
- Knowledge graph with SPARQL support
- REST API with basic authentication
- Named Entity Recognition (regex-based)
- External integrations framework
- Docker support
- MCP server implementation

---

**Note**: For detailed API documentation, see [docs/api/README.md](docs/api/README.md)
