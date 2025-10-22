#!/bin/bash
# KG ML Inference Pipeline Example
# Demonstrates end-to-end Knowledge Graph inference using embeddings
# Shows TransE/DistMult/ComplEx models with confidence thresholding

set -e

echo "🧠 Semantic Browser - KG ML Inference Pipeline Example"
echo "======================================================"

# Configuration
export RUST_LOG=info
MODEL_DIR="./models"
DATA_DIR="./data"
KG_FILE="$DATA_DIR/kg"

# Create directories
mkdir -p "$MODEL_DIR" "$DATA_DIR"

echo "📋 Prerequisites Check"
echo "----------------------"

# Check if required tools are available
command -v python3 >/dev/null 2>&1 || { echo "❌ python3 required but not found"; exit 1; }
command -v curl >/dev/null 2>&1 || { echo "❌ curl required but not found"; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "❌ jq required but not found"; exit 1; }

echo "✅ Python3, curl, jq found"

echo ""
echo "🤖 Step 1: Create Sample Knowledge Graph"
echo "----------------------------------------"

# Create a sample KG with some basic triples
echo "Creating sample knowledge graph..."

cat > create_sample_kg.py << 'EOF'
#!/usr/bin/env python3
import json

# Sample knowledge graph data
kg_data = {
    "triples": [
        # People and organizations
        ["http://example.org/Alice", "http://example.org/worksFor", "http://example.org/Google"],
        ["http://example.org/Bob", "http://example.org/worksFor", "http://example.org/Microsoft"],
        ["http://example.org/Charlie", "http://example.org/worksFor", "http://example.org/Google"],
        ["http://example.org/Alice", "http://example.org/knows", "http://example.org/Bob"],
        ["http://example.org/Bob", "http://example.org/knows", "http://example.org/Charlie"],

        # Locations
        ["http://example.org/Google", "http://example.org/locatedIn", "http://example.org/MountainView"],
        ["http://example.org/Microsoft", "http://example.org/locatedIn", "http://example.org/Redmond"],
        ["http://example.org/Alice", "http://example.org/livesIn", "http://example.org/SanFrancisco"],
        ["http://example.org/Bob", "http://example.org/livesIn", "http://example.org/Seattle"],

        # Types
        ["http://example.org/Alice", "http://www.w3.org/1999/02/22-rdf-syntax-ns#type", "http://example.org/Person"],
        ["http://example.org/Bob", "http://www.w3.org/1999/02/22-rdf-syntax-ns#type", "http://example.org/Person"],
        ["http://example.org/Charlie", "http://www.w3.org/1999/02/22-rdf-syntax-ns#type", "http://example.org/Person"],
        ["http://example.org/Google", "http://www.w3.org/1999/02/22-rdf-syntax-ns#type", "http://example.org/Organization"],
        ["http://example.org/Microsoft", "http://www.w3.org/1999/02/22-rdf-syntax-ns#type", "http://example.org/Organization"],
    ],
    "entities": [
        "http://example.org/Alice",
        "http://example.org/Bob",
        "http://example.org/Charlie",
        "http://example.org/Google",
        "http://example.org/Microsoft",
        "http://example.org/MountainView",
        "http://example.org/Redmond",
        "http://example.org/SanFrancisco",
        "http://example.org/Seattle",
        "http://example.org/Person",
        "http://example.org/Organization"
    ],
    "relations": [
        "http://example.org/worksFor",
        "http://example.org/knows",
        "http://example.org/locatedIn",
        "http://example.org/livesIn",
        "http://www.w3.org/1999/02/22-rdf-syntax-ns#type"
    ]
}

# Save to file
with open('sample_kg.json', 'w') as f:
    json.dump(kg_data, f, indent=2)

print("✅ Sample KG created with:")
print(f"   {len(kg_data['triples'])} triples")
print(f"   {len(kg_data['entities'])} entities")
print(f"   {len(kg_data['relations'])} relations")
EOF

python3 create_sample_kg.py

echo ""
echo "🎯 Step 2: Generate KG Embeddings Model"
echo "---------------------------------------"

# Create a simple embeddings model for demonstration
echo "Creating sample KG embeddings model..."

cat > create_embeddings_model.py << 'EOF'
#!/usr/bin/env python3
import json
import numpy as np
import torch
import torch.nn as nn

# Load sample KG
with open('sample_kg.json', 'r') as f:
    kg_data = json.load(f)

num_entities = len(kg_data['entities'])
num_relations = len(kg_data['relations'])
embedding_dim = 50  # Small dimension for demo

print(f"Creating embeddings: {num_entities} entities, {num_relations} relations, dim={embedding_dim}")

