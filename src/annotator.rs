// Semantic annotator module
//
// Best practices 2025:
// - ONNX models via tract-onnx for production NER
// - Optimized model loading with .into_optimized()
// - Fallback to regex for development/testing
// - Support for common NER models (BERT, DistilBERT, etc.)

use regex::Regex;
use scraper::{Html, Selector};
use std::sync::OnceLock;

#[cfg(feature = "onnx-integration")]
use std::fs;
#[cfg(feature = "onnx-integration")]
use std::path::Path;
#[cfg(feature = "onnx-integration")]
use tokenizers::{Tokenizer, TruncationParams, TruncationStrategy};
#[cfg(feature = "onnx-integration")]
use tract_onnx::prelude::*;

#[cfg(feature = "onnx-integration")]
type TractModel = SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>;

/// Entity with type and text
#[derive(Debug, Clone)]
pub struct Entity {
    pub text: String,
    pub entity_type: String,
    pub confidence: f32,
}

/// NER model wrapper with ONNX support
pub struct NERModel {
    #[cfg(feature = "onnx-integration")]
    model: Option<TractModel>,
    #[cfg(feature = "onnx-integration")]
    tokenizer: Option<Tokenizer>,
    #[cfg(feature = "onnx-integration")]
    labels: Vec<String>,

    #[cfg(not(feature = "onnx-integration"))]
    _phantom: (),

    fallback_to_regex: bool,
}

static NER_MODEL: OnceLock<Option<NERModel>> = OnceLock::new();

impl NERModel {
    /// Load NER model from ONNX file
    ///
    /// Best practices 2025:
    /// - Optimize model with .into_optimized() for better performance
    /// - Support standard NER model architectures (BERT, DistilBERT)
    /// - Fallback to regex if ONNX feature is disabled or model fails to load
    pub fn load(model_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        tracing::info!("Attempting to load NER model from: {}", model_path);

        // Check if file exists first
        if !std::path::Path::new(model_path).exists() {
            tracing::warn!("NER model not found at {}, falling back to regex", model_path);
            return Ok(Self::regex_fallback());
        }

        #[cfg(feature = "onnx-integration")]
        {
            match Self::load_onnx_model(model_path) {
                Ok(model) => {
                    tracing::info!("Successfully loaded ONNX NER model");
                    Ok(model)
                }
                Err(e) => {
                    tracing::error!("Failed to load ONNX model: {}, falling back to regex", e);
                    Ok(Self::regex_fallback())
                }
            }
        }

        #[cfg(not(feature = "onnx-integration"))]
        {
            tracing::warn!(
                "ONNX integration not enabled, falling back to regex. \
                 Enable with --features onnx-integration"
            );
            Ok(Self::regex_fallback())
        }
    }

    #[cfg(feature = "onnx-integration")]
    fn load_onnx_model(model_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Load ONNX model using tract
        // Best practice: optimize the model for better performance
        let model =
            tract_onnx::onnx().model_for_path(model_path)?.into_optimized()?.into_runnable()?;

        tracing::debug!("ONNX model loaded and optimized successfully");

        let tokenizer = match Self::load_tokenizer() {
            Ok(tokenizer) => Some(tokenizer),
            Err(err) => {
                tracing::warn!("NER tokenizer not available ({}), falling back to regex", err);
                return Ok(Self::regex_fallback());
            }
        };

        let labels = match Self::load_labels() {
            Ok(labels) => labels,
            Err(err) => {
                tracing::warn!("NER labels not available ({}), falling back to regex", err);
                return Ok(Self::regex_fallback());
            }
        };

        Ok(Self { model: Some(model), tokenizer, labels, fallback_to_regex: false })
    }

    #[cfg(feature = "onnx-integration")]
    fn load_tokenizer() -> Result<Tokenizer, Box<dyn std::error::Error>> {
        #[allow(clippy::disallowed_methods)]
        let path = std::env::var("NER_TOKENIZER_PATH")
            .map_err(|_| "NER_TOKENIZER_PATH not set in environment")?;

        let tokenizer_path = Path::new(&path);
        if !tokenizer_path.exists() {
            return Err(format!("Tokenizer file not found at {}", path).into());
        }

        let mut tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| format!("Failed to load tokenizer: {}", e))?;

