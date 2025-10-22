# Environment Variables Reference

Complete reference for all environment variables supported by Semantic Browser.

## Table of Contents

- [Quick Start](#quick-start)
- [Authentication](#authentication)
- [LLM Agent Configuration](#llm-agent-configuration)
- [Machine Learning](#machine-learning)
- [Knowledge Graph](#knowledge-graph)
- [Browser Automation](#browser-automation)
- [API Server](#api-server)
- [Security](#security)
- [Logging](#logging)
- [Redis Integration](#redis-integration)
- [Monitoring](#monitoring)
- [Development](#development)

## Quick Start

```bash
# Copy the example file
cp .env.example .env

# Edit the file with your settings
nano .env

# At minimum, set a strong JWT secret
JWT_SECRET=$(openssl rand -base64 48)
```

## Authentication

### `JWT_SECRET` (REQUIRED)

**Description**: Secret key for JWT token signing and verification.

**Type**: String  
**Required**: Yes  
**Minimum Length**: 32 characters  
**Default**: None

**Example**:
```bash
# Generate a secure random key
JWT_SECRET=$(openssl rand -base64 48)
```

**Security Notes**:
- Must be at least 32 characters long
- Use cryptographically random values in production
- Never commit to version control
- Rotate periodically (every 90 days recommended)

---

## LLM Agent Configuration

### `LLM_PROVIDER`

**Description**: Specifies which LLM provider to use for agent operations.

**Type**: Enum  
**Required**: No  
**Options**: `ollama`, `openai`, `anthropic`  
**Default**: `ollama`

**Example**:
```bash
LLM_PROVIDER=ollama
```

---

### Ollama Configuration

#### `OLLAMA_API_URL`

**Description**: Ollama API endpoint URL.

**Type**: URL  
**Required**: No (when using Ollama)  
**Default**: `http://localhost:11434`

**Example**:
```bash
OLLAMA_API_URL=http://localhost:11434
```

#### `OLLAMA_MODEL`

**Description**: Ollama model to use for agent operations.

**Type**: String  
**Required**: No (when using Ollama)  
**Default**: `llama3:8b`

**Popular Models**:
- `llama3:8b` - Fast, general purpose (4.7GB)
- `llama3:70b` - High quality, slower (40GB)
- `mistral:7b` - Efficient, good reasoning (4.1GB)
- `codellama:13b` - Code-focused (7.4GB)

**Example**:
```bash
OLLAMA_MODEL=llama3:8b
```

---

### OpenAI Configuration

#### `OPENAI_API_KEY`

**Description**: OpenAI API key for GPT models.

**Type**: String (API Key)  
**Required**: Yes (when using OpenAI)  
**Security**: Never commit to version control  

**Example**:
```bash
OPENAI_API_KEY=sk-proj-abc123...
```

#### `OPENAI_MODEL`

**Description**: OpenAI model to use.

**Type**: String  
**Required**: No  
**Default**: `gpt-4-turbo-preview`

**Popular Models**:
- `gpt-4-turbo-preview` - Latest GPT-4 Turbo
- `gpt-4` - Standard GPT-4
- `gpt-3.5-turbo` - Faster, cheaper

**Example**:
```bash
OPENAI_MODEL=gpt-4-turbo-preview
```

#### `OPENAI_API_URL`

**Description**: OpenAI API base URL (for custom endpoints).

**Type**: URL  
**Required**: No  
**Default**: `https://api.openai.com/v1`

---

### Anthropic Configuration

#### `ANTHROPIC_API_KEY`

**Description**: Anthropic API key for Claude models.

**Type**: String (API Key)  
**Required**: Yes (when using Anthropic)  
**Security**: Never commit to version control

**Example**:
```bash
ANTHROPIC_API_KEY=sk-ant-api03-abc123...
```

#### `ANTHROPIC_MODEL`

**Description**: Anthropic Claude model to use.

**Type**: String  
**Required**: No  
**Default**: `claude-3-opus-20240229`

**Popular Models**:
- `claude-3-opus-20240229` - Most capable
- `claude-3-sonnet-20240229` - Balanced
- `claude-3-haiku-20240307` - Fastest

**Example**:
```bash
ANTHROPIC_MODEL=claude-3-opus-20240229
```

#### `ANTHROPIC_API_URL`

**Description**: Anthropic API base URL.

**Type**: URL  
**Required**: No  
**Default**: `https://api.anthropic.com/v1`

---

### Agent Behavior

#### `AGENT_MAX_ITERATIONS`

**Description**: Maximum reasoning iterations before stopping.

**Type**: Integer  
**Required**: No  
**Default**: `10`  
**Range**: 1-100

**Example**:
```bash
AGENT_MAX_ITERATIONS=10
```

#### `AGENT_TIMEOUT_SECS`

**Description**: Agent reasoning timeout in seconds.

**Type**: Integer  
**Required**: No  
**Default**: `300` (5 minutes)  
**Range**: 10-3600

**Example**:
```bash
AGENT_TIMEOUT_SECS=300
```

#### `AGENT_DEBUG`

**Description**: Enable detailed agent debug logging.

**Type**: Boolean  
**Required**: No  
**Default**: `false`

**Example**:
```bash
AGENT_DEBUG=true
```

---

## Machine Learning

### `NER_MODEL_PATH`

**Description**: Path to ONNX NER (Named Entity Recognition) model.

**Type**: File Path  
**Required**: No (requires `--features onnx-integration`)  
**Default**: None (uses regex fallback)

**Example**:
```bash
NER_MODEL_PATH=./models/ner-model.onnx
```

**Notes**:
- If not set or file doesn't exist, falls back to regex-based extraction
- Model must be ONNX format
- See [ML Integration Guide](../developer-guide/ml-integration.md) for model format

### `KG_INFERENCE_MODEL_PATH`

**Description**: Path to Knowledge Graph inference model.

**Type**: File Path  
**Required**: No (requires `--features onnx-integration`)  
**Default**: None (uses rule-based inference)

**Example**:
```bash
KG_INFERENCE_MODEL_PATH=./models/kg-inference-model.onnx
```

---

## Knowledge Graph

### `KG_PERSIST_PATH`

**Description**: Path for persistent Knowledge Graph storage.

**Type**: Directory Path  
**Required**: No  
**Default**: None (in-memory storage)

**Example**:
```bash
KG_PERSIST_PATH=./data/kg
```

**Notes**:
- If not set, uses in-memory storage (data lost on restart)
- Directory will be created if it doesn't exist
- Monitor disk usage in production
- Backup regularly for production data

---

## Browser Automation

### `CHROMIUM_PATH`

**Description**: Path to Chromium/Chrome executable.

**Type**: File Path  
**Required**: No (auto-detects if not set)

**Common Paths**:
```bash
# Linux
CHROMIUM_PATH=/usr/bin/chromium

# macOS
CHROMIUM_PATH=/Applications/Google Chrome.app/Contents/MacOS/Google Chrome

# Windows
CHROMIUM_PATH=C:\Program Files\Google\Chrome\Application\chrome.exe
```

### `BROWSER_HEADLESS`

**Description**: Run browser in headless mode (no GUI).

**Type**: Boolean  
**Required**: No  
**Default**: `true`

**Example**:
```bash
BROWSER_HEADLESS=true
```

### `BLOCK_ADS`

**Description**: Block ads and trackers for minimalist browsing.

**Type**: Boolean  
**Required**: No  
**Default**: `true`

**Example**:
```bash
BLOCK_ADS=true
```

**Notes**:
- Adds `--disable-background-networking` to chromium args
- URL-based blocking not fully supported in chromiumoxide 0.7

### `BLOCK_IMAGES`

**Description**: Block images for text-only extraction.

**Type**: Boolean  
**Required**: No  
**Default**: `false`

**Example**:
```bash
BLOCK_IMAGES=false
```

### `BROWSER_TIMEOUT_SECS`

**Description**: Navigation timeout in seconds.

**Type**: Integer  
**Required**: No  
**Default**: `30`  
**Range**: 5-300

**Example**:
```bash
BROWSER_TIMEOUT_SECS=30
```

### `BROWSER_POOL_SIZE`

**Description**: Maximum number of concurrent browser tabs.

**Type**: Integer  
**Required**: No  
**Default**: `2`  
**Range**: 1-10

**Example**:
```bash
BROWSER_POOL_SIZE=2
```

**Notes**:
- Each tab consumes ~100-200MB RAM
- Higher values = more concurrency but more memory

### `CHROMIUMOXIDE_USER_DATA_DIR`

**Description**: Directory for Chromium user data (profile).

**Type**: Directory Path  
**Required**: No  
**Default**: Unique temp folder per run

**Example**:
```bash
CHROMIUMOXIDE_USER_DATA_DIR=/tmp/semantic-browser/chromium-profile
```

---

## API Server

### `SERVER_ADDR`

**Description**: Server bind address.

**Type**: IP Address  
**Required**: No  
**Default**: `0.0.0.0`

**Example**:
```bash
SERVER_ADDR=0.0.0.0
```

**Security Notes**:
- Use `0.0.0.0` to bind to all interfaces
- Use `127.0.0.1` for localhost only
- Use specific IP for single interface

### `SERVER_PORT`

**Description**: Server port number.

**Type**: Integer  
**Required**: No  
**Default**: `3000`  
**Range**: 1024-65535

**Example**:
```bash
SERVER_PORT=3000
```

---

## Security

### `SECURITY_STRICT_MODE`

**Description**: Enable strict security mode.

**Type**: Boolean  
**Required**: No  
**Default**: `false`

**Example**:
```bash
SECURITY_STRICT_MODE=true
```

**Enables**:
- Stricter input validation
- Enhanced CSP headers
- Additional security checks

### `MAX_HTML_SIZE`

**Description**: Maximum HTML input size in bytes.

**Type**: Integer  
**Required**: No  
**Default**: `10000000` (10 MB)

**Example**:
```bash
MAX_HTML_SIZE=10000000
```

### `MAX_QUERY_LENGTH`

**Description**: Maximum SPARQL query length.

**Type**: Integer  
**Required**: No  
**Default**: `10000`

**Example**:
```bash
MAX_QUERY_LENGTH=10000
```

### `RATE_LIMIT_REQUESTS_PER_MINUTE`

**Description**: Maximum requests per minute per IP.

**Type**: Integer  
**Required**: No  
**Default**: `60`

**Example**:
```bash
RATE_LIMIT_REQUESTS_PER_MINUTE=60
```

### `RATE_LIMIT_BURST_SIZE`

**Description**: Burst size for rate limiting.

**Type**: Integer  
**Required**: No  
**Default**: `10`

**Example**:
```bash
RATE_LIMIT_BURST_SIZE=10
```

---

## Logging

### `RUST_LOG`

**Description**: Log level configuration.

**Type**: String  
**Required**: No  
**Default**: `info`

**Levels**: `trace`, `debug`, `info`, `warn`, `error`

**Examples**:
```bash
# Simple level
RUST_LOG=info

# Module-specific
RUST_LOG=semantic_browser=debug

# Multiple modules
RUST_LOG=semantic_browser=debug,semantic_browser::api=trace

# Different levels per module
RUST_LOG=warn,semantic_browser::llm=debug
```

---

## Redis Integration

### `REDIS_URL`

**Description**: Redis URL for token revocation and caching.

**Type**: URL  
**Required**: No (requires `--features redis-integration`)

**Format**: `redis://[username:password@]host[:port][/database]`

**Examples**:
```bash
# Local Redis
REDIS_URL=redis://127.0.0.1:6379

# With authentication
REDIS_URL=redis://username:password@host:6379/0

# Redis Cloud
REDIS_URL=redis://username:password@redis-12345.cloud.redislabs.com:12345
```

---

## Monitoring

### `PROMETHEUS_METRICS`

**Description**: Enable Prometheus metrics endpoint.

**Type**: Boolean  
**Required**: No (requires `--features observability`)  
**Default**: `false`

**Example**:
```bash
PROMETHEUS_METRICS=true
```

### `METRICS_PORT`

**Description**: Port for metrics endpoint.

**Type**: Integer  
**Required**: No  
**Default**: `9090`  
**Range**: 1024-65535

**Example**:
```bash
METRICS_PORT=9090
```

**Access**: `http://localhost:9090/metrics`

---

## Development

### `DEVELOPMENT_MODE`

**Description**: Enable development-friendly settings.

**Type**: Boolean  
**Required**: No  
**Default**: `false`

**Example**:
```bash
DEVELOPMENT_MODE=true
```

**⚠️ NOT FOR PRODUCTION**

**Enables**:
- More verbose logging
- Relaxed validation
- Development helpers

### `DISABLE_AUTH`

**Description**: Disable authentication (NOT FOR PRODUCTION).

**Type**: Boolean  
**Required**: No  
**Default**: `false`

**Example**:
```bash
DISABLE_AUTH=true
```

**⚠️ SECURITY WARNING**: Only use in trusted development environments.

---

## Environment-Specific Files

### Development
```bash
.env.development
```

### Staging
```bash
.env.staging
```

### Production
```bash
.env.production
```

**Usage**:
```bash
# Load specific environment
cp .env.production .env

# Or use direnv
echo "dotenv .env.production" > .envrc
direnv allow
```

---

## Docker Configuration

### Method 1: Volume Mount
```bash
docker run -v $(pwd)/.env:/app/.env semantic-browser
```

### Method 2: Env File
```bash
docker run --env-file .env semantic-browser
```

### Method 3: Individual Variables
```bash
docker run -e JWT_SECRET=... -e LLM_PROVIDER=ollama semantic-browser
```

---

## Security Best Practices

1. **Never commit `.env` files** to version control
2. **Use strong, random secrets** for `JWT_SECRET`
3. **Rotate secrets periodically** (90 days recommended)
4. **Use different secrets** per environment
5. **Store API keys securely** (use secret managers in production)
6. **Audit environment access** regularly
7. **Use HTTPS/TLS** in production
8. **Enable strict security mode** in production
9. **Monitor and log** authentication events
10. **Backup configuration** securely

---

## Validation

### Check Configuration
```bash
# View current environment (sanitized)
cargo run -- config show

# Validate configuration
cargo run -- config validate

# Test LLM connection
cargo run --example test_llm_connection
```

### Common Issues

#### Invalid JWT Secret
```
Error: JWT_SECRET must be at least 32 characters
```
**Solution**: Generate a new secret with `openssl rand -base64 48`

#### LLM Connection Failed
```
Error: Failed to connect to Ollama
```
**Solution**: Ensure Ollama is running: `ollama serve`

#### Browser Not Found
```
Error: Chromium not found
```
**Solution**: Set `CHROMIUM_PATH` or install Chrome/Chromium

---

## Examples

### Minimal Configuration
```bash
JWT_SECRET=$(openssl rand -base64 48)
```

### Development with Local LLM
```bash
JWT_SECRET=dev-secret-at-least-32-characters-long
LLM_PROVIDER=ollama
OLLAMA_MODEL=llama3:8b
AGENT_DEBUG=true
RUST_LOG=debug
BROWSER_HEADLESS=false
```

### Production with OpenAI
```bash
JWT_SECRET=$(openssl rand -base64 48)
LLM_PROVIDER=openai
OPENAI_API_KEY=sk-proj-...
OPENAI_MODEL=gpt-4-turbo-preview
SECURITY_STRICT_MODE=true
PROMETHEUS_METRICS=true
RUST_LOG=warn
KG_PERSIST_PATH=/var/lib/semantic-browser/kg
```

### High-Performance Setup
```bash
JWT_SECRET=$(openssl rand -base64 48)
BROWSER_POOL_SIZE=5
RATE_LIMIT_REQUESTS_PER_MINUTE=120
MAX_HTML_SIZE=20000000
AGENT_MAX_ITERATIONS=15
BROWSER_TIMEOUT_SECS=60
```

---

## Reference

- [Configuration Guide](./configuration.md)
- [Security Guide](../reference/security.md)
- [LLM Integration](../../src/llm/README.md)
- [Docker Setup](./docker-setup.md)
