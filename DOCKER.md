# Docker Environment - Semantic Browser

Complete Docker setup for development, testing, and production deployment.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      Docker Architecture                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                   │
│  Production/Development (docker-compose.yml)                     │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  semantic_browser                                         │  │
│  │  ├─ Rust Application (optimized build)                   │  │
│  │  ├─ Health Checks                                         │  │
│  │  ├─ Volume: /data/kg (persistent KG)                     │  │
│  │  └─ Network: semantic_net (isolated)                     │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                   │
│  Testing Environment (docker-compose.test.yml)                   │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  test_runner         lint_runner        benchmark        │  │
│  │  ├─ Unit Tests       ├─ cargo fmt       ├─ Performance  │  │
│  │  └─ Integration      └─ cargo clippy    └─ Benchmarks   │  │
│  │                                                            │  │
│  │  integration_test ◄──► test_server                       │  │
│  │  ├─ E2E Tests          ├─ Live API                       │  │
│  │  └─ API Testing        └─ Health Checks                  │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                   │
└─────────────────────────────────────────────────────────────────┘
```

## Files Structure

```
.
├── Dockerfile                  # Production/dev multi-stage build
├── Dockerfile.test             # Test environment (multi-target)
├── docker-compose.yml          # Production/dev orchestration
├── docker-compose.test.yml     # Test orchestration
├── .dockerignore               # Build optimization
├── .env.example                # Environment template
│
├── scripts/
│   ├── docker-build.sh         # Build automation
│   ├── docker-up.sh            # Startup automation
│   └── docker-test.sh          # Test automation
│
└── data/
    └── kg/                     # Persistent KG storage
```

## Dockerfile - Production Build

### Multi-Stage Build Strategy

**Stage 1: Dependencies (Cached)**
```dockerfile
FROM rust:1.75-slim as builder
# Build dependencies only (cached layer)
```

**Stage 2: Application Build**
```dockerfile
# Copy source and build app
# Only rebuilds when source changes
```

**Stage 3: Runtime**
```dockerfile
FROM debian:bookworm-slim
# Minimal runtime with:
# - Non-root user (security)
# - Health checks
# - Python for external tools
```

### Optimizations

1. **Layer Caching**: Dependencies built separately
2. **Security**: Non-root user (UID 1000)
3. **Size**: Multi-stage reduces final image size
4. **Health Checks**: Automatic monitoring
5. **Labels**: Metadata for tracking

### Best Practices

**Dockerfile Syntax**:
- ✅ All keywords UPPERCASE: `FROM`, `AS`, `RUN`, `COPY`, `ENV`
- ✅ Consistent casing improves readability
- ✅ Required for BuildKit strict mode compatibility

Example:
```dockerfile
# ✅ Correct
FROM rust:1.75-slim AS builder