        let params = TruncationParams {
            max_length: 512,
            strategy: TruncationStrategy::LongestFirst,
            ..Default::default()
        };
        let _ = tokenizer.with_truncation(Some(params));

        Ok(tokenizer)
    }

    #[cfg(feature = "onnx-integration")]
    fn load_labels() -> Result<Vec<String>, Box<dyn std::error::Error>> {
        #[allow(clippy::disallowed_methods)]
        if let Ok(path) = std::env::var("NER_LABELS_PATH") {
            let content = fs::read_to_string(&path)?;
            if let Ok(labels) = serde_json::from_str::<Vec<String>>(&content) {
                return Self::validate_labels(labels);
            }

            let labels: Vec<String> = content
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(String::from)
                .collect();

            if !labels.is_empty() {
                return Self::validate_labels(labels);
            }
        }

        #[allow(clippy::disallowed_methods)]
        if let Ok(raw) = std::env::var("NER_LABELS") {
            let labels: Vec<String> = raw
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(String::from)
                .collect();

            if !labels.is_empty() {
                return Self::validate_labels(labels);
            }
        }

        // Fallback to default CoNLL style labels
        Self::validate_labels(Self::default_labels())
    }

    #[cfg(feature = "onnx-integration")]
    fn validate_labels(labels: Vec<String>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        if labels.is_empty() {
            return Err("NER labels list cannot be empty".into());
        }

        if !labels.iter().any(|label| label == "O") {
            return Err("NER labels must contain the 'O' (outside) tag".into());
        }

        Ok(labels)
    }

    #[cfg(feature = "onnx-integration")]
    fn default_labels() -> Vec<String> {
        vec!["O", "B-PER", "I-PER", "B-ORG", "I-ORG", "B-LOC", "I-LOC", "B-MISC", "I-MISC"]
            .into_iter()
            .map(String::from)
            .collect()
    }

    fn regex_fallback() -> Self {
        #[cfg(feature = "onnx-integration")]
        {
            Self { model: None, tokenizer: None, labels: Vec::new(), fallback_to_regex: true }
        }

        #[cfg(not(feature = "onnx-integration"))]
        {
            Self { _phantom: (), fallback_to_regex: true }
        }
    }

    /// Extract entities using ONNX model or regex fallback
    ///
    /// For ONNX models, implements:
    /// 1. Simple tokenization (space-based for demo, BERT tokenizer for production)
    /// 2. Tensor conversion
    /// 3. Model inference
    /// 4. Post-processing of predictions
    pub fn extract_entities(&self, text: &str) -> Vec<Entity> {
        if self.fallback_to_regex {
            return extract_entities_regex(text);
        }

        #[cfg(feature = "onnx-integration")]
        {
            if let (Some(model), Some(tokenizer)) = (&self.model, &self.tokenizer) {
                match Self::run_onnx_inference(model, tokenizer, &self.labels, text) {
                    Ok(entities) => {
                        tracing::debug!(
                            "ONNX NER extracted {} entities from text length {}",
                            entities.len(),
                            text.len()
                        );
                        return entities;
                    }
                    Err(e) => {
                        tracing::warn!("ONNX inference failed: {}, using regex fallback", e);
                        return extract_entities_regex(text);
                    }
                }
            }
        }

        // Fallback if no model or feature disabled
        tracing::debug!("Using regex-based NER on text of length {}", text.len());
        extract_entities_regex(text)
    }

    #[cfg(feature = "onnx-integration")]
    fn run_onnx_inference(
        model: &TractModel,
        tokenizer: &Tokenizer,
        labels: &[String],
        text: &str,
    ) -> Result<Vec<Entity>, Box<dyn std::error::Error>> {
        tracing::debug!("Running ONNX inference for NER");

        let encoding = tokenizer
            .encode(text, true)
            .map_err(|e| format!("Tokenizer encoding failed: {}", e))?;

        // Ensure attention mask length matches ids
        let sequence_len = encoding.len();
        if sequence_len == 0 {
            return Ok(Vec::new());
        }

        // Prepare tensors (input_ids, attention_mask, token_type_ids)
        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|id| *id as i64).collect();
        let attention_mask: Vec<i64> =
            encoding.get_attention_mask().iter().map(|mask| *mask as i64).collect();
        let token_type_ids: Vec<i64> =
            encoding.get_type_ids().iter().map(|mask| *mask as i64).collect();

        // Reshape into [1, sequence_len]
        let ids_tensor = Tensor::from(tract_ndarray::Array2::from_shape_vec(
            (1, sequence_len),
            input_ids.clone(),
        )?);
        let mask_tensor = Tensor::from(tract_ndarray::Array2::from_shape_vec(
            (1, sequence_len),
            attention_mask.clone(),
        )?);
        let type_ids_tensor = Tensor::from(tract_ndarray::Array2::from_shape_vec(
            (1, sequence_len),
            token_type_ids.clone(),
        )?);

        let input_count = model
            .model()
            .input_outlets()
            .map_err(|e| format!("Unable to inspect model inputs: {}", e))?
            .len();
        let mut inputs = tvec![ids_tensor.into()];

        if input_count > 1 {
            inputs.push(mask_tensor.into());
        }

        if input_count > 2 {
            inputs.push(type_ids_tensor.into());
        }

        let result = model.run(inputs)?;
        if result.is_empty() {
            tracing::debug!("ONNX model returned no outputs");
            return Ok(Vec::new());
        }

        let logits = result[0].to_array_view::<f32>()?;
        let output_shape = logits.shape();
        if output_shape.len() != 3 {
            tracing::warn!(
                "Unexpected NER output shape {:?}, expected [batch, seq_len, num_labels]",
                output_shape
            );
            return Ok(Vec::new());
        }

        let seq_len = output_shape[1];
        if seq_len != sequence_len {
            tracing::warn!(
                "Tokenizer length ({}) differs from model output ({})",
                sequence_len,
                seq_len
            );
        }

        let mut token_predictions = Vec::with_capacity(seq_len);
        for token_idx in 0..seq_len {
            let logits_slice = logits.index_axis(tract_ndarray::Axis(1), token_idx);
            let logits_vec: Vec<f32> = logits_slice.iter().cloned().collect();
            let (label_idx, confidence) = select_label(&logits_vec)?;
            let label = labels.get(label_idx).map(String::as_str).unwrap_or("O");

            let (start, end) = encoding.get_offsets().get(token_idx).copied().unwrap_or((0, 0));
            let is_special =
                encoding.get_special_tokens_mask().get(token_idx).copied().unwrap_or(0) == 1;

            token_predictions.push(TokenInference {
                start,
                end,
                label: label.to_string(),
                confidence,
                is_special,
            });
        }

        Ok(aggregate_entities(token_predictions, text))
    }
}

