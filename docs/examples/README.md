# API Examples

This directory contains example scripts for using the Semantic Browser API.

## Prerequisites

Make the example scripts executable:
```bash
chmod +x docs/examples/*.sh
```

See [Quick Start](../guides/quickstart.md) for server setup instructions.

## Examples

### Parse HTML
Extract semantic data from HTML:
```bash
./examples/parse_html.sh
```

### Query Knowledge Graph
Query and update the knowledge graph using SPARQL:
```bash
./examples/query_kg.sh
```

### Browse URL
Browse a URL and extract semantic information:
```bash
./examples/browse_url.sh
```

## Authentication and Rate Limiting

See [API Documentation](../api/README.md) for authentication and rate limiting details.

## Environment Variables

See **[Docker Setup Environment Variables](../guides/docker-setup.md#environment-variables)** for all configuration options.

Example:
```bash
RUST_LOG=debug KG_PERSIST_PATH=./kg_data cargo run
```
