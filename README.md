# Semantic Browser for AI Agents

A Rust-based semantic browser designed for the new generation of AI agents, enabling semantic understanding and interaction with web content.

## 📚 Documentation

- **[Quick Start](docs/guides/quickstart.md)** - Get started in 5 minutes
- **[Docker Setup](docs/guides/docker-setup.md)** - Complete Docker setup guide
- **[Testing](docs/guides/testing.md)** - Comprehensive testing guide
- **[API Reference](docs/api/README.md)** - REST API documentation
- **[Architecture](docs/architecture/README.md)** - System architecture overview
- **[Contributing](docs/development/contributing.md)** - Development guidelines
- **[docs/examples/](docs/examples/)** - API usage examples

## 🤝 Community

- **[Code of Conduct](docs/code-of-conduct.md)** - Community guidelines
- **[Security Policy](docs/security.md)** - Vulnerability reporting

## Features

- **HTML5 Parsing**: Extract semantic elements like microdata and JSON-LD.
- **Knowledge Graph**: Build and query RDF graphs.
- **Agent API**: REST API for agent interactions.
- **External Integrations**: Browser automation with browser-use and LangGraph workflows.
- **Security**: Input validation and logging.

## API

REST API with authentication and rate limiting. See **[API Reference](docs/api/README.md)** for details:

- `POST /parse`: Parse HTML and extract semantic data
- `POST /query`: Query Knowledge Graph with SPARQL
- `POST /browse`: Browse URL and extract semantic information

## Architecture

- `parser`: HTML parsing and semantic extraction.
- `annotator`: Entity recognition and RDF conversion.
- `kg`: Knowledge graph management.
- `api`: Agent API server.
- `security`: Security utilities.

## Quick Start

Get started in 5 minutes with our comprehensive guide:

- **[Docker Setup](docs/guides/quickstart.md)** - Complete Docker-based setup
- **[Local Development](docs/guides/quickstart.md#option-2-local-development)** - Run with Cargo

### Basic Usage

```bash
# Copy config and start server
cp config/.env.example .env
./docker/scripts/docker-up.sh -d

# Try examples
./docs/examples/parse_html.sh
./docs/examples/query_kg.sh
```

## Dependencies

- html5ever: HTML parsing
- oxigraph: RDF handling
- axum: Web server
- scraper: HTML querying
- pyo3: Python integration for external tools
- tract-core: ML inference (with ONNX model support)

## Testing

Basic testing commands. See **[Testing Guide](docs/guides/testing.md)** for comprehensive testing information:

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

Complete Docker setup and deployment guide available at **[Docker Setup](docs/guides/docker-setup.md)**.

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

See the `docs/examples/` directory for usage examples:
- `docs/examples/parse_html.sh` - Parse HTML and extract semantic data
- `docs/examples/query_kg.sh` - Query and update the knowledge graph
- `docs/examples/browse_url.sh` - Browse URLs and extract information

Make scripts executable:
```bash
chmod +x docs/examples/*.sh
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

See **[Contributing Guide](docs/development/contributing.md)** for development guidelines and contribution process.

Quick start: Fork, create branch, make changes, run tests, submit PR.

## License

This project is a demonstration of semantic web technologies for AI agents.