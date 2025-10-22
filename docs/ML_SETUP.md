# 🧠 ML/AI Integration Setup Guide

This guide explains how to set up and use the machine learning and AI features in Semantic Browser.

## 📋 Overview

Semantic Browser includes advanced ML/AI capabilities:

- **Knowledge Graph Embeddings** - Semantic understanding of web content
- **Link Prediction** - AI predictions for unknown relationships
- **Named Entity Recognition (NER)** - Extract entities from web pages
- **Multi-Provider LLM Support** - OpenAI, Anthropic, and Ollama

## 🚀 Quick Start

### Option 1: Local LLM (Recommended for Development)

**Setup Ollama** (5 minutes):

```bash
# 1. Install Ollama
brew install ollama  # macOS
# or visit https://ollama.ai for other platforms

# 2. Start Ollama service
ollama serve &

# 3. Pull a model (choose one)
ollama pull llama3:8b      # Fast, good quality (4.7GB)
ollama pull llama3:70b     # High quality, slower (40GB)
ollama pull mistral        # Fast, lighter (4.1GB)

# 4. Verify installation
ollama list
```

**Run agent with Ollama**:

```bash
# Set environment variables
export LLM_PROVIDER=ollama
export OLLAMA_API_URL=http://localhost:11434
export OLLAMA_MODEL=llama3:8b

# Run example
cargo run --example agent_simple_task
```

### Option 2: OpenAI (Cloud-based, High Quality)

**Setup OpenAI**:

```bash
# Get API key from https://platform.openai.com/api-keys
export OPENAI_API_KEY=sk_test_...

# Run agent with OpenAI
OPENAI_API_KEY=$YOUR_KEY cargo run --features llm-openai --example agent_openai_example
```

### Option 3: Anthropic Claude (Cloud-based, Excellent)

**Setup Anthropic**:

```bash
# Get API key from https://console.anthropic.com/
export ANTHROPIC_API_KEY=sk-ant-...

# Run agent with Claude
ANTHROPIC_API_KEY=$YOUR_KEY cargo run --features llm-anthropic --example agent_anthropic_example
```

## 🔧 Knowledge Graph & Embeddings

### Built-in Embeddings (No Extra Setup)

The system works out-of-the-box with local embeddings:

```rust
use semantic_browser::ml::embeddings::{EmbeddingModel, KGEmbedding};

// Create embeddings locally
let embedding = KGEmbedding::new(EmbeddingModel::TransE, 128);

// Train on triples
let triple = ("subject", "predicate", "object");
// ... training code
```

**Embedding Models Available**:
- **TransE** - Translational embeddings (fast, good for simple relationships)
- **DistMult** - Diagonal bilinear (good for symmetric relations)
- **ComplEx** - Complex-valued embeddings (best for complex relationships)

### ONNX Integration (Optional)

For pre-trained NER and embedding models:

```bash
# Enable ONNX support
cargo build --features onnx-integration

# Create models directory
mkdir -p models/
```

## 🤖 Agent Configuration

### Environment Variables

```bash
# LLM Provider Selection
export LLM_PROVIDER=ollama              # ollama | openai | anthropic
export LLM_MODEL=llama3:8b              # Model name/version

# Ollama Configuration
export OLLAMA_API_URL=http://localhost:11434
export OLLAMA_MODEL=llama3:8b

# OpenAI Configuration
export OPENAI_API_KEY=sk_test_...
export OPENAI_MODEL=gpt-3.5-turbo       # gpt-3.5-turbo | gpt-4

# Anthropic Configuration
export ANTHROPIC_API_KEY=sk-ant-...
export ANTHROPIC_MODEL=claude-3-sonnet  # claude-3-opus | claude-3-sonnet | claude-3-haiku

# Agent Control
export AGENT_MAX_ITERATIONS=10
export AGENT_TIMEOUT_SECS=300
export AGENT_DEBUG=true                 # Enable debug logging

# JWT & Security
export JWT_SECRET=your-secret-here
```

