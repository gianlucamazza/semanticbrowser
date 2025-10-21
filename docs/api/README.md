# API Documentation

The Semantic Browser provides a REST API for semantic web operations. All endpoints require authentication and are rate-limited.

## Authentication

All API requests must include the `Authorization` header:
```
Authorization: Bearer secret
```

**⚠️ Security Warning**: The default secret token is `secret` for demonstration purposes. **Change this in production!**

## Rate Limiting

- **Limit**: 10 requests per minute per IP address
- **Headers Considered**: `X-Forwarded-For`, `X-Real-IP` (for proxy support)
- **Response**: Returns error message when limit exceeded

## Endpoints

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
- Uses browser automation tools (browser-use)
- Supports PyO3 integration for Python-based browsing
- Falls back to HTTP requests if advanced browsing fails

**Status Codes:**
- `200`: Success
- `401`: Unauthorized
- `429`: Rate limit exceeded
- `400`: Invalid URL

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
}
```

## Configuration

API behavior can be configured via environment variables:

- `API_SECRET`: Authentication token (default: "secret")
- `RATE_LIMIT_REQUESTS`: Max requests per minute (default: 10)
- `RATE_LIMIT_WINDOW_SECONDS`: Rate limit window (default: 60)

## Examples

See the `examples/` directory for complete API usage examples:
- `examples/parse_html.sh` - HTML parsing
- `examples/query_kg.sh` - Knowledge Graph queries
- `examples/browse_url.sh` - URL browsing