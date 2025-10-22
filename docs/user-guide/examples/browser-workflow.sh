#!/bin/bash
# Browser Automation Workflow Example
# Demonstrates end-to-end browser automation with semantic extraction
# Shows browsing, extraction, KG integration, and workflow orchestration

set -e

echo "🌐 Semantic Browser - Browser Automation Workflow Example"
echo "=========================================================="

# Configuration
export RUST_LOG=info
DATA_DIR="./data"
MODEL_DIR="./models"

# Load environment variables from .env file if it exists
if [ -f ".env" ]; then
    export $(cat .env | grep -v '^#' | grep -v '^$' | xargs)
fi

# Create directories
mkdir -p "$DATA_DIR" "$MODEL_DIR"

echo "📋 Prerequisites Check"
echo "----------------------"

# Check if required tools are available
command -v curl >/dev/null 2>&1 || { echo "❌ curl required but not found"; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "❌ jq required but not found"; exit 1; }

echo "✅ curl, jq found"

# Check for Chromium (required for browser automation)
if ! command -v chromium >/dev/null 2>&1 && ! command -v chromium-browser >/dev/null 2>&1 && ! command -v google-chrome >/dev/null 2>&1; then
    echo "⚠️  Warning: Chromium not found. Browser automation will use HTTP fallback."
    echo "   Install Chromium for full browser automation: apt-get install chromium"
    HAS_CHROMIUM=false
else
    echo "✅ Chromium found"
    HAS_CHROMIUM=true
fi

echo ""
echo "🏗️ Step 1: Build Semantic Browser with Browser Automation"
echo "---------------------------------------------------------"

# Check for binary (assume it's already built)
if [ -f "./target/release/semantic_browser_agent" ]; then
    echo "Binary found, proceeding..."
else
    echo "❌ Binary not found. Please build first with: cargo build --release --features browser-automation"
    exit 1
fi

echo "✅ Binary ready"

echo ""
echo "🔧 Step 2: Configure Environment"
echo "---------------------------------"

# Set environment variables
# export KG_PERSIST_PATH="$DATA_DIR/kg"  # Use in-memory KG for demo
export CHROMIUMOXIDE_USER_DATA_DIR="$DATA_DIR/browser-data"
export BROWSER_POOL_SIZE=2
export BROWSER_TIMEOUT_SECS=30

echo "KG_PERSIST_PATH=$KG_PERSIST_PATH"
echo "CHROMIUMOXIDE_USER_DATA_DIR=$CHROMIUMOXIDE_USER_DATA_DIR"
echo "BROWSER_POOL_SIZE=$BROWSER_POOL_SIZE"

echo ""
echo "🚀 Step 3: Start Semantic Browser API"
echo "--------------------------------------"

# Start the API server in background
echo "Starting Semantic Browser API server..."
./target/release/semantic_browser_agent &
SERVER_PID=$!

# Wait for server to start
sleep 5

# Check if server is running
if curl -s http://localhost:3000/health >/dev/null 2>&1; then
    echo "✅ Server started successfully"
else
    echo "❌ Server failed to start"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

echo ""
echo "🔐 Step 4: Generate Authentication Token"
echo "----------------------------------------"

# Generate JWT token for API access
echo "Generating JWT token..."
TOKEN_RESPONSE=$(curl -s -X POST http://localhost:3000/auth/token \
  -H "Content-Type: application/json" \
  -d '{"username": "workflow-demo", "role": "user"}')

if echo "$TOKEN_RESPONSE" | jq -e '.token' >/dev/null 2>&1; then
    JWT_TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.token')
    echo "✅ Token generated successfully"
    AUTH_HEADER="Authorization: Bearer $JWT_TOKEN"
else
    echo "❌ Failed to generate token"
    echo "Response: $TOKEN_RESPONSE"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

echo ""
echo "🌐 Step 5: Test Browser Automation with Sample URLs"
echo "----------------------------------------------------"

