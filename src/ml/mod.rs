//! Machine Learning module for Knowledge Graph inference
//!
//! Best practices 2025:
//! - Use tract-onnx for ONNX model inference
//! - Support multiple embedding models (TransE, DistMult, ComplEx)
//! - Provide confidence scores for predictions
//! - Enable/disable via feature flag for optional dependency

pub mod embeddings;
pub mod inference;

pub use embeddings::{EmbeddingModel, EmbeddingType};
pub use inference::{LinkPredictor, PredictionResult};
