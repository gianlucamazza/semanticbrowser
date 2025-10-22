//! Prometheus metrics implementation for observability
//!
//! Best practices 2025:
//! - RED metrics (Rate, Errors, Duration) for all services
//! - Prometheus format for metrics exposition
//! - Custom business metrics for KG operations, browser automation, ML inference
//! - Health check endpoints for Kubernetes readiness/liveness probes

use lazy_static::lazy_static;
use prometheus::{
    register_counter_vec, register_gauge_vec, register_histogram_vec, register_int_gauge,
    CounterVec, Encoder, GaugeVec, HistogramVec, TextEncoder,
};

/// Initialize Prometheus metrics registry
pub fn init_metrics() -> Result<(), Box<dyn std::error::Error>> {
    // Force initialization by accessing each metric
    let _ = &*API_REQUESTS_TOTAL;
    let _ = &*API_REQUEST_DURATION;
    let _ = &*API_REQUESTS_ERRORS;
    let _ = &*KG_OPERATIONS_TOTAL;
    let _ = &*KG_INFERENCE_DURATION;
    let _ = &*BROWSER_OPERATIONS_TOTAL;
    let _ = &*BROWSER_OPERATION_DURATION;
    let _ = &*ML_INFERENCE_OPERATIONS;
    let _ = &*ML_INFERENCE_DURATION;
    let _ = &*PARSE_OPERATIONS_TOTAL;
    let _ = &*PARSE_OPERATION_DURATION;
    let _ = &*ACTIVE_CONNECTIONS;
    let _ = &*UPTIME_SECONDS;

    tracing::info!("Prometheus metrics initialized");
    Ok(())
}

/// Get metrics in Prometheus format for HTTP response
pub fn get_metrics_handler() -> Result<String, Box<dyn std::error::Error>> {
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer)?;
    Ok(String::from_utf8(buffer)?)
}

// ===== API METRICS =====

lazy_static! {
    /// Total API requests by endpoint and method
    pub static ref API_REQUESTS_TOTAL: CounterVec = register_counter_vec!(
        "semantic_browser_api_requests_total",
        "Total number of API requests",
        &["endpoint", "method", "status"]
    ).expect("Failed to register API_REQUESTS_TOTAL");

    /// API request duration in seconds
    pub static ref API_REQUEST_DURATION: HistogramVec = register_histogram_vec!(
        "semantic_browser_api_request_duration_seconds",
        "API request duration in seconds",
        &["endpoint", "method"],
        vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 2.0, 5.0, 10.0]
    ).expect("Failed to register API_REQUEST_DURATION");

    /// API request errors by type
    pub static ref API_REQUESTS_ERRORS: CounterVec = register_counter_vec!(
        "semantic_browser_api_requests_errors_total",
        "Total number of API request errors",
        &["endpoint", "error_type"]
    ).expect("Failed to register API_REQUESTS_ERRORS");
}

// ===== KNOWLEDGE GRAPH METRICS =====

lazy_static! {
    /// Total KG operations (insert, query, infer)
    pub static ref KG_OPERATIONS_TOTAL: CounterVec = register_counter_vec!(
        "semantic_browser_kg_operations_total",
        "Total number of Knowledge Graph operations",
        &["operation", "result"]
    ).expect("Failed to register KG_OPERATIONS_TOTAL");

    /// KG inference duration
    pub static ref KG_INFERENCE_DURATION: HistogramVec = register_histogram_vec!(
        "semantic_browser_kg_inference_duration_seconds",
        "Knowledge Graph inference duration in seconds",
        &["inference_type"],
        vec![0.001, 0.01, 0.1, 0.5, 1.0, 5.0, 10.0, 30.0]
    ).expect("Failed to register KG_INFERENCE_DURATION");

    /// Current KG size (number of triples)
    pub static ref KG_SIZE: GaugeVec = register_gauge_vec!(
        "semantic_browser_kg_size",
        "Current Knowledge Graph size (number of triples)",
        &["graph_type"]
    ).expect("Failed to register KG_SIZE");
}

// ===== BROWSER AUTOMATION METRICS =====

lazy_static! {
    /// Total browser operations
    pub static ref BROWSER_OPERATIONS_TOTAL: CounterVec = register_counter_vec!(
        "semantic_browser_browser_operations_total",
        "Total number of browser automation operations",
        &["operation", "result"]
    ).expect("Failed to register BROWSER_OPERATIONS_TOTAL");

    /// Browser operation duration
    pub static ref BROWSER_OPERATION_DURATION: HistogramVec = register_histogram_vec!(
        "semantic_browser_browser_operation_duration_seconds",
        "Browser operation duration in seconds",
        &["operation"],
        vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0]
    ).expect("Failed to register BROWSER_OPERATION_DURATION");

    /// Active browser instances
    pub static ref ACTIVE_BROWSER_INSTANCES: GaugeVec = register_gauge_vec!(
        "semantic_browser_active_browser_instances",
        "Number of active browser instances",
        &["pool_type"]
    ).expect("Failed to register ACTIVE_BROWSER_INSTANCES");
}

// ===== MACHINE LEARNING METRICS =====