### LLMConfig Options

```rust
use semantic_browser::llm::LLMConfig;

let config = LLMConfig {
    model: "llama3:8b".to_string(),
    temperature: 0.7,           // 0.0-1.0 (lower = more deterministic)
    max_tokens: Some(2048),
    top_p: Some(0.9),
    presence_penalty: Some(0.0),
    frequency_penalty: Some(0.0),
};
```

## 📊 ML Features in Detail

### 1. Knowledge Graph Integration

```rust
use semantic_browser::kg::KnowledgeGraph;

// Insert semantic data
kg.insert_triple(
    "https://example.com/article",
    "http://schema.org/author",
    "John Doe"
)?;

// Query with SPARQL
let results = kg.query_sparql(
    "SELECT ?author WHERE { ?article <http://schema.org/author> ?author }"
)?;
```

### 2. Link Prediction

```rust
use semantic_browser::ml::inference::LinkPredictor;

// Predict missing relationships
let predictor = LinkPredictor::new(embedding, 0.7);

// Predict tail entity
let predictions = predictor.predict_tail("company_A", "has_investor")?;
// Returns: vec![("investor_B", 0.85), ("investor_C", 0.72), ...]

// Predict head entity
let predictions = predictor.predict_head("has_investor", "investor_B")?;

// Predict relationship
let predictions = predictor.predict_relation("company_A", "investor_B")?;
```

### 3. Named Entity Recognition (NER)

```rust
use semantic_browser::ml::ner::NERModel;

// Extract entities (with regex fallback)
let model = NERModel::new(None);  // None = use regex fallback
let entities = model.extract_entities("John Doe works at Google")?;
// Returns: vec![Entity { text: "John Doe", entity_type: "PERSON" }, ...]
```

## 🔍 Troubleshooting

### Ollama Connection Issues

**Problem**: "Failed to connect to Ollama"

```bash
# Check if Ollama is running
curl http://localhost:11434/api/tags

# If not running, start it
ollama serve

# Check model availability
ollama list

# If model not found, pull it
ollama pull llama3:8b
```

### OpenAI/Anthropic API Errors

**Problem**: "Invalid API key"

```bash
# Verify API key is set
echo $OPENAI_API_KEY

# Test API key manually
curl https://api.openai.com/v1/models \
  -H "Authorization: Bearer $OPENAI_API_KEY"
```

### Memory/Performance Issues

**Problem**: "Agent running out of memory"

```rust
// Reduce token usage
let config = LLMConfig {
    max_tokens: Some(1024),  // Reduce from 2048
    ..Default::default()
};

// Use smaller model
export OLLAMA_MODEL=mistral  // Lighter than llama3:8b
```

## 📈 Performance Optimization

### LLM Provider Comparison

| Provider | Speed | Quality | Cost | Setup |
|----------|-------|---------|------|-------|
| **Ollama** | Fast | Good | Free | Local |
| **OpenAI GPT-3.5** | Medium | Excellent | $$ | Cloud |
| **Anthropic Claude** | Medium | Excellent | $$ | Cloud |

## 🔐 Security Considerations

1. **API Keys**: Never commit `.env` files with API keys
2. **Rate Limiting**: OpenAI/Anthropic have rate limits
3. **Token Monitoring**: Track API usage for cost control

## 📚 Examples

### Simple Agent Task

```bash
cargo run --example agent_simple_task
```

### Agent with Browser Automation

```bash
OPENAI_API_KEY=sk_... cargo run --features llm-openai,browser-automation \
  --example agent_browser_example
```

### Web Workflow

```bash
cargo run --features browser-automation --example workflow_example
```

## 🎓 Next Steps

1. ✅ Start with Ollama (local, free)
2. ✅ Run basic agent example
3. ✅ Explore browser automation with agents
4. ✅ Build multi-step workflows
5. ✅ Integrate with your application

---

**Version**: 1.0  
**Last Updated**: 2025-10-22  
**Status**: Production-Ready ✅
