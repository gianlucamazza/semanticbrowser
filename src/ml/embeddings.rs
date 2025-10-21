//! Knowledge Graph Embedding Models
//!
//! Supports:
//! - TransE: Translation-based embeddings (Bordes et al., 2013)
//! - DistMult: Bilinear diagonal model (Yang et al., 2015)
//! - ComplEx: Complex-valued embeddings (Trouillon et al., 2016)
//!
//! Best practices 2025:
//! - Use ONNX models via tract for inference
//! - Provide embedding extraction and similarity computation
//! - Support batch processing for efficiency
//! - Enable confidence scoring for predictions

use std::collections::HashMap;

#[cfg(feature = "onnx-integration")]
use std::path::Path;
#[cfg(feature = "onnx-integration")]
use tract_core::prelude::*;
#[cfg(feature = "onnx-integration")]
use tract_onnx::prelude::InferenceModelExt;

#[cfg(feature = "onnx-integration")]
type TractSimplePlan = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

/// Type of embedding model
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingType {
    /// TransE: h + r ≈ t (translation model)
    TransE,
    /// DistMult: <h, r, t> (bilinear model)
    DistMult,
    /// ComplEx: Complex-valued embeddings
    ComplEx,
}

/// Knowledge Graph embedding model
#[allow(dead_code)]
pub struct EmbeddingModel {
    /// Type of embedding
    pub embedding_type: EmbeddingType,
    /// Embedding dimension
    pub dimension: usize,
    /// Entity ID to index mapping
    pub entity_to_idx: HashMap<String, usize>,
    /// Relation ID to index mapping
    pub relation_to_idx: HashMap<String, usize>,
    /// Entity embeddings (num_entities x dimension)
    #[cfg(feature = "onnx-integration")]
    entity_embeddings: Option<Tensor>,
    /// Relation embeddings (num_relations x dimension)
    #[cfg(feature = "onnx-integration")]
    relation_embeddings: Option<Tensor>,
    /// ONNX model for inference
    #[cfg(feature = "onnx-integration")]
    model: Option<TractSimplePlan>,
}

impl EmbeddingModel {
    /// Create new embedding model from ONNX file
    #[cfg(feature = "onnx-integration")]
    pub fn from_onnx(
        path: impl AsRef<Path>,
        embedding_type: EmbeddingType,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let model = tract_onnx::onnx().model_for_path(path)?.into_optimized()?.into_runnable()?;

        // Note: In production, you'd extract entity/relation mappings from model metadata
        // For now, we create empty mappings that will be populated during training/loading
        Ok(Self {
            embedding_type,
            dimension: 100, // Default, should be loaded from model
            entity_to_idx: HashMap::new(),
            relation_to_idx: HashMap::new(),
            entity_embeddings: None,
            relation_embeddings: None,
            model: Some(model),
        })
    }

    /// Create a simple in-memory model (for testing)
    pub fn new_simple(embedding_type: EmbeddingType, dimension: usize) -> Self {
        Self {
            embedding_type,
            dimension,
            entity_to_idx: HashMap::new(),
            relation_to_idx: HashMap::new(),
            #[cfg(feature = "onnx-integration")]
            entity_embeddings: None,
            #[cfg(feature = "onnx-integration")]
            relation_embeddings: None,
            #[cfg(feature = "onnx-integration")]
            model: None,
        }
    }

    /// Add entity to the model
    pub fn add_entity(&mut self, entity: &str) -> usize {
        let idx = self.entity_to_idx.len();
        *self.entity_to_idx.entry(entity.to_string()).or_insert(idx)
    }

    /// Add relation to the model
    pub fn add_relation(&mut self, relation: &str) -> usize {
        let idx = self.relation_to_idx.len();
        *self.relation_to_idx.entry(relation.to_string()).or_insert(idx)
    }

    /// Get entity index
    pub fn get_entity_idx(&self, entity: &str) -> Option<usize> {
        self.entity_to_idx.get(entity).copied()
    }

    /// Get relation index
    pub fn get_relation_idx(&self, relation: &str) -> Option<usize> {
        self.relation_to_idx.get(relation).copied()
    }

    /// Compute score for triple (head, relation, tail) based on embedding type
    pub fn score_triple(&self, head_emb: &[f32], rel_emb: &[f32], tail_emb: &[f32]) -> f32 {
        match self.embedding_type {
            EmbeddingType::TransE => self.score_transe(head_emb, rel_emb, tail_emb),
            EmbeddingType::DistMult => self.score_distmult(head_emb, rel_emb, tail_emb),
            EmbeddingType::ComplEx => self.score_complex(head_emb, rel_emb, tail_emb),
        }
    }

