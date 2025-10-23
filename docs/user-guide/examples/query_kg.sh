#!/bin/bash
# Example: Query the Knowledge Graph using SPARQL
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

# SELECT query
echo "=== SELECT Query ==="
curl -s -X POST "$SERVER_URL/query" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "query": "SELECT * WHERE { ?s ?p ?o } LIMIT 10"
  }' | jq .

echo ""
echo "=== INSERT Update ==="
# INSERT update
curl -s -X POST "$SERVER_URL/query" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "query": "INSERT DATA { <http://example.org/person1> <http://xmlns.com/foaf/0.1/name> \"Alice\" }"
  }' | jq .

echo ""
echo "=== Verify INSERT ==="
# Verify the insert
curl -s -X POST "$SERVER_URL/query" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "query": "SELECT ?name WHERE { <http://example.org/person1> <http://xmlns.com/foaf/0.1/name> ?name }"
  }' | jq .

echo ""
echo "✅ Knowledge Graph queries completed successfully!"
