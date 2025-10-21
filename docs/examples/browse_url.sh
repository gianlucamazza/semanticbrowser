#!/bin/bash
# Example: Browse a URL and extract semantic information

curl -X POST http://localhost:3000/browse \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer secret" \
  -d '{
    "url": "https://wikipedia.com",
    "query": "extract main content"
  }'
