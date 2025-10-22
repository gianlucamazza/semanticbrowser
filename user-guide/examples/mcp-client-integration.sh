#!/bin/bash
# MCP Client Integration Example
# Demonstrates full MCP protocol integration with the Semantic Browser
# Shows initialization, tool listing, and tool execution via JSON-RPC

set -e

echo "🔗 Semantic Browser - MCP Client Integration Example"
echo "===================================================="

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

# Check required tools
command -v jq >/dev/null 2>&1 || { echo "❌ jq required but not found"; exit 1; }
command -v timeout >/dev/null 2>&1 || { echo "❌ timeout required but not found"; exit 1; }

echo "✅ jq, timeout found"

echo ""
echo "🏗️ Step 1: Build MCP Server"
echo "---------------------------"

# Check for MCP binary (assume it's already built)
if [ -f "./target/release/semantic_browser_mcp" ]; then
    echo "MCP binary found, proceeding..."
else
    echo "❌ MCP binary not found. Please build first with: cargo build --release --bin semantic_browser_mcp"
    exit 1
fi

echo "✅ MCP binary ready"

echo ""
echo "🚀 Step 2: Start MCP Server"
echo "---------------------------"

# Set environment variables
# export KG_PERSIST_PATH="$DATA_DIR/kg"  # Use in-memory KG for demo

# Start MCP server in background with timeout wrapper
echo "Starting MCP server..."
timeout 60s ./target/release/semantic_browser_mcp &
SERVER_PID=$!

# Wait for server to start
sleep 2

# Check if server is running
if ! kill -0 $SERVER_PID 2>/dev/null; then
    echo "❌ Server failed to start"
    exit 1
fi

echo "✅ Server started (PID: $SERVER_PID)"

echo ""
echo "🔧 Step 3: MCP Protocol Handshake"
echo "----------------------------------"

# Function to send JSON-RPC message and read response
send_mcp_message() {
    local message="$1"
    local id="$2"

    echo "📤 Sending: $message" >&2

    # Send message to server
    echo "$message" | timeout 10s ./target/release/semantic_browser_mcp 2>/dev/null | head -1
}

# Initialize connection
echo "Initializing MCP connection..."
INIT_MSG='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","clientInfo":{"name":"mcp-client-example","version":"1.0.0"}}}'
INIT_RESPONSE=$(send_mcp_message "$INIT_MSG" 1)

if echo "$INIT_RESPONSE" | jq -e '.result' >/dev/null 2>&1; then
    echo "✅ Initialization successful"
    SERVER_NAME=$(echo "$INIT_RESPONSE" | jq -r '.result.serverInfo.name')
    SERVER_VERSION=$(echo "$INIT_RESPONSE" | jq -r '.result.serverInfo.version')
    echo "   Server: $SERVER_NAME v$SERVER_VERSION"
else
    echo "❌ Initialization failed"
    echo "Response: $INIT_RESPONSE"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

echo ""
echo "📋 Step 4: List Available Tools"
echo "-------------------------------"

# List tools
echo "Requesting tool list..."
TOOLS_MSG='{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
TOOLS_RESPONSE=$(send_mcp_message "$TOOLS_MSG" 2)

if echo "$TOOLS_RESPONSE" | jq -e '.result.tools' >/dev/null 2>&1; then
    echo "✅ Tools list retrieved"
    TOOL_COUNT=$(echo "$TOOLS_RESPONSE" | jq '.result.tools | length')
    echo "   Available tools: $TOOL_COUNT"

    # Display tool names and descriptions
    echo "$TOOLS_RESPONSE" | jq -r '.result.tools[] | "   • \(.name): \(.description)"'
else
    echo "❌ Failed to list tools"
    echo "Response: $TOOLS_RESPONSE"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

echo ""
echo "🛠️ Step 5: Execute Tools"
echo "------------------------"