# ❌ Incorrect (causes BuildKit warnings/errors)
FROM rust:1.75-slim as builder
```

## Dockerfile.test - Testing

### Multiple Build Targets

| Target | Purpose | Tools |
|--------|---------|-------|
| `test` | Unit & integration tests | cargo test |
| `lint` | Code quality | rustfmt, clippy |
| `bench` | Performance | criterion |
| `integration` | E2E testing | curl, jq |
| `coverage` | Code coverage | tarpaulin |

## Docker Compose Files

### Version Specification

**Note**: These files use the modern Compose Specification format (Docker Compose v2+) and do not require a `version` attribute. The format is automatically detected.

## Docker Compose - Production

### Services

**semantic_browser**
- Main API server
- Port: 3000
- Health checks enabled
- Persistent volumes
- Resource limits
- Environment configuration

### Volumes

- `kg_data`: Knowledge Graph persistence
- `models`: ML models (read-only)
- `config`: Configuration files (read-only)

### Networks

- `semantic_net`: Isolated bridge network

## Docker Compose - Testing

### Test Services

1. **test_runner**: Runs cargo test
2. **lint_runner**: Code quality checks
3. **integration_test**: E2E tests with live server
4. **test_server**: Test API instance
5. **benchmark**: Performance tests

### Service Dependencies

```
integration_test → test_server (waits for healthy)
```

### Shared Volumes

- Cargo cache (faster builds)
- Target cache (build artifacts)
- Test results
- Benchmark results

## Scripts

### docker-build.sh

Optimized build with options:

```bash
./scripts/docker-build.sh              # Production build
./scripts/docker-build.sh --dev        # Development build
./scripts/docker-build.sh --test       # Test images
./scripts/docker-build.sh --no-cache   # Force rebuild
./scripts/docker-build.sh --platform linux/amd64  # Cross-platform
```

Features:
- Build type selection
- Cache control
- Platform targeting
- Security scanning (trivy)
- Size reporting

### docker-up.sh

Service management:

```bash
./scripts/docker-up.sh -d              # Start in background
./scripts/docker-up.sh --build -d      # Build & start
./scripts/docker-up.sh --logs          # View logs
./scripts/docker-up.sh --stop          # Stop services
./scripts/docker-up.sh --restart       # Restart
./scripts/docker-up.sh --status        # Status check
```

Features:
- Automatic directory creation
- Environment loading
- Health check waiting
- Status monitoring

### docker-test.sh

Comprehensive testing:

```bash
./scripts/docker-test.sh                    # All tests
./scripts/docker-test.sh --unit-only        # Unit tests only
./scripts/docker-test.sh --integration-only # Integration only
./scripts/docker-test.sh --lint-only        # Linting only
./scripts/docker-test.sh --with-bench       # Include benchmarks
./scripts/docker-test.sh --clean            # Cleanup
```

Features:
- Colored output
- Service orchestration
- Health check waiting
- Automatic cleanup
- Test summary

## Environment Variables

### Required

None (all have defaults)

### Optional

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Log level | `info` |
| `KG_PERSIST_PATH` | KG storage path | In-memory |
| `NER_MODEL_PATH` | NER model | Regex fallback |
| `KG_INFERENCE_MODEL_PATH` | KG inference | Rule-based |

See `.env.example` for complete list.

## Best Practices Implemented

### Security

- ✅ Non-root user execution
- ✅ Minimal base image (Debian slim)
- ✅ No secrets in images
- ✅ Read-only volumes where applicable
- ✅ Network isolation
- ✅ Health checks

### Performance

- ✅ Multi-stage builds
- ✅ Layer caching optimization
- ✅ Cargo dependency caching
- ✅ Shared volumes for builds
- ✅ Resource limits

### Reliability

- ✅ Health checks with retries
- ✅ Graceful shutdown
- ✅ Restart policies
- ✅ Service dependencies
- ✅ Error handling in scripts

### Development

- ✅ Hot reload (volume mounts)
- ✅ Separate test environment
- ✅ Build caching
- ✅ Easy debugging
- ✅ Automated scripts

## Workflows

### Development Workflow

1. Copy environment:
   ```bash
   cp .env.example .env
   ```

2. Start services:
   ```bash
   ./scripts/docker-up.sh -d
   ```

3. Watch logs:
   ```bash
   docker-compose logs -f
   ```

4. Make changes and test:
   ```bash
   ./scripts/docker-test.sh --unit-only
   ```

### CI/CD Workflow

1. Build test image:
   ```bash
   docker-compose -f docker-compose.test.yml build
   ```

2. Run linting:
   ```bash
   docker-compose -f docker-compose.test.yml run --rm lint_runner
   ```

3. Run tests:
   ```bash
   docker-compose -f docker-compose.test.yml run --rm test_runner
   ```

4. Run integration tests:
   ```bash
   docker-compose -f docker-compose.test.yml up --abort-on-container-exit
   ```

5. Build production:
   ```bash
   docker build -t semantic-browser:latest .
   ```

### Production Deployment

1. Configure environment:
   ```bash
   cp .env.example .env
   # Edit .env with production values
   ```

2. Build production image:
   ```bash
   ./scripts/docker-build.sh
   ```

3. Start services:
   ```bash
   docker-compose up -d
   ```

4. Verify health:
   ```bash
   docker-compose ps
   curl http://localhost:3000/
   ```

5. Monitor:
   ```bash
   docker-compose logs -f --tail=100
   ```

## Troubleshooting

### Build Issues

**Problem**: Build fails with dependency errors

**Solution**:
```bash
./scripts/docker-build.sh --no-cache
```

### Runtime Issues

**Problem**: Service won't start

**Solution**:
```bash
docker-compose logs semantic_browser
docker-compose restart semantic_browser
```

**Problem**: Health check fails

**Solution**:
```bash
# Check if port is in use
lsof -i :3000

# Check container logs
docker logs semantic-browser

# Restart with debug logging
RUST_LOG=debug docker-compose up
```

### Test Issues

**Problem**: Integration tests timeout

**Solution**:
```bash
# Increase timeout in docker-compose.test.yml
# Check test server logs
docker-compose -f docker-compose.test.yml logs test_server
```

## Performance Tuning

### Build Performance

1. Enable BuildKit:
   ```bash
   export DOCKER_BUILDKIT=1
   ```

2. Use cache mount (Docker 18.09+):
   ```dockerfile
   RUN --mount=type=cache,target=/usr/local/cargo/registry \
       cargo build --release
   ```

### Runtime Performance

1. Adjust resource limits in docker-compose.yml:
   ```yaml
   deploy:
     resources:
       limits:
         cpus: '4'
         memory: 4G
   ```

2. Use volume for better I/O:
   ```yaml
   volumes:
     - type: volume
       source: kg_data
       target: /data/kg
   ```

## Security Considerations

1. **Never commit .env files**
2. **Change default secrets in production**
3. **Use Docker secrets for sensitive data**
4. **Regularly update base images**
5. **Scan images for vulnerabilities**:
   ```bash
   trivy image semantic-browser:latest
   ```

## Additional Resources

- [Docker Best Practices](https://docs.docker.com/develop/dev-best-practices/)
- [Rust Docker Optimization](https://docs.docker.com/language/rust/)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
