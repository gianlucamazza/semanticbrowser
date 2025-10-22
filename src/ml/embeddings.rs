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
use std::{fs, path::Path};

#[cfg(feature = "onnx-integration")]
use tract_core::ops::konst::Const;
#[cfg(feature = "onnx-integration")]
use tract_core::ops::submodel::InnerModel;
#[cfg(feature = "onnx-integration")]
use tract_core::ops::EvalOp;
#[cfg(feature = "onnx-integration")]
use tract_core::prelude::*;
#[cfg(feature = "onnx-integration")]
use tract_ndarray::prelude::*;
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
    entity_embeddings: Vec<Vec<f32>>,
    /// Relation embeddings (num_relations x dimension)
    relation_embeddings: Vec<Vec<f32>>,
    /// Optional ONNX runtime for advanced inference
    #[cfg(feature = "onnx-integration")]
    #[allow(dead_code)]
    pub(crate) model: Option<TractSimplePlan>,
}

impl EmbeddingModel {
    /// Create new embedding model from ONNX file
    #[cfg(feature = "onnx-integration")]
    pub fn from_onnx(
        path: impl AsRef<Path>,
        embedding_type: EmbeddingType,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();
        tracing::info!("Loading KG embedding model from {}", path.display());

        let inference_model = tract_onnx::onnx().model_for_path(path)?;
        let optimized_model = inference_model.clone().into_optimized()?;
        let typed_model = optimized_model.as_typed();
        let runnable = typed_model.clone().into_runnable()?;

        #[allow(clippy::disallowed_methods)]
        let entity_node = std::env::var("KG_ENTITY_EMBEDDINGS_NODE")
            .unwrap_or_else(|_| "entity_embeddings".into());
        #[allow(clippy::disallowed_methods)]
        let relation_node = std::env::var("KG_RELATION_EMBEDDINGS_NODE")
            .unwrap_or_else(|_| "relation_embeddings".into());

        let entity_embeddings = Self::extract_embeddings(typed_model, &entity_node)
            .map_err(|e| format!("Failed to extract entity embeddings: {}", e))?;
        let relation_embeddings = Self::extract_embeddings(typed_model, &relation_node)
            .map_err(|e| format!("Failed to extract relation embeddings: {}", e))?;

        if entity_embeddings.is_empty() || relation_embeddings.is_empty() {
            return Err("Embedding tensors are empty".into());
        }

        let entity_labels = Self::load_index_mapping("KG_ENTITY_MAPPING_PATH", "entity")?;
        let relation_labels = Self::load_index_mapping("KG_RELATION_MAPPING_PATH", "relation")?;

        if entity_embeddings.len() != entity_labels.len() {
            return Err(format!(
                "Entity embeddings ({}) and mapping ({}) size mismatch",
                entity_embeddings.len(),
                entity_labels.len()
            )
            .into());
        }

        if relation_embeddings.len() != relation_labels.len() {
            return Err(format!(
                "Relation embeddings ({}) and mapping ({}) size mismatch",
                relation_embeddings.len(),
                relation_labels.len()
            )
            .into());
        }

        let dimension = entity_embeddings
            .first()
            .map(|v| v.len())
            .ok_or("Unable to detect embedding dimension")?;

        if relation_embeddings.iter().any(|embedding| embedding.len() != dimension) {
            return Err("Relation embedding dimension mismatch".into());
        }

        let entity_to_idx = build_index_map(entity_labels);
        let relation_to_idx = build_index_map(relation_labels);

        Ok(Self {
            embedding_type,
            dimension,
            entity_to_idx,
            relation_to_idx,
            entity_embeddings,
            relation_embeddings,
            model: Some(runnable),
        })
    }

    #[cfg(feature = "onnx-integration")]
    fn extract_embeddings(
        model: &TypedModel,
        node_name: &str,
    ) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
        let node = model
            .node_by_name(node_name)
            .map_err(|_| format!("Node '{}' not found in ONNX graph", node_name))?;

        let konst = node
            .op_as::<Const>()
            .ok_or_else(|| format!("Node '{}' is not a constant tensor", node_name))?;

        let tensor = konst.eval(tvec!())?.pop().unwrap();
        let tensor = tensor.cast_to::<f32>()?.into_owned();
        let array = tensor.into_array::<f32>()?;
        let matrix = array
            .into_dimensionality::<Ix2>()
            .map_err(|_| format!("Tensor '{}' is not rank-2", node_name))?;

