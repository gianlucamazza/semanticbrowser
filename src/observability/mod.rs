//! Observability module for metrics and monitoring
//!
//! Best practices 2025:
//! - RED metrics (Rate, Errors, Duration) for all services
//! - Prometheus format for metrics exposition
//! - Custom metrics for business logic (KG operations, browser automation)
//! - Health check endpoints for Kubernetes readiness/liveness probes

#[cfg(feature = "observability")]
pub mod metrics;

#[cfg(feature = "observability")]
pub use metrics::{
    get_metrics_handler, init_metrics, record_api_request, record_browser_operation,
    record_kg_operation, record_parse_operation,
};
