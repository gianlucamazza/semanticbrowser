# LLM Agent Testing Roadmap 🤖

**Date**: 2025-01-22  
**Status**: 📋 PLANNING

## 🎯 Obiettivo

Testare **completamente** il progetto con LLM agents reali per validare:
- Correttezza dell'orchestrazione web
- Completezza delle funzionalità
- Usabilità per agent autonomi
- Performance e reliability

---

## 📊 Stato Attuale (Phase 1.5)

### ✅ Cosa Abbiamo

**Core Features (Completato)**:
- ✅ Browser automation (Chromium via CDP)
- ✅ Page navigation & interaction
- ✅ Form interaction (basic + smart)
- ✅ Element finding & clicking
- ✅ Screenshot capture
- ✅ Cookie management
- ✅ Authentication helpers
- ✅ Knowledge Graph integration
- ✅ Observability (tracing)

**Agent-Friendly Features (Completato Phase 1.5)**:
- ✅ FormAnalyzer - Analisi semantica automatica
- ✅ SmartFormFiller - Riempimento con hint semantici
- ✅ Confidence scoring
- ✅ Auto-discovery dei campi
- ✅ Error reporting dettagliato

**Testing**:
- ✅ 69 unit test (tutti passano)
- ✅ Examples funzionanti
- ⚠️  **NESSUN test con LLM reali**

---

## ❌ Cosa Manca per LLM Agent Testing Completo

### 1. 🔴 **CRITICAL: LLM Integration Layer**

**Problema**: Non abbiamo un'interfaccia standardizzata per LLM

**Cosa Serve**:
```rust
// src/llm/mod.rs
pub trait LLMProvider {
    async fn complete(&self, prompt: &str) -> Result<String>;
    async fn chat(&self, messages: Vec<Message>) -> Result<String>;
}

// Implementazioni per provider reali
pub struct OpenAIProvider { /* ... */ }
pub struct AnthropicProvider { /* ... */ }
pub struct OllamaProvider { /* ... */ }  // Local testing
```

**Benefici**:
- Test con modelli reali (GPT-4, Claude, Llama)
- Validazione del prompting
- Misurazione della performance

**Effort**: 2-3 giorni

---

### 2. 🔴 **CRITICAL: Agent Orchestration Framework**

**Problema**: LLM deve decidere QUALE azione fare e QUANDO

**Cosa Serve**:
```rust
// src/agent/mod.rs
pub struct WebAgent {
    llm: Box<dyn LLMProvider>,
    browser: BrowserPool,
    context: AgentContext,
}

impl WebAgent {
    /// Agent autonomo che interpreta task e decide azioni
    pub async fn execute_task(&self, task: &str) -> Result<TaskResult> {
        // 1. Parse task con LLM
        // 2. Planning: decide step da eseguire
        // 3. Execution: esegue con browser API
        // 4. Reflection: valida risultato
        // 5. Loop o completion
    }
    
    /// Fornisce context al LLM su stato corrente
    pub async fn get_current_state(&self) -> PageState {
        // HTML structure, form fields, links, etc.
    }
}
```

**Benefici**:
- LLM può orchestrare autonomamente
- Planning multi-step
- Error recovery automatico

**Effort**: 5-7 giorni (Phase 2.5)

---

### 3. 🟡 **HIGH: Tool Description for LLM**

**Problema**: LLM deve sapere quali tool può chiamare

**Cosa Serve**:
```rust
// src/llm/tools.rs
pub struct ToolRegistry {
    tools: Vec<ToolDefinition>,
}

pub struct ToolDefinition {
    name: String,
    description: String,
    parameters: serde_json::Value,  // JSON Schema
    handler: Box<dyn ToolHandler>,
}

// Auto-generate da API esistente
impl ToolRegistry {
    pub fn from_browser_api() -> Self {
        // goto, click, fill_field_smart, take_screenshot, etc.
    }
    
    pub fn to_openai_format(&self) -> Vec<FunctionCall> { }
    pub fn to_anthropic_format(&self) -> Vec<Tool> { }
}
```

**Benefici**:
- LLM vede tutte le capacità disponibili
- Function calling standard
- Compatibile con OpenAI/Anthropic/etc.

**Effort**: 2-3 giorni

---

### 4. 🟡 **HIGH: Prompt Templates & Context Window Management**

