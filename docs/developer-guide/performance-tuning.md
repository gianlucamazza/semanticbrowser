# Performance Tuning Guide

This guide provides optimization strategies and configuration tips for maximizing the performance of the Semantic Browser.

## System Requirements

### Minimum Requirements
- CPU: 2 cores
- RAM: 4GB
- Storage: 10GB SSD
- Network: 100Mbps

### Recommended for Production
- CPU: 4+ cores
- RAM: 8GB+
- Storage: 50GB+ SSD
- Network: 1Gbps

## Configuration Tuning

### Environment Variables

```bash
# Server Configuration
export PORT=3000
export WORKERS=4  # Match CPU cores

# Knowledge Graph
export KG_PERSIST_PATH=/fast/ssd/kg
export KG_INFERENCE_TOP_K=50
export KG_INFERENCE_CONFIDENCE_THRESHOLD=0.7

# Browser Automation
export BROWSER_POOL_SIZE=4
export BROWSER_TIMEOUT_SECS=30

# Caching
export HTML_CACHE_SIZE_MB=100
export KG_CACHE_SIZE_MB=500

# Rate Limiting
export RATE_LIMIT_REQUESTS_PER_MINUTE=100
export RATE_LIMIT_BURST_SIZE=20
```

## Performance Monitoring

### Key Metrics

Monitor these metrics for performance optimization:

1. **Response Time**: API endpoint latency
2. **Throughput**: Requests per second
3. **Memory Usage**: RSS and virtual memory
4. **CPU Usage**: User and system time
5. **Disk I/O**: Read/write operations for KG
6. **Network I/O**: Bandwidth usage

### Prometheus Metrics

Enable metrics collection:

```bash
export PROMETHEUS_METRICS=true
export METRICS_PORT=9090
```

Key metrics to monitor:
- `http_requests_total` - Request count by endpoint
- `http_request_duration_seconds` - Response time percentiles
- `semantic_browser_kg_triples_total` - Knowledge graph size
- `semantic_browser_memory_usage_bytes` - Memory consumption
- `semantic_browser_browser_pool_active` - Active browser instances

## Optimization Strategies

### 1. Knowledge Graph Optimization

#### Indexing Strategy
```rust
// Enable persistent storage for large KGs
let kg = KnowledgeGraph::with_persistence(path)?;

// Use appropriate page size
kg.set_page_size(4096);
```

#### Query Optimization
- Use LIMIT clauses in SPARQL queries
- Prefer FILTER over complex patterns
- Cache frequent queries

#### Inference Tuning
```bash
# Adjust inference parameters
export KG_INFERENCE_TOP_K=25
export KG_INFERENCE_CONFIDENCE_THRESHOLD=0.8
export KG_INFERENCE_MAX_INSERTS=1000
```

### 2. Browser Automation

#### Pool Sizing
```bash
# Match pool size to concurrent requests
export BROWSER_POOL_SIZE=4

# Adjust timeouts
export BROWSER_TIMEOUT_SECS=20
export BROWSER_NAVIGATION_TIMEOUT_SECS=15
```

#### Resource Blocking
Enable resource blocking to reduce bandwidth:
```bash
export BROWSER_BLOCK_IMAGES=true
export BROWSER_BLOCK_CSS=false  # May affect rendering
export BROWSER_BLOCK_FONTS=true
export BROWSER_BLOCK_ADS=true
```

#### JavaScript Control
```bash
export BROWSER_DISABLE_JAVASCRIPT=false  # Required for dynamic content
export BROWSER_WAIT_FOR_IDLE=true
```

### 3. HTML Parsing

#### Memory Management
```bash
# Limit HTML size
export MAX_HTML_SIZE_BYTES=10485760  # 10MB

# Control parsing depth
export MAX_DOM_DEPTH=100
```

#### Caching
```bash
# Enable HTML result caching
export HTML_CACHE_ENABLED=true
export HTML_CACHE_TTL_SECS=3600
export HTML_CACHE_SIZE_MB=200
```

### 4. Machine Learning

#### ONNX Optimization
```bash
# Model optimization
export NER_MODEL_OPTIMIZED=true
export KG_MODEL_OPTIMIZED=true

# Batch processing
export ML_BATCH_SIZE=16
export ML_MAX_CONCURRENT_INFERENCE=4
```

#### GPU Acceleration (Future)
```bash
export ONNX_USE_GPU=true
export CUDA_VISIBLE_DEVICES=0
```

## Benchmarking

### Built-in Benchmarks

Run performance benchmarks:

```bash
# HTML parsing benchmark
cargo bench --bench parsing_benchmark

# Stress testing
cargo test --test stress_tests --release -- --nocapture
```

### Custom Benchmarks

Create custom benchmarks for your use case:

```rust
#[bench]
fn bench_html_parsing_large(b: &mut Bencher) {
    let html = load_large_html_file();
    b.iter(|| {
        black_box(parse_html(&html));
    });
}
```

### Profiling

Use profiling tools:

```bash
# CPU profiling
cargo flamegraph --bin semantic_browser_agent -- test_scenario

# Memory profiling
valgrind --tool=massif target/release/semantic_browser_agent

# System monitoring
perf record -g target/release/semantic_browser_agent
perf report
```

