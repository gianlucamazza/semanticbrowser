#!/bin/bash
# Example: Query the Knowledge Graph using SPARQL

# SELECT query
echo "=== SELECT Query ==="
curl -X POST http://localhost:3000/query \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer secret" \
  -d '{
    "query": "SELECT * WHERE { ?s ?p ?o } LIMIT 10"
  }'

echo -e "\n\n=== INSERT Update ==="
# INSERT update
curl -X POST http://localhost:3000/query \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer secret" \
  -d '{
    "query": "INSERT DATA { <http://example.org/person1> <http://xmlns.com/foaf/0.1/name> \"Alice\" }"
  }'

echo -e "\n\n=== Verify INSERT ==="
# Verify the insert
curl -X POST http://localhost:3000/query \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer secret" \
  -d '{
    "query": "SELECT ?name WHERE { <http://example.org/person1> <http://xmlns.com/foaf/0.1/name> ?name }"
  }'
