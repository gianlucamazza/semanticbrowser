# Configuration Files

This directory contains configuration files and templates for the Semantic Browser project.

## Environment Configuration

The main configuration is handled through environment variables. Copy `config/.env.example` to `.env` and customize as needed:

```bash
cp config/.env.example .env
# Edit .env with your settings
nano .env
```

### Key Configuration Options

- **Logging**: `RUST_LOG` - Set logging level (trace, debug, info, warn, error)
- **Knowledge Graph**: `KG_PERSIST_PATH` - Path for persistent KG storage
- **ML Models**: `NER_MODEL_PATH`, `KG_INFERENCE_MODEL_PATH` - Paths to ONNX models
- **API Settings**: Server host/port, authentication secrets, rate limiting
- **Docker**: Resource limits and compose settings

See `.env.example` (in this directory) for the complete list of available configuration options with detailed comments.

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
