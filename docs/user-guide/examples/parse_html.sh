#!/bin/bash
# Example: Parse HTML and extract semantic data
#
# Prerequisites:
# 1. Server must be running: cargo run
# 2. jq must be installed for JSON parsing

set -e

SERVER_URL="${SERVER_URL:-http://localhost:3000}"

# Generate authentication token
echo "Generating authentication token..."
TOKEN_RESPONSE=$(curl -s -X POST "$SERVER_URL/auth/token" \
  -H "Content-Type: application/json" \
  -d '{"username":"demo-user","role":"user"}')

TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r .token)

if [ -z "$TOKEN" ] || [ "$TOKEN" = "null" ]; then
  echo "Error: Failed to generate token. Is the server running?"
  echo "Response: $TOKEN_RESPONSE"
  exit 1
fi

echo "Token generated successfully"
echo ""

# Parse HTML with authentication
echo "Parsing HTML and extracting semantic data..."
curl -X POST "$SERVER_URL/parse" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "html": "<html><head><title>Example Page</title></head><body><script type=\"application/ld+json\">{\"@type\": \"Person\", \"name\": \"John Doe\"}</script><div itemscope itemtype=\"http://schema.org/Product\"><span itemprop=\"name\">Widget</span></div></body></html>"
  }'

echo ""
echo "✅ Parse completed successfully!"
