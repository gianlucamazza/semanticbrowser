# API Examples

This directory contains example scripts for using the Semantic Browser API.

## Prerequisites

Make the example scripts executable:
```bash
chmod +x docs/user-guide/examples/*.sh
```

See [Quick Start](../quickstart.md) for server setup instructions.

## Examples

### Core API Examples
Extract semantic data from HTML:
```bash
./docs/user-guide/examples/parse_html.sh
```

Query and update the knowledge graph using SPARQL:
```bash
./docs/user-guide/examples/query_kg.sh
```

Browse a URL and extract semantic information:
```bash
./docs/user-guide/examples/browse_url.sh
```

JWT token generation and revocation with Redis:
```bash
./docs/user-guide/examples/token_revocation.sh
```

### Comprehensive Workflows
Complete ML NER workflow with BERT models:
```bash
./docs/user-guide/examples/ner-bert-workflow.sh
```

Knowledge graph inference with embeddings:
```bash
./docs/user-guide/examples/kg-ml-inference.sh
```

End-to-end browser automation workflow:
```bash
./docs/user-guide/examples/browser-workflow.sh
```

MCP protocol client integration:
```bash
./docs/user-guide/examples/mcp-client-integration.sh
```

## Authentication and Rate Limiting

See [API Documentation](../../api/README.md) for authentication and rate limiting details.

## Environment Variables

See **[Docker Setup Environment Variables](../docker-setup.md#environment-variables)** for all configuration options.

Example:
```bash
RUST_LOG=debug KG_PERSIST_PATH=./kg_data cargo run
```