        Ok(matrix.outer_iter().map(|row| row.to_vec()).collect::<Vec<Vec<f32>>>())
    }

    #[cfg(feature = "onnx-integration")]
    fn load_index_mapping(
        env_var: &str,
        label: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        #[allow(clippy::disallowed_methods)]
        let path = std::env::var(env_var).map_err(|_| {
            format!("{} not set. Provide a mapping file for {} identifiers.", env_var, label)
        })?;

        let raw = fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {} ({}): {}", env_var, path, e))?;
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            return Err(format!("{} is empty ({})", env_var, path).into());
        }

        if trimmed.starts_with('[') {
            let array: Vec<String> = serde_json::from_str(trimmed)
                .map_err(|e| format!("Invalid JSON array in {}: {}", path, e))?;
            return Ok(array);
        }

        if trimmed.starts_with('{') {
            let map: HashMap<String, usize> = serde_json::from_str(trimmed)
                .map_err(|e| format!("Invalid JSON object in {}: {}", path, e))?;
            let mut entries = vec![String::new(); map.len()];
            for (key, idx) in map {
                if idx >= entries.len() {
                    return Err(format!(
                        "Index {} for '{}' exceeds mapping size in {}",
                        idx, key, path
                    )
                    .into());
                }
                entries[idx] = key;
            }
            if entries.iter().any(|s| s.is_empty()) {
                return Err(format!("Sparse mapping detected in {}", path).into());
            }
            return Ok(entries);
        }

        let lines = trimmed
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(String::from)
            .collect::<Vec<String>>();

        if lines.is_empty() {
            return Err(format!("No entries found in {}", path).into());
        }

        Ok(lines)
    }

    /// Create a simple in-memory model (for testing)
    pub fn new_simple(embedding_type: EmbeddingType, dimension: usize) -> Self {
        Self {
            embedding_type,
            dimension,
            entity_to_idx: HashMap::new(),
            relation_to_idx: HashMap::new(),
            entity_embeddings: Vec::new(),
            relation_embeddings: Vec::new(),
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

    /// Insert or update an entity embedding.
    pub fn insert_entity_embedding(
        &mut self,
        entity: &str,
        embedding: Vec<f32>,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        if embedding.len() != self.dimension {
            return Err(format!(
                "Entity embedding for '{}' has dimension {}, expected {}",
                entity,
                embedding.len(),
                self.dimension
            )
            .into());
        }

        let idx = self.add_entity(entity);
        if self.entity_embeddings.len() <= idx {
            self.entity_embeddings.resize(idx + 1, vec![0.0; self.dimension]);
        }
        self.entity_embeddings[idx] = embedding;
        Ok(idx)
    }

    /// Insert or update a relation embedding.
    pub fn insert_relation_embedding(
        &mut self,
        relation: &str,
        embedding: Vec<f32>,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        if embedding.len() != self.dimension {
            return Err(format!(
                "Relation embedding for '{}' has dimension {}, expected {}",
                relation,
                embedding.len(),
                self.dimension
            )
            .into());
        }

        let idx = self.add_relation(relation);
        if self.relation_embeddings.len() <= idx {
            self.relation_embeddings.resize(idx + 1, vec![0.0; self.dimension]);
        }
        self.relation_embeddings[idx] = embedding;
        Ok(idx)
    }

    /// Retrieve entity embedding slice.
    pub fn entity_embedding(&self, idx: usize) -> Option<&[f32]> {
        self.entity_embeddings.get(idx).map(|v| v.as_slice())
    }

    /// Retrieve relation embedding slice.
    pub fn relation_embedding(&self, idx: usize) -> Option<&[f32]> {
        self.relation_embeddings.get(idx).map(|v| v.as_slice())
    }

    /// Iterate over all known entity identifiers.
    pub fn entities(&self) -> impl Iterator<Item = &str> {
        self.entity_to_idx.keys().map(|key| key.as_str())
    }

    /// Iterate over all known relation identifiers.
    pub fn relations(&self) -> impl Iterator<Item = &str> {
        self.relation_to_idx.keys().map(|key| key.as_str())
    }
}

impl EmbeddingType {
    /// Convert raw model score into [0,1] confidence.
    pub fn score_to_confidence(self, raw_score: f32) -> f32 {
        let normalized = match self {
            EmbeddingType::TransE => -raw_score,
            EmbeddingType::DistMult | EmbeddingType::ComplEx => raw_score,
        };
        (1.0 / (1.0 + (-normalized).exp())).clamp(0.0, 1.0)
    }
}

#[cfg(feature = "onnx-integration")]
fn build_index_map(entries: Vec<String>) -> HashMap<String, usize> {
    entries.into_iter().enumerate().map(|(idx, value)| (value, idx)).collect()
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
    fn test_score_to_confidence_transforms() {
        let high = EmbeddingType::TransE.score_to_confidence(-1.5);
        let low = EmbeddingType::TransE.score_to_confidence(-0.1);
        assert!(high > low);
        assert!(high > 0.5);

        let high_mult = EmbeddingType::DistMult.score_to_confidence(1.5);
        let low_mult = EmbeddingType::DistMult.score_to_confidence(0.1);
        assert!(high_mult > low_mult);
        assert!(high_mult > 0.5);
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