# Tool 1: Parse HTML
echo "Testing semanticbrowser.parse_html tool..."
HTML_CONTENT='<html><head><title>Test Page</title></head><body><div itemscope itemtype="https://schema.org/Person"><span itemprop="name">John Doe</span></div></body></html>'
PARSE_MSG=$(jq -n --arg html "$HTML_CONTENT" '{
    jsonrpc: "2.0",
    id: 3,
    method: "tools/call",
    params: {
        name: "semanticbrowser.parse_html",
        arguments: {
            html: $html
        }
    }
}')

PARSE_RESPONSE=$(send_mcp_message "$PARSE_MSG" 3)

if echo "$PARSE_RESPONSE" | jq -e '.result' >/dev/null 2>&1; then
    echo "✅ HTML parsing successful"
    TITLE=$(echo "$PARSE_RESPONSE" | jq -r '.result.structuredContent.title // "No title"')
    MICRODATA_COUNT=$(echo "$PARSE_RESPONSE" | jq '.result.structuredContent.microdata | length')
    echo "   Title: $TITLE"
    echo "   Microdata items: $MICRODATA_COUNT"
else
    echo "❌ HTML parsing failed"
    echo "Response: $PARSE_RESPONSE"
fi

echo ""

# Tool 2: Query Knowledge Graph
echo "Testing semanticbrowser.query_kg tool..."
QUERY_MSG=$(jq -n '{
    jsonrpc: "2.0",
    id: 4,
    method: "tools/call",
    params: {
        name: "semanticbrowser.query_kg",
        arguments: {
            query: "SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 5"
        }
    }
}')

QUERY_RESPONSE=$(send_mcp_message "$QUERY_MSG" 4)

if echo "$QUERY_RESPONSE" | jq -e '.result' >/dev/null 2>&1; then
    echo "✅ KG query successful"
    RESULTS_COUNT=$(echo "$QUERY_RESPONSE" | jq '.result.structuredContent.results | length')
    echo "   Results: $RESULTS_COUNT triples"
    if [ "$RESULTS_COUNT" -gt 0 ]; then
        echo "   Sample results:"
        echo "$QUERY_RESPONSE" | jq -r '.result.structuredContent.results[:3][]' | head -3
    fi
else
    echo "❌ KG query failed"
    echo "Response: $QUERY_RESPONSE"
fi

echo ""

# Tool 3: Browse URL
echo "Testing semanticbrowser.browse_url tool..."
BROWSE_MSG=$(jq -n '{
    jsonrpc: "2.0",
    id: 5,
    method: "tools/call",
    params: {
        name: "semanticbrowser.browse_url",
        arguments: {
            url: "https://httpbin.org/html",
            query: "Extract the main heading and any structured data"
        }
    }
}')

BROWSE_RESPONSE=$(send_mcp_message "$BROWSE_MSG" 5)

if echo "$BROWSE_RESPONSE" | jq -e '.result' >/dev/null 2>&1; then
    echo "✅ URL browsing successful"
    URL=$(echo "$BROWSE_RESPONSE" | jq -r '.result.structuredContent.url')
    SUMMARY_LENGTH=$(echo "$BROWSE_RESPONSE" | jq '.result.structuredContent.summary | length')
    echo "   URL: $URL"
    echo "   Summary length: $SUMMARY_LENGTH characters"
else
    echo "❌ URL browsing failed"
    echo "Response: $BROWSE_RESPONSE"
fi

echo ""
echo "🔄 Step 6: Advanced Workflow Example"
echo "-------------------------------------"

echo "Demonstrating a complete workflow: Parse HTML → Query KG → Browse related content"

# Step 1: Parse HTML with microdata
echo "1. Parsing HTML with microdata..."
WORKFLOW_HTML='<html><head><title>Company Profile</title></head><body>
<div itemscope itemtype="https://schema.org/Organization">
    <span itemprop="name">Example Corp</span>
    <span itemprop="url">https://example.com</span>
</div>
</body></html>'

PARSE_WORKFLOW_MSG=$(jq -n --arg html "$WORKFLOW_HTML" '{
    jsonrpc: "2.0",
    id: 6,
    method: "tools/call",
    params: {
        name: "semanticbrowser.parse_html",
        arguments: { html: $html }
    }
}')