    /// TransE scoring: -||h + r - t||
    fn score_transe(&self, head: &[f32], relation: &[f32], tail: &[f32]) -> f32 {
        let mut distance = 0.0;
        for i in 0..self.dimension {
            let diff = head[i] + relation[i] - tail[i];
            distance += diff * diff;
        }
        -distance.sqrt()
    }

    /// DistMult scoring: <h, r, t> = sum(h_i * r_i * t_i)
    fn score_distmult(&self, head: &[f32], relation: &[f32], tail: &[f32]) -> f32 {
        let mut score = 0.0;
        for i in 0..self.dimension {
            score += head[i] * relation[i] * tail[i];
        }
        score
    }

    /// ComplEx scoring (simplified real part)
    /// Full ComplEx uses complex numbers, this is a simplified version
    fn score_complex(&self, head: &[f32], relation: &[f32], tail: &[f32]) -> f32 {
        // Simplified: treat first half as real, second half as imaginary
        let mid = self.dimension / 2;
        let mut score_real = 0.0;
        let mut score_imag = 0.0;

        for i in 0..mid {
            // Real part
            score_real += head[i] * relation[i] * tail[i];
            // Imaginary part
            if i + mid < self.dimension {
                score_imag += head[i + mid] * relation[i + mid] * tail[i + mid];
            }
        }

        score_real + score_imag
    }

    /// Get number of entities
    pub fn num_entities(&self) -> usize {
        self.entity_to_idx.len()
    }

    /// Get number of relations
    pub fn num_relations(&self) -> usize {
        self.relation_to_idx.len()
    }

    /// Get embedding dimension
    pub fn dimension(&self) -> usize {
        self.dimension
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedding_model_creation() {
        let model = EmbeddingModel::new_simple(EmbeddingType::TransE, 100);
        assert_eq!(model.dimension, 100);
        assert_eq!(model.embedding_type, EmbeddingType::TransE);
    }

    #[test]
    fn test_entity_relation_indexing() {
        let mut model = EmbeddingModel::new_simple(EmbeddingType::DistMult, 50);

        let idx1 = model.add_entity("http://ex.org/Person");
        let idx2 = model.add_entity("http://ex.org/Organization");
        let rel_idx = model.add_relation("http://ex.org/worksFor");

        assert_eq!(idx1, 0);
        assert_eq!(idx2, 1);
        assert_eq!(rel_idx, 0);

        assert_eq!(model.get_entity_idx("http://ex.org/Person"), Some(0));
        assert_eq!(model.get_relation_idx("http://ex.org/worksFor"), Some(0));
        assert_eq!(model.get_entity_idx("nonexistent"), None);
    }

    #[test]
    fn test_transe_scoring() {
        let model = EmbeddingModel::new_simple(EmbeddingType::TransE, 3);

        // Example embeddings (simplified)
        let head = vec![1.0, 0.0, 0.0];
        let relation = vec![0.0, 1.0, 0.0];
        let tail = vec![1.0, 1.0, 0.0];

        let score = model.score_triple(&head, &relation, &tail);

        // TransE: h + r ≈ t
        // [1,0,0] + [0,1,0] = [1,1,0] ≈ [1,1,0] → distance = 0, score = 0
        assert!(score.abs() < 0.1, "TransE score should be near 0 for perfect match");
    }

    #[test]
    fn test_distmult_scoring() {
        let model = EmbeddingModel::new_simple(EmbeddingType::DistMult, 3);

        let head = vec![1.0, 2.0, 3.0];
        let relation = vec![1.0, 1.0, 1.0];
        let tail = vec![1.0, 2.0, 3.0];

        let score = model.score_triple(&head, &relation, &tail);

        // DistMult: sum(h * r * t) = 1*1*1 + 2*1*2 + 3*1*3 = 1 + 4 + 9 = 14
        assert!((score - 14.0).abs() < 0.01, "DistMult score should be 14");
    }

    #[test]
    fn test_complex_scoring() {
        let model = EmbeddingModel::new_simple(EmbeddingType::ComplEx, 4);

        let head = vec![1.0, 2.0, 0.5, 0.5];
        let relation = vec![1.0, 1.0, 1.0, 1.0];
        let tail = vec![1.0, 2.0, 0.5, 0.5];

        let score = model.score_triple(&head, &relation, &tail);

        // ComplEx (simplified): real part + imag part
        assert!(score > 0.0, "ComplEx score should be positive for similar embeddings");
    }

    #[test]
    fn test_model_metadata() {
        let mut model = EmbeddingModel::new_simple(EmbeddingType::TransE, 100);

        model.add_entity("e1");
        model.add_entity("e2");
        model.add_relation("r1");

        assert_eq!(model.num_entities(), 2);
        assert_eq!(model.num_relations(), 1);
        assert_eq!(model.dimension(), 100);
    }
}
