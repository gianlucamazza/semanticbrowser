#!/bin/bash
# Verify Dockerfile syntax and best practices

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_info() {
    echo -e "${YELLOW}ℹ $1${NC}"
}

echo "Verifying Dockerfile syntax..."
echo ""

# Check for lowercase 'as' in FROM statements
if grep -n "FROM.*as " docker/Dockerfile docker/Dockerfile.test 2>/dev/null; then
    print_error "Found lowercase 'as' in FROM statements"
    print_info "All keywords should be UPPERCASE for BuildKit compatibility"
    exit 1
else
    print_success "All FROM...AS statements use correct casing"
fi

# Check for uppercase AS
AS_COUNT=$(grep -c "FROM.*AS" docker/Dockerfile docker/Dockerfile.test 2>/dev/null || echo 0)
print_success "Found $AS_COUNT multi-stage build definitions"

# Verify Dockerfile syntax with buildx
if command -v docker &> /dev/null; then
    if docker buildx build --check -f docker/Dockerfile . 2>&1 | grep -qi "FromAsCasing"; then
        print_error "FromAsCasing warning detected"
        exit 1
    else
        print_success "Dockerfile syntax check passed (no casing warnings)"
    fi
fi

# Verify docker-compose syntax
if docker-compose -f docker/docker-compose.yml config > /dev/null 2>&1; then
    print_success "docker-compose.yml is valid"
else
    print_error "docker-compose.yml has syntax errors"
    exit 1
fi

if docker-compose -f docker/docker-compose.test.yml config > /dev/null 2>&1; then
    print_success "docker-compose.test.yml is valid"
else
    print_error "docker-compose.test.yml has syntax errors"
    exit 1
fi

echo ""
print_success "All syntax checks passed!"
echo ""
print_info "Ready to build with: ./docker/scripts/docker-build.sh"
print_info "Or start with: ./docker/scripts/docker-up.sh"
