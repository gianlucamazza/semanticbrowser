# Semantic Browser for AI Agents

[![CI](https://img.shields.io/github/actions/workflow/status/gianlucamazza/semanticbrowser/ci.yml?branch=main&label=CI&logo=github)](https://github.com/gianlucamazza/semanticbrowser/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-blue.svg?logo=rust)](https://www.rust-lang.org)
[![Security Audit](https://img.shields.io/github/actions/workflow/status/gianlucamazza/semanticbrowser/security.yml?label=security&logo=github)](https://github.com/gianlucamazza/semanticbrowser/actions/workflows/security.yml)
[![Docker](https://img.shields.io/badge/docker-ready-2496ED?logo=docker&logoColor=white)](https://github.com/gianlucamazza/semanticbrowser)

[![Version](https://img.shields.io/badge/version-0.1.3-blue.svg)](https://github.com/gianlucamazza/semanticbrowser)
[![AI Agents](https://img.shields.io/badge/AI-Agents-FF6F00?logo=robot&logoColor=white)](https://github.com/gianlucamazza/semanticbrowser)
[![Semantic Web](https://img.shields.io/badge/Semantic-Web-blue?logo=w3c)](https://github.com/gianlucamazza/semanticbrowser)
[![RDF/SPARQL](https://img.shields.io/badge/RDF-SPARQL-4285F4)](https://github.com/gianlucamazza/semanticbrowser)
[![Documentation](https://img.shields.io/badge/docs-latest-blue?logo=readthedocs&logoColor=white)](./docs/)

A Rust-based semantic browser designed for the new generation of AI agents, enabling semantic understanding and interaction with web content.

## ✨ Features

- **JWT Authentication**: Secure token-based authentication with RBAC and Redis-based token revocation
- **Observability**: Comprehensive Prometheus metrics, distributed tracing, and structured logging
- **Performance Monitoring**: Extensive benchmark suite for HTML parsing, KG operations, and ML inference
- **HTML5 Parsing**: Extract semantic elements like microdata and JSON-LD
- **Knowledge Graph**: Build and query RDF graphs with SPARQL
- **Browser Automation**: Headless browsing with chromiumoxide
- **MCP Protocol**: Full Model Context Protocol server for AI agent integration
- **ML Integration**: ONNX models for NER and knowledge graph embeddings
- **Security**: Input validation, rate limiting, audit logging, seccomp sandboxing

## 🚀 Quick Start

```bash
# Clone repository
git clone https://github.com/gianlucamazza/semanticbrowser.git
cd semanticbrowser

# Start with Docker (recommended)
cp .env.example .env
./docker/scripts/docker-up.sh --build -d

# Test the API
./docs/user-guide/examples/parse_html.sh
```

## 📚 Documentation

- **[User Guide](./docs/user-guide/)** - Getting started, setup, and usage
- **[Developer Guide](./docs/developer-guide/)** - Architecture, testing, contributing
- **[API Reference](./docs/api/)** - REST API documentation
- **[Reference](./docs/reference/)** - Technical references and policies
- **[Examples](./docs/user-guide/examples/)** - Code examples and workflows

## 🤝 Community

- **[Contributing](./docs/developer-guide/contributing.md)** - Development guidelines
- **[Code of Conduct](./docs/reference/code-of-conduct.md)** - Community standards
- **[Security](./docs/reference/security.md)** - Vulnerability reporting

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Testing

Basic testing commands. See **[Testing Guide](developer-guide/testing.md)** for comprehensive testing information:

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test integration_test

# Run benchmarks
cargo bench
```

## Features

See **[Architecture Overview](docs/architecture/README.md)** for detailed feature list and system design.

### Core Capabilities
- HTML5 parsing with semantic extraction
- Knowledge Graph with SPARQL support
- REST API with authentication
- External integrations and browser automation

### Feature Flags

```bash
# PyO3 integration
cargo build --features pyo3-integration

# Seccomp sandboxing (Linux)
cargo build --features seccomp

# All features
cargo build --all-features
```

## Docker

Complete Docker setup and deployment guide available at **[Docker Setup](user-guide/docker-setup.md)**.

### Quick Docker Commands

```bash
# Build and start
./docker/scripts/docker-up.sh --build -d

# Run tests
./docker/scripts/docker-test.sh

# View logs
docker-compose logs -f

# Stop
docker-compose down
```

## Examples

See the `docs/user-guide/examples/` directory for usage examples:
- `docs/user-guide/examples/parse_html.sh` - Parse HTML and extract semantic data
- `docs/user-guide/examples/query_kg.sh` - Query and update the knowledge graph
- `docs/user-guide/examples/browse_url.sh` - Browse URLs and extract information

Make scripts executable:
```bash
chmod +x docs/user-guide/examples/*.sh
```

## Troubleshooting

### Docker Build Issues

If you encounter build errors, try:

```bash
# Verify Dockerfile syntax
./docker/scripts/verify-dockerfile-syntax.sh

# Clean build
    ./docker/scripts/docker-build.sh --no-cache

# Check Docker status
docker info
```

Common issues:
- **BuildKit warnings**: Ensure all Dockerfile keywords are UPPERCASE
- **Credentials errors**: Restart Docker Desktop or re-login
- **Network errors**: Check internet connection and Docker proxy settings

## Contributing

See **[Contributing Guide](developer-guide/contributing.md)** for development guidelines and contribution process.

Quick start: Fork, create branch, make changes, run tests, submit PR.

## License

This project is a demonstration of semantic web technologies for AI agents.