## Scaling Strategies

### Vertical Scaling

1. **CPU Optimization**
   - Increase worker threads: `export WORKERS=8`
   - Enable parallel processing: `export RAYON_NUM_THREADS=8`
   - Use release builds: `cargo build --release`

2. **Memory Optimization**
   - Increase RAM for large KGs
   - Use SSD storage for KG persistence
   - Enable memory pooling

3. **Storage Optimization**
   - Use NVMe SSDs for KG storage
   - Enable compression for large datasets
   - Implement data partitioning

### Horizontal Scaling

1. **Load Balancing**
   ```nginx
   upstream semantic_browser {
       server 127.0.0.1:3000;
       server 127.0.0.1:3001;
       server 127.0.0.1:3002;
   }
   ```

2. **Shared Storage**
   - Use PostgreSQL/Neo4j for shared KG
   - Implement distributed caching (Redis)
   - Use shared file systems (NFS, Ceph)

3. **Service Mesh**
   - Implement service discovery
   - Use circuit breakers
   - Enable distributed tracing

## Database Optimization

### Knowledge Graph Storage

#### Oxigraph Tuning
```rust
// Custom storage configuration
let store = Store::open_with_capacity(path, 1_000_000)?;

// Enable query optimization
store.set_query_optimizer(true);
```

#### External Databases
For large-scale deployments:

- **PostgreSQL**: Use with SPARQL-SQL translation
- **Neo4j**: Native graph database
- **Virtuoso**: High-performance RDF store

### Caching Strategy

#### Multi-level Caching
1. **In-memory cache**: Fast access for hot data
2. **Disk cache**: Persistence for warm data
3. **Distributed cache**: Redis for cluster coordination

#### Cache Configuration
```rust
// Application-level caching
let cache = Cache::builder()
    .max_capacity(1000)
    .time_to_live(Duration::from_secs(3600))
    .build();
```

## Network Optimization

### Connection Pooling
```rust
// HTTP client configuration
let client = Client::builder()
    .pool_max_idle_per_host(10)
    .pool_idle_timeout(Duration::from_secs(90))
    .timeout(Duration::from_secs(30))
    .build()?;
```

### Compression
Enable response compression:
```rust
// Axum compression middleware
let app = Router::new()
    .layer(CompressionLayer::new());
```

### CDN Integration
Use CDN for static assets and cached responses.

## Monitoring and Alerting

### Key Alerts

1. **High Latency**
   ```
   http_request_duration_seconds{quantile="0.95"} > 5.0
   ```

2. **High Error Rate**
   ```
   rate(http_requests_total{status="500"}[5m]) > 0.05
   ```

3. **Memory Usage**
   ```
   semantic_browser_memory_usage_bytes > 1e9
   ```

4. **Disk Space**
   ```
   disk_used_percent{mountpoint="/var/lib/semantic-browser"} > 85
   ```

### Logging Optimization

```bash
# Structured logging
export RUST_LOG=info,semantic_browser=warn
export LOG_FORMAT=json

# Log sampling
export LOG_SAMPLING_RATE=0.1
```

## Troubleshooting Performance Issues

### Slow API Responses

1. **Check KG size**: Large KGs slow down queries
2. **Profile queries**: Use EXPLAIN for SPARQL queries
3. **Check browser pool**: Ensure sufficient browser instances
4. **Monitor I/O**: Disk bottlenecks affect performance

### High Memory Usage

1. **KG memory mapping**: Large KGs consume memory
2. **Browser instances**: Each browser uses ~100MB
3. **Cache sizes**: Adjust cache limits
4. **Memory leaks**: Monitor with valgrind

### High CPU Usage

1. **ML inference**: Heavy computation
2. **HTML parsing**: Complex DOM processing
3. **Concurrent requests**: Too many simultaneous operations
4. **Garbage collection**: Inefficient memory management

## Performance Testing

### Load Testing

Use tools like:

```bash
# Apache Bench
ab -n 1000 -c 10 http://localhost:3000/parse

# Hey
hey -n 1000 -c 10 http://localhost:3000/parse

# Artillery
artillery quick --count 100 --num 10 http://localhost:3000/parse
```

### Stress Testing

Built-in stress tests:

```bash
cargo test --test stress_tests --release -- --nocapture
```

### Capacity Planning

1. **Determine baseline**: Measure performance with typical load
2. **Scale testing**: Gradually increase load to find limits
3. **Resource monitoring**: Track CPU, memory, disk, network
4. **Bottleneck identification**: Find limiting factors

## Best Practices

### Development
- Use `--release` builds for performance testing
- Profile regularly during development
- Write performance tests for critical paths

### Production
- Monitor key metrics continuously
- Set up alerting for performance degradation
- Regular performance audits
- Keep dependencies updated

### Maintenance
- Regular benchmark runs
- Performance regression testing
- Capacity planning reviews
- Infrastructure upgrades based on usage patterns

## References

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Oxigraph Performance Tuning](https://oxigraph.org/performance/)
- [ONNX Runtime Performance](https://onnxruntime.ai/docs/performance/)
- [System Performance Tuning](https://www.brendangregg.com/linuxperf.html)