**Problema**: Context troppo grande, prompt inefficienti

**Cosa Serve**:
```rust
// src/llm/prompting.rs
pub struct PromptBuilder {
    system_prompt: String,
    context_window: usize,
    compressor: ContextCompressor,
}

impl PromptBuilder {
    /// Smart HTML summarization per LLM
    pub fn summarize_page(&self, html: &str) -> String {
        // Extract: forms, links, headings, interactive elements
        // Ignore: scripts, styles, tracking pixels
        // Compress: similar elements
    }
    
    /// Build prompt per task execution
    pub fn build_task_prompt(
        &self, 
        task: &str,
        page_state: &PageState,
        previous_actions: &[Action],
    ) -> String {
        // System prompt + task + context + history
    }
}
```

**Benefici**:
- Riduzione token usage
- Prompt più efficaci
- Cost optimization

**Effort**: 3-4 giorni

---

### 5. 🟢 **MEDIUM: Multi-Step Workflow Engine**

**Problema**: Task complessi richiedono più step sequenziali

**Cosa Serve**:
```rust
// src/workflow/mod.rs
pub struct WebWorkflow {
    steps: Vec<WorkflowStep>,
    state: WorkflowState,
}

pub enum WorkflowStep {
    Navigate { url: String },
    FillForm { data: HashMap<String, String> },
    Click { hint: String },
    ExtractData { selector: String },
    Conditional { condition: Box<dyn Fn(&PageState) -> bool>, then: Vec<WorkflowStep> },
}

impl WebWorkflow {
    pub async fn execute(&mut self, agent: &WebAgent) -> Result<WorkflowResult> {
        // Execute steps with auto-retry
        // Save state between steps
        // Screenshot on errors
    }
}
```

**Benefici**:
- Workflow declarativi
- Retry logic automatico
- State persistence

**Effort**: 3-4 giorni (già pianificato Phase 2.5)

---

### 6. 🟢 **MEDIUM: Observability per LLM Decisions**

**Problema**: Non possiamo debuggare decisioni LLM

**Cosa Serve**:
```rust
// src/observability/llm_traces.rs
#[derive(Debug, Serialize)]
pub struct LLMDecision {
    timestamp: DateTime<Utc>,
    prompt: String,
    response: String,
    tool_calls: Vec<ToolCall>,
    reasoning: Option<String>,
    confidence: f32,
}

pub struct LLMTracer {
    decisions: Vec<LLMDecision>,
    exporter: Box<dyn TraceExporter>,
}

impl LLMTracer {
    pub async fn trace_decision(&mut self, decision: LLMDecision) {
        // Log to file/DB/OpenTelemetry
        // Enable replay & debugging
    }
}
```

**Benefici**:
- Debug delle decisioni agent
- Replay dei task falliti
- Training data per fine-tuning

**Effort**: 2-3 giorni

---

### 7. 🟢 **MEDIUM: Test Scenarios Realistici**

**Problema**: Non abbiamo benchmark standardizzati

**Cosa Serve**:
```rust
// tests/agent_scenarios/mod.rs
pub struct AgentScenario {
    name: String,
    task: String,
    starting_url: String,
    expected_outcome: Box<dyn Fn(&PageState) -> bool>,
    max_steps: usize,
}

// Scenari realistici
pub fn get_test_scenarios() -> Vec<AgentScenario> {
    vec![
        AgentScenario {
            name: "Login to GitHub",
            task: "Login to GitHub with username 'testuser' and password 'testpass'",
            starting_url: "https://github.com/login",
            expected_outcome: Box::new(|state| state.url.contains("github.com") && !state.url.contains("login")),
            max_steps: 10,
        },
        AgentScenario {
            name: "Search and extract prices from e-commerce",
            task: "Search for 'laptop' on Amazon and extract top 5 prices",
            // ...
        },
        // Add 20+ realistic scenarios
    ]
}
```

**Benefici**:
- Benchmark riproducibili
- Regression testing
- Performance comparison tra LLM

**Effort**: 3-5 giorni (writing scenarios)

---

### 8. 🔵 **LOW: Cost & Performance Monitoring**

**Problema**: Non misuriamo costi/performance

