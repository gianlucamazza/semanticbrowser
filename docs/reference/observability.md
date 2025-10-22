# Observability Guide

The Semantic Browser provides comprehensive observability features for monitoring, debugging, and performance analysis. This guide covers metrics collection, distributed tracing, and logging configuration.

## Overview

Observability in the Semantic Browser includes:

- **Metrics**: Prometheus-compatible metrics for monitoring system health and performance
- **Tracing**: OpenTelemetry distributed tracing for request tracking
- **Logging**: Structured logging with configurable levels and formats

## Metrics Collection

### Enabling Metrics

Build with observability features:

```bash
cargo build --features observability
```

Configure metrics in `.env`:

```bash
# Enable Prometheus metrics endpoint
PROMETHEUS_METRICS=true

# Port for metrics endpoint (default: 9090)
METRICS_PORT=9090
```

### Accessing Metrics

Metrics are exposed at `/metrics` endpoint:

```bash
curl http://localhost:9090/metrics
```

### Available Metrics

#### HTTP Request Metrics
- `semantic_browser_http_requests_total{endpoint,method,status}` - Total HTTP requests
- `semantic_browser_http_request_duration_seconds{endpoint,method}` - Request duration histogram

#### Knowledge Graph Metrics
- `semantic_browser_kg_operations_total{operation}` - KG operations (insert, query, delete)
- `semantic_browser_kg_operation_duration_seconds{operation}` - Operation duration
- `semantic_browser_kg_size{graph_type}` - Current KG size by graph type

#### Browser Automation Metrics
- `semantic_browser_browser_operations_total{result}` - Browser operations by result
- `semantic_browser_browser_operation_duration_seconds` - Operation duration

#### ML Inference Metrics
- `semantic_browser_ml_inference_total{model_type}` - ML inference operations
- `semantic_browser_ml_inference_duration_seconds{model_type,confidence}` - Inference duration with confidence

#### Parse Operations
- `semantic_browser_parse_operations_total{content_type,result}` - Parse operations
- `semantic_browser_parse_operation_duration_seconds{content_type}` - Parse duration

#### System Metrics
- `semantic_browser_active_connections{type}` - Active connections by type
- `semantic_browser_uptime_seconds` - Server uptime

### Prometheus Configuration

```yaml
scrape_configs:
  - job_name: 'semantic-browser'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 15s
    metrics_path: '/metrics'
```

### Grafana Dashboard

Example panels:
- Request rate and latency by endpoint
- Error rates and 5xx responses
- KG growth over time
- ML inference performance and accuracy
- Browser operation success rates
- Memory and CPU usage

## Distributed Tracing

### OpenTelemetry Setup

Configure tracing in `.env`:

```bash
# OpenTelemetry endpoint
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317

# Service identification
OTEL_SERVICE_NAME=semantic-browser
OTEL_SERVICE_VERSION=0.1.0

# Tracing configuration
OTEL_TRACES_EXPORTER=otlp
OTEL_TRACES_SAMPLER=always_on
```

### Supported Tracing Backends

- **Jaeger**: `OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:16686/api/traces`
- **Zipkin**: `OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:9411/api/v2/spans`
- **Honeycomb**: `OTEL_EXPORTER_OTLP_ENDPOINT=https://api.honeycomb.io:443`
- **DataDog**: `OTEL_EXPORTER_OTLP_ENDPOINT=https://trace.agent.datadoghq.com`

### Trace Context

The Semantic Browser automatically instruments:

- HTTP request/response cycles
- Knowledge Graph operations
- Browser automation workflows
- ML inference calls
- External API calls

### Custom Tracing

Add custom spans in code:

```rust
use tracing::{info, instrument};

#[instrument]
pub async fn my_function(param: &str) {
    info!("Processing {}", param);
    // Function logic here
}
```

## Logging Configuration

### Log Levels

Configure global log level:

```bash
# Available levels: trace, debug, info, warn, error
RUST_LOG=info
```

### Module-Specific Logging

