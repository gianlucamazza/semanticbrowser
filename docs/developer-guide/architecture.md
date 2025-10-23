# Architecture Overview

This document provides a high-level overview of the Semantic Browser architecture, design decisions, and system components.

## System Overview

The Semantic Browser is a Rust-based semantic web agent designed to extract, process, and query semantic information from web content. It combines HTML parsing, natural language processing, and knowledge graph technologies to provide intelligent web content analysis.

## Core Architecture

### Component Diagram

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web Content   │───▶│   HTML Parser   │───▶│  Entity Extract │
│                 │    │                 │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                                         │
┌─────────────────┐    ┌─────────────────┐             │
│  External Tools │───▶│ Browser         │◀────────────┘
│                 │    │ Automation      │
└─────────────────┘    └─────────────────┘
          │                       │             ┌─────────────────┐
          └───────────────────────┼────────────▶│   LLM Provider  │
                                  ▼             │  (Vision/Text)   │
                     ┌─────────────────┐        └─────────────────┘
                     │ Knowledge Graph │
                     │                 │
                     └─────────────────┘
                              │
                              ▼
                     ┌─────────────────┐
                     │   REST API      │
                     │                 │
                     └─────────────────┘
```

## Key Components

### 1. HTML Parser (`src/parser.rs`)

**Purpose**: Parse HTML content and extract semantic information.

**Key Features**:
- HTML5 parsing with semantic element extraction
- Microdata and JSON-LD structured data extraction
- Title and metadata extraction

**Dependencies**: `html5ever`, `scraper`

### 2. Entity Annotator (`src/annotator.rs`)

**Purpose**: Identify and classify named entities in text.

**Key Features**:
- Named Entity Recognition (NER)
- ML-based entity classification
- Fallback to regex-based extraction

**Dependencies**: `tract-core` (optional)

### 3. Knowledge Graph (`src/kg.rs`)

**Purpose**: Store and query semantic relationships.

**Key Features**:
- RDF triple storage
- SPARQL query support
- Optional persistence to disk
- Inference capabilities

**Dependencies**: `oxigraph`

### 4. REST API (`src/api.rs`)

**Purpose**: Provide HTTP interface for client applications.

**Key Features**:
- RESTful endpoints for parsing, querying, and browsing
- Authentication and rate limiting
- Real IP detection for proxy support
- Comprehensive error handling

**Dependencies**: `axum`, `tokio`

### 5. External Integrations (`src/external.rs`)

**Purpose**: Interface with external tools and services.

**Key Features**:
- Browser automation integration
- Python interoperability (PyO3)
- LangGraph workflow support

**Dependencies**: `pyo3` (optional)

### 6. LLM Provider (`src/llm/`)

**Purpose**: Interface with Large Language Models for intelligent content analysis and vision capabilities.

**Key Features**:
- Multi-provider support (OpenAI, Anthropic, Ollama)
- Vision model integration for image analysis
- Streaming responses for real-time interaction
- Tool/function calling for agent workflows
- Content blocks supporting text and images

**Dependencies**: `reqwest`, `tokio`, `serde`

### 7. Browser Automation (`src/browser.rs`)

**Purpose**: Headless browser automation for dynamic content extraction and multi-tab orchestration.

**Key Features**:
- Chromium-based headless browsing
- Multi-tab management with resource pooling
- Semantic data extraction from web pages
- Screenshot capture and JavaScript execution
- Concurrent tab operations for efficiency

**Dependencies**: `chromiumoxide`, `futures`

### 8. Security (`src/security.rs`)

**Purpose**: Provide security utilities and validation.

**Key Features**:
- Input validation and sanitization
- HTML and SPARQL query validation
- Security event logging
- Optional seccomp sandboxing

## Data Flow

1. **Content Ingestion**: HTML content is received via API or file input
2. **Parsing**: HTML is parsed to extract structured data and text content
3. **Entity Recognition**: Text is analyzed for named entities and semantic relationships
4. **LLM Analysis**: Content is analyzed using LLMs for deeper understanding (including vision models for images)
5. **Knowledge Graph**: Extracted information is stored as RDF triples
6. **Query Processing**: SPARQL queries retrieve and manipulate stored knowledge
7. **Browser Automation**: Dynamic content is extracted using headless browser with multi-tab support
8. **External Integration**: Additional tools and workflows provide extended capabilities

## Design Decisions

### Why Rust?

- **Performance**: Critical for real-time web content processing
- **Memory Safety**: Prevents common security vulnerabilities
- **Concurrency**: Efficient handling of multiple requests
- **Ecosystem**: Rich crate ecosystem for web technologies

### Why RDF/SPARQL?

- **Standards-Based**: W3C standards for semantic web
- **Interoperability**: Compatible with other semantic web tools
- **Query Power**: Expressive query language for complex relationships
- **Extensibility**: Schema-less design allows flexible data models

### Why Axum for HTTP?

- **Async-Native**: Built on Tokio for high performance
- **Type Safety**: Compile-time guarantees for HTTP handling
- **Modular**: Composable middleware and routing
- **Ecosystem**: Growing ecosystem of axum-based tools

## Performance Considerations

### Memory Management
- Streaming parsing for large HTML documents
- Efficient RDF triple storage with indexing
- Connection pooling for external services

### Concurrency
- Async/await throughout the stack
- Non-blocking I/O operations
- Configurable thread pools

### Caching
- Optional KG persistence to disk
- In-memory caching for frequent queries
- Docker layer caching for builds

## Security Architecture

### Input Validation
- HTML sanitization and size limits
- SPARQL query complexity limits
- URL validation for browsing operations

### Authentication & Authorization
- Bearer token authentication
- Rate limiting per IP address
- Request logging and monitoring

### Sandboxing
- Optional seccomp system call filtering
- Container isolation via Docker
- Resource limits and timeouts

## Deployment Architecture

### Docker-Based Deployment
- Multi-stage builds for optimized images
- Non-root user execution
- Health checks and graceful shutdown

### Configuration Management
- Environment-based configuration
- Validation of configuration values
- Secure credential handling

## Monitoring & Observability

### Logging
- Structured logging with `tracing`
- Configurable log levels
- Security event logging

### Metrics
- Performance benchmarking
- Request/response metrics
- Error rate monitoring

## Future Architecture Considerations

### Scalability
- Horizontal scaling with load balancing
- Database integration for larger KGs
- Caching layer for improved performance

### Extensibility
- Plugin architecture for custom parsers
- Webhook support for external integrations
- GraphQL API for complex queries

### Cloud-Native Features
- Kubernetes deployment manifests
- Service mesh integration
- Distributed tracing support