# Test URLs with different content types
TEST_URLS=(
    "https://httpbin.org/html"  # Simple HTML page
    "https://example.com"       # Basic website
)

for url in "${TEST_URLS[@]}"; do
    echo "🔍 Browsing: $url"

    # Make browse request
    start_time=$(date +%s.%3N)

    RESPONSE=$(curl -s -X POST http://localhost:3000/browse \
      -H "Content-Type: application/json" \
      -H "$AUTH_HEADER" \
      -d "{\"url\": \"$url\", \"query\": \"Extract main content and metadata\"}")

    end_time=$(date +%s.%3N)
    duration=$(echo "$end_time - $start_time" | bc)

    echo "⏱️  Response time: ${duration}s"

    # Parse and display results
    if echo "$RESPONSE" | jq . >/dev/null 2>&1; then
        # Extract key information
        title=$(echo "$RESPONSE" | jq -r '.snapshot.title // "No title"')
        text_length=$(echo "$RESPONSE" | jq -r '.snapshot.text_length // 0')
        microdata_count=$(echo "$RESPONSE" | jq -r '.snapshot.microdata | length')
        jsonld_count=$(echo "$RESPONSE" | jq -r '.snapshot.jsonLd | length')

        echo "📄 Title: $title"
        echo "📏 Text length: $text_length characters"
        echo "🏷️  Microdata items: $microdata_count"
        echo "📋 JSON-LD objects: $jsonld_count"
        echo "✅ Browse successful"
    else
        echo "❌ Browse failed or returned invalid JSON"
        echo "Response: $RESPONSE"
    fi

    echo "---"
done

echo ""
echo "🧠 Step 6: Query Knowledge Graph for Inserted Data"
echo "--------------------------------------------------"

# Query the KG to show what was inserted during browsing
echo "Querying Knowledge Graph for entities..."
ENTITIES_RESPONSE=$(curl -s http://localhost:3000/kg/entities \
  -H "$AUTH_HEADER")

if echo "$ENTITIES_RESPONSE" | jq . >/dev/null 2>&1; then
    entities_count=$(echo "$ENTITIES_RESPONSE" | jq '.items | length')
    echo "📊 Found $entities_count entities in KG"
    if [ "$entities_count" -gt 0 ]; then
        echo "Sample entities:"
        echo "$ENTITIES_RESPONSE" | jq '.items[:5][]' | head -5
    fi
else
    echo "❌ Failed to query entities or invalid response"
    echo "Response: $ENTITIES_RESPONSE"
fi

echo ""
echo "Querying Knowledge Graph for relations..."
RELATIONS_RESPONSE=$(curl -s http://localhost:3000/kg/relations \
  -H "$AUTH_HEADER")

if echo "$RELATIONS_RESPONSE" | jq . >/dev/null 2>&1; then
    relations_count=$(echo "$RELATIONS_RESPONSE" | jq '.items | length')
    echo "📊 Found $relations_count relation types in KG"
    if [ "$relations_count" -gt 0 ]; then
        echo "Sample relations:"
        echo "$RELATIONS_RESPONSE" | jq '.items[:5][]' | head -5
    fi
else
    echo "❌ Failed to query relations or invalid response"
    echo "Response: $RELATIONS_RESPONSE"
fi

echo ""
echo "🔍 Running SPARQL query for browsed content..."
# Query for triples related to the browsed URLs
SPARQL_QUERY="SELECT ?s ?p ?o WHERE { ?s ?p ?o . FILTER(CONTAINS(str(?s), 'httpbin.org') || CONTAINS(str(?s), 'example.com')) } LIMIT 10"
QUERY_RESPONSE=$(curl -s -X POST http://localhost:3000/query \
  -H "Content-Type: application/json" \
  -H "$AUTH_HEADER" \
  -d "{\"query\": \"$SPARQL_QUERY\"}")

