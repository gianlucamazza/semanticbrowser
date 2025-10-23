#!/bin/bash
# Test Ollama Integration in Docker Dev Environment
set -e

echo "🧪 Testing Semantic Browser Docker Dev Environment"
echo "=================================================="
echo ""

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}1. Testing Semantic Browser API...${NC}"
if curl -s http://localhost:3000/ > /dev/null 2>&1; then
    echo -e "${GREEN}✓ API is responding${NC}"
else
    echo -e "${RED}✗ API not responding${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}2. Testing Redis connection...${NC}"
if docker exec semantic-browser-redis-dev redis-cli ping > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Redis is healthy${NC}"
else
    echo -e "${RED}✗ Redis not responding${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}3. Testing Ollama connection from container...${NC}"
if docker exec semantic-browser-dev curl -s http://host.docker.internal:11434/api/tags | grep -q "llama3"; then
    echo -e "${GREEN}✓ Ollama is accessible from container${NC}"
    echo -e "${YELLOW}  Available models:${NC}"
    docker exec semantic-browser-dev curl -s http://host.docker.internal:11434/api/tags | \
        jq -r '.models[].name' | head -5 | sed 's/^/    - /'
else
    echo -e "${RED}✗ Ollama not accessible${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}4. Testing Ollama LLM inference...${NC}"
RESPONSE=$(docker exec semantic-browser-dev curl -s http://host.docker.internal:11434/api/generate \
    -d '{
        "model": "llama3.2",
        "prompt": "What is Docker? Answer in one sentence.",
        "stream": false
    }' 2>/dev/null)

if echo "$RESPONSE" | jq -e '.response' > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Ollama LLM inference working${NC}"
    echo -e "${YELLOW}  Response:${NC}"
    echo "$RESPONSE" | jq -r '.response' | sed 's/^/    /'
else
    echo -e "${RED}✗ Ollama inference failed${NC}"
    echo "$RESPONSE"
    exit 1
fi

echo ""
echo -e "${BLUE}5. Checking container logs for errors...${NC}"
if docker logs semantic-browser-dev 2>&1 | grep -i "error\|panic" | grep -v "no error" > /dev/null; then
    echo -e "${YELLOW}⚠ Found errors in logs:${NC}"
    docker logs semantic-browser-dev 2>&1 | grep -i "error\|panic" | grep -v "no error" | tail -5 | sed 's/^/    /'
else
    echo -e "${GREEN}✓ No errors in logs${NC}"
fi

echo ""
echo -e "${GREEN}================================${NC}"
echo -e "${GREEN}✅ All tests passed!${NC}"
echo -e "${GREEN}================================${NC}"
echo ""
echo "Environment is ready for development:"
echo "  - API:    http://localhost:3000"
echo "  - Ollama: http://localhost:11434"
echo "  - Redis:  localhost:6379"
echo ""
echo "Try editing files in src/ - cargo-watch will auto-rebuild!"
