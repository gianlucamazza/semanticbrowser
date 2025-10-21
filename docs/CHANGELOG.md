# Changelog

All notable changes to the Semantic Browser project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Updated Docker Compose files to use modern Compose Specification (v2+)
- Removed deprecated `version` attribute from docker-compose.yml and docker-compose.test.yml
- Added descriptive comments to Docker Compose files
- Updated all Dockerfile keywords to UPPERCASE for BuildKit compatibility

### Fixed
- Eliminated warning: "the attribute `version` is obsolete" in Docker Compose v2.40.0+
- Fixed Docker BuildKit error: "FromAsCasing: 'as' and 'FROM' keywords' casing do not match"
- Changed all `as` to `AS` in multi-stage builds (Dockerfile and Dockerfile.test)
- Resolved "failed to execute bake" error caused by BuildKit strict mode

## [0.1.0] - 2025-10-21

### Added
- Initial release of Semantic Browser
- HTML5 parsing with semantic extraction (microdata, JSON-LD)
- Knowledge Graph with SPARQL support (SELECT, INSERT, DELETE, CONSTRUCT, ASK, DESCRIBE)
- Named Entity Recognition framework with ML support
- REST API with authentication and rate limiting
- Docker environment with multi-stage builds
- Comprehensive test suite (unit, integration, benchmarks)
- CI/CD pipeline with GitHub Actions
- Complete documentation (README, DOCKER, TESTING, QUICKSTART)
- Example scripts for API usage
- Python integration via PyO3 (optional)
- Security features: input validation, sandboxing framework, rate limiting
- Persistent storage for Knowledge Graph
- Health checks and monitoring
- Automated build and test scripts

### Security
- Non-root Docker user execution
- Input validation for HTML and SPARQL
- Rate limiting (10 requests/min per IP)
- Bearer token authentication
- Sandboxing framework (seccomp on Linux)

### Performance
- Multi-stage Docker builds with layer caching
- Cargo dependency caching
- Optimized build process
- Resource limits configuration

[Unreleased]: https://github.com/yourusername/semanticbrowser/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/semanticbrowser/releases/tag/v0.1.0
