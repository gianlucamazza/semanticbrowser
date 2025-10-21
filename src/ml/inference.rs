//! Link Prediction for Knowledge Graphs
//!
//! Uses embedding models to predict missing links in KG:
//! - Head prediction: Given (?, r, t), predict h
//! - Tail prediction: Given (h, r, ?), predict t
//! - Relation prediction: Given (h, ?, t), predict r
//!
//! Best practices 2025:
//! - Provide confidence scores for predictions
//! - Support batch prediction for efficiency
//! - Filter out known triples (filtered setting)
//! - Return top-k predictions with ranking

use super::embeddings::EmbeddingModel;
use std::collections::HashSet;

/// Prediction result with confidence
#[derive(Debug, Clone)]
pub struct PredictionResult {
    /// Predicted entity/relation URI
    pub uri: String,
    /// Confidence score (higher is better)
    pub score: f32,
    /// Rank in predictions (1-based)
    pub rank: usize,
}

/// Link predictor for Knowledge Graph completion
pub struct LinkPredictor {
    /// Embedding model
    model: EmbeddingModel,
    /// Known triples (for filtering)
    known_triples: HashSet<(String, String, String)>,
}

impl LinkPredictor {
    /// Create new link predictor with embedding model
    pub fn new(model: EmbeddingModel) -> Self {
        Self { model, known_triples: HashSet::new() }
    }

    /// Add known triple (for filtered prediction)
    pub fn add_known_triple(&mut self, head: &str, relation: &str, tail: &str) {
        self.known_triples.insert((head.to_string(), relation.to_string(), tail.to_string()));
    }

    /// Predict tail given head and relation: (h, r, ?)
    ///
    /// Returns top-k candidates with scores
    pub fn predict_tail(
        &self,
        head: &str,
        relation: &str,
        k: usize,
        filtered: bool,
    ) -> Result<Vec<PredictionResult>, Box<dyn std::error::Error>> {
        let head_idx = self.model.get_entity_idx(head).ok_or("Head entity not in model")?;
        let rel_idx = self.model.get_relation_idx(relation).ok_or("Relation not in model")?;

        // Generate candidate scores
        let mut candidates = Vec::new();

        for (entity, &tail_idx) in &self.model.entity_to_idx {
            // Skip if this is a known triple (in filtered setting)
            if filtered
                && self.known_triples.contains(&(
                    head.to_string(),
                    relation.to_string(),
                    entity.clone(),
                ))
            {
                continue;
            }

            // In real implementation, we'd fetch actual embeddings
            // For now, use dummy embeddings for demonstration
            let score = self.score_triple_by_idx(head_idx, rel_idx, tail_idx);

            candidates.push((entity.clone(), score));
        }

        // Sort by score (descending)
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top-k
        Ok(candidates
            .into_iter()
            .take(k)
            .enumerate()
            .map(|(rank, (uri, score))| PredictionResult { uri, score, rank: rank + 1 })
            .collect())
    }

    /// Predict head given relation and tail: (?, r, t)
    pub fn predict_head(
        &self,
        relation: &str,
        tail: &str,
        k: usize,
        filtered: bool,
    ) -> Result<Vec<PredictionResult>, Box<dyn std::error::Error>> {
        let rel_idx = self.model.get_relation_idx(relation).ok_or("Relation not in model")?;
        let tail_idx = self.model.get_entity_idx(tail).ok_or("Tail entity not in model")?;

        let mut candidates = Vec::new();

        for (entity, &head_idx) in &self.model.entity_to_idx {
            // Skip known triples in filtered setting
            if filtered
                && self.known_triples.contains(&(
                    entity.clone(),
                    relation.to_string(),
                    tail.to_string(),
                ))
            {
                continue;
            }

            let score = self.score_triple_by_idx(head_idx, rel_idx, tail_idx);
            candidates.push((entity.clone(), score));
        }

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(candidates
            .into_iter()
            .take(k)
            .enumerate()
            .map(|(rank, (uri, score))| PredictionResult { uri, score, rank: rank + 1 })
            .collect())
    }

    /// Predict relation given head and tail: (h, ?, t)
    pub fn predict_relation(
        &self,
        head: &str,
        tail: &str,
        k: usize,
        filtered: bool,
    ) -> Result<Vec<PredictionResult>, Box<dyn std::error::Error>> {
        let head_idx = self.model.get_entity_idx(head).ok_or("Head entity not in model")?;
        let tail_idx = self.model.get_entity_idx(tail).ok_or("Tail entity not in model")?;

        let mut candidates = Vec::new();

        for (relation, &rel_idx) in &self.model.relation_to_idx {
            // Skip known triples in filtered setting
            if filtered
                && self.known_triples.contains(&(
                    head.to_string(),
                    relation.clone(),
                    tail.to_string(),
                ))
            {
                continue;
            }

            let score = self.score_triple_by_idx(head_idx, rel_idx, tail_idx);
            candidates.push((relation.clone(), score));
        }

        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(candidates
            .into_iter()
            .take(k)
            .enumerate()
            .map(|(rank, (uri, score))| PredictionResult { uri, score, rank: rank + 1 })
            .collect())
    }

