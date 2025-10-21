# Use Rust official image
FROM rust:1.84-slim AS builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y \
        build-essential \
        pkg-config \
        libssl-dev \
        libclang-dev \
    && rm -rf /var/lib/apt/lists/*

# Set environment for pyo3 compatibility
ENV PYO3_USE_ABI3_FORWARD_COMPATIBILITY=1

# Set working directory
WORKDIR /app

# Copy Cargo files first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY benches/ ./benches/

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn dummy() {}" > src/lib.rs

# Build dependencies (this layer will be cached)
# Note: --bins excludes benchmarks and examples
RUN cargo build --release --bins && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build the actual application (only rebuilds if source changed)
RUN cargo build --release --bins

# Runtime image
FROM debian:bookworm-slim

# Add labels for metadata
LABEL maintainer="Semantic Browser Team"
LABEL description="Semantic Browser for AI Agents"
LABEL version="0.1.0"

# Install Python, curl (for health checks) and other runtime dependencies
RUN apt-get update && \
    apt-get install -y \
        python3 \
        python3-pip \
        curl \
        ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Install Python dependencies for browser automation and agent workflows
RUN pip install --no-cache-dir --break-system-packages \
    requests \
    beautifulsoup4 \
    lxml \
    || true

# Create non-root user for security
RUN useradd -m -u 1000 semantic && \
    mkdir -p /data && \
    chown -R semantic:semantic /data

# Copy the binary from builder
COPY --from=builder /app/target/release/semantic_browser_agent /usr/local/bin/

# Switch to non-root user
USER semantic

# Set working directory
WORKDIR /data

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/ || exit 1

# Environment variables
ENV RUST_LOG=info

# Run the server
CMD ["semantic_browser_agent"]