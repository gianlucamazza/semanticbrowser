# API Documentation

The Semantic Browser provides a REST API for semantic web operations. All endpoints require JWT authentication and are rate-limited.

## Authentication

All API requests must include the `Authorization` header with a valid JWT token:

```
Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...
```

### Prerequisites

**IMPORTANT**: Before using the API, you must configure JWT authentication:

1. **Set JWT_SECRET** in `.env`:
   ```bash
   # Generate a secure random secret (at least 32 characters)
   JWT_SECRET=$(openssl rand -base64 48)
   echo "JWT_SECRET=$JWT_SECRET" >> .env
   ```

2. **Start the server** with the configured secret:
   ```bash
   cargo run
   ```

**⚠️ Security Warning**: 
- Never use the default JWT_SECRET in production!
- Generate a strong, random secret at least 32 characters long
- Keep JWT_SECRET confidential and never commit to version control
- Rotate JWT_SECRET periodically

### Token Generation

Generate a JWT token using the authentication endpoint:

```bash
# Generate token
TOKEN_RESPONSE=$(curl -s -X POST http://localhost:3000/auth/token \
  -H "Content-Type: application/json" \
  -d '{"username": "my-user", "role": "user"}')

# Extract token
TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r .token)

echo "Token: $TOKEN"
```

**Response:**
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "expires_in": 86400
}
```

**Token Parameters:**
- `username`: Any string identifier for the user
- `role`: Optional role for RBAC (e.g., "user", "admin")

**Token Expiration:**
- Default: 24 hours (86400 seconds)
- Tokens automatically expire after this period
- Generate a new token when expired

## Rate Limiting

- **Limit**: 10 requests per minute per IP address
- **Headers Considered**: `X-Forwarded-For`, `X-Real-IP` (for proxy support)
- **Response**: Returns error message when limit exceeded

## Endpoints

### POST `/auth/token`

Generate a JWT authentication token.

**Request Body:**
```json
{
  "username": "my-user",
  "role": "user"
}
```

**Response:**
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "expires_in": 86400
}
```

**Status Codes:**
- `200`: Success
- `500`: Token generation failed

### POST `/parse`

Parse HTML content and extract semantic data.

**Request Body:**
```json
{
  "html": "<html><head><title>Test</title></head><body>...</body></html>"
}
```

**Response:**
```json
{
  "title": "Test",
  "entities": ["schema:Article", "schema:Person"]
}
```

**Status Codes:**
- `200`: Success
- `401`: Unauthorized
- `429`: Rate limit exceeded
- `400`: Invalid input

### POST `/query`

Query or update the Knowledge Graph using SPARQL.

**Request Body:**
```json
{
  "query": "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 10"
}
```

**Response:**
```json
{
  "results": ["result1", "result2", "..."]
}
```

**Supported Operations:**
- `SELECT` - Query data
- `INSERT` - Add triples
- `DELETE` - Remove triples
- `CONSTRUCT` - Transform results
- `ASK` - Boolean queries
- `DESCRIBE` - Describe resources

**Status Codes:**
- `200`: Success
- `401`: Unauthorized
- `429`: Rate limit exceeded
- `400`: Invalid SPARQL query

### POST `/browse`

Browse a URL and extract semantic information using external tools.

**Request Body:**
```json
{
  "url": "https://example.com",
  "query": "Extract main article content"
}
```

**Response:**
```json
{
  "data": "Extracted content from the webpage..."
}
```

**Features:**
- Headless browser automation with chromiumoxide
- JavaScript execution and dynamic content support
- Resource blocking (ads, trackers, images)
- Semantic data extraction (microdata, JSON-LD, Open Graph)
- Automatic fallback to HTTP if browser fails

**Status Codes:**
- `200`: Success
- `401`: Unauthorized
- `429`: Rate limit exceeded
- `400`: Invalid URL

### POST `/browse_kg`

Browse a URL, extract semantic information, and automatically insert into the Knowledge Graph.

**Request Body:**
```json
{
  "url": "https://example.com"
}
```