#[cfg(feature = "onnx-integration")]
fn select_label(logits: &[f32]) -> Result<(usize, f32), Box<dyn std::error::Error>> {
    if logits.is_empty() {
        return Err("Model returned empty logits".into());
    }

    let mut max_idx = 0usize;
    let mut max_val = f32::NEG_INFINITY;
    for (idx, &value) in logits.iter().enumerate() {
        if value > max_val {
            max_val = value;
            max_idx = idx;
        }
    }

    let exp_sum: f32 = logits.iter().map(|&value| (value - max_val).exp()).sum();
    let confidence = if exp_sum.is_finite() && exp_sum > 0.0 {
        ((logits[max_idx] - max_val).exp() / exp_sum).clamp(0.0, 1.0)
    } else {
        0.0
    };

    Ok((max_idx, confidence))
}

#[cfg(feature = "onnx-integration")]
#[derive(Debug, Clone)]
struct TokenInference {
    start: usize,
    end: usize,
    label: String,
    confidence: f32,
    is_special: bool,
}

#[cfg(feature = "onnx-integration")]
#[derive(Debug)]
struct CurrentEntity {
    entity_type: String,
    start: usize,
    end: usize,
    confidences: Vec<f32>,
}

#[cfg(feature = "onnx-integration")]
fn aggregate_entities(tokens: Vec<TokenInference>, text: &str) -> Vec<Entity> {
    let mut entities = Vec::new();
    let mut current: Option<CurrentEntity> = None;

    for token in tokens {
        let parsed = parse_bio_label(&token.label);

        if token.is_special || token.start >= token.end || parsed.is_none() {
            finalize_entity(&mut current, text, &mut entities);
            continue;
        }

        let (tag, entity_type) = parsed.unwrap();

        match tag {
            Tag::Outside => {
                finalize_entity(&mut current, text, &mut entities);
            }
            Tag::Single => {
                finalize_entity(&mut current, text, &mut entities);
                push_entity(
                    text,
                    &mut entities,
                    entity_type,
                    token.start,
                    token.end,
                    &[token.confidence],
                );
            }
            Tag::Begin => {
                finalize_entity(&mut current, text, &mut entities);
                current = Some(CurrentEntity {
                    entity_type: entity_type.to_string(),
                    start: token.start,
                    end: token.end,
                    confidences: vec![token.confidence],
                });
            }
            Tag::Inside => {
                if let Some(active) = &mut current {
                    if active.entity_type == entity_type {
                        active.end = token.end.max(active.end);
                        active.confidences.push(token.confidence);
                        continue;
                    }
                    finalize_entity(&mut current, text, &mut entities);
                }

                // Treat stray I- as standalone entity for robustness
                current = Some(CurrentEntity {
                    entity_type: entity_type.to_string(),
                    start: token.start,
                    end: token.end,
                    confidences: vec![token.confidence],
                });
            }
            Tag::End => {
                if let Some(active) = &mut current {
                    if active.entity_type == entity_type {
                        active.end = token.end.max(active.end);
                        active.confidences.push(token.confidence);
                        finalize_entity(&mut current, text, &mut entities);
                        continue;
                    }
                    finalize_entity(&mut current, text, &mut entities);
                }

                // Treat stray E- as single entity
                push_entity(
                    text,
                    &mut entities,
                    entity_type,
                    token.start,
                    token.end,
                    &[token.confidence],
                );
            }
        }
    }

    finalize_entity(&mut current, text, &mut entities);
    entities
}

