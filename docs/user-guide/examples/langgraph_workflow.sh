#!/bin/bash
# LangGraph Workflow Example
# Demonstrates the complete semantic browser workflow using LangGraph

set -e

# Configuration
SERVER_URL="http://localhost:3000"
TOKEN="your-jwt-token-here"  # Replace with actual token

echo "=== LangGraph Workflow Example ==="
echo "Testing complete browse → extract → query pipeline"
echo

# Example 1: Simple browse and extract workflow
echo "1. Simple Browse Workflow"
echo "Graph definition:"
GRAPH_DEF='{
  "entry_point": "start",
  "edges": {
    "start": "browse",
    "browse": "extract",
    "extract": "end"
  }
}'

echo "$GRAPH_DEF" | jq .
echo

echo "Input: https://example.com"
curl -X POST "$SERVER_URL/langgraph" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"graph_definition\": $GRAPH_DEF,
    \"input\": \"https://example.com\"
  }" | jq .
echo

# Example 2: Browse with KG query
echo "2. Browse and Query Workflow"
echo "Graph definition:"
GRAPH_DEF_QUERY='{
  "entry_point": "start",
  "edges": {
    "start": "browse",
    "browse": "extract",
    "extract": "query"
  }
}'

echo "$GRAPH_DEF_QUERY" | jq .
echo

echo "Input with query: {\"url\": \"https://example.com\", \"query\": \"SELECT * WHERE { ?s ?p ?o } LIMIT 5\"}"
curl -X POST "$SERVER_URL/langgraph" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"graph_definition\": $GRAPH_DEF_QUERY,
    \"input\": \"{\\\\"url\\\": \\\\"https://example.com\\\", \\\\"query\\\": \\\\"SELECT * WHERE { ?s ?p ?o } LIMIT 5\\\"}\"
  }" | jq .
echo

# Example 3: Conditional workflow
echo "3. Conditional Workflow (with error handling)"
echo "Graph definition:"
GRAPH_DEF_CONDITIONAL='{
  "entry_point": "start",
  "edges": {
    "start": "browse",
    "browse": "extract"
  },
  "conditional_edges": {
    "extract": {
      "type": "has_data"
    }
  }
}'

echo "$GRAPH_DEF_CONDITIONAL" | jq .
echo

echo "Input: https://httpbin.org/html (reliable test page)"
curl -X POST "$SERVER_URL/langgraph" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d "{
    \"graph_definition\": $GRAPH_DEF_CONDITIONAL,
    \"input\": \"https://httpbin.org/html\"
  }" | jq .
echo

echo "=== Workflow Examples Complete ==="
echo "LangGraph enables flexible agent workflows with:"
echo "- Real browse operations with chromiumoxide fallback"
echo "- Semantic data extraction and entity recognition"
echo "- Knowledge Graph queries with SPARQL"
echo "- Conditional execution based on workflow state"
echo "- Error recovery and state persistence"