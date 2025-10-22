# Production Deployment Guide

This guide covers deploying the Semantic Browser in production environments with security, scalability, and monitoring best practices.

## Prerequisites

- Linux server (Ubuntu 20.04+ or RHEL 8+)
- Rust 1.70+
- 4GB RAM minimum, 8GB recommended
- SSL certificate for HTTPS

## Building for Production

### Optimized Build

```bash
# Build with all features and optimizations
cargo build --release --all-features

# Strip debug symbols for smaller binary
strip target/release/semantic_browser_mcp
strip target/release/semantic_browser_agent

# Verify binary
file target/release/semantic_browser_mcp
ldd target/release/semantic_browser_mcp
```

### Feature Selection

Choose appropriate features for your deployment:

```bash
# Minimal production build
cargo build --release --features browser-automation,seccomp

# Full-featured build
cargo build --release --all-features

# With telemetry
cargo build --release --features browser-automation,seccomp,telemetry,observability
```

## Configuration

### Environment Variables

Create `.env` file:

```bash
# Server
HOST=0.0.0.0
PORT=3000
WORKERS=4

# Security
JWT_SECRET=your-32-char-minimum-secret-key-here
CORS_ORIGINS=https://yourdomain.com

# Knowledge Graph
KG_PERSIST_PATH=/var/lib/semantic-browser/kg
KG_INFERENCE_MODEL_PATH=/opt/models/kg-model.onnx

# Browser Automation
CHROMIUMOXIDE_USER_DATA_DIR=/var/lib/semantic-browser/browser-data

# Logging
RUST_LOG=info,semantic_browser=warn,tower_http=debug
LOG_FORMAT=json

# Rate Limiting
RATE_LIMIT_REQUESTS_PER_MINUTE=60
RATE_LIMIT_BURST_SIZE=10

# Monitoring
PROMETHEUS_METRICS=true
METRICS_PORT=9090
```

### Directory Structure

```bash
/opt/semantic-browser/
├── bin/
│   ├── semantic_browser_mcp
│   └── semantic_browser_agent
├── config/
│   ├── .env
│   └── mcp-manifest.json
├── data/
│   ├── kg/
│   └── browser-data/
├── models/
│   ├── ner-model.onnx
│   └── kg-model.onnx
└── logs/
```

## Systemd Service

### MCP Server Service

Create `/etc/systemd/system/semantic-browser-mcp.service`:

```ini
[Unit]
Description=Semantic Browser MCP Server
After=network.target
Wants=network.target

[Service]
Type=simple
User=semantic-browser
Group=semantic-browser
WorkingDirectory=/opt/semantic-browser
EnvironmentFile=/opt/semantic-browser/config/.env
ExecStart=/opt/semantic-browser/bin/semantic_browser_mcp
Restart=always
RestartSec=5
LimitNOFILE=65536

# Security hardening
NoNewPrivileges=yes
PrivateTmp=yes
ProtectHome=yes
ProtectSystem=strict
ReadWritePaths=/opt/semantic-browser/data /var/log/semantic-browser
ProtectKernelTunables=yes
ProtectControlGroups=yes

# Resource limits
MemoryLimit=1G
CPUQuota=200%

[Install]
WantedBy=multi-user.target
```

### API Server Service

Create `/etc/systemd/system/semantic-browser-api.service`:

```ini
[Unit]
Description=Semantic Browser API Server
After=network.target semantic-browser-mcp.service
Wants=network.target

[Service]
Type=simple
User=semantic-browser
Group=semantic-browser
WorkingDirectory=/opt/semantic-browser
EnvironmentFile=/opt/semantic-browser/config/.env
ExecStart=/opt/semantic-browser/bin/semantic_browser_agent
Restart=always
RestartSec=5
LimitNOFILE=65536

# Security
NoNewPrivileges=yes
PrivateTmp=yes
ProtectHome=yes
ProtectSystem=strict
ReadWritePaths=/opt/semantic-browser/data /tmp
ProtectKernelTunables=yes
ProtectControlGroups=yes

# Resources
MemoryLimit=2G
CPUQuota=400%

[Install]
WantedBy=multi-user.target
```