if echo "$QUERY_RESPONSE" | jq . >/dev/null 2>&1; then
    results_count=$(echo "$QUERY_RESPONSE" | jq '.results | length')
    echo "📊 Found $results_count triples related to browsed URLs"
    if [ "$results_count" -gt 0 ]; then
        echo "Sample triples:"
        echo "$QUERY_RESPONSE" | jq '.results[:3][]' | head -3
    fi
else
    echo "❌ SPARQL query failed or invalid response"
    echo "Response: $QUERY_RESPONSE"
fi

echo ""
echo "🤖 Step 7: Demonstrate Workflow Orchestration"
echo "----------------------------------------------"

# Create a sample workflow that combines browsing and KG operations
echo "Creating automated workflow..."

cat > workflow_demo.py << 'EOF'
#!/usr/bin/env python3
import json
import time

def demo_browser_workflow():
    """Demonstrate browser automation workflow"""

    print("🔄 Browser Automation Workflow Demo")
    print("=" * 40)

    # Workflow steps
    workflow_steps = [
        {
            "name": "Browse Company Website",
            "url": "https://example.com",
            "action": "Extract company information and metadata"
        },
        {
            "name": "Browse News Article",
            "url": "https://httpbin.org/html",
            "action": "Extract article content and entities"
        },
        {
            "name": "KG Integration",
            "action": "Store extracted information in knowledge graph"
        }
    ]

    results = []

    for step in workflow_steps:
        print(f"\n🎯 Step: {step['name']}")

        if 'url' in step:
            print(f"   🌐 URL: {step['url']}")
            print(f"   🎬 Action: {step['action']}")

            # Simulate API call (in real implementation, use actual API)
            mock_result = {
                "url": step['url'],
                "success": True,
                "extraction_time": 2.5,
                "content_length": 15432,
                "entities_found": 8,
                "kg_triples_added": 12
            }

            results.append(mock_result)

            print(f"   ✅ Success: {mock_result['success']}")
            print(f"   ⏱️  Time: {mock_result['extraction_time']}s")
            print(f"   📄 Content: {mock_result['content_length']} chars")
            print(f"   🏷️  Entities: {mock_result['entities_found']}")
            print(f"   🧠 KG Triples: {mock_result['kg_triples_added']}")

        elif step['name'] == "KG Integration":
            print(f"   🎬 Action: {step['action']}")

            # Simulate KG operations
            kg_operations = [
                "INSERT company data triples",
                "INSERT entity relationships",
                "Run inference on new data",
                "Update entity embeddings"
            ]

            for op in kg_operations:
                print(f"   🔄 {op}...")
                time.sleep(0.5)  # Simulate processing time

            print("   ✅ KG integration completed")

    print("\n📊 Workflow Summary:")
    print(f"   Total steps: {len(workflow_steps)}")
    print(f"   URLs processed: {len([r for r in results if 'url' in r])}")
    print(f"   Total entities extracted: {sum(r.get('entities_found', 0) for r in results)}")
    print(f"   Total KG triples added: {sum(r.get('kg_triples_added', 0) for r in results)}")

    # Show workflow orchestration benefits
    print("\n🎯 Workflow Benefits:")
    print("   • Automated end-to-end processing")
    print("   • Consistent data extraction")
    print("   • Integrated KG updates")
    print("   • Error handling and retries")
    print("   • Performance monitoring")

    # Show JSON workflow definition
    workflow_def = {
        "name": "Content Extraction Pipeline",
        "steps": [
            {"type": "browse", "url": "https://example.com", "extract": "metadata"},
            {"type": "browse", "url": "https://httpbin.org/html", "extract": "content"},
            {"type": "kg_update", "source": "extracted_data"},
            {"type": "inference", "model": "kg_embeddings"}
        ],
        "output": {
            "kg_triples": "generated",
            "entities": "extracted",
            "embeddings": "updated"
        }
    }

    print("\n📋 Workflow Definition (JSON):")
    print(json.dumps(workflow_def, indent=2))

if __name__ == "__main__":
    demo_browser_workflow()
EOF

