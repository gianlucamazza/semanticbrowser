# 🧠 LLM Agent Integration

This module provides LLM-based autonomous agents for orchestrating web browsing tasks.

## 🚀 Quick Start

### Prerequisites

1. **Install Ollama** (for local/free LLM):
   ```bash
   # macOS
   brew install ollama
   
   # Linux
   curl https://ollama.ai/install.sh | sh
   
   # Windows
   # Download from https://ollama.ai
   ```

2. **Pull a model**:
   ```bash
   # Recommended: Llama 3 8B (fast, good quality)
   ollama pull llama3:8b
   
   # Or: Llama 3 70B (slower, better quality)
   ollama pull llama3:70b
   ```

3. **Start Ollama** (if not running as service):
   ```bash
   ollama serve
   ```

### Run the Example

```bash
cargo run --example agent_simple_task
```

Expected output:
```
🤖 Semantic Browser - Agent Example
====================================

Checking Ollama connection...
✅ Ollama is running
Loaded 5 tools

📋 Task 1: Navigate to github.com and find the trending repositories
─────────────────────────────────────────────────────────────────────
Agent iteration 1/5
Thought: I need to navigate to GitHub first
Action: navigate_to
...
✅ Success! (iterations: 3)
Result: Found trending repositories: rust-lang/rust, ...
```

## 📖 Usage

### Basic Agent Task

```rust
use semantic_browser::llm::{
    OllamaProvider, OllamaConfig, AgentOrchestrator,
    AgentTask, LLMConfig, ToolRegistry,
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create LLM provider
    let provider = Arc::new(OllamaProvider::new(OllamaConfig::default()));
    
    // 2. Configure LLM
    let config = LLMConfig {
        model: "llama3:8b".to_string(),
        temperature: 0.7,
        max_tokens: Some(2048),
        ..Default::default()
    };
    
    // 3. Create tool registry
    let tools = ToolRegistry::with_browser_tools();
    
    // 4. Create agent
    let agent = AgentOrchestrator::new(provider, config, tools);
    
    // 5. Define and execute task
    let task = AgentTask::new("Fill out a contact form")
        .with_context("Form has fields: name, email, message")
        .with_max_iterations(5);
    
    let response = agent.execute(task).await?;
    
    if response.success {
        println!("✅ Success: {}", response.result);
    } else {
        println!("❌ Failed: {:?}", response.error);
    }
    
    Ok(())
}
```

### Custom System Prompt

```rust
let agent = AgentOrchestrator::new(provider, config, tools)
    .with_system_prompt(r#"
        You are a specialized web scraping agent.
        Extract data efficiently and handle errors gracefully.
        Always validate data before returning results.
    "#);
```

## 🛠️ Available Tools

The default `ToolRegistry::with_browser_tools()` provides:

| Tool | Description | Parameters |
|------|-------------|------------|
| `navigate_to` | Navigate to a URL | `url: string` |
| `fill_form` | Fill form fields | `form_data: object` |
| `click_element` | Click an element | `selector: string` |
| `get_page_content` | Get page HTML/text | `format: "html" \| "text"` |
| `extract_data` | Extract structured data | `selectors: object` |

## 🧩 Architecture

```
┌─────────────────────────────────┐
│    AgentOrchestrator            │
│  (ReAct: Think → Act → Observe) │
└──────────┬──────────────────────┘
           │
    ┌──────┴──────┐
    │             │
┌───▼────┐   ┌───▼─────┐
│  LLM   │   │  Tools  │
│Provider│   │Registry │
└────────┘   └─────────┘
    │             │
    └──────┬──────┘
           │
    ┌──────▼──────┐
    │   Browser   │
    │ Automation  │
    └─────────────┘
```

## 🎯 ReAct Pattern

The agent follows the **ReAct** (Reasoning + Acting) pattern:

1. **THOUGHT**: Analyze situation and decide next action
2. **ACTION**: Choose a tool to execute
3. **ACTION INPUT**: Provide parameters for the tool
4. **OBSERVATION**: Observe the result

Example iteration:
```
THOUGHT: I need to navigate to the website first
ACTION: navigate_to
ACTION INPUT: {"url": "https://example.com"}
OBSERVATION: Successfully navigated to: https://example.com

THOUGHT: Now I can fill the form
ACTION: fill_form
ACTION INPUT: {"form_data": {"name": "John", "email": "john@example.com"}}
OBSERVATION: Form filled successfully

THOUGHT: Task complete
ACTION: FINISH
ACTION INPUT: Successfully filled contact form with user data
```

## 🔌 Adding New LLM Providers

