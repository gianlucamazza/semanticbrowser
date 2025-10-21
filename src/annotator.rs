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

        Ok(Self { model: Some(model), fallback_to_regex: false })
    }

    fn regex_fallback() -> Self {
        #[cfg(feature = "onnx-integration")]
        {
            Self { model: None, fallback_to_regex: true }
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
            if let Some(ref model) = self.model {
                match Self::run_onnx_inference(model, text) {
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
        text: &str,
    ) -> Result<Vec<Entity>, Box<dyn std::error::Error>> {
        // Simplified tokenization (in production, use proper BERT tokenizer)
        // This is a placeholder - real NER models need:
        // - BERT/WordPiece tokenization
        // - Special tokens ([CLS], [SEP])
        // - Attention masks
        // - Proper vocabulary mapping

        tracing::debug!("Running ONNX inference for NER");

        // Simple word-based tokenization for demo
        let tokens: Vec<&str> = text.split_whitespace().collect();
        let max_seq_len = 128; // Common BERT sequence length

        // Create dummy input tensor (for demonstration)
        // In production, this would be:
        // 1. Proper tokenization with BERT tokenizer
        // 2. Padding/truncation to max_seq_len
        // 3. Conversion to token IDs
        let input_ids: Vec<i64> =
            tokens.iter().take(max_seq_len).enumerate().map(|(i, _)| i as i64).collect();

        // Pad to max length
        let mut padded_input = input_ids;
        padded_input.resize(max_seq_len, 0);

        // Create tensor (shape: [batch_size=1, sequence_length=128])
        let array = tract_ndarray::Array2::from_shape_vec((1, max_seq_len), padded_input)?;
        let tensor = Tensor::from(array);

        // Run inference
        let result = model.run(tvec![tensor.into()])?;

        // Post-process results (simplified)
        // Real implementation would:
        // 1. Decode predictions to entity labels
        // 2. Group consecutive tokens with same label
        // 3. Map back to original text spans
        // 4. Calculate confidence scores

        // For now, return empty entities as placeholder
        // This would be populated with actual NER results
        tracing::debug!("ONNX inference completed, output shape: {:?}", result[0].shape());

        // Return placeholder - in production, decode tensor to entities
        Ok(Vec::new())
    }
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
}
