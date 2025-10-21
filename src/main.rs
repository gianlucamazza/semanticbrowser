#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber with env filter
    // Set RUST_LOG environment variable to control log level
    // Example: RUST_LOG=debug cargo run
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();

    tracing::info!("Starting Semantic Browser Agent");

    // Initialize NER model if configured
    semantic_browser::annotator::init_ner_model();

    semantic_browser::api::run_server().await
}