### Service Management

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable services
sudo systemctl enable semantic-browser-mcp
sudo systemctl enable semantic-browser-api

# Start services
sudo systemctl start semantic-browser-mcp
sudo systemctl start semantic-browser-api

# Check status
sudo systemctl status semantic-browser-*

# View logs
sudo journalctl -u semantic-browser-mcp -f
sudo journalctl -u semantic-browser-api -f
```

## Docker Deployment

### Dockerfile

```dockerfile
FROM rust:1.70-slim AS builder

WORKDIR /app
COPY . .

# Build with all features
RUN cargo build --release --all-features

# Strip binaries
RUN strip target/release/semantic_browser_mcp
RUN strip target/release/semantic_browser_agent

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    chromium \
    && rm -rf /var/lib/apt/lists/*

# Create user
RUN useradd --create-home --shell /bin/bash semantic-browser

# Copy binaries
COPY --from=builder /app/target/release/semantic_browser_mcp /usr/local/bin/
COPY --from=builder /app/target/release/semantic_browser_agent /usr/local/bin/

# Create directories
RUN mkdir -p /app/data /app/logs && \
    chown -R semantic-browser:semantic-browser /app

USER semantic-browser
WORKDIR /app

EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["semantic_browser_agent"]
```

### Docker Compose

```yaml
version: '3.8'

services:
  semantic-browser:
    build: .
    ports:
      - "3000:3000"
      - "9090:9090"  # Metrics
    environment:
      - JWT_SECRET=${JWT_SECRET}
      - KG_PERSIST_PATH=/app/data/kg
      - PROMETHEUS_METRICS=true
    volumes:
      - ./data:/app/data
      - ./models:/app/models:ro
    restart: unless-stopped
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    read_only: true
    tmpfs:
      - /tmp

  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.console.libraries=/etc/prometheus/console_libraries'
      - '--web.console.templates=/etc/prometheus/consoles'
      - '--storage.tsdb.retention.time=200h'
      - '--web.enable-lifecycle'
```

## Kubernetes Deployment

### Deployment Manifest

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: semantic-browser
  labels:
    app: semantic-browser
spec:
  replicas: 3
  selector:
    matchLabels:
      app: semantic-browser
  template:
    metadata:
      labels:
        app: semantic-browser
    spec:
      securityContext:
        runAsNonRoot: true
        runAsUser: 65534
        fsGroup: 65534
      containers:
      - name: semantic-browser
        image: semantic-browser:latest
        ports:
        - containerPort: 3000
          name: http
        - containerPort: 9090
          name: metrics
        env:
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: semantic-browser-secrets
              key: jwt-secret
        - name: KG_PERSIST_PATH
          value: "/data/kg"
        volumeMounts:
        - name: data
          mountPath: /data
        - name: models
          mountPath: /models
          readOnly: true
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
        securityContext:
          allowPrivilegeEscalation: false
          capabilities:
            drop:
            - ALL
          readOnlyRootFilesystem: true
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: semantic-browser-data
      - name: models
        configMap:
          name: semantic-browser-models
```

### Service Manifest

```yaml
apiVersion: v1
kind: Service
metadata:
  name: semantic-browser
spec:
  selector:
    app: semantic-browser
  ports:
  - name: http
    port: 80
    targetPort: 3000
  - name: metrics
    port: 9090
    targetPort: 9090
  type: ClusterIP
```

### Ingress

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: semantic-browser
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
spec:
  tls:
  - hosts:
    - api.yourdomain.com
    secretName: semantic-browser-tls
  rules:
  - host: api.yourdomain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: semantic-browser
            port:
              number: 80
```

## Monitoring and Observability

### Prometheus Metrics

Enable metrics collection:

```bash
export PROMETHEUS_METRICS=true
export METRICS_PORT=9090
```

### Health Checks

The application provides health endpoints:

- `GET /health` - Basic health check
- `GET /health/detailed` - Detailed health with component status
- `GET /metrics` - Prometheus metrics (if enabled)

### Logging

Configure structured logging:

```bash
export RUST_LOG=info,semantic_browser=warn
export LOG_FORMAT=json
```

### Distributed Tracing

Enable OpenTelemetry tracing:

```bash
export OTEL_SERVICE_NAME=semantic-browser
export OTEL_TRACES_EXPORTER=otlp
export OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:14268/api/traces
```

## Security Hardening

### Network Security

- Use HTTPS with valid certificates
- Configure CORS properly
- Rate limiting enabled by default
- Input validation on all endpoints

### Secrets Management

- Store JWT secrets in secure vaults (HashiCorp Vault, AWS Secrets Manager)
- Rotate secrets regularly
- Use Kubernetes secrets or Docker secrets

### Backup and Recovery

```bash
# Backup knowledge graph
cp -r /var/lib/semantic-browser/kg /backup/kg-$(date +%Y%m%d)

# Backup configuration (without secrets)
cp /opt/semantic-browser/config/.env /backup/config-$(date +%Y%m%d)
```

## Scaling Considerations

### Horizontal Scaling

- Stateless design allows horizontal scaling
- Use load balancer for multiple instances
- Shared KG storage (PostgreSQL, Neo4j) for consistency

### Vertical Scaling

- Monitor memory usage (KG can grow large)
- Adjust worker threads based on CPU cores
- Use connection pooling for external services

### Performance Tuning

```bash
# Environment variables for tuning
export RAYON_NUM_THREADS=8  # Parallel processing
export KG_INFERENCE_TOP_K=50  # Limit inference results
export BROWSER_POOL_SIZE=4  # Concurrent browsers
```

## Troubleshooting

### Common Issues

1. **High Memory Usage**
   - Check KG size: `du -sh /var/lib/semantic-browser/kg`
   - Monitor with `htop` or Prometheus
   - Consider KG compaction

2. **Slow Responses**
   - Check browser pool size
   - Monitor external API calls
   - Enable query logging

3. **Connection Refused**
   - Verify service is running: `systemctl status`
   - Check firewall rules
   - Validate configuration

### Debug Mode

Enable debug logging temporarily:

```bash
export RUST_LOG=debug
systemctl restart semantic-browser-api
journalctl -u semantic-browser-api -f
```

## Backup and Maintenance

### Automated Backups

```bash
#!/bin/bash
# Daily backup script
DATE=$(date +%Y%m%d)
BACKUP_DIR="/backup/semantic-browser-$DATE"

mkdir -p "$BACKUP_DIR"
cp -r /var/lib/semantic-browser/kg "$BACKUP_DIR/"
cp /opt/semantic-browser/config/.env "$BACKUP_DIR/"

# Compress
tar -czf "$BACKUP_DIR.tar.gz" -C /backup "semantic-browser-$DATE"
rm -rf "$BACKUP_DIR"

# Upload to S3 or other storage
aws s3 cp "$BACKUP_DIR.tar.gz" "s3://backups/semantic-browser/"
```

### Log Rotation

Configure logrotate:

```bash
# /etc/logrotate.d/semantic-browser
/var/log/semantic-browser/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
    create 0644 semantic-browser semantic-browser
    postrotate
        systemctl reload semantic-browser-api
    endscript
}
```

## References

- [Systemd Service Documentation](https://www.freedesktop.org/software/systemd/man/systemd.service.html)
- [Kubernetes Best Practices](https://kubernetes.io/docs/concepts/configuration/overview/)
- [Docker Security](https://docs.docker.com/develop/dev-best-practices/security/)
- [Prometheus Monitoring](https://prometheus.io/docs/introduction/overview/)