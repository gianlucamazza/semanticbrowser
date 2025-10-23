# Makefile for Semantic Browser Development

.PHONY: help build test run clean lint fmt check docker-build docker-up docker-down docker-test docker-logs bench docs install-deps

# Default target
help: ## Show this help message
	@echo "Semantic Browser Development Commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

# Development
build: ## Build the project in debug mode
	cargo build

build-release: ## Build the project in release mode
	cargo build --release

run: ## Run the application locally
	cargo run

run-release: ## Run the application in release mode
	cargo run --release

# Testing
test: ## Run all tests
	cargo test

test-unit: ## Run only unit tests
	cargo test --lib

test-integration: ## Run only integration tests
	cargo test --test integration_test

bench: ## Run benchmarks
	cargo bench

# Code Quality
lint: ## Run clippy linter
	cargo clippy --all-targets --all-features -- -D warnings

fmt: ## Format code with rustfmt
	cargo fmt

fmt-check: ## Check code formatting
	cargo fmt --all -- --check

check: ## Run cargo check
	cargo check

# Docker Production
docker-build: ## Build Docker image
	./docker/scripts/docker-build.sh

docker-up: ## Start Docker containers
	./docker/scripts/docker-up.sh -d

docker-down: ## Stop Docker containers
	./docker/scripts/docker-up.sh --stop

docker-logs: ## Show Docker logs
	./docker/scripts/docker-up.sh --logs

docker-test: ## Run tests in Docker
	./docker/scripts/docker-test.sh

docker-test-unit: ## Run unit tests in Docker
	./docker/scripts/docker-test.sh --unit-only

docker-test-integration: ## Run integration tests in Docker
	./docker/scripts/docker-test.sh --integration-only

# Docker Development Environment (with Ollama, Redis, Hot-Reload)
docker-dev-up: ## Start development environment with Ollama and Redis
	./docker/scripts/docker-dev.sh up

docker-dev-down: ## Stop development environment
	./docker/scripts/docker-dev.sh down

docker-dev-restart: ## Restart development environment
	./docker/scripts/docker-dev.sh restart

docker-dev-logs: ## Show development environment logs
	./docker/scripts/docker-dev.sh logs

docker-dev-logs-app: ## Show application logs only
	./docker/scripts/docker-dev.sh logs semantic_browser

docker-dev-status: ## Show status of all development services
	./docker/scripts/docker-dev.sh status

docker-dev-build: ## Rebuild development image
	./docker/scripts/docker-dev.sh build

docker-dev-rebuild: ## Force rebuild development image (no cache)
	./docker/scripts/docker-dev.sh rebuild

docker-dev-shell: ## Open shell in development container
	./docker/scripts/docker-dev.sh shell

docker-dev-redis: ## Open Redis CLI in development
	./docker/scripts/docker-dev.sh redis-cli

docker-dev-test: ## Run quick API tests in development
	./docker/scripts/docker-dev.sh test

docker-dev-health: ## Check health of all development services
	./docker/scripts/docker-dev.sh health

docker-dev-clean: ## Clean development environment (removes data)
	./docker/scripts/docker-dev.sh clean

docker-dev-clean-cache: ## Clean cargo cache in development
	./docker/scripts/docker-dev.sh clean-cache

# Ollama Management
ollama-pull: ## Pull Ollama model (usage: make ollama-pull MODEL=llama3.2)
	./docker/scripts/docker-dev.sh ollama-pull $(MODEL)

ollama-list: ## List available Ollama models
	./docker/scripts/docker-dev.sh ollama-list

ollama-run: ## Run interactive Ollama chat (usage: make ollama-run MODEL=llama3.2)
	./docker/scripts/docker-dev.sh ollama-run $(MODEL)

ollama-rm: ## Remove Ollama model (usage: make ollama-rm MODEL=llama3.2)
	./docker/scripts/docker-dev.sh ollama-rm $(MODEL)

# Quick shortcuts for common development tasks
dev: docker-dev-up ## Alias for docker-dev-up
dev-logs: docker-dev-logs ## Alias for docker-dev-logs
dev-shell: docker-dev-shell ## Alias for docker-dev-shell

# Documentation
docs: ## Generate documentation
	cargo doc --open

# Cleanup
clean: ## Clean build artifacts
	cargo clean

clean-docker: ## Clean Docker containers and images
	docker-compose -f docker/docker-compose.yml down --rmi all --volumes --remove-orphans
	docker-compose -f docker/docker-compose.test.yml down --rmi all --volumes --remove-orphans

clean-all: clean clean-docker ## Clean everything

# Dependencies
install-deps: ## Install development dependencies
	cargo install cargo-tarpaulin
	cargo install cargo-audit

audit: ## Run security audit
	cargo audit

# CI Simulation
ci: fmt-check lint test ## Run CI checks locally

# Examples
examples: ## Make example scripts executable
	chmod +x docs/user-guide/examples/*.sh

# Setup
setup: examples ## Initial project setup
	@echo "Project setup complete!"
	@echo "Run 'make build' to build the project"
	@echo "Run 'make test' to run tests"
	@echo "Run 'make docker-up' to start with Docker"