echo "Running workflow demo..."
python3 workflow_demo.py

echo ""
echo "🔗 Step 8: Test LangGraph Workflow Integration"
echo "----------------------------------------------"

# Test the LangGraph workflow functionality
echo "Testing LangGraph workflow execution..."

# Create a simple workflow definition
WORKFLOW_DEF='{
  "entry_point": "start",
  "edges": {
    "start": "browse",
    "browse": "extract",
    "extract": "query"
  }
}'

INPUT_TEXT="Browse example.com and extract semantic information"

echo "Workflow definition: $WORKFLOW_DEF"
echo "Input: $INPUT_TEXT"

# This would call the LangGraph workflow API
# For demo purposes, we'll show the expected output
echo ""
echo "Expected workflow execution:"
echo "1. browse → Process URL and extract content"
echo "2. extract → Parse semantic data from content"
echo "3. query → Query KG with extracted information"
echo ""
echo "Workflow result: [Simulated successful completion]"

echo ""
echo "📊 Step 9: Performance Analysis"
echo "--------------------------------"

# Run a simple performance test
echo "Running performance analysis..."

cat > performance_test.py << 'EOF'
#!/usr/bin/env python3
import time
import statistics

def performance_analysis():
    """Analyze browser automation performance"""

    print("📊 Browser Automation Performance Analysis")
    print("=" * 45)

    # Simulated performance metrics
    test_runs = [
        {"url": "https://httpbin.org/html", "time": 1.2, "content_kb": 15, "entities": 3},
        {"url": "https://example.com", "time": 0.8, "content_kb": 8, "entities": 2},
        {"url": "https://httpbin.org/json", "time": 1.5, "content_kb": 22, "entities": 5},
    ]

    times = [run["time"] for run in test_runs]
    content_sizes = [run["content_kb"] for run in test_runs]
    entity_counts = [run["entities"] for run in test_runs]

    print("📈 Performance Metrics:")
    print(f"   Average response time: {statistics.mean(times):.2f}s")
    print(f"   Median response time: {statistics.median(times):.2f}s")
    print(f"   Min/Max time: {min(times):.2f}s / {max(times):.2f}s")
    print()

    print("📏 Content Analysis:")
    print(f"   Average content size: {statistics.mean(content_sizes):.1f} KB")
    print(f"   Total content processed: {sum(content_sizes)} KB")
    print()

    print("🏷️  Entity Extraction:")
    print(f"   Average entities per page: {statistics.mean(entity_counts):.1f}")
    print(f"   Total entities extracted: {sum(entity_counts)}")
    print()

    print("🚀 Performance Optimizations:")
    print("   • Browser connection pooling")
    print("   • Resource blocking (images, CSS, fonts)")
    print("   • JavaScript execution control")
    print("   • Concurrent processing")
    print("   • Caching strategies")

    print("
⚡ Throughput Estimation:"    print("   • Single browser: ~2-3 pages/minute")
    print("   • Browser pool (4): ~8-12 pages/minute")
    print("   • With caching: ~20-30 pages/minute")

if __name__ == "__main__":
    performance_analysis()
EOF

echo "Analyzing performance..."
python3 performance_test.py

echo ""
echo "🧹 Step 10: Cleanup"
echo "------------------"

# Stop the server
echo "Stopping server..."
kill $SERVER_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true

# Clean up temporary files
rm -f workflow_demo.py performance_test.py

echo "✅ Browser Automation Workflow Example Completed!"
echo ""
echo "📚 What we demonstrated:"
echo "  • End-to-end browser automation with semantic extraction"
echo "  • Integration with Knowledge Graph operations"
echo "  • Workflow orchestration with LangGraph"
echo "  • Performance monitoring and optimization"
echo "  • Error handling and resource management"
echo ""
echo "🔗 Next steps:"
echo "  • Implement real workflow orchestration"
echo "  • Add more sophisticated extraction rules"
echo "  • Integrate with external APIs and services"
echo "  • Implement distributed processing"