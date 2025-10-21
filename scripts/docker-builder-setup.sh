#!/bin/bash
# Docker Build Cloud Setup Script
# This script sets up Docker Build Cloud for multi-architecture builds

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

print_header "Docker Build Cloud Setup"

# Configuration
DOCKER_ORG="${DOCKER_ORG:-homen3}"
BUILDER_NAME="${BUILDER_NAME:-cloud-homen3-linux}"
CLOUD_BUILDER="${DOCKER_ORG}/linux"

echo ""
print_info "This script will set up Docker Build Cloud for multi-architecture builds"
print_info "Organization: $DOCKER_ORG"
print_info "Cloud Builder: $CLOUD_BUILDER"
print_info "Local Builder Name: $BUILDER_NAME"
echo ""

# Step 1: Check Docker Desktop version
print_info "Checking Docker Desktop version..."
DOCKER_VERSION=$(docker version --format '{{.Client.Version}}' 2>/dev/null || echo "unknown")
print_success "Docker version: $DOCKER_VERSION"

# Step 2: Check Docker login
print_info "Step 1: Checking Docker Hub Authentication"
echo ""

if docker info 2>/dev/null | grep -q "Username:"; then
    CURRENT_USER=$(docker info 2>/dev/null | grep "Username:" | awk '{print $2}')
    print_success "Logged in as: $CURRENT_USER"
else
    print_info "Docker CLI not authenticated"
    print_info "Checking Docker Desktop authentication..."

    # Try to use Docker Desktop credentials
    if [ -f ~/.docker/config.json ]; then
        print_info "Found Docker config, will use credential helper"
    fi

    print_info "If this fails, please login first:"
    print_info "  docker login"
    print_info ""
fi
echo ""

# Step 3: Remove old cloud builder if it exists
print_info "Step 2: Removing old cloud builder (if exists)..."

if docker buildx ls | grep -q "$BUILDER_NAME"; then
    print_info "Found existing builder: $BUILDER_NAME"
    docker buildx rm $BUILDER_NAME || true
    print_success "Old builder removed"
else
    print_info "No existing builder found"
fi
echo ""

# Step 4: Create new cloud builder
print_info "Step 3: Creating Docker Build Cloud builder..."
print_info "Cloud endpoint: cloud://$CLOUD_BUILDER"

docker buildx create \
    --driver cloud \
    $CLOUD_BUILDER \
    --name $BUILDER_NAME || {
    print_error "Failed to create cloud builder"
    print_info "Make sure you have access to Docker Build Cloud"
    print_info "Visit: https://app.docker.com/build/"
    exit 1
}

print_success "Cloud builder created: $BUILDER_NAME"
echo ""

# Step 5: Set as default builder
print_info "Step 4: Setting cloud builder as default..."
docker buildx use $BUILDER_NAME
print_success "Cloud builder is now the default"
echo ""

# Step 6: Bootstrap and verify
print_info "Step 5: Bootstrapping and verifying builder..."
docker buildx inspect --bootstrap || {
    print_error "Failed to bootstrap builder"
    exit 1
}
print_success "Builder bootstrapped successfully"
echo ""

# Step 7: List all builders
print_header "Builder Configuration"
echo ""
docker buildx ls
echo ""

# Summary
print_header "Setup Complete"
echo ""
print_success "Docker Build Cloud is configured and ready to use"
echo ""
print_info "Current default builder: $BUILDER_NAME"
print_info "Cloud endpoint: cloud://$CLOUD_BUILDER"
print_info "Supported platforms: linux/amd64, linux/arm64"
echo ""
print_info "You can now build multi-architecture images with:"
echo "  ./scripts/docker-build.sh"
echo "  ./scripts/docker-up.sh --build -d"
echo ""
print_info "To switch back to local builder:"
echo "  docker buildx use desktop-linux"
echo ""
print_info "To view builder details:"
echo "  docker buildx inspect $BUILDER_NAME"
echo ""