**Response:**
```json
{
  "data": "Browsed https://example.com and inserted into KG...",
  "triples_inserted": 15,
  "final_url": "https://example.com",
  "snapshot": {
    "title": "Example Domain",
    "description": "...",
    "json_ld_count": 2,
    "microdata": [...]
  }
}
```

**Features:**
- Same browsing capabilities as `/browse`
- Automatic RDF triple extraction and insertion
- Returns count of triples inserted
- Enables immediate SPARQL queries on browsed content

**Status Codes:**
- `200`: Success
- `401`: Unauthorized
- `429`: Rate limit exceeded
- `400`: Invalid URL

### GET `/kg/entities`

List all entities currently in the Knowledge Graph.

**Response:**
```json
{
  "items": [
    "https://schema.org/WebPage",
    "https://example.com",
    "https://schema.org/Person"
  ]
}
```

**Status Codes:**
- `200`: Success
- `401`: Unauthorized
- `429`: Rate limit exceeded

### GET `/kg/relations`

List all relations currently in the Knowledge Graph.

**Response:**
```json
{
  "items": [
    "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
    "https://schema.org/name",
    "https://schema.org/url"
  ]
}
```

**Status Codes:**
- `200`: Success
- `401`: Unauthorized
- `429`: Rate limit exceeded

### POST `/auth/revoke`

Revoke a JWT token immediately (requires Redis integration).

**Request Body:**
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
}
```

**Response:**
```json
{
  "message": "Token revoked successfully"
}
```

**Requirements:**
- `--features redis-integration`
- `REDIS_URL` configured

**Status Codes:**
- `200`: Success
- `400`: Invalid token format
- `401`: Unauthorized
- `500`: Redis connection failed

### GET `/metrics`

Expose Prometheus-compatible metrics for monitoring.

**Response:**
```
# HELP semantic_browser_http_requests_total Total HTTP requests
# TYPE semantic_browser_http_requests_total counter
semantic_browser_http_requests_total{endpoint="/parse",method="POST",status="200"} 42

