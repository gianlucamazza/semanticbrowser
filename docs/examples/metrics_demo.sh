#!/bin/bash
# Prometheus Metrics Demo
# Demonstrates observability metrics collection and export
#
# Prerequisites:
# - Build with observability support: cargo build --features observability
# - Start the server: cargo run --features observability
#
# Usage: ./metrics_demo.sh

set -e

# Configuration
SERVER_URL="http://localhost:3000"
JWT_SECRET="demo-secret-key-that-is-long-enough-for-jwt-validation-32chars-minimum"

echo "📊 Prometheus Metrics Demo"
echo "==========================="
echo

# Set environment variables
export JWT_SECRET="$JWT_SECRET"

# Check if server is running
echo "🔍 Checking server status..."
if ! curl -s "$SERVER_URL/health" > /dev/null; then
    echo "❌ Server not running at $SERVER_URL"
    echo "   Start with: cargo run --features observability"
    exit 1
fi
echo "✅ Server is running"
echo

# Generate a token for authentication
echo "🎫 Generating JWT token..."
TOKEN_RESPONSE=$(curl -s -X POST "$SERVER_URL/auth/token" \
    -H "Content-Type: application/json" \
    -d '{"username": "metrics-demo", "role": "admin"}')

if echo "$TOKEN_RESPONSE" | jq -e '.token' > /dev/null 2>&1; then
    TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.token')
    echo "✅ Token generated successfully"
else
    echo "❌ Failed to generate token"
    echo "Response: $TOKEN_RESPONSE"
    exit 1
fi
echo

# Make some API calls to generate metrics
echo "📈 Generating metrics with API calls..."

# Parse HTML
echo "   📄 Parsing HTML..."
curl -s -H "Authorization: Bearer $TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"html": "<html><body><h1>Test</h1><p>Metrics demo content</p></body></html>"}' \
     "$SERVER_URL/parse" > /dev/null

# Query KG
echo "   🔍 Querying Knowledge Graph..."
curl -s -H "Authorization: Bearer $TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"query": "SELECT * WHERE { ?s ?p ?o }"}' \
     "$SERVER_URL/query" > /dev/null

# List KG entities
echo "   📋 Listing KG entities..."
curl -s -H "Authorization: Bearer $TOKEN" \
     "$SERVER_URL/kg/entities" > /dev/null

echo "✅ API calls completed"
echo

# Fetch metrics
echo "📊 Fetching Prometheus metrics..."
METRICS=$(curl -s "$SERVER_URL/metrics")

if [ -z "$METRICS" ]; then
    echo "❌ No metrics received"
    exit 1
fi

echo "✅ Metrics collected successfully"
echo

# Display metrics summary
echo "📈 Metrics Summary:"
echo "==================="

# Count total metrics
METRICS_COUNT=$(echo "$METRICS" | grep -c "^[a-zA-Z_]")
echo "   📊 Total metrics: $METRICS_COUNT"

# Show semantic_browser metrics
SEMANTIC_METRICS=$(echo "$METRICS" | grep "semantic_browser" | wc -l)
echo "   🎯 Semantic Browser metrics: $SEMANTIC_METRICS"

# Show uptime
UPTIME=$(echo "$METRICS" | grep "semantic_browser_uptime_seconds" | awk '{print $2}')
if [ -n "$UPTIME" ]; then
    echo "   ⏱️  Service uptime: ${UPTIME}s"
fi

echo
echo "🔍 Sample Metrics:"
echo "=================="
echo "$METRICS" | grep "semantic_browser" | head -5

echo
echo "🎉 Demo completed!"
echo
echo "📚 Notes:"
echo "   - Metrics are exported in Prometheus format"
echo "   - Use with: cargo build --features observability"
echo "   - Integrate with Prometheus for monitoring dashboards"
echo "   - RED metrics: Rate, Errors, Duration for all services"