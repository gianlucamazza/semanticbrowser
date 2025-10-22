#!/bin/bash
# Quick Test Script for LLM Agent Integration
# Usage: ./scripts/test_agent.sh

set -e

echo "🤖 Semantic Browser - LLM Agent Test Suite"
echo "==========================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if Ollama is installed
if ! command -v ollama &> /dev/null; then
    echo -e "${RED}❌ Ollama not found!${NC}"
    echo ""
    echo "Please install Ollama:"
    echo "  macOS:   brew install ollama"
    echo "  Linux:   curl https://ollama.ai/install.sh | sh"
    echo "  Windows: Download from https://ollama.ai"
    echo ""
    exit 1
fi

echo -e "${GREEN}✅ Ollama installed${NC}"

# Check if Ollama is running
if ! curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
    echo -e "${YELLOW}⚠️  Ollama is not running${NC}"
    echo ""
    echo "Starting Ollama..."
    ollama serve &
    OLLAMA_PID=$!
    echo "Started Ollama (PID: $OLLAMA_PID)"
    sleep 3
fi

echo -e "${GREEN}✅ Ollama is running${NC}"

# Check if model is available
MODEL="llama3:8b"
if ! ollama list | grep -q "$MODEL"; then
    echo -e "${YELLOW}⚠️  Model $MODEL not found${NC}"
    echo ""
    echo "Pulling $MODEL (this may take a few minutes)..."
    ollama pull $MODEL
fi

echo -e "${GREEN}✅ Model $MODEL available${NC}"
echo ""

# Run tests
echo "📋 Running LLM module tests..."
echo "─────────────────────────────────────"

# Unit tests
echo ""
echo "1️⃣  Unit tests..."
cargo test --lib llm

# Integration tests (with Ollama)
echo ""
echo "2️⃣  Integration tests (requires Ollama)..."
cargo test --lib llm -- --ignored

# Example
echo ""
echo "3️⃣  Running agent example..."
echo "─────────────────────────────────────"
cargo run --example agent_simple_task

echo ""
echo -e "${GREEN}🎉 All tests completed!${NC}"
echo ""

# Cleanup
if [ ! -z "$OLLAMA_PID" ]; then
    echo "Stopping Ollama (PID: $OLLAMA_PID)..."
    kill $OLLAMA_PID
fi