# HELP semantic_browser_http_request_duration_seconds HTTP request duration
# TYPE semantic_browser_http_request_duration_seconds histogram
...
```

**Requirements:**
- `--features observability`
- `PROMETHEUS_METRICS=true`

**Status Codes:**
- `200`: Success
- `404`: Metrics not enabled

### POST `/langgraph`

Execute a LangGraph workflow for complex multi-step operations using a custom StateGraph implementation.

**Request Body:**
```json
{
  "graph_definition": {
    "entry_point": "start",
    "edges": {
      "start": "browse",
      "browse": "extract",
      "extract": "query"
    },
    "conditional_edges": {
      "extract": {
        "type": "has_data"
      }
    }
  },
  "input": "initial input data"
}
```

**Graph Definition Schema:**
- `entry_point`: Starting node name (default: "start")
- `edges`: Object mapping from-node to to-node for linear flow
- `conditional_edges`: Object mapping nodes to conditional routing logic

**Conditional Edge Types:**
- `has_data`: Routes to "query" if extract_result exists, otherwise to "end"

**Built-in Nodes:**
- `browse`: Browses a URL and extracts semantic data
- `extract`: Extracts entities from semantic data using annotator
- `query`: Executes SPARQL queries against the Knowledge Graph

**Response:**
```json
{
  "result": "Workflow completed. Final step: query\nbrowse_result: Browsed https://example.com...\nextract_result: Extracted 5 entities...\nquery_result: Query returned 3 results..."
}
```

**Features:**
- Custom StateGraph with nodes, edges, and conditional routing
- Built-in nodes for browse, extract, and query operations
- State persistence across workflow steps
- Error recovery and logging
- Integration with Knowledge Graph and browser automation

**Status Codes:**
- `200`: Success
- `400`: Invalid workflow definition
- `401`: Unauthorized
- `500`: Workflow execution failed

## Error Handling

All endpoints return consistent error responses:

```json
{
  "title": null,
  "entities": ["Error message"]
}
```

## Data Models

### ParseRequest
```rust
struct ParseRequest {
    html: String,
}
```

### ParseResponse
```rust
struct ParseResponse {
    title: Option<String>,
    entities: Vec<String>,
}
```

### QueryRequest
```rust
struct QueryRequest {
    query: String,
}
```

### QueryResponse
```rust
struct QueryResponse {
    results: Vec<String>,
}
```

### BrowseRequest
```rust
struct BrowseRequest {
    url: String,
    query: String,
}
```

### BrowseResponse
```rust
struct BrowseResponse {
    data: String,
    snapshot: Option<SemanticSnapshot>,
}
```

`data` contains the legacy plain-text summary for backward compatibility. When semantic
extraction succeeds, `snapshot` provides a structured view of the page that downstream
clients and the knowledge graph can ingest directly.

### SemanticSnapshot
// Requires `use std::collections::HashMap;`
```rust
struct SemanticSnapshot {
    title: Option<String>,
    description: Option<String>,
    language: Option<String>,
    canonical_url: Option<String>,
    final_url: String,
    keywords: Vec<String>,
    open_graph: HashMap<String, String>,
    twitter_card: HashMap<String, String>,
    json_ld_count: usize,
    microdata: Vec<MicrodataSummary>,
    text_preview: String,
    text_length: usize,
    query_matches: Vec<QueryMatch>,
}
```

### MicrodataSummary
```rust
struct MicrodataSummary {
    item_type: String,
    properties: usize,
}
```

### QueryMatch
```rust
struct QueryMatch {
    excerpt: String,
    element: String,
    score: f32,
}
```

### BrowseKGResponse
```rust
struct BrowseKGResponse {
    data: String,
    triples_inserted: usize,
    final_url: String,
    snapshot: Option<SemanticSnapshot>,
}
```

## Metrics and Monitoring

The Semantic Browser provides Prometheus-compatible metrics for monitoring and observability.

### Setup

1. **Enable Metrics**:
   ```bash
   cargo build --features observability
   ```

2. **Configure Metrics**:
   ```bash
   # Add to .env
   PROMETHEUS_METRICS=true
   METRICS_PORT=9090
   ```

3. **Access Metrics**:
   ```bash
   curl http://localhost:9090/metrics
   ```

### Available Metrics

#### HTTP Request Metrics
- `semantic_browser_http_requests_total` - Total HTTP requests by endpoint, method, and status
- `semantic_browser_http_request_duration_seconds` - Request duration histogram

#### Knowledge Graph Metrics
- `semantic_browser_kg_operations_total` - KG operations (insert, query, delete)
- `semantic_browser_kg_operation_duration_seconds` - Operation duration
- `semantic_browser_kg_size` - Current KG size by graph type

#### Browser Automation Metrics
- `semantic_browser_browser_operations_total` - Browser operations by result
- `semantic_browser_browser_operation_duration_seconds` - Operation duration

#### ML Inference Metrics
- `semantic_browser_ml_inference_total` - ML inference operations by model type
- `semantic_browser_ml_inference_duration_seconds` - Inference duration with confidence

#### Parse Operations
- `semantic_browser_parse_operations_total` - Parse operations by content type and result
- `semantic_browser_parse_operation_duration_seconds` - Parse duration

#### System Metrics
- `semantic_browser_active_connections` - Active connections by type
- `semantic_browser_uptime_seconds` - Server uptime

### Example Prometheus Configuration

```yaml
scrape_configs:
  - job_name: 'semantic-browser'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 15s