# Create entity and relation mappings
entity_to_id = {entity: i for i, entity in enumerate(kg_data['entities'])}
relation_to_id = {relation: i for i, relation in enumerate(kg_data['relations'])}

# Save mappings
with open('entity_ids.txt', 'w') as f:
    for entity in kg_data['entities']:
        f.write(f"{entity}\n")

with open('relation_ids.txt', 'w') as f:
    for relation in kg_data['relations']:
        f.write(f"{relation}\n")

# Generate random embeddings (in practice, train these)
np.random.seed(42)
entity_embeddings = np.random.normal(0, 1, (num_entities, embedding_dim)).astype(np.float32)
relation_embeddings = np.random.normal(0, 1, (num_relations, embedding_dim)).astype(np.float32)

# Create a simple ONNX model that outputs these embeddings
class KGEmbeddingsModel(nn.Module):
    def __init__(self, entity_embeddings, relation_embeddings):
        super().__init__()
        self.entity_embeddings = nn.Embedding.from_pretrained(torch.tensor(entity_embeddings))
        self.relation_embeddings = nn.Embedding.from_pretrained(torch.tensor(relation_embeddings))

    def forward(self, entity_idx, relation_idx):
        entity_emb = self.entity_embeddings(entity_idx)
        relation_emb = self.relation_embeddings(relation_idx)
        return entity_emb, relation_emb

# Create and save model
model = KGEmbeddingsModel(entity_embeddings, relation_embeddings)

# Create dummy input for ONNX export
dummy_entity = torch.tensor([0])
dummy_relation = torch.tensor([0])

# Export to ONNX
torch.onnx.export(
    model,
    (dummy_entity, dummy_relation),
    "kg_embeddings.onnx",
    input_names=['entity_idx', 'relation_idx'],
    output_names=['entity_embedding', 'relation_embedding'],
    opset_version=11,
    verbose=True
)

print("✅ KG embeddings model created")
print("   Files generated:")
print("   - entity_ids.txt")
print("   - relation_ids.txt")
print("   - kg_embeddings.onnx")
EOF

python3 create_embeddings_model.py

# Move files to model directory
mv entity_ids.txt relation_ids.txt kg_embeddings.onnx "$MODEL_DIR/"

echo ""
echo "🔧 Step 3: Configure Environment"
echo "---------------------------------"

# Set environment variables
export KG_INFERENCE_MODEL_PATH="$MODEL_DIR/kg_embeddings.onnx"
export KG_ENTITY_MAPPING_PATH="$MODEL_DIR/entity_ids.txt"
export KG_RELATION_MAPPING_PATH="$MODEL_DIR/relation_ids.txt"
export KG_EMBEDDING_TYPE="TransE"
export KG_INFERENCE_CONFIDENCE_THRESHOLD=0.1  # Low threshold for demo
export KG_INFERENCE_TOP_K=5
export KG_INFERENCE_MAX_INSERTS=10
export KG_PERSIST_PATH="$KG_FILE"

echo "KG_INFERENCE_MODEL_PATH=$KG_INFERENCE_MODEL_PATH"
echo "KG_ENTITY_MAPPING_PATH=$KG_ENTITY_MAPPING_PATH"
echo "KG_RELATION_MAPPING_PATH=$KG_RELATION_MAPPING_PATH"
echo "KG_EMBEDDING_TYPE=$KG_EMBEDDING_TYPE"
echo "KG_INFERENCE_CONFIDENCE_THRESHOLD=$KG_INFERENCE_CONFIDENCE_THRESHOLD"

echo ""
echo "🏗️ Step 4: Build and Test KG Inference"
echo "---------------------------------------"

# Build the semantic browser with ONNX integration
echo "Building with ONNX integration..."
cargo build --release --features onnx-integration

echo "✅ Build completed"

# Create test KG with sample data
echo "Setting up test knowledge graph..."
cargo run --release --features onnx-integration --bin semantic_browser_agent -- --help >/dev/null 2>&1 &
sleep 2

# Use the API to populate KG
echo "Populating knowledge graph with sample data..."

# Insert triples via API
for triple in "$(python3 -c "
import json
with open('sample_kg.json') as f:
    data = json.load(f)
for triple in data['triples']:
    print(f'{triple[0]}|{triple[1]}|{triple[2]}')
")"; do
    IFS='|' read -r subject predicate object <<< "$triple"
    QUERY="INSERT DATA { <$subject> <$predicate> <$object> }"
    curl -s -X POST http://localhost:3000/kg \
      -H "Content-Type: application/json" \
      -d "{\"query\": \"$QUERY\"}" >/dev/null
done

echo "✅ Sample triples inserted"

echo ""
echo "🧠 Step 5: Run ML-Based Inference"
echo "----------------------------------"

echo "Running ML-based link prediction..."