**Cosa Serve**:
```rust
// src/metrics/mod.rs
#[derive(Debug)]
pub struct AgentMetrics {
    total_tokens_used: usize,
    total_cost_usd: f64,
    api_calls: usize,
    avg_response_time_ms: u64,
    success_rate: f32,
    steps_per_task: f32,
}

pub struct MetricsCollector {
    pub fn record_llm_call(&mut self, tokens: usize, cost: f64);
    pub fn record_task_completion(&mut self, success: bool, steps: usize);
    pub fn export_report(&self) -> MetricsReport;
}
```

**Effort**: 1-2 giorni

---

## 🗺️ Roadmap Completo

### **Phase 2: LLM Integration** (2 settimane)
**Priorità**: 🔴 CRITICAL

1. [ ] LLM Provider Interface (2-3 giorni)
   - OpenAI integration
   - Anthropic integration
   - Ollama (local) integration
   
2. [ ] Tool Registry & Descriptions (2-3 giorni)
   - Auto-generate da API
   - OpenAI function calling format
   - Anthropic tool format

3. [ ] Prompt Engineering (3-4 giorni)
   - System prompts
   - HTML summarization
   - Context management

4. [ ] Basic Agent Orchestrator (3-4 giorni)
   - Task parsing
   - Action execution loop
   - Simple error handling

**Deliverables**:
- ✅ LLM può chiamare browser API
- ✅ Agent esegue task semplici (navigate, fill form, click)
- ✅ Basic error recovery

---

### **Phase 2.5: Workflow Engine** (1 settimana)
**Priorità**: 🟡 HIGH

1. [ ] WebWorkflow builder (2-3 giorni)
2. [ ] Conditional branching (1-2 giorni)
3. [ ] State management (1-2 giorni)
4. [ ] Auto-recovery logic (1-2 giorni)

**Deliverables**:
- ✅ Workflow declarativi
- ✅ Multi-step orchestration
- ✅ Automatic retry

---

### **Phase 3: Testing & Validation** (1 settimana)
**Priorità**: 🟡 HIGH

1. [ ] Write 20+ test scenarios (3-4 giorni)
2. [ ] Run against GPT-4, Claude, Llama (2-3 giorni)
3. [ ] Collect metrics & analyze (1-2 giorni)
4. [ ] Bug fixes & optimization (ongoing)

**Deliverables**:
- ✅ Benchmark suite
- ✅ Performance report
- ✅ Known limitations documented

---

### **Phase 4: Advanced Features** (2 settimane)
**Priorità**: 🟢 MEDIUM

1. [ ] Vision support (screenshot → LLM)
2. [ ] Memory & context persistence
3. [ ] Multi-page workflows
4. [ ] Parallel task execution
5. [ ] Fine-tuning data collection

---

## 📝 Test Plan Dettagliato

### **Unit Tests** (Current: ✅ 69 tests)
```bash
cargo test --lib --features browser-automation
```

### **Integration Tests con LLM Mock**
```rust
// tests/integration/mock_llm_agent.rs
#[tokio::test]
async fn test_agent_can_login() {
    let mock_llm = MockLLM::new()
        .expect_call("What should I do?")
        .respond_with("fill_field_smart('username', 'test')");
    
    let agent = WebAgent::new(mock_llm, browser_pool);
    let result = agent.execute_task("Login to example.com").await;
    
    assert!(result.is_ok());
}
```

### **E2E Tests con LLM Reali**
```bash
# With OpenAI
OPENAI_API_KEY=xxx cargo test --test agent_e2e --features llm-openai

# With Anthropic
ANTHROPIC_API_KEY=xxx cargo test --test agent_e2e --features llm-anthropic

# With Ollama (local)
cargo test --test agent_e2e --features llm-ollama
```

### **Benchmark Tests**
```bash
cargo run --example agent_benchmark --release
```

Output:
```
🤖 Agent Benchmark Results
═══════════════════════════════════════

Model: GPT-4 Turbo
  ✅ Success Rate: 85% (17/20 scenarios)
  💰 Avg Cost/Task: $0.12
  ⏱️  Avg Time/Task: 15.3s
  📊 Avg Steps/Task: 4.2

Model: Claude 3 Opus
  ✅ Success Rate: 82% (16/20 scenarios)
  💰 Avg Cost/Task: $0.09
  ⏱️  Avg Time/Task: 12.8s
  📊 Avg Steps/Task: 3.8

Model: Llama 3 70B (local)
  ✅ Success Rate: 65% (13/20 scenarios)
  💰 Avg Cost/Task: $0.00
  ⏱️  Avg Time/Task: 8.2s
  📊 Avg Steps/Task: 5.1
```

