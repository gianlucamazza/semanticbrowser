#!/bin/bash
# Docker Up Script - Start the Semantic Browser in Docker

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
MODE="foreground"
BUILD=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -d|--detach)
            MODE="background"
            shift
            ;;
        --build)
            BUILD=true
            shift
            ;;
        --logs)
            docker-compose -f docker/docker-compose.yml logs -f semantic_browser
            exit 0
            ;;
        --stop)
            print_info "Stopping Semantic Browser..."
            docker-compose -f docker/docker-compose.yml down
            print_success "Stopped"
            exit 0
            ;;
        --restart)
            print_info "Restarting Semantic Browser..."
            docker-compose -f docker/docker-compose.yml restart
            print_success "Restarted"
            exit 0
            ;;
        --status)
            docker-compose -f docker/docker-compose.yml ps
            exit 0
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -d, --detach    Run in background (detached mode)"
            echo "  --build         Build before starting"
            echo "  --logs          Show logs"
            echo "  --stop          Stop the service"
            echo "  --restart       Restart the service"
            echo "  --status        Show service status"
            echo "  --help          Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

print_header "Semantic Browser - Docker Startup"

# Create necessary directories
print_info "Creating data directories..."
mkdir -p data/kg models config

# Load environment variables if .env exists
if [ -f .env ]; then
    print_info "Loading environment variables from .env"
    export $(cat .env | grep -v '^#' | xargs)
else
    print_info "No .env file found, using defaults"
    print_info "Copy config/.env.example to .env to customize"
fi

# Build if requested
if [ "$BUILD" = true ]; then
    print_info "Building images..."
    docker-compose -f docker/docker-compose.yml build || {
        print_error "Build failed"
        exit 1
    }
    print_success "Build completed"
fi

# Start services
print_info "Starting Semantic Browser..."

if [ "$MODE" = "background" ]; then
    docker-compose -f docker/docker-compose.yml up -d || {
        print_error "Failed to start service"
        exit 1
    }

    # Wait for health check
    print_info "Waiting for service to be healthy..."
    timeout=60
    while [ $timeout -gt 0 ]; do
        if docker-compose -f docker/docker-compose.yml ps semantic_browser | grep -q "healthy"; then
            print_success "Service is healthy and running"
            break
        fi
        sleep 1
        ((timeout--))
        echo -n "."
    done
    echo ""

    if [ $timeout -eq 0 ]; then
        print_error "Service failed to become healthy"
        docker-compose -f docker/docker-compose.yml logs semantic_browser
        exit 1
    fi

    print_success "Semantic Browser started in background"
    echo ""
    print_info "API available at: http://localhost:3000"
    echo ""
    echo "Useful commands:"
    echo "  View logs:    $0 --logs"
    echo "  Stop:         $0 --stop"
    echo "  Restart:      $0 --restart"
    echo "  Status:       $0 --status"
    echo ""
    print_info "Try the examples:"
    echo "  ./docs/examples/parse_html.sh"
    echo "  ./docs/examples/query_kg.sh"
    echo "  ./docs/examples/browse_url.sh"

else
    print_info "Starting in foreground mode (Ctrl+C to stop)"
    echo ""
    docker-compose -f docker/docker-compose.yml up || {
        print_error "Failed to start service"
        exit 1
    }
fi
