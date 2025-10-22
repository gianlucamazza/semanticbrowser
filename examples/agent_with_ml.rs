use semantic_browser::kg::KnowledgeGraph;
use semantic_browser::llm::{
    AgentOrchestrator, AgentTask, LLMConfig, OllamaConfig, OllamaProvider, ToolRegistry,
};
use semantic_browser::ml::embeddings::{EmbeddingModel, EmbeddingType};
use semantic_browser::ml::inference::LinkPredictor;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 Semantic Browser - Agent with ML Integration");
    println!("================================================");

    // 1. Setup LLM Provider
    let provider = Arc::new(OllamaProvider::new(OllamaConfig::default()));
    let config = LLMConfig {
        model: "llama3:8b".to_string(),
        temperature: 0.7,
        max_tokens: Some(1024),
        ..Default::default()
    };

    // 2. Setup Knowledge Graph
    let mut kg = KnowledgeGraph::new();

    // Add some sample data
    kg.insert("http://ex.org/Alice", "http://ex.org/knows", "http://ex.org/Bob")?;
    kg.insert("http://ex.org/Bob", "http://ex.org/worksAt", "http://ex.org/Company")?;
    kg.insert("http://ex.org/Alice", "http://ex.org/name", "Alice Smith")?;

    // 3. Setup ML Link Predictor
    let mut embedding_model = EmbeddingModel::new_simple(EmbeddingType::TransE, 50);
    embedding_model.insert_entity_embedding("http://ex.org/Alice", vec![1.0; 50])?;
    embedding_model.insert_entity_embedding("http://ex.org/Bob", vec![0.8; 50])?;
    embedding_model.insert_entity_embedding("http://ex.org/Company", vec![0.5; 50])?;
    embedding_model.insert_relation_embedding("http://ex.org/knows", vec![0.2; 50])?;
    embedding_model.insert_relation_embedding("http://ex.org/worksAt", vec![0.3; 50])?;

    let mut predictor = LinkPredictor::new(embedding_model);
    predictor.add_known_triple("http://ex.org/Alice", "http://ex.org/knows", "http://ex.org/Bob");

    // 4. Create Agent with ML capabilities
    let tools = ToolRegistry::with_browser_tools();
    let agent =
        AgentOrchestrator::new(provider, config, tools).with_kg(kg).with_predictor(predictor);

    // 5. Define task that uses ML reasoning
    let task = AgentTask::new(
        "Find out who Alice knows and predict where she might work. \
         Use the knowledge graph to query existing relationships and \
         ML predictions to suggest new connections.",
    )
    .with_context("Alice is connected to Bob who works at Company. Use this to make predictions.")
    .with_max_iterations(5);

    // 6. Execute task
    println!("\n📋 Task: {}", task.goal);
    println!("{}", "─".repeat(50));

    let response = agent.execute(task).await?;

    println!("\n✅ Result: {}", response.result);
    println!("Iterations: {}", response.iterations);
    if let Some(error) = response.error {
        println!("❌ Error: {}", error);
    }

    Ok(())
}