---

## 🚀 Quick Start per Testare OGGI

### Opzione 1: Test con Ollama (Locale, Gratis)

```bash
# 1. Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# 2. Pull modello
ollama pull llama3:70b

# 3. Run test
cd semanticbrowser
cargo run --example agent_simple_task --features llm-ollama
```

### Opzione 2: Test con OpenAI

```bash
export OPENAI_API_KEY="sk-..."
cargo run --example agent_simple_task --features llm-openai
```

### Esempio di Task
```rust
let agent = WebAgent::new(openai_provider, browser_pool).await?;

let task = "Go to https://the-internet.herokuapp.com/login \
            and login with username 'tomsmith' and password 'SuperSecretPassword!'";

let result = agent.execute_task(task).await?;

println!("✅ Task completed!");
println!("Final URL: {}", result.final_url);
println!("Steps taken: {}", result.steps.len());
println!("Total cost: ${:.4}", result.total_cost);
```

---

## 📈 Success Metrics

### **Obiettivi Minimi (MVP)**:
- ✅ 70%+ success rate su 10 scenari base
- ✅ Avg < 10 steps per task semplice
- ✅ Avg < $0.20/task con GPT-4
- ✅ Errors con retry automatico

### **Obiettivi Target**:
- ✅ 85%+ success rate su 20+ scenari
- ✅ Avg < 6 steps per task
- ✅ Avg < $0.10/task
- ✅ Full observability & replay

### **Obiettivi Stretch**:
- ✅ 95%+ success rate
- ✅ Vision support (screenshot analysis)
- ✅ Multi-page workflows
- ✅ Parallel execution

---

## 🎯 Timeline Realistica

**Immediate (1 settimana)**:
- Implement basic LLM provider (OpenAI + Ollama)
- Simple agent orchestrator
- 5 test scenarios
- **Deliverable**: Working agent prototype

**Short-term (1 mese)**:
- Full Phase 2 + 2.5
- 20+ test scenarios
- Benchmark suite
- **Deliverable**: Production-ready agent system

**Long-term (3 mesi)**:
- Advanced features (vision, memory, etc.)
- Fine-tuning data collection
- Multi-agent coordination
- **Deliverable**: Enterprise-grade platform

---

## 💡 Raccomandazioni

### **Start Immediately**:
1. ✅ Implement `LLMProvider` trait
2. ✅ Integrate Ollama (free, local testing)
3. ✅ Write 3-5 simple scenarios
4. ✅ Build basic orchestrator loop

### **Quick Wins**:
- Use existing `SmartFormFiller` (already agent-friendly!)
- Leverage `FormAnalyzer` for context
- Use screenshots for debugging

### **Avoid**:
- ❌ Premature optimization
- ❌ Supporting too many LLM providers initially
- ❌ Complex workflow engine before basic works

---

## 📚 Resources Needed

**APIs**:
- OpenAI API key ($10-50/month testing)
- Anthropic API key (optional)
- Ollama (free, local)

**Infrastructure**:
- CI/CD for automated testing
- Metrics storage (Prometheus/Grafana)
- Trace storage (Jaeger/Tempo)

**Documentation**:
- Agent API docs
- Prompt engineering guide
- Best practices

---

## ✅ Checklist per Test Completo

### **Funzionalità Core**:
- [ ] LLM può navigare a URL
- [ ] LLM può riempire form (con SmartFormFiller)
- [ ] LLM può cliccare elementi
- [ ] LLM può estrarre dati
- [ ] LLM può fare screenshot
- [ ] LLM può gestire errori

### **Orchestration**:
- [ ] Multi-step task execution
- [ ] Conditional logic
- [ ] Error recovery
- [ ] State management
- [ ] Cost tracking

### **Observability**:
- [ ] Trace ogni decisione LLM
- [ ] Metrics export
- [ ] Error logs dettagliati
- [ ] Replay capability

### **Performance**:
- [ ] < 10s per task semplice
- [ ] < $0.20/task (GPT-4)
- [ ] 70%+ success rate
- [ ] Graceful degradation

---

**Status**: 📋 Ready to implement  
**Next Step**: Implement `LLMProvider` trait + Ollama integration  
**ETA**: 1 week for working prototype

