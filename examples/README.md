# API Examples

This directory contains example scripts for using the Semantic Browser API.

## Prerequisites

1. Start the server:
   ```bash
   cargo run
   ```

2. Make the example scripts executable:
   ```bash
   chmod +x examples/*.sh
   ```

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

## Authentication

All endpoints require the `Authorization: Bearer secret` header. This is a hardcoded token for demonstration purposes.

## Rate Limiting

All endpoints are rate limited to 10 requests per minute per IP address.

## Environment Variables

- `RUST_LOG=debug` - Enable debug logging
- `KG_PERSIST_PATH=./kg_data` - Persist knowledge graph to disk
- `NER_MODEL_PATH=./models/ner.onnx` - Use ML model for Named Entity Recognition
- `KG_INFERENCE_MODEL_PATH=./models/kg_inference.onnx` - Use ML model for KG inference

Example with all variables:
```bash
RUST_LOG=debug KG_PERSIST_PATH=./kg_data cargo run
```
