#!/bin/bash
# Docker Test Runner - Executes all tests in Docker containers

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
print_header() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

# Parse arguments
RUN_UNIT=true
RUN_INTEGRATION=true
RUN_LINT=true
RUN_BENCH=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --unit-only)
            RUN_INTEGRATION=false
            RUN_LINT=false
            shift
            ;;
        --integration-only)
            RUN_UNIT=false
            RUN_LINT=false
            shift
            ;;
        --lint-only)
            RUN_UNIT=false
            RUN_INTEGRATION=false
            shift
            ;;
        --with-bench)
            RUN_BENCH=true
            shift
            ;;
        --clean)
            print_info "Cleaning up Docker resources..."
            docker-compose -f docker-compose.test.yml down -v
            print_success "Cleanup complete"
            exit 0
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --unit-only         Run only unit tests"
            echo "  --integration-only  Run only integration tests"
            echo "  --lint-only         Run only linting checks"
            echo "  --with-bench        Include benchmarks"
            echo "  --clean             Clean up Docker resources"
            echo "  --help              Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Main test execution
print_header "Semantic Browser - Docker Test Suite"

# Build test images
print_info "Building test images..."
docker-compose -f docker-compose.test.yml build || {
    print_error "Failed to build test images"
    exit 1
}
print_success "Test images built"

# Run linting
if [ "$RUN_LINT" = true ]; then
    print_header "Running Code Quality Checks"
    docker-compose -f docker-compose.test.yml run --rm lint_runner || {
        print_error "Linting failed"
        exit 1
    }
    print_success "Code quality checks passed"
fi

# Run unit tests
if [ "$RUN_UNIT" = true ]; then
    print_header "Running Unit Tests"
    docker-compose -f docker-compose.test.yml run --rm test_runner || {
        print_error "Unit tests failed"
        exit 1
    }
    print_success "Unit tests passed"
fi

# Run integration tests
if [ "$RUN_INTEGRATION" = true ]; then
    print_header "Running Integration Tests"

    # Start test server
    print_info "Starting test server..."
    docker-compose -f docker-compose.test.yml up -d test_server

    # Wait for server to be healthy
    print_info "Waiting for test server to be ready..."
    timeout=30
    while [ $timeout -gt 0 ]; do
        if docker-compose -f docker-compose.test.yml ps test_server | grep -q "healthy"; then
            print_success "Test server is ready"
            break
        fi
        sleep 1
        ((timeout--))
    done

    if [ $timeout -eq 0 ]; then
        print_error "Test server failed to start"
        docker-compose -f docker-compose.test.yml logs test_server
        docker-compose -f docker-compose.test.yml down
        exit 1
    fi

    # Run integration tests
    docker-compose -f docker-compose.test.yml run --rm integration_test || {
        print_error "Integration tests failed"
        docker-compose -f docker-compose.test.yml logs test_server
        docker-compose -f docker-compose.test.yml down
        exit 1
    }
    print_success "Integration tests passed"

    # Cleanup
    docker-compose -f docker-compose.test.yml down
fi

# Run benchmarks (optional)
if [ "$RUN_BENCH" = true ]; then
    print_header "Running Benchmarks"
    docker-compose -f docker-compose.test.yml run --rm benchmark || {
        print_error "Benchmarks failed"
        exit 1
    }
    print_success "Benchmarks completed"
fi

# Final summary
print_header "Test Summary"
echo ""
if [ "$RUN_LINT" = true ]; then
    print_success "Linting: PASSED"
fi
if [ "$RUN_UNIT" = true ]; then
    print_success "Unit Tests: PASSED"
fi
if [ "$RUN_INTEGRATION" = true ]; then
    print_success "Integration Tests: PASSED"
fi
if [ "$RUN_BENCH" = true ]; then
    print_success "Benchmarks: COMPLETED"
fi
echo ""
print_success "All tests completed successfully!"
