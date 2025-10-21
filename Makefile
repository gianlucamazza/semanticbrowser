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

# Docker
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
	chmod +x docs/examples/*.sh

# Setup
setup: examples ## Initial project setup
	@echo "Project setup complete!"
	@echo "Run 'make build' to build the project"
	@echo "Run 'make test' to run tests"
	@echo "Run 'make docker-up' to start with Docker"