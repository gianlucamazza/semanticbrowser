# Machine Learning Models

This directory contains machine learning models used by the Semantic Browser for enhanced semantic processing.

## Overview

## Supported Models

### Named Entity Recognition (NER)

- **Purpose**: Identifies and classifies named entities in text (persons, organizations, locations, etc.)
- **Format**: ONNX models
- **Configuration**: Set `NER_MODEL_PATH` in `.env`
- **Fallback**: Regex-based extraction if no model provided

### Knowledge Graph Inference

- **Purpose**: Performs semantic inference on RDF triples to discover new relationships
- **Format**: ONNX models
- **Configuration**: Set `KG_INFERENCE_MODEL_PATH` in `.env`
- **Fallback**: Rule-based inference if no model provided

## Model Requirements

Models should be in ONNX format for cross-platform compatibility. The system uses the `tract-core` library for inference.

## Adding Models

1. Place ONNX model files in this directory
2. Update `.env` with the appropriate `*_MODEL_PATH` variables
3. Restart the application to load the models

## Performance Notes

- Models are loaded at startup for optimal performance
- Large models may increase memory usage and startup time
- GPU acceleration may be supported in future versions

## Current Status

This is a placeholder directory. ML model integration is implemented but no specific models are included in the repository. Users can add their own ONNX models or use the built-in fallback mechanisms.

## Using Real ONNX Models

This guide explains how to use real ONNX models for Named Entity Recognition (NER) and Knowledge Graph (KG) inference in the Semantic Browser.

## Prerequisites

- ONNX-compatible ML models
- Hugging Face tokenizers (for NER)
- Model optimization tools

## NER Models

### Supported Architectures

The Semantic Browser supports BERT, DistilBERT, and other transformer-based NER models exported to ONNX format.

### Model Preparation

1. **Export from Hugging Face**:
```python
from transformers import AutoTokenizer, AutoModelForTokenClassification
from transformers.onnx import export
import torch

# Load model and tokenizer
model_name = "dbmdz/bert-large-cased-finetuned-conll03-english"
tokenizer = AutoTokenizer.from_pretrained(model_name)
model = AutoModelForTokenClassification.from_pretrained(model_name)

# Export to ONNX
export(
    preprocessor=tokenizer,
    model=model,
    config=model.config,
    opset=13,
    output="ner-model.onnx"
)
```

2. **Optimize the model**:
```bash
# Use onnxruntime-tools for optimization
python -m onnxruntime_tools.optimizer_cli \
  --input ner-model.onnx \
  --output ner-model-opt.onnx \
  --optimization_level 2
```

### Configuration

Set environment variables:

```bash
export NER_MODEL_PATH=/path/to/ner-model.onnx
export NER_TOKENIZER_PATH=/path/to/tokenizer.json
export NER_LABELS_PATH=/path/to/labels.json
```

### Labels Configuration

Create `labels.json` mapping model outputs to entity types:

```json
{
  "O": "O",
  "B-PER": "PERSON",
  "I-PER": "PERSON",
  "B-ORG": "ORGANIZATION",
  "I-ORG": "ORGANIZATION",
  "B-LOC": "LOCATION",
  "I-LOC": "LOCATION",
  "B-MISC": "MISC",
  "I-MISC": "MISC"
}
```

## KG Inference Models

### TransE Model

Translation-based embedding model for link prediction.

**Training with PyTorch**:
```python
import torch
import torch.nn as nn

class TransE(nn.Module):
    def __init__(self, num_entities, num_relations, embedding_dim):
        super().__init__()
        self.entity_embeddings = nn.Embedding(num_entities, embedding_dim)
        self.relation_embeddings = nn.Embedding(num_relations, embedding_dim)

    def forward(self, head, relation, tail):
        head_emb = self.entity_embeddings(head)
        rel_emb = self.relation_embeddings(relation)
        tail_emb = self.entity_embeddings(tail)

        # TransE scoring: -||head + relation - tail||
        score = -torch.norm(head_emb + rel_emb - tail_emb, p=2, dim=-1)
        return score
```

### DistMult Model

Bilinear diagonal model for knowledge graph completion.

```python
class DistMult(nn.Module):
    def __init__(self, num_entities, num_relations, embedding_dim):
        super().__init__()
        self.entity_embeddings = nn.Embedding(num_entities, embedding_dim)
        self.relation_embeddings = nn.Embedding(num_relations, embedding_dim)

    def forward(self, head, relation, tail):
        head_emb = self.entity_embeddings(head)
        rel_emb = self.relation_embeddings(relation)
        tail_emb = self.entity_embeddings(tail)

        # DistMult scoring: sum(head * relation * tail)
        score = torch.sum(head_emb * rel_emb * tail_emb, dim=-1)
        return score
```

### ComplEx Model

Complex-valued embeddings for better expressiveness.

```python
class ComplEx(nn.Module):
    def __init__(self, num_entities, num_relations, embedding_dim):
        super().__init__()
        # embedding_dim should be even for real/imaginary split
        self.entity_embeddings = nn.Embedding(num_entities, embedding_dim * 2)
        self.relation_embeddings = nn.Embedding(num_relations, embedding_dim * 2)

    def forward(self, head, relation, tail):
        head_emb = self.entity_embeddings(head)
        rel_emb = self.relation_embeddings(relation)
        tail_emb = self.entity_embeddings(tail)

        # Split into real and imaginary parts
        head_real, head_imag = head_emb.chunk(2, dim=-1)
        rel_real, rel_imag = rel_emb.chunk(2, dim=-1)
        tail_real, tail_imag = tail_emb.chunk(2, dim=-1)

        # ComplEx scoring
        real_part = head_real * rel_real * tail_real + head_imag * rel_imag * tail_real
        imag_part = head_real * rel_imag * tail_imag + head_imag * rel_real * tail_imag
        score = real_part + imag_part
        return torch.sum(score, dim=-1)
```