    /// Score triple by indices (helper function)
    ///
    /// In real implementation, this would fetch embeddings and use model.score_triple()
    /// For now, returns a dummy score for demonstration
    fn score_triple_by_idx(&self, _head_idx: usize, _rel_idx: usize, _tail_idx: usize) -> f32 {
        // Placeholder: In production, fetch actual embeddings and compute score
        // Example:
        // let head_emb = &self.model.entity_embeddings[head_idx];
        // let rel_emb = &self.model.relation_embeddings[rel_idx];
        // let tail_emb = &self.model.entity_embeddings[tail_idx];
        // self.model.score_triple(head_emb, rel_emb, tail_emb)

        // For now, return random-ish score based on indices
        (((_head_idx * 13 + _rel_idx * 17 + _tail_idx * 19) % 100) as f32) / 100.0
    }

    /// Get confidence threshold for filtering predictions
    ///
    /// Returns percentile-based threshold (e.g., 90th percentile)
    pub fn get_confidence_threshold(&self, percentile: f32) -> f32 {
        // In production, compute from score distribution
        // For now, return a simple threshold
        percentile
    }

    /// Get model metadata
    pub fn num_entities(&self) -> usize {
        self.model.num_entities()
    }

    pub fn num_relations(&self) -> usize {
        self.model.num_relations()
    }

    pub fn num_known_triples(&self) -> usize {
        self.known_triples.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ml::EmbeddingType;

    #[test]
    fn test_link_predictor_creation() {
        let model = EmbeddingModel::new_simple(EmbeddingType::TransE, 50);
        let predictor = LinkPredictor::new(model);

        assert_eq!(predictor.num_entities(), 0);
        assert_eq!(predictor.num_relations(), 0);
        assert_eq!(predictor.num_known_triples(), 0);
    }

    #[test]
    fn test_predict_tail() {
        let mut model = EmbeddingModel::new_simple(EmbeddingType::DistMult, 100);

        // Add entities and relations
        model.add_entity("http://ex.org/Alice");
        model.add_entity("http://ex.org/Bob");
        model.add_entity("http://ex.org/Company");
        model.add_relation("http://ex.org/worksFor");

        let predictor = LinkPredictor::new(model);

        // Predict: (Alice, worksFor, ?)
        let predictions = predictor
            .predict_tail("http://ex.org/Alice", "http://ex.org/worksFor", 2, false)
            .expect("Prediction should succeed");

        assert_eq!(predictions.len(), 2, "Should return top-2 predictions");
        assert_eq!(predictions[0].rank, 1, "First prediction should have rank 1");
        assert!(predictions[0].score >= predictions[1].score, "Scores should be sorted");
    }

    #[test]
    fn test_predict_head() {
        let mut model = EmbeddingModel::new_simple(EmbeddingType::TransE, 100);

        model.add_entity("http://ex.org/Alice");
        model.add_entity("http://ex.org/Bob");
        model.add_relation("http://ex.org/knows");

        let predictor = LinkPredictor::new(model);

        // Predict: (?, knows, Bob)
        let predictions = predictor
            .predict_head("http://ex.org/knows", "http://ex.org/Bob", 1, false)
            .expect("Prediction should succeed");

        assert_eq!(predictions.len(), 1, "Should return top-1 prediction");
        assert_eq!(predictions[0].rank, 1);
    }

    #[test]
    fn test_predict_relation() {
        let mut model = EmbeddingModel::new_simple(EmbeddingType::ComplEx, 100);

        model.add_entity("http://ex.org/Alice");
        model.add_entity("http://ex.org/Bob");
        model.add_relation("http://ex.org/knows");
        model.add_relation("http://ex.org/likes");

        let predictor = LinkPredictor::new(model);

        // Predict: (Alice, ?, Bob)
        let predictions = predictor
            .predict_relation("http://ex.org/Alice", "http://ex.org/Bob", 2, false)
            .expect("Prediction should succeed");

        assert!(predictions.len() <= 2, "Should return at most 2 predictions");
    }

    #[test]
    fn test_filtered_prediction() {
        let mut model = EmbeddingModel::new_simple(EmbeddingType::DistMult, 100);

        model.add_entity("http://ex.org/Alice");
        model.add_entity("http://ex.org/Bob");
        model.add_entity("http://ex.org/Charlie");
        model.add_relation("http://ex.org/knows");

        let mut predictor = LinkPredictor::new(model);

        // Add known triple
        predictor.add_known_triple(
            "http://ex.org/Alice",
            "http://ex.org/knows",
            "http://ex.org/Bob",
        );

        // Predict with filtering
        let filtered = predictor
            .predict_tail(
                "http://ex.org/Alice",
                "http://ex.org/knows",
                10,
                true, // filtered = true
            )
            .expect("Prediction should succeed");

        // Bob should be excluded from predictions
        assert!(
            !filtered.iter().any(|p| p.uri == "http://ex.org/Bob"),
            "Known triple should be filtered out"
        );
    }

    #[test]
    fn test_confidence_threshold() {
        let model = EmbeddingModel::new_simple(EmbeddingType::TransE, 100);
        let predictor = LinkPredictor::new(model);

        let threshold = predictor.get_confidence_threshold(0.9);
        assert!(threshold > 0.0 && threshold <= 1.0, "Threshold should be in [0,1]");
    }

    #[test]
    fn test_prediction_error_handling() {
        let model = EmbeddingModel::new_simple(EmbeddingType::DistMult, 100);
        let predictor = LinkPredictor::new(model);

        // Predict with non-existent entities
        let result = predictor.predict_tail("nonexistent", "also_nonexistent", 1, false);
        assert!(result.is_err(), "Should error on non-existent entities");
    }
}