```

### Example Grafana Dashboard

Metrics can be visualized in Grafana with panels for:
- Request rate and latency
- Error rates by endpoint
- KG growth over time
- ML inference performance
- Browser operation success rates

## Configuration

API behavior can be configured via environment variables:

- `JWT_SECRET`: JWT signing secret (REQUIRED for production)
- `RATE_LIMIT_REQUESTS`: Max requests per minute per IP (default: 10)
- `RATE_LIMIT_WINDOW_SECONDS`: Rate limit window in seconds (default: 60)
- `SECURITY_STRICT_MODE`: Enable strict input validation (default: false)
- `MAX_HTML_SIZE`: Maximum HTML input size in bytes (default: 10MB)
- `MAX_QUERY_LENGTH`: Maximum SPARQL query length (default: 10KB)
- `PROMETHEUS_METRICS`: Enable Prometheus metrics endpoint (default: false)
- `METRICS_PORT`: Port for metrics endpoint (default: 9090)

## Examples

See the `docs/user-guide/examples/` directory for complete API usage examples:

### Core API Examples
- `parse_html.sh` - HTML parsing and semantic extraction
- `query_kg.sh` - Knowledge Graph SPARQL queries
- `browse_url.sh` - URL browsing with semantic extraction

### Comprehensive Workflows
- `ner-bert-workflow.sh` - ML-based named entity recognition
- `kg-ml-inference.sh` - Knowledge graph inference with embeddings
- `browser-workflow.sh` - Complete browser automation workflow
- `mcp-client-integration.sh` - MCP protocol client integration

## MCP Protocol Server

The Semantic Browser provides an MCP (Model Context Protocol) server binary for integration with MCP-compatible clients like Claude Desktop.

### Running the MCP Server

```bash
cargo run --bin semantic_browser_mcp
```

The server communicates via JSON-RPC 2.0 over stdin/stdout.

### MCP Protocol Details

**Protocol Version:** 2025-06-18
**Server Name:** semantic-browser-mcp
**Capabilities:**
- Tools: listChanged=false (static tool list)

### MCP Messages

#### Initialize
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2025-06-18",
    "clientInfo": {
      "name": "claude-desktop",
      "version": "1.0.0"
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "protocolVersion": "2025-06--18",
    "capabilities": {
      "tools": {
        "listChanged": false
      }
    },
    "serverInfo": {
      "name": "semantic-browser-mcp",
      "title": "Semantic Browser MCP",
      "version": "0.1.0"
    },
    "instructions": "Provides HTML parsing, knowledge graph querying, and browsing tools backed by the Semantic Browser."
  }
}
```

#### List Tools
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list",
  "params": {}
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "tools": [
      {
        "name": "semanticbrowser.parse_html",
        "description": "Parse HTML content and extract semantic annotations.",
        "inputSchema": {
          "type": "object",
          "properties": {
            "html": {
              "type": "string",
              "description": "Raw HTML content to parse."
            }
          },
          "required": ["html"]
        }
      },
      {
        "name": "semanticbrowser.query_kg",
        "description": "Execute read or write operations against the Semantic Browser knowledge graph.",
        "inputSchema": {
          "type": "object",
          "properties": {
            "query": {
              "type": "string",
              "description": "SPARQL query or update statement."
            }
          },
          "required": ["query"]
        }
      },
      {
        "name": "semanticbrowser.browse_url",
        "description": "Fetch a URL and summarize semantic signals relevant to a query.",
        "inputSchema": {
          "type": "object",
          "properties": {
            "url": {
              "type": "string",
              "format": "uri",
              "description": "Target URL to browse."
            },
            "query": {
              "type": "string",
              "description": "Optional focus or extraction instruction.",
              "default": ""
            }
          },
          "required": ["url"]
        }
      }
    ]
  }
}
```

#### Call Tool
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "semanticbrowser.browse_url",
    "arguments": {
      "url": "https://example.com",
      "query": "Extract main content"
    }
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "content": [
      {
        "type": "text",
        "text": "Browsed https://example.com. Summary length: 1234 characters."
      }
    ],
    "structuredContent": {
      "url": "https://example.com",
      "query": "Extract main content",
      "summary": "...",
      "snapshot": {...}
    }
  }
}
```

### MCP Tools

#### semanticbrowser.parse_html
Parses HTML content and extracts semantic annotations including microdata and JSON-LD.

#### semanticbrowser.query_kg
Executes SPARQL queries or updates against the Knowledge Graph.

#### semanticbrowser.browse_url
Browses a URL and extracts semantic information, automatically inserting into the Knowledge Graph.

### Configuration
The MCP server uses the same environment variables as the main API server. Set `KG_PERSIST_PATH` for persistent Knowledge Graph storage.
