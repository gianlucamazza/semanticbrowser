# Machine Learning Models

This directory contains machine learning models used by the Semantic Browser for enhanced semantic processing.

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
