# Semantic Browser for AI Agents

A Rust-based semantic browser designed for the new generation of AI agents, enabling semantic understanding and interaction with web content.

## 📚 Documentation

- **[QUICKSTART.md](QUICKSTART.md)** - Get started in 5 minutes
- **[DOCKER.md](DOCKER.md)** - Complete Docker setup guide
- **[TESTING.md](TESTING.md)** - Comprehensive testing guide
- **[examples/](examples/)** - API usage examples

## Features

- **HTML5 Parsing**: Extract semantic elements like microdata and JSON-LD.
- **Knowledge Graph**: Build and query RDF graphs.
- **Agent API**: REST API for agent interactions.
- **External Integrations**: Browser automation with browser-use and LangGraph workflows.
- **Security**: Input validation and logging.

## Architecture

- `parser`: HTML parsing and semantic extraction.
- `annotator`: Entity recognition and RDF conversion.
- `kg`: Knowledge graph management.
- `api`: Agent API server.
- `security`: Security utilities.

## Quick Start

### Using Docker (Recommended)

1. **Copy environment configuration:**
   ```bash
   cp .env.example .env
   # Edit .env as needed
   ```

2. **Start the server:**
   ```bash
   ./scripts/docker-up.sh -d
   ```

3. **Try the examples:**
   ```bash
   ./examples/parse_html.sh
   ./examples/query_kg.sh
   ```

4. **View logs:**
   ```bash
   ./scripts/docker-up.sh --logs
   ```

5. **Stop the server:**
   ```bash
   ./scripts/docker-up.sh --stop
   ```

### Using Cargo (Development)

Run the server directly:

```bash
cargo run
```

### Environment Variables

- `RUST_LOG`: Set logging level (e.g., `debug`, `info`, `warn`, `error`). Default: `info`
- `KG_PERSIST_PATH`: Path to persist the Knowledge Graph. If not set, uses in-memory storage.
- `NER_MODEL_PATH`: Path to NER ONNX model (optional, uses regex fallback if not set)
- `KG_INFERENCE_MODEL_PATH`: Path to KG inference model (optional)

Example:
```bash
RUST_LOG=debug KG_PERSIST_PATH=./kg_data cargo run
```

### API Endpoints

All endpoints require `Authorization: Bearer secret` header and are rate limited to 10 requests/min per IP:

- `POST /parse`: Parse HTML and extract semantic data
- `POST /query`: Query Knowledge Graph with SPARQL
- `POST /browse`: Browse URL and extract semantic information using external tools

## Dependencies

- html5ever: HTML parsing
- oxigraph: RDF handling
- axum: Web server
- scraper: HTML querying
- pyo3: Python integration for external tools
- tract-core: ML inference (placeholder)

## Testing

Run all tests:
```bash
cargo test
```

Run only integration tests:
```bash
cargo test --test integration_test
```

Run benchmarks:
```bash
cargo bench
```

## Features

### Core Features
- ✅ HTML5 parsing with semantic extraction (microdata, JSON-LD)
- ✅ Named Entity Recognition (NER) with ML support via tract-core
- ✅ Knowledge Graph with SPARQL support (SELECT, INSERT, DELETE, CONSTRUCT, ASK, DESCRIBE)
- ✅ RDF triple storage with optional persistence
- ✅ ML-based inference for knowledge graphs
- ✅ REST API with authentication and rate limiting
- ✅ Real IP extraction (supports X-Forwarded-For, X-Real-IP headers)

### Security
- ✅ Input validation for HTML and SPARQL
- ✅ Rate limiting (10 requests/min per IP)
- ✅ Bearer token authentication
- ✅ Sandboxing framework (seccomp on Linux with feature flag)

### External Integrations
- ✅ PyO3 support for Python integration (optional feature)
- ✅ Browser automation placeholder (browser-use)
- ✅ LangGraph workflow support (placeholder)

### Operations
- ✅ Structured logging with tracing
- ✅ Environment-based configuration
- ✅ Docker support
- ✅ CI/CD with GitHub Actions

## Feature Flags

Build with PyO3 integration:
```bash
cargo build --features pyo3-integration
```

Build with seccomp sandboxing (Linux only):
```bash
cargo build --features seccomp
```

Build with all features:
```bash
cargo build --all-features
```

## Docker

### Build Docker Image

```bash
# Production build
./scripts/docker-build.sh

# Test build
./scripts/docker-build.sh --test

# Build without cache
./scripts/docker-build.sh --no-cache
```

### Run with Docker Compose

```bash
# Start in background
./scripts/docker-up.sh -d

# Start in foreground
./scripts/docker-up.sh

# Build and start
./scripts/docker-up.sh --build -d

# View logs
docker-compose logs -f

# Stop
docker-compose down
```

### Docker Testing

Run the complete test suite in Docker:

```bash
# Run all tests
./scripts/docker-test.sh

# Run only unit tests
./scripts/docker-test.sh --unit-only

# Run only integration tests
./scripts/docker-test.sh --integration-only

# Run with benchmarks
./scripts/docker-test.sh --with-bench

# Clean up test containers
./scripts/docker-test.sh --clean
```

### Docker Compose Services

**docker-compose.yml** (Production/Development):
- `semantic_browser`: Main API server
- Persistent volume for Knowledge Graph
- Health checks enabled

**docker-compose.test.yml** (Testing):
- `test_runner`: Unit tests
- `lint_runner`: Code quality checks (fmt, clippy)
- `integration_test`: Integration tests with live server
- `test_server`: Test server for integration tests
- `benchmark`: Performance benchmarks

## Examples

See the `examples/` directory for usage examples:
- `examples/parse_html.sh` - Parse HTML and extract semantic data
- `examples/query_kg.sh` - Query and update the knowledge graph
- `examples/browse_url.sh` - Browse URLs and extract information

Make scripts executable:
```bash
chmod +x examples/*.sh
```

## Troubleshooting

### Docker Build Issues

If you encounter build errors, try:

```bash
# Verify Dockerfile syntax
./scripts/verify-dockerfile-syntax.sh

# Clean build
./scripts/docker-build.sh --no-cache

# Check Docker status
docker info
```

Common issues:
- **BuildKit warnings**: Ensure all Dockerfile keywords are UPPERCASE
- **Credentials errors**: Restart Docker Desktop or re-login
- **Network errors**: Check internet connection and Docker proxy settings

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests: `cargo test`
5. Run formatter: `cargo fmt`
6. Run linter: `cargo clippy`
7. Verify Docker syntax: `./scripts/verify-dockerfile-syntax.sh`
8. Submit a pull request

## License

This project is a demonstration of semantic web technologies for AI agents.