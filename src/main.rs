/// Initialize tracing with OpenTelemetry support if configured
///
/// Best practices 2025:
/// - Use OpenTelemetry for distributed tracing and observability
/// - Fall back to simple fmt subscriber if OTEL not configured
/// - Configure via environment variables (OTEL_EXPORTER_OTLP_ENDPOINT)
#[cfg(feature = "telemetry")]
fn init_telemetry() -> Result<(), Box<dyn std::error::Error>> {
    use opentelemetry::trace::TracerProvider as _;
    use opentelemetry::KeyValue;
    use opentelemetry_sdk::Resource;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    // Get service name from env or use default
    #[allow(clippy::disallowed_methods)]
    let service_name =
        std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "semantic-browser".to_string());

    // Create OTLP exporter using new 0.27+ API
    let exporter = opentelemetry_otlp::SpanExporter::builder().with_tonic().build()?;

    // Build tracer provider with resource
    let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_resource(Resource::new(vec![KeyValue::new("service.name", service_name.clone())]))
        .build();

    // Get tracer
    let tracer = tracer_provider.tracer("semantic-browser");

    // Set as global provider
    opentelemetry::global::set_tracer_provider(tracer_provider);

    // Create OpenTelemetry tracing layer
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Create env filter layer
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        tracing_subscriber::EnvFilter::new(
            "warn,semantic_browser=info,chromiumoxide::conn=off,chromiumoxide::handler=off",
        )
    });

    // Create fmt layer
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true);

    // Combine layers
    tracing_subscriber::registry().with(env_filter).with(fmt_layer).with(telemetry_layer).init();

    tracing::info!("OpenTelemetry initialized with service name: {}", service_name);
    Ok(())
}

#[cfg(not(feature = "telemetry"))]
fn init_telemetry() -> Result<(), Box<dyn std::error::Error>> {
    // Simple fmt subscriber when telemetry feature is disabled
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(
            |_| {
                tracing_subscriber::EnvFilter::new(
                    "warn,semantic_browser=info,chromiumoxide::conn=off,chromiumoxide::handler=off",
                )
            },
        ))
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .init();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber with OpenTelemetry if enabled
    // Set RUST_LOG environment variable to control log level (default: warn,semantic_browser=info,chromiumoxide::conn=off,chromiumoxide::handler=off)
    // Set OTEL_EXPORTER_OTLP_ENDPOINT to enable OpenTelemetry (e.g., http://localhost:4317)
    // Set OTEL_SERVICE_NAME to override service name (default: semantic-browser)
    // Example: RUST_LOG=debug OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 cargo run --features telemetry
    init_telemetry()?;

    tracing::info!("Starting Semantic Browser Agent");

    // Initialize JWT authentication
    semantic_browser::auth::JwtConfig::init()
        .map_err(|e| format!("Failed to initialize JWT config: {}", e))?;

    // Initialize NER model if configured
    semantic_browser::annotator::init_ner_model();

    let result = semantic_browser::api::run_server().await;

    // Shutdown OpenTelemetry on exit
    #[cfg(feature = "telemetry")]
    {
        tracing::info!("Shutting down OpenTelemetry");
        opentelemetry::global::shutdown_tracer_provider();
    }

    result
}
