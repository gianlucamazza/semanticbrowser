#!/bin/bash
# Docker Build Script - Optimized build with caching

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

print_header() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Parse arguments
BUILD_TYPE="production"
NO_CACHE=false
PLATFORM=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --dev)
            BUILD_TYPE="development"
            shift
            ;;
        --test)
            BUILD_TYPE="test"
            shift
            ;;
        --no-cache)
            NO_CACHE=true
            shift
            ;;
        --platform)
            PLATFORM="--platform $2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --dev         Build development image"
            echo "  --test        Build test images"
            echo "  --no-cache    Build without using cache"
            echo "  --platform    Specify platform (e.g., linux/amd64)"
            echo "  --help        Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Prepare build args
CACHE_ARG=""
if [ "$NO_CACHE" = true ]; then
    CACHE_ARG="--no-cache"
fi

print_header "Semantic Browser - Docker Build"

# Build based on type
case $BUILD_TYPE in
    production)
        print_info "Building production image..."
        docker build \
            $CACHE_ARG \
            $PLATFORM \
            -t semantic-browser:latest \
            -f Dockerfile \
            . || {
            print_error "Production build failed"
            exit 1
        }
        print_success "Production image built: semantic-browser:latest"

        # Show image size
        IMAGE_SIZE=$(docker images semantic-browser:latest --format "{{.Size}}")
        print_info "Image size: $IMAGE_SIZE"
        ;;

    development)
        print_info "Building development image..."
        docker-compose build || {
            print_error "Development build failed"
            exit 1
        }
        print_success "Development environment built"
        ;;

    test)
        print_info "Building test images..."
        docker-compose -f docker-compose.test.yml build $CACHE_ARG || {
            print_error "Test images build failed"
            exit 1
        }
        print_success "Test images built"
        ;;
esac

# Optional: Run security scan
print_info "Running security scan (optional)..."
if command -v trivy &> /dev/null; then
    trivy image semantic-browser:latest || true
else
    print_info "Trivy not installed, skipping security scan"
    print_info "Install with: brew install trivy"
fi

# Show build summary
print_header "Build Summary"
echo ""
print_success "Build completed successfully"
echo ""
echo "Available images:"
docker images | grep semantic-browser || true
echo ""
print_info "To run: ./scripts/docker-up.sh"
print_info "To test: ./scripts/docker-test.sh"