#[cfg(feature = "onnx-integration")]
fn finalize_entity(current: &mut Option<CurrentEntity>, text: &str, entities: &mut Vec<Entity>) {
    if let Some(active) = current.take() {
        push_entity(
            text,
            entities,
            &active.entity_type,
            active.start,
            active.end,
            &active.confidences,
        );
    }
}

#[cfg(feature = "onnx-integration")]
fn push_entity(
    text: &str,
    entities: &mut Vec<Entity>,
    entity_type: &str,
    start: usize,
    end: usize,
    confidences: &[f32],
) {
    if let Some(span) = text.get(start..end) {
        let entity_text = span.trim();
        if entity_text.is_empty() {
            return;
        }

        let confidence = if confidences.is_empty() {
            0.0
        } else {
            let sum: f32 = confidences.iter().copied().filter(|c| c.is_finite()).sum();
            let count = confidences.iter().filter(|c| c.is_finite()).count();
            if count == 0 {
                0.0
            } else {
                (sum / count as f32).clamp(0.0, 1.0)
            }
        };

        entities.push(Entity {
            text: entity_text.to_string(),
            entity_type: entity_type.to_string(),
            confidence,
        });
    }
}

#[cfg(feature = "onnx-integration")]
enum Tag {
    Outside,
    Single,
    Begin,
    Inside,
    End,
}

#[cfg(feature = "onnx-integration")]
fn parse_bio_label(label: &str) -> Option<(Tag, &str)> {
    if label == "O" {
        return Some((Tag::Outside, ""));
    }

    if let Some(rest) = label.strip_prefix("B-") {
        return Some((Tag::Begin, rest));
    }

    if let Some(rest) = label.strip_prefix("I-") {
        return Some((Tag::Inside, rest));
    }

    if let Some(rest) = label.strip_prefix("E-") {
        return Some((Tag::End, rest));
    }

    if let Some(rest) = label.strip_prefix("S-") {
        return Some((Tag::Single, rest));
    }

    None
}

