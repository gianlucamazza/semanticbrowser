#!/bin/bash
# Example: Browse a URL and extract semantic information
#
# Browser Automation Features:
# - Uses chromiumoxide headless browser (when --features browser-automation enabled)
# - Full JavaScript support
# - Automatic fallback to HTTP if chromium unavailable
# - Extracts: title, JSON-LD, microdata, meta tags, Open Graph/Twitter cards, text preview
# - Computes query-aware matches to surface the most relevant snippets for the provided query
# - Blocks ads and trackers (configurable via BLOCK_ADS env var)
#
# Prerequisites:
# 1. Start server with browser automation:
#    cargo run --features browser-automation
# 2. Install Chromium (optional, falls back to HTTP):
#    - Ubuntu: sudo apt install chromium-browser
#    - macOS: brew install --cask chromium

# Generate JWT token
echo "Generating authentication token..."
TOKEN=$(curl -s -X POST http://localhost:3000/auth/token \
  -H "Content-Type: application/json" \
  -d '{"username":"demo","role":"user"}' | jq -r .token)

if [ -z "$TOKEN" ] || [ "$TOKEN" = "null" ]; then
  echo "Error: Failed to generate token. Is the server running?"
  exit 1
fi

echo "Token: $TOKEN"
echo ""

# Browse URL with chromiumoxide (or HTTP fallback)
echo "Browsing URL with semantic data extraction (summary + structured snapshot)..."
RESPONSE=$(curl -s -X POST http://localhost:3000/browse \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $TOKEN" \
  -d '{
    "url": "https://schema.org",
    "query": "extract structured data"
  }')

echo "$RESPONSE" | jq .

echo ""
echo "Key fields (summary text, final URL, top matches):"
echo "$RESPONSE" | jq '{summary: .data, final_url: .snapshot.final_url, matches: .snapshot.query_matches}'

echo ""
echo "---"
echo "Browser automation features:"
echo "- JavaScript rendering: ✅ (with --features browser-automation)"
echo "- Microdata extraction: ✅"
echo "- JSON-LD extraction: ✅"
echo "- Ad blocking: ✅ (set BLOCK_ADS=true)"
echo "- Screenshot support: ✅ (configurable)"
echo "- Cookie management: ✅ (configurable)"
