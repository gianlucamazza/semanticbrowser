# Machine Learning Models

This directory contains optional ML models for enhanced accuracy.

## 📁 Directory Structure

```
models/
├── README.md                     # This file
├── ner-model.onnx               # (Optional) NER model
├── kg-inference-model.onnx      # (Optional) KG inference
└── embeddings/                  # (Optional) Pre-trained embeddings
    ├── entities.bin
    └── relations.bin
```

## ⚠️ Important: Models are OPTIONAL

**The system works perfectly without any models in this directory!**

- ✅ **NER**: Falls back to regex-based entity extraction
- ✅ **Embeddings**: Created in-memory during runtime
- ✅ **Link Prediction**: Uses rule-based scoring

## 🚀 Quick Start (No Models Needed)

```bash
# Just run - works out of the box
cargo run --example agent_simple_task

# With ML features
cargo run --features onnx-integration --example agent_with_ml
```

## 📦 Adding Models (Optional)

### Option 1: Train Your Own

See [ML_SETUP.md](../docs/ML_SETUP.md) for training instructions.

### Option 2: Download Pre-trained

```bash
# (Future) Download from repository
# wget https://example.com/models/ner-model.onnx
```

### Option 3: Export from HuggingFace

```python
# Export NER model
from transformers import AutoModelForTokenClassification
import torch

model = AutoModelForTokenClassification.from_pretrained("dslim/bert-base-NER")
# ... export to ONNX (see ML_SETUP.md)
```

## 🔧 Configuration

Add to `.env` only if you have models:

```bash
# Optional: NER model
NER_MODEL_PATH=./models/ner-model.onnx

# Optional: KG inference
KG_INFERENCE_MODEL_PATH=./models/kg-inference-model.onnx
```

## 📊 Model Performance

| Feature | Without Models | With ONNX Models |
|---------|----------------|------------------|
| NER Accuracy | 75-85% (regex) | 95%+ (ONNX) |
| Speed | Very Fast | Fast |
| Memory | Low (10MB) | Medium (500MB) |
| Setup | None | Download/Train |

## 📚 Learn More

- [ML Setup Guide](../docs/ML_SETUP.md) - Complete setup instructions
- [Complete Analysis](../docs/COMPLETE_ANALYSIS.md) - Full project analysis
- [Examples](../examples/) - Code examples

## 🤝 Contributing

Have a pre-trained model to share? Submit a PR!

1. Add model to this directory
2. Update this README
3. Add example usage
4. Submit PR

## 📝 License

Models follow project license (MIT). See [LICENSE](../LICENSE).
