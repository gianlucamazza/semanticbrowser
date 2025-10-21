# Quick Start Guide - Semantic Browser

Get up and running with the Semantic Browser in 5 minutes.

## Prerequisites

Choose one of:
- **Docker** (Recommended): Docker and Docker Compose
- **Local**: Rust 1.75+, Python 3.x (optional)

## Option 1: Docker (Recommended) 🐳

### Step 1: Setup Environment

```bash
# Clone repository (if not already done)
cd semanticbrowser

# Copy environment template
cp config/.env.example .env

# (Optional) Edit .env for custom configuration
# nano .env
```

### Step 2: Start Server

```bash
# Build and start in one command
./docker/scripts/docker-up.sh --build -d

# Or separately:
./docker/scripts/docker-build.sh
./docker/scripts/docker-up.sh -d
```

Wait for the health check (about 5 seconds):
```
✓ Service is healthy and running
```

### Step 3: Test the API

```bash
# Parse HTML
./examples/parse_html.sh

# Query Knowledge Graph
./examples/query_kg.sh

# Browse URL
./examples/browse_url.sh
```

### Step 4: View Logs (Optional)

```bash
# Follow logs
./docker/scripts/docker-up.sh --logs

# Or with docker-compose
docker-compose logs -f
```

### Step 5: Stop Server

```bash
./docker/scripts/docker-up.sh --stop
```

That's it! 🎉

---

## Option 2: Local Development 💻

### Step 1: Build

```bash
# Build the project
cargo build --release
```

### Step 2: Run

```bash
# Start the server
cargo run
```

You should see:
```
INFO  Starting Semantic Browser Agent
INFO  Initializing in-memory Knowledge Graph
INFO  Server running on http://127.0.0.1:3000
```

### Step 3: Test

Open a new terminal:

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

---

## Testing 🧪

### Quick Test

```bash
# Using Docker (complete test suite)
./docker/scripts/docker-test.sh

# Or locally
cargo test
```

### Detailed Testing

```bash
# Unit tests only
./docker/scripts/docker-test.sh --unit-only

# Integration tests only
./docker/scripts/docker-test.sh --integration-only

# With benchmarks
./docker/scripts/docker-test.sh --with-bench
```

---

## API Endpoints

All endpoints require: `Authorization: Bearer secret`

### 1. Parse HTML

Extract semantic data from HTML:

```bash
curl -X POST http://localhost:3000/parse \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer secret" \
  -d '{
    "html": "<html><head><title>Example</title><script type=\"application/ld+json\">{\"@type\": \"Person\", \"name\": \"John\"}</script></head><body></body></html>"
  }'
```

Response:
```json
{
  "title": "Example",
  "entities": ["http://schema.org/Person"]
}
```

### 2. Query Knowledge Graph

Execute SPARQL queries:

```bash
curl -X POST http://localhost:3000/query \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer secret" \
  -d '{
    "query": "SELECT * WHERE { ?s ?p ?o } LIMIT 10"
  }'
```

### 3. Browse URL

Extract semantic information from URLs:

```bash
curl -X POST http://localhost:3000/browse \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer secret" \
  -d '{
    "url": "https://example.com",
    "query": "extract main content"
  }'
```

---

## Configuration ⚙️

See **[Docker Setup Environment Variables](../guides/docker-setup.md#environment-variables)** for complete configuration options.

### Basic Configuration

```bash
# Copy template
cp config/.env.example .env

# Edit as needed
nano .env
```

### With Docker

```bash
# Restart to apply changes
./docker/scripts/docker-up.sh --restart
```

### With Cargo

```bash
# Set environment variables
RUST_LOG=debug KG_PERSIST_PATH=./data/kg cargo run
```

---

## Useful Commands 🛠️

### Docker Commands

```bash
# Status
./docker/scripts/docker-up.sh --status
docker-compose ps

# Logs
./docker/scripts/docker-up.sh --logs
docker-compose logs -f semantic_browser

# Restart
./docker/scripts/docker-up.sh --restart
docker-compose restart

# Stop
./docker/scripts/docker-up.sh --stop
docker-compose down

# Rebuild
./docker/scripts/docker-build.sh
docker-compose build
```

### Cargo Commands

```bash
# Build
cargo build
cargo build --release

# Run
cargo run

# Test
cargo test
cargo test --test integration_test

# Benchmark
cargo bench

# Format
cargo fmt

# Lint
cargo clippy
```

---

## Common Issues 🔧

### Port Already in Use

**Problem**: `Address already in use (os error 48)`

**Solution**:
```bash
# Find and kill process on port 3000
lsof -ti:3000 | xargs kill -9

# Or change port in .env
# SERVER_PORT=3001
```

### Permission Denied

**Problem**: `./docker/scripts/docker-up.sh: Permission denied`

**Solution**:
```bash
chmod +x docker/scripts/*.sh
chmod +x docs/examples/*.sh
```

### Docker Build Slow

**Problem**: Build takes too long

**Solution**:
```bash
# Enable BuildKit
export DOCKER_BUILDKIT=1

# Use cached layers
./docker/scripts/docker-build.sh
```

### BuildKit Casing Error

**Problem**: `FromAsCasing: 'as' and 'FROM' keywords' casing do not match`

**Solution**:
This has been fixed in the Dockerfiles. All keywords are now UPPERCASE.

To verify:
```bash
./docker/scripts/verify-dockerfile-syntax.sh
```

### Credentials Error

**Problem**: `error getting credentials - err: exit status 1`

**Solution**:
```bash
# Restart Docker Desktop
# Or clear Docker credentials
rm ~/.docker/config.json
docker login
```

### Health Check Fails

**Problem**: Service won't become healthy

**Solution**:
```bash
# Check logs
docker-compose logs semantic_browser

# Restart with debug logging
RUST_LOG=debug docker-compose up
```

---

## Next Steps 📚

1. **Read Documentation**:
   - [README.md](../README.md) - Full documentation
   - [Docker Setup](docker-setup.md) - Docker details
   - [Testing](testing.md) - Testing guide

2. **Try Examples**:
   - [examples/](examples/) - API usage examples

3. **Explore Features**:
   - ML Models for NER
   - SPARQL queries
   - Knowledge Graph inference

4. **Contribute**:
   - Run tests: `./docker/scripts/docker-test.sh`
   - Check linting: `cargo fmt && cargo clippy`
   - Submit PR

---

## Support 💬

- **Issues**: GitHub Issues
- **Documentation**: See docs in repository
- **Examples**: `examples/` directory

## Summary

### Docker Workflow
```bash
cp config/.env.example .env
./docker/scripts/docker-up.sh -d
./examples/parse_html.sh
./docker/scripts/docker-up.sh --stop
```

### Local Workflow
```bash
cargo run &
curl -X POST http://localhost:3000/parse \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer secret" \
  -d '{"html": "<html><title>Test</title></html>"}'
```

Happy coding! 🚀
