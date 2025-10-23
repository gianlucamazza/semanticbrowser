#!/bin/bash
# Docker Development Environment Manager for Semantic Browser
# Provides convenient commands for managing the development Docker stack

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
COMPOSE_FILE="docker/docker-compose.dev.yml"
ENV_FILE="docker/.env.dev"
PROJECT_NAME="semantic-browser-dev"

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

print_cyan() {
    echo -e "${CYAN}$1${NC}"
}

show_help() {
    cat << EOF
Semantic Browser Development Environment Manager

Usage: $0 <command> [options]

Commands:
  up              Start the development environment
  down            Stop and remove containers
  restart         Restart all services
  stop            Stop services without removing containers
  start           Start stopped services

  logs [service]  Show logs (optionally for specific service)
  ps              Show running containers
  status          Show detailed status of all services

  build           Rebuild the semantic browser image
  rebuild         Force rebuild (no cache) and restart

  shell           Open bash shell in semantic browser container
  redis-cli       Open Redis CLI

  ollama-pull     Pull an Ollama model (e.g., llama3.2, mistral)
  ollama-list     List available Ollama models
  ollama-run      Run interactive chat with Ollama model
  ollama-rm       Remove an Ollama model

  health          Check health of all services
  test            Run a quick API test

  clean           Stop services and remove volumes (⚠️  deletes data)
  clean-cache     Clean cargo cache and target directory

  help            Show this help message

Examples:
  $0 up                    # Start all services
  $0 logs semantic_browser # Show app logs
  $0 ollama-pull llama3.2  # Download llama3.2 model
  $0 shell                 # Interactive shell
  $0 rebuild               # Full rebuild

Environment:
  Compose file: $COMPOSE_FILE
  Env file:     $ENV_FILE
  Project:      $PROJECT_NAME

EOF
}

# Ensure we're in the project root
cd "$(dirname "$0")/../.." || exit 1