```bash
# Set specific levels for modules
RUST_LOG=semantic_browser=debug,semantic_browser::api=trace,tract=info
```

### Production Logging

```bash
# Structured JSON logging for production
RUST_LOG=warn
# Use log aggregation system (ELK, Loki, etc.)
```

### Log Format

Default format includes:
- Timestamp
- Log level
- Module path
- Message
- Structured fields (when applicable)

### Debug Logging Examples

```bash
# Debug KG operations
RUST_LOG=semantic_browser::kg=debug

# Debug browser automation
RUST_LOG=semantic_browser::browser=debug

# Debug ML inference
RUST_LOG=semantic_browser::ml=debug
```

## Monitoring Best Practices

### Alerting Rules

Example Prometheus alerting rules:

```yaml
groups:
  - name: semantic_browser
    rules:
      - alert: HighErrorRate
        expr: rate(semantic_browser_http_requests_total{status=~"5.."}[5m]) / rate(semantic_browser_http_requests_total[5m]) > 0.05
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"

      - alert: KGSizeAnomaly
        expr: increase(semantic_browser_kg_size[1h]) < 0
        for: 5m
        labels:
          severity: info
        annotations:
          summary: "Knowledge Graph size decreased unexpectedly"
```

### Health Checks

The application provides health check endpoints:

- `GET /health` - Basic health check
- `GET /metrics` - Detailed metrics (when enabled)

### Performance Monitoring

Key metrics to monitor:

1. **Latency**: P95 request duration should be < 500ms
2. **Error Rate**: Should be < 1% for production workloads
3. **Resource Usage**: Memory and CPU usage within limits
4. **KG Performance**: Query/insert operations should complete within SLA
5. **ML Inference**: Model inference time and accuracy

## Troubleshooting with Observability

### High Latency Issues

1. Check `semantic_browser_http_request_duration_seconds` histogram
2. Identify slow endpoints and operations
3. Review traces for bottlenecks
4. Check resource utilization

### Memory Leaks

1. Monitor `semantic_browser_active_connections`
2. Check KG size growth with `semantic_browser_kg_size`
3. Review browser pool usage
4. Enable memory profiling if needed

### ML Performance Issues

1. Check `semantic_browser_ml_inference_duration_seconds`
2. Monitor model accuracy and confidence scores
3. Review input data quality
4. Consider model optimization or retraining

### Browser Automation Failures

1. Check `semantic_browser_browser_operations_total{result="error"}`
2. Review browser pool configuration
3. Check network connectivity and timeouts
4. Enable debug logging for browser operations

## Configuration Reference

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `PROMETHEUS_METRICS` | `false` | Enable Prometheus metrics |
| `METRICS_PORT` | `9090` | Metrics endpoint port |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | - | OTLP endpoint URL |
| `OTEL_SERVICE_NAME` | `semantic-browser` | Service name for tracing |
| `OTEL_TRACES_EXPORTER` | - | Tracing exporter (otlp, jaeger, zipkin) |
| `RUST_LOG` | `info` | Global log level |

### Feature Flags

- `--features observability` - Enable metrics and tracing
- Individual flags for specific observability components

## Integration Examples

### Docker Compose Monitoring

```yaml
version: '3.8'
services:
  semantic-browser:
    # ... app config ...
    environment:
      - PROMETHEUS_METRICS=true
      - OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector:4317

  prometheus:
    image: prom/prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml

  otel-collector:
    image: otel/opentelemetry-collector
    # ... collector config ...
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: semantic-browser
spec:
  template:
    spec:
      containers:
      - name: semantic-browser
        env:
        - name: PROMETHEUS_METRICS
          value: "true"
        - name: OTEL_EXPORTER_OTLP_ENDPOINT
          value: "http://opentelemetry-collector:4317"
        ports:
        - containerPort: 9090
          name: metrics
```

This comprehensive observability setup ensures you can monitor, debug, and optimize the Semantic Browser effectively in production environments.