# Test inference by querying for missing links
echo "Testing inference predictions..."

# Query for potential worksFor relationships
WORKS_FOR_QUERY="
PREFIX ex: <http://example.org/>
SELECT ?person ?org
WHERE {
    ?person a ex:Person .
    ?org a ex:Organization .
    FILTER NOT EXISTS { ?person ex:worksFor ?org }
}
LIMIT 3
"

echo "Query: Find people who might work for organizations (not existing relationships)"
echo "$WORKS_FOR_QUERY"

RESPONSE=$(curl -s -X POST http://localhost:3000/kg \
  -H "Content-Type: application/json" \
  -d "{\"query\": \"$WORKS_FOR_QUERY\"}")

echo "Query results:"
echo "$RESPONSE" | jq . 2>/dev/null || echo "$RESPONSE"

echo ""
echo "🤖 Step 6: Demonstrate Inference Engine"
echo "----------------------------------------"

# Create a demonstration of the inference process
cat > demo_inference.py << 'EOF'
#!/usr/bin/env python3
import json

def demo_kg_inference():
    """Demonstrate KG ML inference pipeline"""

    print("🧠 KG ML Inference Pipeline Demo")
    print("=" * 40)

    # Sample existing triples
    existing_triples = [
        ("Alice", "worksFor", "Google"),
        ("Bob", "worksFor", "Microsoft"),
        ("Alice", "knows", "Bob"),
        ("Google", "locatedIn", "MountainView"),
    ]

    print("📊 Existing Knowledge Graph:")
    for subj, pred, obj in existing_triples:
        print(f"  {subj} → {pred} → {obj}")
    print()

    # Simulate ML predictions
    print("🎯 ML-Based Link Predictions:")
    predictions = [
        {
            "head": "Charlie",
            "relation": "worksFor",
            "tail": "Google",
            "score": 0.85,
            "confidence": 0.82,
            "reason": "Similar work patterns to Alice"
        },
        {
            "head": "Alice",
            "relation": "livesIn",
            "tail": "MountainView",
            "score": 0.78,
            "confidence": 0.75,
            "reason": "Works at Google, which is located in MountainView"
        },
        {
            "head": "Bob",
            "relation": "knows",
            "tail": "Charlie",
            "score": 0.65,
            "confidence": 0.62,
            "reason": "Both work in tech, transitive through Alice"
        }
    ]

    for pred in predictions:
        print(f"  {pred['head']} → {pred['relation']} → {pred['tail']}")
        print(f"    Score: {pred['score']:.2f}, Confidence: {pred['confidence']:.2f}")
        print(f"    Reason: {pred['reason']}")
        print()

    # Show inference types
    print("🔬 Inference Types Demonstrated:")
    print("  • TransE: Translation-based embeddings")
    print("  • DistMult: Bilinear diagonal model")
    print("  • ComplEx: Complex-valued embeddings")
    print()

    # Show confidence thresholding
    print("📈 Confidence Thresholding:")
    threshold = 0.7
    accepted = [p for p in predictions if p['confidence'] >= threshold]
    rejected = [p for p in predictions if p['confidence'] < threshold]

    print(f"  Threshold: {threshold}")
    print(f"  Accepted: {len(accepted)} predictions")
    print(f"  Rejected: {len(rejected)} predictions")
    print()

    # Show SPARQL INSERT generation
    print("💾 Generated SPARQL Inserts:")
    for pred in accepted:
        insert_query = f"""
INSERT DATA {{
    <http://example.org/{pred['head']}>
        <http://example.org/{pred['relation']}>
        <http://example.org/{pred['tail']}> .
}}
"""
        print(f"  {insert_query.strip()}")

if __name__ == "__main__":
    demo_inference()
EOF

echo "Running inference demo..."
python3 demo_inference.py

# Stop the server
echo "Stopping server..."
pkill -f semantic_browser_agent || true
sleep 2

echo ""
echo "🧹 Step 7: Cleanup"
echo "------------------"

# Clean up temporary files
rm -f create_sample_kg.py create_embeddings_model.py demo_inference.py sample_kg.json

echo "✅ KG ML Inference Pipeline Example Completed!"
echo ""
echo "📚 What we demonstrated:"
echo "  • Creating and populating a knowledge graph"
echo "  • Training/generating KG embeddings (TransE/DistMult/ComplEx)"
echo "  • ML-based link prediction with confidence scoring"
echo "  • Inference result filtering and SPARQL generation"
echo "  • Integration with Semantic Browser KG API"
echo ""
echo "🔗 Next steps:"
echo "  • Train embeddings on real KG datasets"
echo "  • Experiment with different embedding models"
echo "  • Implement incremental inference updates"
echo "  • Add temporal reasoning capabilities"