# Configuration Files

This directory contains configuration files and templates for the Semantic Browser project.

## Environment Configuration

The main configuration is handled through environment variables. Copy `.env.example` to `.env` and customize as needed:

```bash
cp .env.example .env
# Edit .env with your settings
nano .env
```

### Key Configuration Options

- **Logging**: `RUST_LOG` - Set logging level (trace, debug, info, warn, error)
- **Knowledge Graph**: `KG_PERSIST_PATH` - Path for persistent KG storage
- **ML Models**: `NER_MODEL_PATH`, `KG_INFERENCE_MODEL_PATH` - Paths to ONNX models
- **API Settings**: Server host/port, authentication secrets, rate limiting
- **Docker**: Resource limits and compose settings

See `.env.example` (in the project root) for the complete list of available configuration options with detailed comments.

## Advanced Configuration

### Rate Limiting Configuration

The Semantic Browser implements sophisticated rate limiting to protect against abuse while allowing legitimate usage.

#### Basic Rate Limiting

```bash
# Maximum requests per minute per IP address
RATE_LIMIT_REQUESTS_PER_MINUTE=60

# Burst size - additional requests allowed beyond the rate limit
RATE_LIMIT_BURST_SIZE=10
```

#### How Rate Limiting Works

- **Token Bucket Algorithm**: Each IP gets a bucket that fills at a constant rate
- **Burst Handling**: Extra capacity allows for legitimate traffic spikes
- **IP Detection**: Supports `X-Forwarded-For`, `X-Real-IP` headers for proxy setups
- **Response**: Returns `429 Too Many Requests` when limit exceeded

#### Production Tuning

```bash
# High-traffic production
RATE_LIMIT_REQUESTS_PER_MINUTE=1000
RATE_LIMIT_BURST_SIZE=100

# API gateway behind load balancer
RATE_LIMIT_REQUESTS_PER_MINUTE=5000
RATE_LIMIT_BURST_SIZE=500
```

### Browser Automation Configuration

Browser automation provides JavaScript execution and dynamic content access through headless Chromium.

#### Basic Browser Setup

```bash
# Enable browser automation
cargo build --features browser-automation

# Auto-detect Chromium (recommended)
# CHROMIUM_PATH=/usr/bin/chromium

# Run headless
BROWSER_HEADLESS=true
```

#### Resource Blocking

```bash
# Block ads and trackers for faster, cleaner browsing
BLOCK_ADS=true

# Block images for text-only extraction
BLOCK_IMAGES=false

# Block background network activity
# (automatically enabled when BLOCK_ADS=true)
```

#### Performance Tuning

```bash
# Navigation timeout in seconds
BROWSER_TIMEOUT_SECS=30

# Maximum concurrent browser tabs
BROWSER_POOL_SIZE=2

# User data directory for persistent sessions
CHROMIUMOXIDE_USER_DATA_DIR=/tmp/semantic-browser/profile
```

#### Chromium Path Configuration

```bash
# Linux
CHROMIUM_PATH=/usr/bin/chromium
# or
CHROMIUM_PATH=/usr/bin/chromium-browser

# macOS
CHROMIUM_PATH=/Applications/Google\ Chrome.app/Contents/MacOS/Google\ Chrome

# Windows
CHROMIUM_PATH=C:\Program Files\Google\Chrome\Application\chrome.exe
```

#### Pool Management

The browser pool automatically manages Chromium instances:

- **Warm Pool**: Pre-warmed browsers reduce startup latency
- **Concurrent Limits**: `BROWSER_POOL_SIZE` controls resource usage
- **Automatic Cleanup**: Failed browsers are replaced automatically
- **Session Isolation**: Each request gets a clean browser context

### Knowledge Graph Persistence

```bash
# Enable persistence (recommended for production)
KG_PERSIST_PATH=./data/kg

# Directory structure created automatically:
# ./data/kg/
# ├── current-0/
# ├── current-1/
# └── ...
```

### ML Model Configuration

```bash
# NER model (optional)
NER_MODEL_PATH=./models/ner-model.onnx
NER_TOKENIZER_PATH=./models/ner-tokenizer.json
NER_LABELS_PATH=./models/ner-labels.txt

# KG inference model (optional)
KG_INFERENCE_MODEL_PATH=./models/kg-inference-model.onnx
KG_EMBEDDING_TYPE=TransE  # TransE, DistMult, ComplEx
KG_ENTITY_MAPPING_PATH=./models/kg-entities.json
KG_RELATION_MAPPING_PATH=./models/kg-relations.json

# KG inference parameters
KG_INFERENCE_CONFIDENCE_THRESHOLD=0.8
KG_INFERENCE_TOP_K=10
KG_INFERENCE_MAX_INSERTS=100
KG_INFERENCE_SAMPLE_SIZE=1000
```

### Logging Configuration

```bash
# Global log level
RUST_LOG=info

# Module-specific logging
RUST_LOG=semantic_browser=debug,semantic_browser::api=trace,tract=info

# Production logging
RUST_LOG=warn
```

### Security Configuration

```bash
# Enable strict input validation
SECURITY_STRICT_MODE=true

# Maximum HTML input size (bytes)
MAX_HTML_SIZE=10485760  # 10MB

# Maximum SPARQL query length
MAX_QUERY_LENGTH=10000  # 10KB
```

### Observability Configuration

```bash
# Enable Prometheus metrics
PROMETHEUS_METRICS=true

# Metrics endpoint port
METRICS_PORT=9090

# OpenTelemetry tracing
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
OTEL_SERVICE_NAME=semantic-browser
OTEL_TRACES_EXPORTER=otlp
```

## Docker Configuration

Docker-related configuration is handled through:
- `docker-compose.yml` - Production/development services
- `docker-compose.test.yml` - Testing environment
- `Dockerfile` - Main application container
- `Dockerfile.test` - Test container

## Security Notes

- Never commit `.env` files to version control
- Use strong, unique secrets for `API_SECRET` in production
- Review rate limiting settings for your deployment environment