/// Initialize NER model from environment variable
pub fn init_ner_model() {
    NER_MODEL.get_or_init(|| {
        #[allow(clippy::disallowed_methods)]
        if let Ok(model_path) = std::env::var("NER_MODEL_PATH") {
            tracing::info!("Initializing NER model from {}", model_path);
            match NERModel::load(&model_path) {
                Ok(model) => Some(model),
                Err(e) => {
                    tracing::error!("Failed to load NER model: {}", e);
                    None
                }
            }
        } else {
            tracing::info!("NER_MODEL_PATH not set, using regex-based entity extraction");
            None
        }
    });
}

/// Annotate HTML with semantic information and extract entities
pub fn annotate_html(html: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let document = Html::parse_document(html);

    // Extract text content
    let body_selector = Selector::parse("body").unwrap();
    let body_text = document
        .select(&body_selector)
        .next()
        .map(|b| b.text().collect::<String>())
        .unwrap_or_default();

    // Get entities using ML model if available, otherwise regex
    let entities = get_entities(&body_text);

    // Convert to strings for backward compatibility
    Ok(entities.into_iter().map(|e| e.text).collect())
}

/// Get entities using the initialized model or fallback
fn get_entities(text: &str) -> Vec<Entity> {
    if let Some(Some(model)) = NER_MODEL.get() {
        model.extract_entities(text)
    } else {
        extract_entities_regex(text)
    }
}

/// Regex-based entity extraction (fallback)
fn extract_entities_regex(text: &str) -> Vec<Entity> {
    let re = Regex::new(r"\b[A-Z][a-z]+(?:\s+[A-Z][a-z]+)*\b").unwrap(); // Capitalized words/phrases
    re.find_iter(text)
        .map(|m| Entity {
            text: m.as_str().to_string(),
            entity_type: "UNKNOWN".to_string(),
            confidence: 0.5, // Low confidence for regex
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_entities() {
        let text = "John Smith works at Microsoft in New York.";
        let entities = extract_entities_regex(text);
        assert!(!entities.is_empty());
        assert!(entities.iter().any(|e| e.text == "John Smith"));
    }

    #[cfg(feature = "onnx-integration")]
    #[test]
    fn test_aggregate_entities_merges_sequences() {
        use super::{aggregate_entities, TokenInference};

        let text = "Alice works at Example Corp in London.";
        let tokens = vec![
            TokenInference {
                start: 0,
                end: 5,
                label: "B-PER".to_string(),
                confidence: 0.92,
                is_special: false,
            },
            TokenInference {
                start: 6,
                end: 11,
                label: "O".to_string(),
                confidence: 0.0,
                is_special: false,
            },
            TokenInference {
                start: 15,
                end: 22,
                label: "B-ORG".to_string(),
                confidence: 0.88,
                is_special: false,
            },
            TokenInference {
                start: 23,
                end: 27,
                label: "I-ORG".to_string(),
                confidence: 0.84,
                is_special: false,
            },
            TokenInference {
                start: 25,
                end: 27,
                label: "O".to_string(),
                confidence: 0.0,
                is_special: false,
            },
            TokenInference {
                start: 31,
                end: 37,
                label: "B-LOC".to_string(),
                confidence: 0.81,
                is_special: false,
            },
        ];

        let entities = aggregate_entities(tokens, text);
        assert_eq!(entities.len(), 3);
        assert_eq!(entities[0].entity_type, "PER");
        assert_eq!(entities[0].text, "Alice");
        assert!((entities[0].confidence - 0.92).abs() < f32::EPSILON);

        assert_eq!(entities[1].entity_type, "ORG");
        assert_eq!(entities[1].text, "Example Corp");
        assert!((entities[1].confidence - 0.86).abs() < 0.02);

        assert_eq!(entities[2].entity_type, "LOC");
        assert_eq!(entities[2].text, "London");
        assert!((entities[2].confidence - 0.81).abs() < f32::EPSILON);
    }
}