case "${1:-help}" in
    up)
        print_header "Starting Development Environment"

        # Create necessary directories
        print_info "Creating data directories..."
        mkdir -p data/kg-dev models config

        # Check if .env.dev exists, create from template if not
        if [ ! -f "$ENV_FILE" ]; then
            print_info "Creating $ENV_FILE from template..."
            cp docker/.env.dev.template docker/.env.dev 2>/dev/null || true
        fi

        # Start services
        print_info "Starting services with docker-compose..."
        docker-compose -f "$COMPOSE_FILE" --env-file "$ENV_FILE" up -d

        # Wait for services to be healthy
        print_info "Waiting for services to be healthy..."
        sleep 5

        # Check Ollama
        if docker-compose -f "$COMPOSE_FILE" ps ollama | grep -q "Up"; then
            print_success "Ollama is running at http://localhost:11434"
        fi

        # Check Redis
        if docker-compose -f "$COMPOSE_FILE" ps redis | grep -q "Up"; then
            print_success "Redis is running at localhost:6379"
        fi

        # Check Semantic Browser
        print_info "Waiting for Semantic Browser to be ready..."
        timeout=60
        while [ $timeout -gt 0 ]; do
            if curl -sf http://localhost:3000/ > /dev/null 2>&1; then
                print_success "Semantic Browser is ready at http://localhost:3000"
                break
            fi
            sleep 2
            ((timeout-=2))
        done

        if [ $timeout -eq 0 ]; then
            print_error "Semantic Browser failed to start"
            print_info "Check logs with: $0 logs semantic_browser"
        fi

        echo ""
        print_header "Development Environment Ready"
        print_cyan "API:    http://localhost:3000"
        print_cyan "Ollama: http://localhost:11434"
        print_cyan "Redis:  localhost:6379"
        echo ""
        print_info "Next steps:"
        echo "  1. Pull an Ollama model: $0 ollama-pull llama3.2"
        echo "  2. View logs:            $0 logs"
        echo "  3. Run tests:            $0 test"
        echo "  4. Open shell:           $0 shell"
        ;;

    down)
        print_info "Stopping development environment..."
        docker-compose -f "$COMPOSE_FILE" down
        print_success "Stopped"
        ;;

    restart)
        print_info "Restarting services..."
        docker-compose -f "$COMPOSE_FILE" restart
        print_success "Restarted"
        ;;

    stop)
        print_info "Stopping services..."
        docker-compose -f "$COMPOSE_FILE" stop
        print_success "Stopped"
        ;;

    start)
        print_info "Starting services..."
        docker-compose -f "$COMPOSE_FILE" start
        print_success "Started"
        ;;

    logs)
        if [ -n "$2" ]; then
            docker-compose -f "$COMPOSE_FILE" logs -f "$2"
        else
            docker-compose -f "$COMPOSE_FILE" logs -f
        fi
        ;;

    ps)
        docker-compose -f "$COMPOSE_FILE" ps
        ;;

    status)
        print_header "Service Status"
        docker-compose -f "$COMPOSE_FILE" ps
        echo ""

        print_header "Service Health"

        # Check Ollama
        echo -n "Ollama:           "
        if docker exec semantic-browser-ollama-dev ollama list > /dev/null 2>&1; then
            print_success "Healthy"
        else
            print_error "Unhealthy"
        fi

        # Check Redis
        echo -n "Redis:            "
        if docker exec semantic-browser-redis-dev redis-cli ping > /dev/null 2>&1; then
            print_success "Healthy"
        else
            print_error "Unhealthy"
        fi

        # Check Semantic Browser
        echo -n "Semantic Browser: "
        if curl -sf http://localhost:3000/ > /dev/null 2>&1; then
            print_success "Healthy"
        else
            print_error "Unhealthy"
        fi
        ;;

    build)
        print_info "Building semantic browser image..."
        docker-compose -f "$COMPOSE_FILE" build semantic_browser
        print_success "Build completed"
        ;;

    rebuild)
        print_info "Force rebuilding semantic browser (no cache)..."
        docker-compose -f "$COMPOSE_FILE" build --no-cache semantic_browser
        print_info "Restarting service..."
        docker-compose -f "$COMPOSE_FILE" up -d semantic_browser
        print_success "Rebuild and restart completed"
        ;;

    shell)
        print_info "Opening shell in semantic browser container..."
        docker exec -it semantic-browser-dev bash
        ;;

    redis-cli)
        print_info "Opening Redis CLI..."
        docker exec -it semantic-browser-redis-dev redis-cli
        ;;

    ollama-pull)
        if [ -z "$2" ]; then
            print_error "Please specify a model name"
            echo "Example: $0 ollama-pull llama3.2"
            echo ""
            echo "Popular models:"
            echo "  - llama3.2       (8B, latest, recommended)"
            echo "  - llama3.2:1b    (1B, fastest)"
            echo "  - mistral        (7B, alternative)"
            echo "  - codellama      (code-optimized)"
            echo "  - phi3           (3.8B, efficient)"
            exit 1
        fi
        print_info "Pulling Ollama model: $2"
        docker exec semantic-browser-ollama-dev ollama pull "$2"
        print_success "Model $2 pulled successfully"
        ;;

    ollama-list)
        print_header "Available Ollama Models"
        docker exec semantic-browser-ollama-dev ollama list
        ;;

    ollama-run)
        if [ -z "$2" ]; then
            print_error "Please specify a model name"
            echo "Example: $0 ollama-run llama3.2"
            exit 1
        fi
        print_info "Starting interactive chat with $2"
        docker exec -it semantic-browser-ollama-dev ollama run "$2"
        ;;

    ollama-rm)
        if [ -z "$2" ]; then
            print_error "Please specify a model name"
            echo "Example: $0 ollama-rm llama3.2"
            exit 1
        fi
        print_info "Removing Ollama model: $2"
        docker exec semantic-browser-ollama-dev ollama rm "$2"
        print_success "Model $2 removed"
        ;;

    health)
        print_header "Health Checks"

        echo "Ollama:"
        curl -sf http://localhost:11434/ || echo "  Ollama not responding"
        echo ""

        echo "Redis:"
        docker exec semantic-browser-redis-dev redis-cli ping || echo "  Redis not responding"
        echo ""

        echo "Semantic Browser:"
        curl -sf http://localhost:3000/ || echo "  API not responding"
        echo ""
        ;;

    test)
        print_header "Quick API Test"

        print_info "Testing root endpoint..."
        curl -sf http://localhost:3000/ && print_success "Root endpoint OK" || print_error "Root endpoint failed"

        print_info "Testing health endpoint..."
        curl -sf http://localhost:3000/health && print_success "Health endpoint OK" || print_error "Health endpoint failed"

        print_info "Testing Ollama connection..."
        curl -sf http://localhost:11434/ && print_success "Ollama OK" || print_error "Ollama failed"

        print_info "Testing Redis connection..."
        docker exec semantic-browser-redis-dev redis-cli ping > /dev/null && print_success "Redis OK" || print_error "Redis failed"
        ;;

    clean)
        print_error "⚠️  WARNING: This will delete all data (KG, models, cache)"
        read -p "Are you sure? (yes/no): " -r
        if [[ $REPLY == "yes" ]]; then
            print_info "Stopping and removing containers and volumes..."
            docker-compose -f "$COMPOSE_FILE" down -v
            print_info "Removing local data directories..."
            rm -rf data/kg-dev
            print_success "Cleaned"
        else
            print_info "Cancelled"
        fi
        ;;

    clean-cache)
        print_info "Cleaning cargo cache and target directory..."
        docker-compose -f "$COMPOSE_FILE" down semantic_browser
        docker volume rm ${PROJECT_NAME}_cargo_cache ${PROJECT_NAME}_cargo_git_cache ${PROJECT_NAME}_target_cache 2>/dev/null || true
        print_success "Cache cleaned"
        print_info "Next build will download dependencies again"
        ;;

    help|--help|-h)
        show_help
        ;;

    *)
        print_error "Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac
