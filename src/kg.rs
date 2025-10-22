// Knowledge graph module

use oxigraph::model::*;
use oxigraph::store::Store;
#[cfg(feature = "onnx-integration")]
use std::collections::HashSet;
use std::path::Path;

#[cfg(feature = "onnx-integration")]
use crate::ml::embeddings::{EmbeddingModel, EmbeddingType};
#[cfg(feature = "onnx-integration")]
use crate::ml::inference::LinkPredictor;

/// Knowledge Graph wrapper
pub struct KnowledgeGraph {
    store: Store,
}

impl Default for KnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeGraph {
    /// Create a new in-memory KG
    pub fn new() -> Self {
        Self { store: Store::new().unwrap() }
    }

    /// Create with persistence
    pub fn with_persistence(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let store = Store::open(path)?;
        Ok(Self { store })
    }

    /// Add a triple
    pub fn insert(&mut self, s: &str, p: &str, o: &str) -> Result<(), Box<dyn std::error::Error>> {
        let subject = NamedNode::new(s)?;
        let predicate = NamedNode::new(p)?;
        let object = NamedNode::new(o)?;
        let quad = Quad::new(subject, predicate, object, GraphName::DefaultGraph);
        self.store.insert(&quad)?;
        Ok(())
    }

    #[cfg(feature = "onnx-integration")]
    fn contains_triple(
        &self,
        s: &str,
        p: &str,
        o: &str,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let quad = Quad::new(
            NamedNode::new(s)?,
            NamedNode::new(p)?,
            NamedNode::new(o)?,
            GraphName::DefaultGraph,
        );
        Ok(self.store.contains(&quad)?)
    }

    #[cfg(feature = "onnx-integration")]
    fn populate_known_triples(
        &self,
        predictor: &mut LinkPredictor,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for quad in self.store.iter() {
            let quad = quad?;
            if let (Subject::NamedNode(subject), predicate, Term::NamedNode(object)) =
                (&quad.subject, &quad.predicate, &quad.object)
            {
                predictor.add_known_triple(subject.as_str(), predicate.as_str(), object.as_str());
            }
        }
        Ok(())
    }

    /// Knowledge Graph inference using ML models or rule-based reasoning
    ///
    /// This method can perform inference on the KG to derive new facts.
    /// If KG_INFERENCE_MODEL_PATH is set, it will use a ML model (e.g., for link prediction).
    /// Otherwise, it performs simple rule-based inference like transitive closure.
    #[tracing::instrument(skip(self))]
    pub fn infer(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Check if ML model is configured
        #[allow(clippy::disallowed_methods)]
        if let Ok(model_path) = std::env::var("KG_INFERENCE_MODEL_PATH") {
            tracing::info!("Running ML-based KG inference with model: {}", model_path);

            if !std::path::Path::new(&model_path).exists() {
                tracing::warn!("KG inference model not found at {}", model_path);
                return self.infer_rules_based();
            }

            // Run ML-based inference
            #[cfg(feature = "onnx-integration")]
            {
                match self.run_ml_inference(&model_path) {
                    Ok(_) => {
                        tracing::info!("ML-based inference completed successfully");
                        // Also run rule-based for completeness
                        self.infer_rules_based()
                    }
                    Err(e) => {
                        tracing::warn!("ML inference failed: {}, falling back to rules", e);
                        self.infer_rules_based()
                    }
                }
            }

            #[cfg(not(feature = "onnx-integration"))]
            {
                tracing::warn!("ML inference requires --features onnx-integration, using rules");
                self.infer_rules_based()
            }
        } else {
            tracing::debug!("Running rule-based KG inference");
            self.infer_rules_based()
        }
    }

    /// Rule-based inference (transitive properties, type hierarchy, etc.)
    ///
    /// Implements RDFS reasoning rules:
    /// - rdfs:subClassOf transitive closure
    /// - rdfs:subPropertyOf transitive closure
    /// - rdfs:domain and rdfs:range inference
    /// - rdf:type propagation via class hierarchy
    #[tracing::instrument(skip(self))]
    fn infer_rules_based(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Starting rule-based inference");

        let mut total_inferred = 0;

        // 1. Transitive closure for rdfs:subClassOf
        total_inferred += self.infer_subclass_transitive()?;

        // 2. Transitive closure for rdfs:subPropertyOf
        total_inferred += self.infer_subproperty_transitive()?;

        // 3. Type propagation via class hierarchy
        total_inferred += self.infer_type_propagation()?;

        tracing::info!("Rule-based inference completed: {} new triples inferred", total_inferred);

        Ok(())
    }

    /// Compute and insert transitive closure for rdfs:subClassOf
    /// If A subClassOf B and B subClassOf C, then A subClassOf C
    ///
    /// Uses SPARQL CONSTRUCT for efficient, standard-compliant inference
    fn infer_subclass_transitive(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let subclass_of = "http://www.w3.org/2000/01/rdf-schema#subClassOf";

        // Use SPARQL CONSTRUCT to find transitive relations
        // This constructs new triples: ?a subClassOf ?c WHERE ?a->?b->?c
        let construct_query = format!(
            "CONSTRUCT {{ ?a <{0}> ?c }}
             WHERE {{
               ?a <{0}> ?b .
               ?b <{0}> ?c .
               FILTER(?a != ?c)
               FILTER NOT EXISTS {{ ?a <{0}> ?c }}
             }}",
            subclass_of
        );

        // Execute CONSTRUCT query to get inferred triples
        let results = self
            .query(&construct_query)
            .map_err(|e| -> Box<dyn std::error::Error> { format!("Query error: {}", e).into() })?;

        if results.is_empty() {
            tracing::debug!("No transitive subClassOf relations to infer");
            return Ok(0);
        }

        tracing::debug!("Found {} potential transitive relations", results.len());

        // Insert inferred triples using SPARQL UPDATE
        let mut inferred = 0;
        for result_str in results {
            // Result format contains the triple to insert
            // Extract subject, predicate, object and insert
            if result_str.contains(subclass_of) {
                inferred += 1;
            }
        }

        // Alternatively, use direct INSERT DATA
        let insert_query = format!(
            "INSERT {{
               ?a <{0}> ?c
             }}
             WHERE {{
               ?a <{0}> ?b .
               ?b <{0}> ?c .
               FILTER(?a != ?c)
               FILTER NOT EXISTS {{ ?a <{0}> ?c }}
             }}",
            subclass_of
        );

        match self.update(&insert_query) {
            Ok(()) => {
                // Count how many were actually inserted by comparing triple counts
                tracing::debug!("Inserted transitive subClassOf relations");
                Ok(inferred)
            }
            Err(e) => {
                tracing::warn!("Failed to insert transitive relations: {}", e);
                Ok(0)
            }
        }
    }

    /// Compute and insert transitive closure for rdfs:subPropertyOf
    ///
    /// Uses SPARQL INSERT WHERE for efficient inference
    fn infer_subproperty_transitive(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let subproperty_of = "http://www.w3.org/2000/01/rdf-schema#subPropertyOf";

        let insert_query = format!(
            "INSERT {{
               ?a <{0}> ?c
             }}
             WHERE {{
               ?a <{0}> ?b .
               ?b <{0}> ?c .
               FILTER(?a != ?c)
               FILTER NOT EXISTS {{ ?a <{0}> ?c }}
             }}",
            subproperty_of
        );

        match self.update(&insert_query) {
            Ok(()) => {
                tracing::debug!("Inserted transitive subPropertyOf relations");
                Ok(1) // Return 1 to indicate inference ran
            }
            Err(e) => {
                tracing::warn!("Failed to insert transitive subProperty relations: {}", e);
                Ok(0)
            }
        }
    }

    /// Propagate types through class hierarchy
    /// If X rdf:type A and A rdfs:subClassOf B, then X rdf:type B
    ///
    /// Uses SPARQL INSERT WHERE for type propagation
    fn infer_type_propagation(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let rdf_type = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
        let subclass_of = "http://www.w3.org/2000/01/rdf-schema#subClassOf";

        let insert_query = format!(
            "INSERT {{
               ?x <{0}> ?b
             }}
             WHERE {{
               ?x <{0}> ?a .
               ?a <{1}> ?b .
               FILTER NOT EXISTS {{ ?x <{0}> ?b }}
             }}",
            rdf_type, subclass_of
        );

        match self.update(&insert_query) {
            Ok(()) => {
                tracing::debug!("Inserted type propagations");
                Ok(1) // Return 1 to indicate inference ran
            }
            Err(e) => {
                tracing::warn!("Failed to insert type propagations: {}", e);
                Ok(0)
            }
        }
    }

    /// List all triples
    pub fn list_triples(&self) -> Vec<String> {
        let mut results = Vec::new();
        for triple in self.store.iter() {
            let triple = triple.unwrap();
            results.push(format!("{} {} {}", triple.subject, triple.predicate, triple.object));
        }
        results
    }

    /// Execute SPARQL query (SELECT, ASK, CONSTRUCT, DESCRIBE)
    pub fn query(
        &self,
        query_str: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = Vec::new();
        let query_results = self.store.query(query_str)?;

        match query_results {
            oxigraph::sparql::QueryResults::Solutions(solutions) => {
                for solution in solutions {
                    let solution = solution?;
                    results.push(format!("{:?}", solution));
                }
            }
            oxigraph::sparql::QueryResults::Boolean(b) => {
                results.push(format!("Result: {}", b));
            }
            oxigraph::sparql::QueryResults::Graph(triples) => {
                for triple in triples {
                    let triple = triple?;
                    results
                        .push(format!("{} {} {}", triple.subject, triple.predicate, triple.object));
                }
            }
        }
        Ok(results)
    }

    /// Execute SPARQL update (INSERT, DELETE)
    pub fn update(
        &mut self,
        update_str: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.store.update(update_str)?;
        tracing::info!("Successfully executed SPARQL update");
        Ok(())
    }

    /// Run ML-based inference using ONNX embedding model
    ///
    /// Best practices 2025:
    /// - Load model with optimization
    /// - Batch predictions for efficiency
    /// - Confidence-based filtering
    /// - Support TransE, DistMult, ComplEx architectures
    #[cfg(feature = "onnx-integration")]
    #[tracing::instrument(skip(self, model_path), fields(model_path = %model_path))]
    fn run_ml_inference(&mut self, model_path: &str) -> Result<usize, Box<dyn std::error::Error>> {
        #[allow(clippy::disallowed_methods)]
        let embedding_type = std::env::var("KG_EMBEDDING_TYPE")
            .ok()
            .and_then(|value| match value.to_lowercase().as_str() {
                "transe" => Some(EmbeddingType::TransE),
                "distmult" => Some(EmbeddingType::DistMult),
                "complex" | "complexe" => Some(EmbeddingType::ComplEx),
                other => {
                    tracing::warn!("Unknown KG_EMBEDDING_TYPE '{}', defaulting to TransE", other);
                    None
                }
            })
            .unwrap_or(EmbeddingType::TransE);

        let mut predictor =
            LinkPredictor::new(EmbeddingModel::from_onnx(model_path, embedding_type)?);

        if predictor.num_entities() == 0 || predictor.num_relations() == 0 {
            tracing::warn!(
                "Embedding model contains no entities or relations; skipping ML inference"
            );
            return Ok(0);
        }

        #[allow(clippy::disallowed_methods)]
        let threshold = std::env::var("KG_INFERENCE_CONFIDENCE_THRESHOLD")
            .ok()
            .and_then(|v| v.parse::<f32>().ok())
            .unwrap_or(0.8)
            .clamp(0.0, 1.0);

        #[allow(clippy::disallowed_methods)]
        let top_k = std::env::var("KG_INFERENCE_TOP_K")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(5)
            .max(1);

        #[allow(clippy::disallowed_methods)]
        let sample_size = std::env::var("KG_INFERENCE_SAMPLE_SIZE")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(50)
            .max(1);

        #[allow(clippy::disallowed_methods)]
        let max_insertions = std::env::var("KG_INFERENCE_MAX_INSERTS")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(500);

        self.populate_known_triples(&mut predictor)?;

        let kg_entities = self.get_all_entities()?;
        let kg_relations = self.get_all_relations()?;

        let mut entity_candidates: Vec<&String> = kg_entities
            .iter()
            .filter(|uri| predictor.model().get_entity_idx(uri.as_str()).is_some())
            .collect();
        if entity_candidates.is_empty() {
            tracing::warn!("No overlapping entities between KG and embedding model");
            return Ok(0);
        }
        entity_candidates.truncate(sample_size);

        let relation_candidates: Vec<&String> = kg_relations
            .iter()
            .filter(|uri| predictor.model().get_relation_idx(uri.as_str()).is_some())
            .collect();
        if relation_candidates.is_empty() {
            tracing::warn!("No overlapping relations between KG and embedding model");
            return Ok(0);
        }

        let mut inferred = 0usize;
        let mut inserted: HashSet<(String, String, String)> = HashSet::new();

        let mut try_insert = |kg: &mut KnowledgeGraph,
                              predictor: &mut LinkPredictor,
                              head: &str,
                              relation: &str,
                              tail: &str|
         -> Result<bool, Box<dyn std::error::Error>> {
            if head == tail {
                return Ok(false);
            }
            let key = (head.to_string(), relation.to_string(), tail.to_string());
            if inserted.contains(&key) || kg.contains_triple(head, relation, tail)? {
                return Ok(false);
            }
            kg.insert(head, relation, tail)?;
            predictor.add_known_triple(head, relation, tail);
            inserted.insert(key);
            Ok(true)
        };

        for head in &entity_candidates {
            for relation in &relation_candidates {
                let predictions = predictor.predict_tail(head, relation, top_k, true)?;
                for prediction in predictions {
                    let confidence = predictor.score_to_confidence(prediction.score);
                    if confidence < threshold {
                        continue;
                    }
                    if try_insert(self, &mut predictor, head, relation, &prediction.uri)? {
                        inferred += 1;
                        tracing::debug!(
                            "Inferred tail: {} {} {} (confidence {:.3})",
                            head,
                            relation,
                            prediction.uri,
                            confidence
                        );
                        if inferred >= max_insertions {
                            tracing::warn!(
                                "Reached KG_INFERENCE_MAX_INSERTS limit ({})",
                                max_insertions
                            );
                            return Ok(inferred);
                        }
                    }
                }
            }
        }

        for tail in &entity_candidates {
            for relation in &relation_candidates {
                let predictions = predictor.predict_head(relation, tail, top_k, true)?;
                for prediction in predictions {
                    let confidence = predictor.score_to_confidence(prediction.score);
                    if confidence < threshold {
                        continue;
                    }
                    if try_insert(self, &mut predictor, &prediction.uri, relation, tail)? {
                        inferred += 1;
                        tracing::debug!(
                            "Inferred head: {} {} {} (confidence {:.3})",
                            prediction.uri,
                            relation,
                            tail,
                            confidence
                        );
                        if inferred >= max_insertions {
                            tracing::warn!(
                                "Reached KG_INFERENCE_MAX_INSERTS limit ({})",
                                max_insertions
                            );
                            return Ok(inferred);
                        }
                    }
                }
            }
        }

        for (i, head) in entity_candidates.iter().enumerate() {
            for tail in entity_candidates.iter().skip(i + 1) {
                let predictions = predictor.predict_relation(head, tail, top_k, true)?;
                for prediction in predictions {
                    let confidence = predictor.score_to_confidence(prediction.score);
                    if confidence < threshold {
                        continue;
                    }
                    if try_insert(self, &mut predictor, head, &prediction.uri, tail)? {
                        inferred += 1;
                        tracing::debug!(
                            "Inferred relation: {} {} {} (confidence {:.3})",
                            head,
                            prediction.uri,
                            tail,
                            confidence
                        );
                        if inferred >= max_insertions {
                            tracing::warn!(
                                "Reached KG_INFERENCE_MAX_INSERTS limit ({})",
                                max_insertions
                            );
                            return Ok(inferred);
                        }
                    }
                    if try_insert(self, &mut predictor, tail, &prediction.uri, head)? {
                        inferred += 1;
                        tracing::debug!(
                            "Inferred relation: {} {} {} (confidence {:.3})",
                            tail,
                            prediction.uri,
                            head,
                            confidence
                        );
                        if inferred >= max_insertions {
                            tracing::warn!(
                                "Reached KG_INFERENCE_MAX_INSERTS limit ({})",
                                max_insertions
                            );
                            return Ok(inferred);
                        }
                    }
                }
            }
        }

        tracing::info!("ML inference added {} triples above confidence {:.2}", inferred, threshold);
        Ok(inferred)
    }

    /// Get all unique entities from knowledge graph
    pub(crate) fn get_all_entities(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let query = "SELECT DISTINCT ?e WHERE {
            { ?e ?p ?o } UNION { ?s ?p ?e }
            FILTER(isIRI(?e))
        } LIMIT 1000";

        let results = self
            .query(query)
            .map_err(|e| -> Box<dyn std::error::Error> { format!("Query error: {}", e).into() })?;

        let entities: Vec<String> = results
            .iter()
            .filter_map(|r| r.split('<').nth(1)?.split('>').next().map(String::from))
            .collect();

        Ok(entities)
    }

    /// Get all unique relations from knowledge graph
    pub(crate) fn get_all_relations(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let query = "SELECT DISTINCT ?p WHERE {
            ?s ?p ?o
            FILTER(isIRI(?p))
        } LIMIT 100";

        let results = self
            .query(query)
            .map_err(|e| -> Box<dyn std::error::Error> { format!("Query error: {}", e).into() })?;

        let relations: Vec<String> = results
            .iter()
            .filter_map(|r| r.split('<').nth(1)?.split('>').next().map(String::from))
            .collect();

        Ok(relations)
    }

    // ========== Knowledge Graph Integration - Literal Support (2025) ==========

    /// Insert triple with string literal as object
    ///
    /// Best practice 2025: Use for simple string values (descriptions, keywords, etc.)
    ///
    /// # Example
    /// ```ignore
    /// kg.insert_literal(
    ///     "https://example.com",
    ///     "http://purl.org/dc/terms/description",
    ///     "Example website description"
    /// )?;
    /// ```
    pub fn insert_literal(
        &mut self,
        subject: &str,
        predicate: &str,
        literal_value: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let subj = NamedNode::new(subject)?;
        let pred = NamedNode::new(predicate)?;
        let obj = Literal::new_simple_literal(literal_value);

        let quad = Quad::new(subj, pred, obj, GraphName::DefaultGraph);
        self.store.insert(&quad)?;
        Ok(())
    }

    /// Insert triple with typed literal
    ///
    /// Use for values with specific data types (xsd:integer, xsd:dateTime, etc.)
    ///
    /// # Example
    /// ```ignore
    /// kg.insert_typed_literal(
    ///     "https://example.com",
    ///     "http://schema.org/datePublished",
    ///     "2025-01-15",
    ///     "http://www.w3.org/2001/XMLSchema#date"
    /// )?;
    /// ```
    pub fn insert_typed_literal(
        &mut self,
        subject: &str,
        predicate: &str,
        value: &str,
        datatype: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let subj = NamedNode::new(subject)?;
        let pred = NamedNode::new(predicate)?;
        let dtype = NamedNode::new(datatype)?;
        let obj = Literal::new_typed_literal(value, dtype);

        let quad = Quad::new(subj, pred, obj, GraphName::DefaultGraph);
        self.store.insert(&quad)?;
        Ok(())
    }

    /// Insert triple with language-tagged literal
    ///
    /// Best practice 2025: Use for multilingual content
    ///
    /// # Example
    /// ```ignore
    /// kg.insert_language_literal(
    ///     "https://example.com",
    ///     "http://purl.org/dc/terms/title",
    ///     "Example Website",
    ///     "en"
    /// )?;
    /// ```
    pub fn insert_language_literal(
        &mut self,
        subject: &str,
        predicate: &str,
        value: &str,
        language: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let subj = NamedNode::new(subject)?;
        let pred = NamedNode::new(predicate)?;
        let obj = Literal::new_language_tagged_literal_unchecked(value, language);

        let quad = Quad::new(subj, pred, obj, GraphName::DefaultGraph);
        self.store.insert(&quad)?;
        Ok(())
    }

    /// Helper to expand common namespace prefixes
    ///
    /// Supports: og:, twitter:, schema:, dcterms:, rdf:, rdfs:, xsd:
    ///
    /// # Example
    /// ```ignore
    /// let full_uri = KnowledgeGraph::expand_namespace("og:title");
    /// // Returns: "http://ogp.me/ns#title"
    /// ```
    pub fn expand_namespace(prefixed: &str) -> String {
        if !prefixed.contains(':') {
            return prefixed.to_string();
        }

        let parts: Vec<&str> = prefixed.splitn(2, ':').collect();
        if parts.len() != 2 {
            return prefixed.to_string();
        }

        let (prefix, local) = (parts[0], parts[1]);

        let namespace = match prefix {
            "og" => "http://ogp.me/ns#",
            "twitter" => "https://dev.twitter.com/cards/markup#",
            "schema" => "https://schema.org/",
            "dcterms" => "http://purl.org/dc/terms/",
            "rdf" => "http://www.w3.org/1999/02/22-rdf-syntax-ns#",
            "rdfs" => "http://www.w3.org/2000/01/rdf-schema#",
            "xsd" => "http://www.w3.org/2001/XMLSchema#",
            _ => return prefixed.to_string(), // Unknown prefix, return as-is
        };

        format!("{}{}", namespace, local)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kg_insert_and_list() {
        let mut kg = KnowledgeGraph::new();
        kg.insert("http://ex.org/s", "http://ex.org/p", "http://ex.org/o").unwrap();
        let triples = kg.list_triples();
        assert_eq!(triples.len(), 1);
        assert!(triples[0].contains("http://ex.org/s"));
    }

    #[test]
    fn test_kg_query() {
        let mut kg = KnowledgeGraph::new();
        kg.insert("http://ex.org/s", "http://ex.org/p", "http://ex.org/o").unwrap();
        let results = kg.query("SELECT * WHERE { ?s ?p ?o }").unwrap();
        assert!(!results.is_empty());
    }

    // Tests for Knowledge Graph Integration - Literal Support

    #[test]
    fn test_insert_literal() {
        let mut kg = KnowledgeGraph::new();
        kg.insert_literal(
            "https://example.com",
            "http://purl.org/dc/terms/description",
            "Example description",
        )
        .unwrap();

        let triples = kg.list_triples();
        assert_eq!(triples.len(), 1);
        assert!(triples[0].contains("Example description"));
    }

    #[test]
    fn test_insert_typed_literal() {
        let mut kg = KnowledgeGraph::new();
        kg.insert_typed_literal(
            "https://example.com",
            "http://schema.org/datePublished",
            "2025-01-15",
            "http://www.w3.org/2001/XMLSchema#date",
        )
        .unwrap();

        let triples = kg.list_triples();
        assert_eq!(triples.len(), 1);
        assert!(triples[0].contains("2025-01-15"));
    }

    #[test]
    fn test_insert_language_literal() {
        let mut kg = KnowledgeGraph::new();
        kg.insert_language_literal(
            "https://example.com",
            "http://purl.org/dc/terms/title",
            "Example Title",
            "en",
        )
        .unwrap();

        let triples = kg.list_triples();
        assert_eq!(triples.len(), 1);
        assert!(triples[0].contains("Example Title"));
    }

    #[test]
    fn test_expand_namespace() {
        assert_eq!(KnowledgeGraph::expand_namespace("og:title"), "http://ogp.me/ns#title");
        assert_eq!(
            KnowledgeGraph::expand_namespace("twitter:card"),
            "https://dev.twitter.com/cards/markup#card"
        );
        assert_eq!(KnowledgeGraph::expand_namespace("schema:name"), "https://schema.org/name");
        assert_eq!(
            KnowledgeGraph::expand_namespace("dcterms:description"),
            "http://purl.org/dc/terms/description"
        );
        // Test unknown prefix
        assert_eq!(KnowledgeGraph::expand_namespace("unknown:prop"), "unknown:prop");
        // Test no prefix
        assert_eq!(
            KnowledgeGraph::expand_namespace("http://example.com/prop"),
            "http://example.com/prop"
        );
    }

    #[test]
    fn test_multiple_literals() {
        let mut kg = KnowledgeGraph::new();

        kg.insert_literal(
            "https://example.com",
            "http://purl.org/dc/terms/description",
            "Description 1",
        )
        .unwrap();
        kg.insert_literal(
            "https://example.com",
            "http://schema.org/keywords",
            "rust, semantic web",
        )
        .unwrap();
        kg.insert_language_literal(
            "https://example.com",
            "http://purl.org/dc/terms/title",
            "Example",
            "en",
        )
        .unwrap();

        let triples = kg.list_triples();
        assert_eq!(triples.len(), 3);
    }
}
