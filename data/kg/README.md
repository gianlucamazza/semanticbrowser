# Knowledge Graph Data

This directory contains persistent data for the Semantic Browser's Knowledge Graph (KG) functionality.

## Overview

The Knowledge Graph stores semantic relationships extracted from web content using RDF (Resource Description Framework) triples. It supports:

- **SPARQL queries** for complex data retrieval
- **RDF triple storage** with optional persistence
- **Named Entity Recognition** integration
- **Semantic inference** capabilities

## Data Storage

When `KG_PERSIST_PATH` is configured in `.env`, the KG data is stored here as RDF triples. The data includes:

- Extracted entities and relationships from parsed HTML
- Microdata and JSON-LD structured data
- User-defined triples from API interactions
- Inferred relationships from ML models

## File Structure

- RDF data files (created automatically when persistence is enabled)
- Backup files (created during maintenance operations)

## Configuration

Set the persistence path in your `.env` file:

```bash
KG_PERSIST_PATH=./data/kg
```

If not set, the KG operates in-memory only (data lost on restart).

## Management

The KG can be queried and updated via the REST API endpoints:
- `POST /query` - SPARQL queries (SELECT, INSERT, DELETE, etc.)
- `POST /parse` - Add data from HTML parsing

See the [API documentation](../guides/quickstart.md) for usage examples.