### 1. Implement the `LLMProvider` trait

```rust
use async_trait::async_trait;
use crate::llm::provider::{LLMProvider, LLMConfig, LLMResult, LLMResponse, Message};

pub struct MyCustomProvider {
    // your fields
}

#[async_trait]
impl LLMProvider for MyCustomProvider {
    async fn chat_completion(
        &self,
        messages: Vec<Message>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse> {
        // Your implementation
        todo!()
    }
    
    async fn chat_completion_with_tools(
        &self,
        messages: Vec<Message>,
        tools: Vec<serde_json::Value>,
        config: &LLMConfig,
    ) -> LLMResult<LLMResponse> {
        // Your implementation
        todo!()
    }
    
    fn provider_name(&self) -> &str {
        "my-custom-provider"
    }
    
    async fn health_check(&self) -> LLMResult<bool> {
        // Your implementation
        Ok(true)
    }
}
```

### 2. Use it with the agent

```rust
let provider = Arc::new(MyCustomProvider::new());
let agent = AgentOrchestrator::new(provider, config, tools);
```

## 📋 Roadmap

See [docs/LLM_AGENT_ROADMAP.md](../../docs/LLM_AGENT_ROADMAP.md) for the complete roadmap.

### ✅ Phase 1 - Complete
- [x] LLMProvider trait
- [x] Ollama integration
- [x] Tool registry
- [x] Agent orchestrator
- [x] Example

### 🚧 Phase 2 - In Progress
- [ ] OpenAI provider (GPT-4)
- [ ] Anthropic provider (Claude)
- [ ] Browser integration (real tool execution)
- [ ] Prompt engineering

### 🔜 Phase 3 - Planned
- [ ] Memory & state management
- [ ] Multi-agent orchestration
- [ ] Advanced tool use

### 🔮 Phase 4 - Future
- [ ] Observability (traces, metrics)
- [ ] Error handling & retry
- [ ] Security & safety
- [ ] Evaluation framework

## 🧪 Testing

### Unit Tests
```bash
cargo test --lib llm
```

### Integration Tests (requires Ollama)
```bash
# Start Ollama first
ollama serve

# Pull model
ollama pull llama3:8b

# Run tests
cargo test --lib llm -- --ignored
```

### Example
```bash
cargo run --example agent_simple_task
```

## 📊 Performance

### Token Usage

| Model | Speed | Quality | Cost (local) |
|-------|-------|---------|--------------|
| llama3:8b | ~50 tok/s | Good | Free |
| llama3:70b | ~10 tok/s | Excellent | Free |
| GPT-4 Turbo | Fast | Best | $0.01/1K tokens |
| Claude 3 Opus | Fast | Best | $0.015/1K tokens |

### Recommendations

- **Development**: Use `llama3:8b` (fast, local, free)
- **Production**: Use GPT-4 or Claude 3 (higher quality, more reliable)
- **Budget-conscious**: Use `llama3:70b` (best free option)

## 🐛 Troubleshooting

### "Ollama is not running"
```bash
# Check if Ollama is running
curl http://localhost:11434/api/tags

# Start Ollama
ollama serve
```

### "Model not found"
```bash
# List installed models
ollama list

# Pull the model
ollama pull llama3:8b
```

### "Connection timeout"
```rust
// Increase timeout in config
let ollama_config = OllamaConfig {
    base_url: "http://localhost:11434".to_string(),
    timeout: Duration::from_secs(300), // 5 minutes
};
```

### Agent gets stuck in loop
- Reduce `max_iterations` to fail faster
- Check system prompt clarity
- Verify tool descriptions are accurate
- Add more context to the task

## 📚 Resources

### Papers
- [ReAct: Synergizing Reasoning and Acting in Language Models](https://arxiv.org/abs/2210.03629)
- [Toolformer: Language Models Can Teach Themselves to Use Tools](https://arxiv.org/abs/2302.04761)

### LLM Providers
- [Ollama](https://ollama.ai) - Local LLM runner
- [OpenAI API](https://platform.openai.com/docs/api-reference)
- [Anthropic Claude](https://docs.anthropic.com/claude/reference/getting-started-with-the-api)

### Agent Frameworks
- [LangChain](https://python.langchain.com/)
- [AutoGPT](https://github.com/Significant-Gravitas/AutoGPT)
- [BabyAGI](https://github.com/yoheinakajima/babyagi)

## 🤝 Contributing

See [CONTRIBUTING.md](../../CONTRIBUTING.md)

## 📄 License

MIT License - see [LICENSE](../../LICENSE)
