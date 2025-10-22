#!/bin/bash
# Token Revocation Example
# Demonstrates JWT token generation and revocation using Redis
#
# Prerequisites:
# - Build with Redis support: cargo build --features redis-integration
# - Set REDIS_URL environment variable
# - Start the server: cargo run --features redis-integration
#
# Usage: ./token_revocation.sh

set -e

# Configuration
SERVER_URL="http://localhost:3000"
JWT_SECRET="demo-secret-key-that-is-long-enough-for-jwt-validation-32chars-minimum"

echo "🔐 JWT Token Revocation Demo"
echo "=============================="
echo

# Set environment variables
export JWT_SECRET="$JWT_SECRET"
export REDIS_URL="${REDIS_URL:-redis://127.0.0.1:6379}"

echo "📋 Configuration:"
echo "   Server: $SERVER_URL"
echo "   Redis: $REDIS_URL"
echo

# Check if server is running
echo "🔍 Checking server status..."
if ! curl -s "$SERVER_URL/health" > /dev/null; then
    echo "❌ Server not running at $SERVER_URL"
    echo "   Start with: cargo run --features redis-integration"
    exit 1
fi
echo "✅ Server is running"
echo

# Generate a token
echo "🎫 Generating JWT token..."
TOKEN_RESPONSE=$(curl -s -X POST "$SERVER_URL/auth/token" \
    -H "Content-Type: application/json" \
    -d '{"username": "demo_user", "role": "admin"}')

if echo "$TOKEN_RESPONSE" | jq -e '.token' > /dev/null 2>&1; then
    TOKEN=$(echo "$TOKEN_RESPONSE" | jq -r '.token')
    EXPIRES_IN=$(echo "$TOKEN_RESPONSE" | jq -r '.expires_in')
    echo "✅ Token generated successfully"
    echo "   Token: ${TOKEN:0:50}..."
    echo "   Expires in: $EXPIRES_IN seconds"
else
    echo "❌ Failed to generate token"
    echo "Response: $TOKEN_RESPONSE"
    exit 1
fi
echo

# Test token validation before revocation
echo "🔍 Testing token validation (before revocation)..."
VALIDATE_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" "$SERVER_URL/health")
if echo "$VALIDATE_RESPONSE" | grep -q "healthy"; then
    echo "✅ Token is valid"
else
    echo "❌ Token validation failed"
    echo "Response: $VALIDATE_RESPONSE"
fi
echo

# Revoke the token
echo "🚫 Revoking token..."
REVOKE_RESPONSE=$(curl -s -X POST "$SERVER_URL/auth/revoke" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer $TOKEN" \
    -d "{\"token\": \"$TOKEN\"}")

if echo "$REVOKE_RESPONSE" | jq -e '.revoked' > /dev/null 2>&1; then
    REVOKED=$(echo "$REVOKE_RESPONSE" | jq -r '.revoked')
    MESSAGE=$(echo "$REVOKE_RESPONSE" | jq -r '.message')
    if [ "$REVOKED" = "true" ]; then
        echo "✅ Token revoked successfully"
        echo "   Message: $MESSAGE"
    else
        echo "❌ Token revocation failed"
        echo "   Message: $MESSAGE"
    fi
else
    echo "❌ Revocation request failed"
    echo "Response: $REVOKE_RESPONSE"
fi
echo

# Test token validation after revocation
echo "🔍 Testing token validation (after revocation)..."
VALIDATE_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" "$SERVER_URL/health")
if echo "$VALIDATE_RESPONSE" | grep -q "healthy"; then
    echo "❌ Token is still valid (revocation failed)"
else
    echo "✅ Token is properly revoked"
fi
echo

# Generate a new token to show the system still works
echo "🎫 Generating new token (system still functional)..."
NEW_TOKEN_RESPONSE=$(curl -s -X POST "$SERVER_URL/auth/token" \
    -H "Content-Type: application/json" \
    -d '{"username": "demo_user2", "role": "user"}')

if echo "$NEW_TOKEN_RESPONSE" | jq -e '.token' > /dev/null 2>&1; then
    NEW_TOKEN=$(echo "$NEW_TOKEN_RESPONSE" | jq -r '.token')
    echo "✅ New token generated successfully"
    echo "   Token: ${NEW_TOKEN:0:50}..."

    # Test the new token
    NEW_VALIDATE_RESPONSE=$(curl -s -H "Authorization: Bearer $NEW_TOKEN" "$SERVER_URL/health")
    if echo "$NEW_VALIDATE_RESPONSE" | grep -q "healthy"; then
        echo "✅ New token is valid"
    else
        echo "❌ New token validation failed"
    fi
else
    echo "❌ Failed to generate new token"
fi
echo

echo "🎉 Demo completed!"
echo
echo "📚 Notes:"
echo "   - Revoked tokens are stored in Redis with expiration"
echo "   - Token validation checks Redis for revocation status"
echo "   - Build with: cargo build --features redis-integration"
echo "   - Set REDIS_URL environment variable for Redis connection"