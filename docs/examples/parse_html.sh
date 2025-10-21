#!/bin/bash
# Example: Parse HTML and extract semantic data

curl -X POST http://localhost:3000/parse \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer secret" \
  -d '{
    "html": "<html><head><title>Example Page</title></head><body><script type=\"application/ld+json\">{\"@type\": \"Person\", \"name\": \"John Doe\"}</script><div itemscope itemtype=\"http://schema.org/Product\"><span itemprop=\"name\">Widget</span></div></body></html>"
  }'
