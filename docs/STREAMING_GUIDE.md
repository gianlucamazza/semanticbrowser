# LLM Streaming Guide

This guide covers how to use real-time token streaming with the Semantic Browser's LLM integration.

## Table of Contents

1. [Overview](#overview)
2. [Why Streaming?](#why-streaming)
3. [Getting Started](#getting-started)
4. [OpenAI Streaming](#openai-streaming)
5. [Handling Streams](#handling-streams)
6. [Best Practices](#best-practices)
7. [Troubleshooting](#troubleshooting)

---

## Overview

Streaming allows you to receive LLM responses token-by-token as they are generated, instead of waiting for the complete response. This enables:

- **Real-time User Feedback**: Show responses as they arrive
- **Progressive Rendering**: Display content incrementally
- **Lower Latency Perception**: Users see output faster
- **Bandwidth Efficiency**: Start processing tokens before full response arrives

### Supported Providers

| Provider | Status | Method |
|----------|--------|--------|
| OpenAI | ✅ Implemented | `stream_chat_completion()` |
| Anthropic | 🔄 In Progress | Coming soon |
| Ollama | ⏳ Planned | Future release |

---

## Why Streaming?

### Traditional Non-Streaming
```text
Request → [============================] → Response (5-10 seconds)
          Waiting period (user sees nothing)
```

### With Streaming
```text
Request → Token1 → Token2 → Token3 → ... → Complete (still 5-10s, but user sees progress)
```

**Key Benefits:**
- Perceived latency reduction
- Better UX for long-form content
- Progressive rendering possible
- Real-time token counting
- Cancellable operations

---

## Getting Started

### Setup

1. **Enable the LLM feature:**
   ```bash
   cargo build --features llm-openai
   ```

2. **Set API key:**
   ```bash
   export OPENAI_API_KEY=sk-your-key-here
   ```

3. **Run the streaming example:**
   ```bash
   cargo run --features llm-openai --example streaming_example
   ```

---

## OpenAI Streaming

### Basic Usage

```rust
use semantic_browser::llm::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create provider
    let provider = OpenAIProvider::new("sk-your-key".to_string());

    // Prepare message
    let messages = vec![
        Message::user("Write a haiku about programming.".to_string())
    ];

    // Configure LLM
    let config = LLMConfig {
        model: "gpt-3.5-turbo".to_string(),
        temperature: 0.7,
        max_tokens: Some(100),
        ..Default::default()
    };

    // Get streaming receiver
    let mut rx = provider
        .stream_chat_completion(messages, &config)
        .await?;

    // Consume tokens as they arrive
    let mut full_response = String::new();
    while let Some(token) = rx.recv().await {
        print!("{}", token);  // Display token immediately
        full_response.push_str(&token);
        std::io::stdout().flush()?;
    }

    println!("\n✅ Stream complete!");
    Ok(())
}
```

### Advanced Usage: Token Counting

```rust
// Track tokens in real-time
let mut token_count = 0;
let mut char_count = 0;

while let Some(token) = rx.recv().await {
    token_count += 1;
    char_count += token.len();

    // Real-time statistics
    if token_count % 10 == 0 {
        println!("Processed {} tokens ({} chars)", token_count, char_count);
    }

    process_token(&token)?;
}

println!("Final: {} tokens, {} characters", token_count, char_count);
println!("Avg token size: {:.2} chars", char_count as f64 / token_count as f64);
```

### Advanced Usage: Timeout Handling

```rust
use tokio::time::{timeout, Duration};

let timeout_duration = Duration::from_secs(30);
let mut full_response = String::new();

loop {
    match timeout(timeout_duration, rx.recv()).await {
        Ok(Some(token)) => {
            full_response.push_str(&token);
            print!("{}", token);
        }
        Ok(None) => {
            println!("\n✅ Stream completed normally");
            break;
        }
        Err(_) => {
            println!("\n⚠️  Stream timeout after 30 seconds");
            break;
        }
    }
}
```

### Advanced Usage: Streaming with Tools

Currently, streaming is supported for basic chat completion. Tool-based streaming will be available in a future release.

```rust
// Non-streaming with tools (current approach)
let response = provider
    .chat_completion_with_tools(messages, tools, &config)
    .await?;

println!("Response: {}", response.content);
if let Some(tool_calls) = response.tool_calls {
    println!("Tool calls: {:?}", tool_calls);
}
```

---

## Handling Streams

### Channel Behavior

The `stream_chat_completion` method returns a `tokio::sync::mpsc::Receiver<String>`:

```rust
pub async fn stream_chat_completion(
    &self,
    messages: Vec<Message>,
    config: &LLMConfig,
) -> LLMResult<tokio::sync::mpsc::Receiver<String>>
```

**Key characteristics:**

- **Buffer Size**: 100 tokens
- **Async**: Requires `.await` and runs in tokio runtime
- **Non-blocking**: `recv()` awaits next token without blocking
- **Ordered**: Tokens arrive in generation order
- **Closure**: Channel closes automatically when stream completes

### Error Handling

```rust
match provider.stream_chat_completion(messages, &config).await {
    Ok(mut rx) => {
        // Stream created successfully
        while let Some(token) = rx.recv().await {
            process(token)?;
        }
    }
    Err(LLMError::Network(e)) => {
        eprintln!("Network error: {}", e);
    }
    Err(LLMError::Api(msg)) => {
        eprintln!("API error: {}", msg);
    }
    Err(e) => {
        eprintln!("Unknown error: {}", e);
    }
}
```

### Early Termination

If you need to stop consuming tokens early:

```rust
let mut rx = provider.stream_chat_completion(messages, &config).await?;
let mut count = 0;

while let Some(token) = rx.recv().await {
    count += 1;
    print!("{}", token);

    // Stop after 50 tokens
    if count >= 50 {
        println!("\n... (truncated)");
        drop(rx);  // Close receiver
        break;
    }
}
```

---

## Best Practices

### 1. Always Use stdout::flush() for Real-time Display

```rust
while let Some(token) = rx.recv().await {
    print!("{}", token);
    std::io::stdout().flush()?;  // Important!
}
```

### 2. Handle Stream Interruptions Gracefully

```rust
let result = provider.stream_chat_completion(messages, &config).await;

if let Err(e) = result {
    eprintln!("Stream failed to initialize: {}", e);
    return;
}

match result {
    Ok(mut rx) => {
        while let Some(token) = rx.recv().await {
            // Process token
        }
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

### 3. Set Appropriate Timeouts for Long Tasks

```rust
use tokio::time::{timeout, Duration};

let mut rx = provider.stream_chat_completion(messages, &config).await?;
let max_wait = Duration::from_secs(60);

while let Ok(Some(token)) = timeout(max_wait, rx.recv()).await {
    process_token(token);
}
```

### 4. Monitor Token Generation

```rust
let mut previous_time = std::time::Instant::now();
let mut token_count = 0;

while let Some(token) = rx.recv().await {
    token_count += 1;

    if token_count % 20 == 0 {
        let elapsed = previous_time.elapsed();
        let tokens_per_sec = 20.0 / elapsed.as_secs_f64();
        println!("📊 Speed: {:.1} tokens/sec", tokens_per_sec);
        previous_time = std::time::Instant::now();
    }
}
```

### 5. Buffer Multiple Streams

```rust
use futures::stream::StreamExt;

let mut rx1 = provider.stream_chat_completion(msg1, &config).await?;
let mut rx2 = provider.stream_chat_completion(msg2, &config).await?;

// Option A: Sequential
while let Some(token) = rx1.recv().await {
    process_a(token);
}
while let Some(token) = rx2.recv().await {
    process_b(token);
}

// Option B: Merge streams
use tokio_stream::StreamExt as _;
// Future enhancement
```

---

## Troubleshooting

### Issue: No tokens received

**Possible causes:**
- API key invalid
- Network connectivity issue
- Model doesn't exist
- API rate limited

**Solution:**
```rust
// Check health first
match provider.health_check().await {
    Ok(true) => println!("✅ API is accessible"),
    Ok(false) => println!("❌ Health check failed"),
    Err(e) => println!("❌ Network error: {}", e),
}

// Test with non-streaming first
let test_config = LLMConfig::default();
let result = provider.chat_completion(test_msg, &test_config).await;
println!("Non-streaming result: {:?}", result);
```

### Issue: Incomplete responses

**Possible causes:**
- Stream interrupted
- Timeout occurred
- Channel buffer full

**Solution:**
```rust
// Increase buffer size (if implementing custom version)
// Default: 100 tokens

// Add logging
while let Some(token) = rx.recv().await {
    tracing::debug!("Token: {:?}", token);
}

// Set RUST_LOG=debug
// RUST_LOG=debug cargo run --example streaming_example
```

### Issue: High latency

**Possible causes:**
- First token latency (normal)
- Network latency
- Model processing time
- Buffering delay

**Solution:**
```rust
// Measure time-to-first-token
let start = std::time::Instant::now();
while let Some(first_token) = rx.recv().await {
    let ttft = start.elapsed();
    println!("Time to first token: {:.2}s", ttft.as_secs_f64());
    break;
}

// Use faster model for lower latency
let config = LLMConfig {
    model: "gpt-3.5-turbo".to_string(),  // Faster than gpt-4
    ..Default::default()
};
```

---

## Architecture Details

### How Streaming Works

```rust
// Internal flow in stream_chat_completion:

1. Create tokio::sync::mpsc channel (100-token buffer)
2. Spawn background task
3. Send streaming request to OpenAI API (stream: true)
4. Parse Server-Sent Events (SSE) format
5. Extract delta.content from each event
6. Send token through channel
7. Channel closes when [DONE] received
8. Return receiver to caller
```

### SSE Format

OpenAI streams responses as Server-Sent Events:

```
data: {"object":"chat.completion.chunk","choices":[{"delta":{"content":"Hello"}}]}

data: {"object":"chat.completion.chunk","choices":[{"delta":{"content":" world"}}]}

data: [DONE]
```

The streaming implementation:
- Reads response as byte stream
- Buffers incomplete lines
- Parses `data: ` prefix
- Extracts JSON and delta.content
- Sends non-empty tokens to channel

---

## Performance Metrics

Typical performance with gpt-3.5-turbo:

| Metric | Value |
|--------|-------|
| Time-to-first-token | 200-500ms |
| Token generation rate | 40-100 tokens/sec |
| Buffer capacity | 100 tokens |
| Channel overhead | < 1μs per token |

---

## API Reference

### LLMProvider Trait

```rust
#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn stream_chat_completion(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
    ) -> LLMResult<tokio::sync::mpsc::Receiver<String>>;
}
```

### OpenAIProvider Implementation

```rust
impl LLMProvider for OpenAIProvider {
    async fn stream_chat_completion(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
    ) -> LLMResult<tokio::sync::mpsc::Receiver<String>> {
        // Detailed implementation in src/llm/openai.rs
    }
}
```

---

## Future Enhancements

- [ ] Streaming with tools (function calling)
- [ ] Streaming with vision models
- [ ] Anthropic streaming support
- [ ] Ollama streaming support
- [ ] Token validation
- [ ] Cost estimation during streaming
- [ ] Metrics/observability integration
- [ ] Multiple stream merging

---

**Last Updated**: 2025-10-22
**Status**: Production Ready
**Maintainer**: Technical Team
