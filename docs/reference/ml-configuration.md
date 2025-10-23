# Machine Learning Configuration Guide

Complete guide to configuring ONNX models for NER and Knowledge Graph inference.

## Table of Contents

- [Overview](#overview)
- [Named Entity Recognition (NER)](#named-entity-recognition-ner)
- [Knowledge Graph ML Inference](#knowledge-graph-ml-inference)
- [Model Formats](#model-formats)
- [Troubleshooting](#troubleshooting)

---

## Overview

The Semantic Browser supports ML-based inference through ONNX models for:
1. **Named Entity Recognition (NER)**: Extract entities from text
2. **Knowledge Graph Inference**: Predict missing relationships

All ML features require the `onnx-integration` feature flag:

```bash
cargo build --features onnx-integration
```

---

## Named Entity Recognition (NER)

### Configuration

NER requires three components:

1. **ONNX Model**: The neural network model
2. **Tokenizer**: Hugging Face tokenizer for text preprocessing
3. **Labels**: BIO labels for entity types

### Environment Variables

```bash
# Path to ONNX NER model (REQUIRED)
NER_MODEL_PATH=./models/ner-model.onnx

# Path to tokenizer.json (REQUIRED for ONNX NER)
NER_TOKENIZER_PATH=./models/tokenizer.json

# Path to labels file (OPTIONAL)
NER_LABELS_PATH=./models/labels.txt

# Labels as comma-separated string (ALTERNATIVE to file)
NER_LABELS=B-PER,I-PER,B-ORG,I-ORG,B-LOC,I-LOC,O
```

### Model Requirements

**ONNX Model Format**:
- Input: `input_ids` (tensor of token IDs)
- Output: `logits` (tensor of class probabilities)
- Shape: `[batch_size, sequence_length, num_labels]`

**Tokenizer Format**:
- Standard Hugging Face `tokenizer.json`
- Must be compatible with model's tokenization

**Labels Format** (if using file):
```
B-PER
I-PER
B-ORG
I-ORG
B-LOC
I-LOC
O
```

### Example Setup

**1. Download a pre-trained model**:
```bash
mkdir -p models
cd models

# Download BERT-based NER model (example)
wget https://huggingface.co/dslim/bert-base-NER/resolve/main/model.onnx -O ner-model.onnx
wget https://huggingface.co/dslim/bert-base-NER/resolve/main/tokenizer.json
```

**2. Configure labels**:
```bash
cat > labels.txt << EOF
B-PER
I-PER
B-ORG
I-ORG
B-LOC
I-LOC
B-MISC
I-MISC
O
EOF
```

**3. Set environment variables**:
```bash
export NER_MODEL_PATH=./models/ner-model.onnx
export NER_TOKENIZER_PATH=./models/tokenizer.json
export NER_LABELS_PATH=./models/labels.txt
```

**4. Test**:
```rust
use semantic_browser::annotator::EntityAnnotator;

let annotator = EntityAnnotator::new();
let entities = annotator.annotate("Apple Inc. is based in Cupertino, California.")?;

for entity in entities {
    println!("{}: {} (confidence: {:.2})", entity.label, entity.text, entity.confidence);
}
```

### Fallback Behavior

If NER model is not configured or fails to load:
- **Automatic fallback** to regex-based entity extraction
- No errors thrown
- Warning logged to `tracing`

---

## Knowledge Graph ML Inference

### Configuration

KG ML inference predicts missing relationships using entity/relation embeddings.

### Environment Variables

```bash
# Path to KG inference model (REQUIRED)
KG_INFERENCE_MODEL_PATH=./models/kg-inference-model.onnx

# Embedding type: TransE, DistMult, or ComplEx (DEFAULT: TransE)
KG_EMBEDDING_TYPE=TransE

# Entity embeddings node name in ONNX model (DEFAULT: entity_embeddings)
KG_ENTITY_EMBEDDINGS_NODE=entity_embeddings

# Relation embeddings node name in ONNX model (DEFAULT: relation_embeddings)
KG_RELATION_EMBEDDINGS_NODE=relation_embeddings

# Confidence threshold (0.0-1.0) for predictions (DEFAULT: 0.7)
KG_INFERENCE_CONFIDENCE_THRESHOLD=0.7

# Top-K predictions per entity pair (DEFAULT: 5)
KG_INFERENCE_TOP_K=5

# Sample size for inference (DEFAULT: 100)
KG_INFERENCE_SAMPLE_SIZE=100

# Maximum triples to insert from predictions (DEFAULT: 1000)
KG_INFERENCE_MAX_INSERTS=1000
```

### Model Requirements

**ONNX Model Format**:
- Input: Entity and relation indices
- Output: Score/probability for each candidate triple
- Supports: TransE, DistMult, ComplEx scoring functions

**Embedding Files** (alternative to ONNX):
If using pre-trained embeddings:
```bash
# Entity embeddings mapping file
KG_ENTITY_EMBEDDINGS_PATH=./models/entity_embeddings.txt

# Relation embeddings mapping file
KG_RELATION_EMBEDDINGS_PATH=./models/relation_embeddings.txt
```

### Supported Embedding Models

1. **TransE**: Translation-based embeddings
   - Formula: `h + r ≈ t`
   - Good for: Hierarchical relationships

2. **DistMult**: Bilinear diagonal model
   - Formula: `h^T M_r t`
   - Good for: Symmetric relationships

3. **ComplEx**: Complex-valued embeddings
   - Formula: `Re(h^T M_r conj(t))`
   - Good for: Asymmetric relationships

### Example Setup

**1. Create mock embeddings** (for testing):
```python
import numpy as np
import onnx
from onnx import numpy_helper

# Create mock entity embeddings (100 entities, 50 dimensions)
entity_emb = np.random.randn(100, 50).astype(np.float32)
relation_emb = np.random.randn(20, 50).astype(np.float32)

# Save as ONNX
# (implementation depends on your embedding model)
```

**2. Configure environment**:
```bash
export KG_INFERENCE_MODEL_PATH=./models/kg-inference.onnx
export KG_EMBEDDING_TYPE=TransE
export KG_INFERENCE_CONFIDENCE_THRESHOLD=0.8
export KG_INFERENCE_TOP_K=3
```

**3. Run inference**:
```rust
use semantic_browser::kg::KnowledgeGraph;

let mut kg = KnowledgeGraph::new();

// Insert some known triples
kg.insert("http://example.org/Alice", "http://xmlns.com/foaf/0.1/knows", "http://example.org/Bob")?;

// Run ML-based inference
let inserted_count = kg.ml_inference()?;
println!("Inserted {} predicted triples", inserted_count);
```

### Inference Process

1. **Sample entities**: Random sample of N entities from KG
2. **Generate candidates**: All possible (head, relation, tail) combinations
3. **Score predictions**: Use embedding model to score candidates
4. **Filter by confidence**: Keep only predictions above threshold
5. **Top-K selection**: Take top K predictions per entity pair
6. **Deduplicate**: Remove existing triples
7. **Insert**: Add new triples to KG (up to max limit)

### Safety Limits

- **CONFIDENCE_THRESHOLD**: Prevents low-quality predictions
- **TOP_K**: Limits predictions per entity pair
- **SAMPLE_SIZE**: Prevents processing entire large KGs
- **MAX_INSERTS**: Prevents KG explosion

---

## Model Formats

### ONNX Model Export

**From PyTorch**:
```python
import torch

# Your NER model
model = YourNERModel()

# Dummy input
dummy_input = torch.randint(0, 30000, (1, 128))

# Export to ONNX
torch.onnx.export(
    model,
    dummy_input,
    "ner-model.onnx",
    input_names=['input_ids'],
    output_names=['logits'],
    dynamic_axes={
        'input_ids': {0: 'batch_size', 1: 'sequence_length'},
        'logits': {0: 'batch_size', 1: 'sequence_length'}
    }
)
```

**From TensorFlow**:
```python
import tf2onnx

# Convert SavedModel to ONNX
python -m tf2onnx.convert \
    --saved-model ./tf_model \
    --output ner-model.onnx
```

### Tokenizer Export

**From Hugging Face Transformers**:
```python
from transformers import AutoTokenizer

tokenizer = AutoTokenizer.from_pretrained("bert-base-uncased")
tokenizer.save_pretrained("./models/")
# This creates tokenizer.json
```

---

## Troubleshooting

### NER Model Not Loading

**Problem**: Model fails to load or gives errors

**Solutions**:
1. Verify ONNX file is valid:
   ```bash
   python -c "import onnx; onnx.checker.check_model('ner-model.onnx')"
   ```

2. Check file permissions:
   ```bash
   ls -l models/ner-model.onnx
   ```

3. Verify path is absolute or relative to working directory:
   ```bash
   realpath ./models/ner-model.onnx
   ```

4. Enable debug logging:
   ```bash
   RUST_LOG=debug cargo run
   ```

### Tokenizer Errors

**Problem**: Tokenization fails or produces wrong tokens

**Solutions**:
1. Ensure tokenizer.json format is correct
2. Verify tokenizer matches model's training tokenizer
3. Check for special tokens configuration

### Low NER Confidence

**Problem**: All entities have low confidence scores

**Solutions**:
1. Verify model is appropriate for your text domain
2. Check if input text needs preprocessing
3. Consider fine-tuning model on your domain

### KG Inference Not Working

**Problem**: No triples inserted from ML inference

**Solutions**:
1. Lower confidence threshold:
   ```bash
   KG_INFERENCE_CONFIDENCE_THRESHOLD=0.5
   ```

2. Increase sample size:
   ```bash
   KG_INFERENCE_SAMPLE_SIZE=500
   ```

3. Check if KG has enough entities:
   ```bash
   # KG needs at least 2 entities for inference
   ```

4. Verify embedding dimensions match

### Performance Issues

**Problem**: Inference is too slow

**Solutions**:
1. Reduce sample size:
   ```bash
   KG_INFERENCE_SAMPLE_SIZE=50
   ```

2. Use smaller models
3. Enable ONNX optimization:
   ```rust
   // Models are automatically optimized with .into_optimized()
   ```

4. Use GPU acceleration (if available):
   - Compile tract-onnx with CUDA support
   - Set appropriate execution providers

---

## Best Practices

1. **Start with pre-trained models**: Use Hugging Face models
2. **Test with small datasets first**: Validate before scaling
3. **Monitor confidence scores**: Adjust thresholds based on results
4. **Version your models**: Track model versions with embeddings
5. **Regular retraining**: Update models as KG grows
6. **Backup before inference**: ML inference modifies KG
7. **Use appropriate thresholds**: Balance precision vs. recall

---

## References

- [ONNX Runtime Documentation](https://onnxruntime.ai/docs/)
- [Hugging Face Transformers](https://huggingface.co/docs/transformers/)
- [Tract ONNX](https://github.com/sonos/tract)
- [Knowledge Graph Embeddings Survey](https://arxiv.org/abs/1503.00759)

---

**Last Updated**: 2025-01-23
**Status**: Production Ready
**Maintainer**: Technical Team
