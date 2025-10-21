// Semantic annotator module

use regex::Regex;
use scraper::{Html, Selector};
use std::sync::OnceLock;

/// Entity with type and text
#[derive(Debug, Clone)]
pub struct Entity {
    pub text: String,
    pub entity_type: String,
    pub confidence: f32,
}

/// NER model wrapper
pub struct NERModel {
    // Placeholder for tract model
    // In a real implementation, this would be: tract_core::Model
    fallback_to_regex: bool,
}

static NER_MODEL: OnceLock<Option<NERModel>> = OnceLock::new();

impl NERModel {
    /// Load NER model from path
    pub fn load(model_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        // Try to load ONNX model using tract
        // For now, this is a placeholder - a real implementation would:
        // 1. Load the ONNX model
        // 2. Optimize it
        // 3. Store it for inference

        tracing::info!("Attempting to load NER model from: {}", model_path);

        // Placeholder: in real impl, use tract_core::onnx to load model
        // let model = tract_onnx::onnx()
        //     .model_for_path(model_path)?
        //     .into_optimized()?
        //     .into_runnable()?;

        // For now, if file doesn't exist, fall back to regex
        if !std::path::Path::new(model_path).exists() {
            tracing::warn!(
                "NER model not found at {}, falling back to regex",
                model_path
            );
            Ok(Self {
                fallback_to_regex: true,
            })
        } else {
            tracing::warn!("NER model found but tract-onnx integration not yet implemented, falling back to regex");
            Ok(Self {
                fallback_to_regex: true,
            })
        }
    }

    /// Extract entities using the model
    pub fn extract_entities(&self, text: &str) -> Vec<Entity> {
        if self.fallback_to_regex {
            return extract_entities_regex(text);
        }

        // Real ML inference would go here
        // 1. Tokenize text
        // 2. Convert to tensor
        // 3. Run model inference
        // 4. Decode predictions
        // 5. Return entities with types and confidence scores

        tracing::debug!("Running ML-based NER on text of length {}", text.len());
        extract_entities_regex(text)
    }
}

/// Initialize NER model from environment variable
pub fn init_ner_model() {
    NER_MODEL.get_or_init(|| {
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