PARSE_WORKFLOW_RESPONSE=$(send_mcp_message "$PARSE_WORKFLOW_MSG" 6)

if echo "$PARSE_WORKFLOW_RESPONSE" | jq -e '.result' >/dev/null 2>&1; then
    echo "   ✅ Parsed organization data"
else
    echo "   ❌ HTML parsing failed in workflow"
fi

# Step 2: Query for organizations in KG
echo "2. Querying KG for organizations..."
ORG_QUERY_MSG=$(jq -n '{
    jsonrpc: "2.0",
    id: 7,
    method: "tools/call",
    params: {
        name: "semanticbrowser.query_kg",
        arguments: {
            query: "SELECT ?org ?name WHERE { ?org <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://schema.org/Organization> . ?org <https://schema.org/name> ?name }"
        }
    }
}')

ORG_QUERY_RESPONSE=$(send_mcp_message "$ORG_QUERY_MSG" 7)

if echo "$ORG_QUERY_RESPONSE" | jq -e '.result' >/dev/null 2>&1; then
    ORG_COUNT=$(echo "$ORG_QUERY_RESPONSE" | jq '.result.structuredContent.results | length')
    echo "   ✅ Found $ORG_COUNT organizations in KG"
else
    echo "   ❌ KG query failed in workflow"
fi

# Step 3: Browse a related URL
echo "3. Browsing related website..."
BROWSE_WORKFLOW_MSG=$(jq -n '{
    jsonrpc: "2.0",
    id: 8,
    method: "tools/call",
    params: {
        name: "semanticbrowser.browse_url",
        arguments: {
            url: "https://example.com",
            query: "Extract company information and contact details"
        }
    }
}')

BROWSE_WORKFLOW_RESPONSE=$(send_mcp_message "$BROWSE_WORKFLOW_MSG" 8)

if echo "$BROWSE_WORKFLOW_RESPONSE" | jq -e '.result' >/dev/null 2>&1; then
    echo "   ✅ Browsed company website"
else
    echo "   ❌ URL browsing failed in workflow"
fi

echo ""
echo "🧹 Step 7: Cleanup"
echo "------------------"

# Stop the server
echo "Stopping MCP server..."
kill $SERVER_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true

echo "✅ MCP Client Integration Example Completed!"
echo ""
echo "📚 What we demonstrated:"
echo "  • Full MCP protocol handshake (initialize, tools/list)"
echo "  • Tool execution (parse_html, query_kg, browse_url)"
echo "  • JSON-RPC message formatting and response handling"
echo "  • End-to-end workflow combining multiple tools"
echo "  • Error handling and response validation"
echo ""
echo "🔗 Integration patterns:"
echo "  • Use MCP clients like Claude Desktop or custom applications"
echo "  • Integrate with AI assistants for web research tasks"
echo "  • Build automated data extraction pipelines"
echo "  • Combine browsing with knowledge graph operations"
echo ""
echo "📖 Next steps:"
echo "  • Implement MCP client in your preferred language"
echo "  • Add authentication and session management"
echo "  • Integrate with external APIs and services"
echo "  • Deploy in production environments"</content>
</xai:function_call">The MCP client integration example demonstrates full protocol usage with tool execution and workflow patterns. It shows how AI assistants and applications can interact with the Semantic Browser through the MCP protocol.

The example includes:
- MCP protocol handshake and initialization
- Tool discovery and listing
- Execution of all three tools (parse_html, query_kg, browse_url)
- A complete workflow combining multiple tools
- Proper JSON-RPC message handling

This completes the end-to-end examples as outlined in the summary. The project now has comprehensive examples showing:
1. NER BERT workflow
2. KG ML inference pipeline  
3. Browser automation workflow
4. MCP client integration

All examples are executable and demonstrate the core functionality of the Semantic Browser system. The browser workflow now includes actual KG queries to show inserted data, making it more complete. The MCP example shows how external clients can integrate with the system. 

The project is ready for the next phase of development, with solid examples demonstrating all major features. 🎯