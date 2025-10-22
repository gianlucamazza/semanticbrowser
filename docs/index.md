# Semantic Browser Documentation

Welcome to the comprehensive documentation for the Semantic Browser, a Rust-based semantic web agent designed for AI agents.

## 🚀 Getting Started

New to Semantic Browser? Start here:

- **[Quick Start](./user-guide/quickstart.md)** - Get up and running in 5 minutes
- **[Docker Setup](./user-guide/docker-setup.md)** - Complete Docker environment setup
- **[Core Operations](./user-guide/core-operations.md)** - Essential operations and workflows

## 📚 Documentation Structure

### User Guide
Practical guides for using Semantic Browser:

- **[Quick Start](./user-guide/quickstart.md)** - Installation and basic usage
- **[Docker Setup](./user-guide/docker-setup.md)** - Docker deployment guide
- **[Core Operations](./user-guide/core-operations.md)** - Main workflows and operations
- **[Browser Automation](./user-guide/browser-automation.md)** - Headless browsing features
- **[Examples](./user-guide/examples/)** - Code examples and scripts

### Developer Guide
Technical documentation for contributors:

- **[Architecture](./developer-guide/architecture.md)** - System design and components
- **[Testing](./developer-guide/testing.md)** - Testing strategies and guidelines
- **[Contributing](./developer-guide/contributing.md)** - Development workflow
- **[Performance Tuning](./developer-guide/performance-tuning.md)** - Optimization guide
- **[Production Deployment](./developer-guide/production-deployment.md)** - Production setup
- **[MCP Extension](./developer-guide/mcp-extension-guide.md)** - Model Context Protocol integration

### API Reference
Technical API documentation:

- **[REST API](./api/README.md)** - Complete API reference
- **[Authentication](./reference/authentication.md)** - JWT authentication guide
- **[ML Models](./reference/ml-models.md)** - Machine learning integration
- **[Security](./reference/security.md)** - Security features and policies

### Reference
Additional reference materials:

- **[Changelog](./reference/changelog.md)** - Version history
- **[Security Policy](./reference/security.md)** - Vulnerability reporting
- **[Code of Conduct](./reference/code-of-conduct.md)** - Community guidelines
- **[ML ONNX Guide](./reference/ml-onnx-guide.md)** - ONNX model usage
- **[Seccomp](./reference/seccomp.md)** - Sandboxing configuration

## ✨ Key Features

- **JWT Authentication**: Secure token-based authentication with RBAC and Redis-based token revocation
- **Observability**: Comprehensive Prometheus metrics, distributed tracing, and structured logging
- **Performance Monitoring**: Extensive benchmark suite for HTML parsing, KG operations, and ML inference
- **HTML5 Parsing**: Extract semantic elements like microdata and JSON-LD
- **Knowledge Graph**: Build and query RDF graphs with SPARQL
- **Browser Automation**: Headless browsing with chromiumoxide
- **MCP Protocol**: Full Model Context Protocol server for AI agent integration
- **ML Integration**: ONNX models for NER and knowledge graph embeddings
- **Security**: Input validation, rate limiting, audit logging, seccomp sandboxing

## 🏗️ Architecture Overview

The Semantic Browser consists of several key modules:

- `auth`: JWT authentication and authorization system
- `parser`: HTML parsing and semantic extraction
- `annotator`: ML-based entity recognition and RDF conversion
- `kg`: Knowledge graph management with SPARQL support
- `kg_integration`: Knowledge graph population and inference
- `browser`: Headless browser automation with chromiumoxide
- `external`: External tool integrations and workflow orchestration
- `api`: REST API server with rate limiting
- `security`: Input validation, audit logging, and sandboxing
- `ml`: ONNX model inference for embeddings and NER
- `observability`: Metrics and monitoring (optional)

## 📖 API Overview

REST API with JWT authentication, rate limiting, and observability:

- `POST /auth/token`: Generate JWT authentication tokens
- `POST /auth/revoke`: Revoke JWT tokens using Redis (requires admin role)
- `POST /parse`: Parse HTML and extract semantic data
- `POST /query`: Query/update Knowledge Graph with SPARQL
- `POST /browse`: Browse URL and extract semantic information
- `POST /browse_kg`: Browse URL and automatically insert into Knowledge Graph
- `GET /kg/entities`: List all entities in the Knowledge Graph
- `GET /kg/relations`: List all relations in the Knowledge Graph
- `GET /metrics`: Expose Prometheus metrics for monitoring (optional)

## 🐳 Quick Docker Start

```bash
# Copy config and start server
cp .env.example .env
./docker/scripts/docker-up.sh --build -d

# Test with examples
./docs/user-guide/examples/parse_html.sh
./docs/user-guide/examples/query_kg.sh
```

## 🤝 Contributing

We welcome contributions! See our [Contributing Guide](./developer-guide/contributing.md) for details.

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/gianlucamazza/semanticbrowser/issues)
- **Discussions**: [GitHub Discussions](https://github.com/gianlucamazza/semanticbrowser/discussions)
- **Documentation**: This documentation site
