# Docker Development Environment Guide

Complete guide for developing Semantic Browser using Docker with Ollama, Redis, and hot-reload capabilities.

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Services](#services)
- [Development Workflow](#development-workflow)
- [Ollama Configuration](#ollama-configuration)
- [Hot Reload](#hot-reload)
- [Debugging](#debugging)
- [Common Tasks](#common-tasks)
- [Troubleshooting](#troubleshooting)
- [Performance Tips](#performance-tips)

## Overview

The development Docker environment provides:

- **Ollama**: Local LLM inference (no API keys required)
- **Redis**: Caching and token revocation
- **Hot Reload**: Automatic rebuilds on code changes
- **Full Features**: All Rust features enabled
- **Debugging Tools**: gdb, lldb, verbose logging
- **Isolated Environment**: Clean, reproducible setup

### Benefits

✅ **Zero API Costs**: Use Ollama instead of OpenAI/Anthropic  
✅ **Fast Iteration**: Hot-reload rebuilds in seconds  
✅ **Full Stack**: All services in one command  
✅ **Reproducible**: Same environment for all developers  
✅ **Debugging**: Full toolchain and verbose logging  

## Quick Start

### Prerequisites

- Docker 24.0+ and Docker Compose 2.0+
- 8GB RAM minimum (16GB recommended for Ollama)
- 10GB free disk space

### Start Development Environment

```bash
# Start all services
./docker/scripts/docker-dev.sh up

# Pull an Ollama model (required for LLM features)
./docker/scripts/docker-dev.sh ollama-pull llama3.2

# View logs
./docker/scripts/docker-dev.sh logs

# Test the API
curl http://localhost:3000/
```

### Stop Environment

```bash
# Stop all services
./docker/scripts/docker-dev.sh down

# Or use Makefile shortcuts
make docker-dev-down
```

## Architecture

### Service Stack

```
┌─────────────────────────────────────────────┐
│         Semantic Browser (Port 3000)        │
│  - Cargo Watch (Hot Reload)                 │
│  - All Features Enabled                     │
│  - Debug Symbols                            │
└────────┬────────────────────────┬───────────┘
         │                        │
         ▼                        ▼
┌─────────────────┐      ┌─────────────────┐
│  Ollama (11434) │      │  Redis (6379)   │
│  - Local LLMs   │      │  - Cache        │
│  - No API Keys  │      │  - Token Store  │
└─────────────────┘      └─────────────────┘
```

### Volume Mounts

```yaml
# Source code (read-only, for hot-reload)
../src → /app/src:ro

# Cargo cache (persistent, speeds up rebuilds)
cargo_cache → /usr/local/cargo/registry
target_cache → /app/target

# Data (persistent)
kg_data_dev → /data/kg
ollama_models → /root/.ollama
redis_data → /data
```

## Services

### Semantic Browser

**Container**: `semantic-browser-dev`  
**Port**: 3000  
**Features**: All enabled (`browser-automation`, `redis-integration`, `onnx-integration`)  

Environment variables from `docker/.env.dev`:
- `RUST_LOG=debug,semantic_browser=trace`
- `OLLAMA_API_BASE=http://ollama:11434`
- `REDIS_URL=redis://redis:6379`

### Ollama

**Container**: `semantic-browser-ollama-dev`  
**Port**: 11434  
**Purpose**: Local LLM inference  

Recommended models:
- `llama3.2` (8B) - Best balance
- `llama3.2:1b` - Fastest
- `mistral` - Alternative
- `codellama` - Code-optimized

### Redis

**Container**: `semantic-browser-redis-dev`  
**Port**: 6379  
**Purpose**: Caching and token revocation  

Configuration:
- Max memory: 256MB
- Eviction policy: allkeys-lru
- Persistence: AOF enabled

## Development Workflow

### 1. Initial Setup

```bash
# Start environment
./docker/scripts/docker-dev.sh up

# Pull Ollama model
./docker/scripts/docker-dev.sh ollama-pull llama3.2

# Verify services
./docker/scripts/docker-dev.sh status
```

### 2. Development Loop

```bash
# 1. Edit code in src/
vim src/llm/agent.rs

# 2. Container automatically rebuilds (cargo-watch)
# Watch logs: ./docker/scripts/docker-dev.sh logs semantic_browser

# 3. Test changes
curl http://localhost:3000/

# 4. Repeat
```

### 3. Testing

```bash
# Quick API test
./docker/scripts/docker-dev.sh test

# Run Rust tests in container
docker exec semantic-browser-dev cargo test --all-features

# Run specific example
docker exec semantic-browser-dev cargo run --example agent_with_browser --all-features
```

### 4. Debugging

```bash
# Open shell in container
./docker/scripts/docker-dev.sh shell

# View logs with filtering
docker-compose -f docker/docker-compose.dev.yml logs -f semantic_browser | grep ERROR

# Check Redis data
./docker/scripts/docker-dev.sh redis-cli
> KEYS *
> GET some_key
```

## Ollama Configuration

### Pull Models

```bash
# Recommended: llama3.2 (8B)
./docker/scripts/docker-dev.sh ollama-pull llama3.2

# Fast testing: llama3.2 (1B)
./docker/scripts/docker-dev.sh ollama-pull llama3.2:1b

# Code tasks: codellama
./docker/scripts/docker-dev.sh ollama-pull codellama
```

### List Models

```bash
./docker/scripts/docker-dev.sh ollama-list
```

### Interactive Chat

```bash
# Test model directly
./docker/scripts/docker-dev.sh ollama-run llama3.2
```

### Remove Models

```bash
./docker/scripts/docker-dev.sh ollama-rm llama3.2:1b
```

### Model Storage

Models are stored in the `ollama_models` volume and persist across restarts.

Location: `/root/.ollama` inside container

## Hot Reload

### How It Works

1. `cargo-watch` monitors `src/` and `Cargo.toml`
2. On file change, triggers `cargo build --all-features`
3. On successful build, runs `cargo run --all-features`
4. Service automatically restarts with new code

### Monitoring Rebuilds

```bash
# Watch rebuild output
./docker/scripts/docker-dev.sh logs semantic_browser
```

You'll see:
```
[Running 'cargo build --all-features']
   Compiling semantic_browser v0.1.3
    Finished dev [unoptimized + debuginfo] target(s) in 3.2s
[Running 'cargo run --all-features --bin semantic_browser_agent']
```

### Rebuild Time

- **First build**: 5-10 minutes (downloads dependencies)
- **Incremental**: 10-30 seconds (only changed files)
- **Cargo cache**: Persists in volume for speed

### Disabling Hot Reload

Edit `docker/Dockerfile.dev` CMD:

```dockerfile
# Instead of cargo-watch
CMD ["cargo", "run", "--all-features", "--bin", "semantic_browser_agent"]
```

## Debugging

### Enable Debug Logging

Already enabled in `docker/.env.dev`:

```bash
RUST_LOG=debug,semantic_browser=trace
RUST_BACKTRACE=full
```

### Attach Debugger

```bash
# Install gdb in container (already included)
./docker/scripts/docker-dev.sh shell

# Find process ID
ps aux | grep semantic_browser_agent

# Attach gdb
gdb -p <PID>
```

### View Backtraces

With `RUST_BACKTRACE=full`, panics show full stack traces in logs:

```bash
./docker/scripts/docker-dev.sh logs semantic_browser | grep -A 50 "panicked"
```

### Debug Prints

Add debug prints in code:

```rust
tracing::debug!("Debug message: {:?}", variable);
tracing::info!("Info message");
tracing::error!("Error: {}", error);
```

View in logs:

```bash
./docker/scripts/docker-dev.sh logs semantic_browser | grep "Debug message"
```

## Common Tasks

### Access Services

```bash
# Semantic Browser API
curl http://localhost:3000/

# Ollama API
curl http://localhost:11434/api/tags

# Redis CLI
./docker/scripts/docker-dev.sh redis-cli
```

### Rebuild Image

```bash
# Normal rebuild (uses cache)
./docker/scripts/docker-dev.sh build

# Force rebuild (no cache)
./docker/scripts/docker-dev.sh rebuild
```

### Clean Environment

```bash
# Remove containers and volumes (⚠️ deletes data)
./docker/scripts/docker-dev.sh clean

# Clean cargo cache only
./docker/scripts/docker-dev.sh clean-cache
```

### Check Health

```bash
# All services
./docker/scripts/docker-dev.sh health

# Detailed status
./docker/scripts/docker-dev.sh status
```

### View Container Processes

```bash
./docker/scripts/docker-dev.sh ps
```

### Restart Services

```bash
# Restart all
./docker/scripts/docker-dev.sh restart

# Restart specific service
docker-compose -f docker/docker-compose.dev.yml restart semantic_browser
```

## Troubleshooting

### Service Won't Start

**Check logs:**
```bash
./docker/scripts/docker-dev.sh logs semantic_browser
```

**Common issues:**
- Port 3000 already in use: `lsof -i :3000` and kill process
- Cargo build failing: Check Rust syntax errors in logs
- Out of memory: Increase Docker memory limit (8GB minimum)

### Ollama Model Not Found

**Pull the model:**
```bash
./docker/scripts/docker-dev.sh ollama-pull llama3.2
```

**Verify:**
```bash
./docker/scripts/docker-dev.sh ollama-list
```

### Hot Reload Not Working

**Check cargo-watch is running:**
```bash
docker exec semantic-browser-dev ps aux | grep cargo-watch
```

**Restart container:**
```bash
docker-compose -f docker/docker-compose.dev.yml restart semantic_browser
```

### Slow Builds

**Check cargo cache volumes:**
```bash
docker volume ls | grep semantic-browser-dev
```

**Should see:**
- `semantic-browser-dev_cargo_cache`
- `semantic-browser-dev_target_cache`

**If missing, rebuild:**
```bash
./docker/scripts/docker-dev.sh rebuild
```

### Redis Connection Failed

**Check Redis is running:**
```bash
docker exec semantic-browser-redis-dev redis-cli ping
# Should return: PONG
```

**Check network:**
```bash
docker network inspect semantic-browser-dev_semantic_dev_net
```

### Port Conflicts

**Change ports in `docker-compose.dev.yml`:**

```yaml
ports:
  - "3001:3000"  # Semantic Browser
  - "11435:11434"  # Ollama
  - "6380:6379"  # Redis
```

## Performance Tips

### 1. Allocate More Resources

Increase Docker Desktop resources:
- **Memory**: 16GB (for Ollama)
- **CPUs**: 4+ cores
- **Disk**: 50GB

### 2. Use Faster Models

For quick testing, use smaller models:

```bash
./docker/scripts/docker-dev.sh ollama-pull llama3.2:1b  # 1B params
```

### 3. Disable Features You Don't Need

Edit `docker-compose.dev.yml` build args:

```yaml
args:
  CARGO_FEATURES: "browser-automation"  # Only features you need
```

### 4. Use Pre-built Binary

For faster startup (no rebuild), change `Dockerfile.dev` CMD:

```dockerfile
CMD ["/app/target/debug/semantic_browser_agent"]
```

Then manually rebuild when code changes:

```bash
docker exec semantic-browser-dev cargo build --all-features
docker-compose -f docker/docker-compose.dev.yml restart semantic_browser
```

### 5. Persistent Cargo Cache

Cargo cache volumes persist between runs. Don't delete unless necessary:

```bash
# Keep these volumes!
docker volume ls | grep cargo_cache
docker volume ls | grep target_cache
```

## Integration with Makefile

Use Makefile shortcuts:

```bash
make docker-dev-up          # Start environment
make docker-dev-down        # Stop environment
make docker-dev-logs        # View logs
make docker-dev-shell       # Open shell
make docker-dev-test        # Run tests
```

See `Makefile` for all available targets.

## Next Steps

- **Run Examples**: See `examples/` directory
- **API Documentation**: See `docs/api/README.md`
- **LLM Integration**: See `docs/STREAMING_GUIDE.md`
- **Browser Automation**: See `docs/user-guide/browser-automation.md`

## Additional Resources

- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [Ollama Documentation](https://ollama.ai/docs)
- [Redis Documentation](https://redis.io/docs/)
- [Cargo Watch](https://crates.io/crates/cargo-watch)

---

**Need Help?**

- Check logs: `./docker/scripts/docker-dev.sh logs`
- Check status: `./docker/scripts/docker-dev.sh status`
- Open issue: [GitHub Issues](https://github.com/gianlucamazza/semanticbrowser/issues)
