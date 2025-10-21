# Contributing to Semantic Browser

Thank you for your interest in contributing to the Semantic Browser project! This document provides guidelines and information for contributors.

## Development Setup

### Prerequisites

- Rust 1.75 or later
- Docker and Docker Compose (recommended)
- Python 3.x (optional, for external integrations)

### Quick Start

1. **Clone and setup**:
   ```bash
   git clone <repository-url>
   cd semanticbrowser
    cp config/.env.example .env
   ```

2. **Build and test**:
   ```bash
   make build
   make test
   ```

3. **Run locally**:
   ```bash
   make run
   ```

## Development Workflow

### 1. Choose an Issue

- Check [GitHub Issues](https://github.com/your-repo/issues) for open tasks
- Comment on the issue to indicate you're working on it
- Create a new branch: `git checkout -b feature/your-feature-name`

### 2. Code Changes

- Follow Rust coding standards
- Run `make lint` to check code quality
- Add tests for new functionality
- Update documentation as needed

### 3. Testing

```bash
# Run all tests
make test

# Run with Docker (recommended)
make docker-test

# Run benchmarks
make bench
```

### 4. Commit Guidelines

- Use clear, descriptive commit messages
- Follow conventional commits format when possible:
  - `feat:` for new features
  - `fix:` for bug fixes
  - `docs:` for documentation
  - `test:` for tests
  - `refactor:` for code refactoring

Example:
```
feat: add support for microdata extraction

- Extract microdata from HTML documents
- Add unit tests for microdata parsing
- Update documentation
```

### 5. Pull Request

- Push your branch: `git push origin feature/your-feature-name`
- Create a Pull Request on GitHub
- Ensure CI checks pass
- Request review from maintainers

## Code Standards

### Rust Guidelines

- Use `rustfmt` for formatting: `cargo fmt`
- Use `clippy` for linting: `cargo clippy`
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Write comprehensive documentation comments (`///`)

### Testing

- Write unit tests for all public functions
- Add integration tests for API endpoints
- Include benchmarks for performance-critical code
- Aim for >80% test coverage

### Documentation

- Update README.md for significant changes
- Add examples in `examples/` for new features
- Document public APIs with examples

## Project Structure

```
semanticbrowser/
├── src/                    # Source code
│   ├── lib.rs             # Library entry point
│   ├── main.rs            # Binary entry point
│   ├── parser.rs          # HTML parsing
│   ├── annotator.rs       # Entity annotation
│   ├── kg.rs              # Knowledge graph
│   ├── api.rs             # REST API
│   ├── security.rs        # Security utilities
│   └── external.rs        # External integrations
├── tests/                 # Integration tests
├── benches/               # Performance benchmarks
├── examples/              # Usage examples
├── scripts/               # Build/deployment scripts
├── docker/                # Docker configuration
└── docs/                  # Documentation
```

## Docker Development

For consistent development environment:

```bash
# Start development environment
make docker-up

# Run tests in Docker
make docker-test

# View logs
make docker-logs
```

## Security Considerations

- Validate all inputs (HTML, SPARQL queries)
- Follow secure coding practices
- Report security issues privately to maintainers

## Getting Help

- **Documentation**: Check `README.md`, `docs/guides/quickstart.md`, `docs/guides/docker-setup.md`, `docs/guides/testing.md`
- **Issues**: Use GitHub Issues for bugs and feature requests
- **Discussions**: Use GitHub Discussions for questions

## License

By contributing to this project, you agree that your contributions will be licensed under the same MIT License that covers the project.