### Export to ONNX

```python
import torch.onnx

# Example for TransE
model = TransE(num_entities=1000, num_relations=100, embedding_dim=200)
dummy_input = (torch.randint(0, 1000, (1,)), torch.randint(0, 100, (1,)), torch.randint(0, 1000, (1,)))

torch.onnx.export(
    model,
    dummy_input,
    "kg-model.onnx",
    input_names=['head', 'relation', 'tail'],
    output_names=['score'],
    opset_version=11
)
```

### Configuration

```bash
export KG_INFERENCE_MODEL_PATH=/path/to/kg-model.onnx
export KG_ENTITY_MAPPING_PATH=/path/to/entity_ids.txt
export KG_RELATION_MAPPING_PATH=/path/to/relation_ids.txt
export KG_EMBEDDING_TYPE=TransE
export KG_INFERENCE_CONFIDENCE_THRESHOLD=0.8
export KG_INFERENCE_TOP_K=10
```

### Entity/Relation Mappings

Create `entity_ids.txt` and `relation_ids.txt` with ordered mappings:

```
# Entity IDs (one per line)
http://example.org/Person
http://example.org/Organization
http://example.org/Location
...

# Relation IDs
http://example.org/worksFor
http://example.org/locatedIn
http://example.org/knows
...
```

## Integration in Semantic Browser

### Automatic KG Inference

When KG inference is enabled, the Semantic Browser automatically discovers new relationships:

1. **Entity Extraction**: Parse HTML to extract entities using NER
2. **Triple Insertion**: Store extracted triples in the Knowledge Graph
3. **Inference**: Use ML models to predict missing relationships
4. **Confidence Filtering**: Only add triples above confidence threshold
5. **SPARQL Queries**: Query both explicit and inferred knowledge

### Inference Workflow

```rust
// Example: Inferring worksFor relationships
use semantic_browser::kg::KnowledgeGraph;
use semantic_browser::ml::inference::LinkPredictor;

let kg = KnowledgeGraph::new();
// ... populate KG with explicit triples ...

let predictor = LinkPredictor::new("path/to/kg-model.onnx").await?;
let inferences = predictor.predict_links(&kg, confidence_threshold: 0.8).await?;

// Add inferred triples to KG
for inference in inferences {
    kg.insert(&inference.head, &inference.relation, &inference.tail)?;
}
```

### Supported Inference Types

- **Link Prediction**: Predict missing relationships between known entities
- **Entity Similarity**: Find similar entities for clustering
- **Relation Discovery**: Identify patterns in relationship data
- **Path Finding**: Discover multi-hop relationships

### Performance Considerations

- **Batch Inference**: Process multiple predictions in parallel
- **Caching**: Cache frequent inference results
- **Incremental Updates**: Only re-infer when KG changes significantly
- **Resource Limits**: Configure `KG_INFERENCE_TOP_K` to limit computation

### Monitoring Inference

Enable detailed logging for inference operations:

```bash
RUST_LOG=semantic_browser::ml::inference=debug
```

Metrics available when observability is enabled:
- `semantic_browser_ml_inference_total{model_type="kg_inference"}`
- `semantic_browser_ml_inference_duration_seconds{model_type="kg_inference"}`

## Performance Optimization

### Model Optimization

1. **ONNX Runtime Optimization**:
```bash
# Use onnxruntime-tools
python -c "
import onnxruntime as ort
from onnxruntime_tools import optimizer

optimized_model = optimizer.optimize_model(
    'model.onnx',
    'model-opt.onnx',
    optimization_level=ort.GraphOptimizationLevel.ORT_ENABLE_ALL
)
"
```

2. **Quantization** (optional):
```python
from onnxruntime.quantization import quantize_dynamic, QuantType

quantize_dynamic(
    model_input='model.onnx',
    model_output='model-quant.onnx',
    weight_type=QuantType.QInt8
)
```

### Memory Management

- Use appropriate batch sizes
- Enable model caching for repeated inferences
- Monitor memory usage with `RUST_LOG=debug`

## Troubleshooting

### Common Issues

1. **Model Loading Fails**:
   - Check ONNX opset compatibility
   - Verify input/output tensor shapes
   - Ensure model is exported correctly

2. **Tokenizer Mismatch**:
   - Use the same tokenizer that was used for training
   - Check tokenization consistency

3. **Poor Performance**:
   - Optimize models before deployment
   - Use appropriate batch sizes
   - Consider GPU acceleration if available

### Debugging

Enable debug logging:
```bash
export RUST_LOG=semantic_browser=debug,tract=info
```

Check model information:
```bash
# Use netron or onnx-runtime tools
python -c "
import onnxruntime as ort
session = ort.InferenceSession('model.onnx')
print('Inputs:', [inp.name for inp in session.get_inputs()])
print('Outputs:', [out.name for out in session.get_outputs()])
"
```

## Example Workflow

1. Prepare your trained models
2. Export to ONNX format
3. Optimize for inference
4. Configure environment variables
5. Test with sample data
6. Deploy to production

## References

- [Hugging Face ONNX Export](https://huggingface.co/docs/transformers/serialization)
- [ONNX Runtime Optimization](https://onnxruntime.ai/docs/performance/optimization.html)
- [Knowledge Graph Embeddings Survey](https://arxiv.org/abs/2002.00388)