lazy_static! {
    /// Total ML inference operations
    pub static ref ML_INFERENCE_OPERATIONS: CounterVec = register_counter_vec!(
        "semantic_browser_ml_inference_operations_total",
        "Total number of ML inference operations",
        &["model_type", "operation", "result"]
    ).expect("Failed to register ML_INFERENCE_OPERATIONS");

    /// ML inference duration
    pub static ref ML_INFERENCE_DURATION: HistogramVec = register_histogram_vec!(
        "semantic_browser_ml_inference_duration_seconds",
        "ML inference duration in seconds",
        &["model_type", "operation"],
        vec![0.001, 0.01, 0.1, 0.5, 1.0, 2.0, 5.0]
    ).expect("Failed to register ML_INFERENCE_DURATION");

    /// ML model confidence scores
    pub static ref ML_MODEL_CONFIDENCE: HistogramVec = register_histogram_vec!(
        "semantic_browser_ml_model_confidence",
        "ML model confidence scores",
        &["model_type"],
        vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
    ).expect("Failed to register ML_MODEL_CONFIDENCE");
}

// ===== PARSING METRICS =====

lazy_static! {
    /// Total parsing operations
    pub static ref PARSE_OPERATIONS_TOTAL: CounterVec = register_counter_vec!(
        "semantic_browser_parse_operations_total",
        "Total number of HTML parsing operations",
        &["content_type", "result"]
    ).expect("Failed to register PARSE_OPERATIONS_TOTAL");

    /// Parsing operation duration
    pub static ref PARSE_OPERATION_DURATION: HistogramVec = register_histogram_vec!(
        "semantic_browser_parse_operation_duration_seconds",
        "HTML parsing duration in seconds",
        &["content_type"],
        vec![0.001, 0.01, 0.1, 0.5, 1.0, 2.0]
    ).expect("Failed to register PARSE_OPERATION_DURATION");
}

// ===== SYSTEM METRICS =====

lazy_static! {
    /// Active connections
    pub static ref ACTIVE_CONNECTIONS: GaugeVec = register_gauge_vec!(
        "semantic_browser_active_connections",
        "Number of active connections",
        &["connection_type"]
    ).expect("Failed to register ACTIVE_CONNECTIONS");

    /// Service uptime in seconds
    pub static ref UPTIME_SECONDS: prometheus::IntGauge = register_int_gauge!(
        "semantic_browser_uptime_seconds",
        "Service uptime in seconds"
    ).expect("Failed to register UPTIME_SECONDS");
}

// ===== METRICS RECORDING FUNCTIONS =====

/// Record API request metrics
pub fn record_api_request(endpoint: &str, method: &str, status: &str, duration: f64) {
    API_REQUESTS_TOTAL.with_label_values(&[endpoint, method, status]).inc();
    API_REQUEST_DURATION.with_label_values(&[endpoint, method]).observe(duration);

    if !status.starts_with('2') {
        API_REQUESTS_ERRORS.with_label_values(&[endpoint, "http_error"]).inc();
    }
}

/// Record KG operation metrics
pub fn record_kg_operation(operation: &str, result: &str, duration: Option<f64>) {
    KG_OPERATIONS_TOTAL.with_label_values(&[operation, result]).inc();

    if let Some(dur) = duration {
        KG_INFERENCE_DURATION.with_label_values(&[operation]).observe(dur);
    }
}

/// Record browser operation metrics
pub fn record_browser_operation(operation: &str, result: &str, duration: f64) {
    BROWSER_OPERATIONS_TOTAL.with_label_values(&[operation, result]).inc();
    BROWSER_OPERATION_DURATION.with_label_values(&[operation]).observe(duration);
}

/// Record ML inference metrics
pub fn record_ml_inference(
    model_type: &str,
    operation: &str,
    result: &str,
    duration: f64,
    confidence: Option<f64>,
) {
    ML_INFERENCE_OPERATIONS.with_label_values(&[model_type, operation, result]).inc();
    ML_INFERENCE_DURATION.with_label_values(&[model_type, operation]).observe(duration);

    if let Some(conf) = confidence {
        ML_MODEL_CONFIDENCE.with_label_values(&[model_type]).observe(conf);
    }
}

/// Record parsing operation metrics
pub fn record_parse_operation(content_type: &str, result: &str, duration: f64) {
    PARSE_OPERATIONS_TOTAL.with_label_values(&[content_type, result]).inc();
    PARSE_OPERATION_DURATION.with_label_values(&[content_type]).observe(duration);
}

/// Update KG size metric
pub fn update_kg_size(graph_type: &str, size: f64) {
    KG_SIZE.with_label_values(&[graph_type]).set(size);
}

/// Update active connections metric
pub fn update_active_connections(connection_type: &str, count: f64) {
    ACTIVE_CONNECTIONS.with_label_values(&[connection_type]).set(count);
}

/// Update uptime metric
pub fn update_uptime(seconds: i64) {
    UPTIME_SECONDS.set(seconds);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_initialization() {
        init_metrics().expect("Metrics initialization should succeed");
    }

    #[test]
    fn test_metrics_recording() {
        // Test API metrics
        record_api_request("/parse", "POST", "200", 0.1);
        record_api_request("/parse", "POST", "500", 0.05);

        // Test KG metrics
        record_kg_operation("infer", "success", Some(0.5));

        // Test browser metrics
        record_browser_operation("browse", "success", 2.0);

        // Test ML metrics
        record_ml_inference("bert", "ner", "success", 0.1, Some(0.95));

        // Test parsing metrics
        record_parse_operation("html", "success", 0.01);

        // Test system metrics
        update_kg_size("main", 1000.0);
        update_active_connections("http", 5.0);
        update_uptime(3600);
    }

    #[test]
    fn test_metrics_export() {
        init_metrics().expect("Metrics initialization should succeed");
        let metrics = get_metrics_handler().expect("Metrics export should succeed");
        assert!(!metrics.is_empty());
        // Check for semantic_browser metrics
        assert!(metrics.contains("semantic_browser"));
    }
}
