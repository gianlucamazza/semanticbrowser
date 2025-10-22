pub mod agent;
#[cfg(feature = "llm-anthropic")]
pub mod anthropic;
pub mod browser_executor;
pub mod ollama;
#[cfg(feature = "llm-openai")]
pub mod openai;
/// LLM Integration Layer
///
/// Provides a unified interface for interacting with different LLM providers
/// (OpenAI, Anthropic, Ollama, etc.) for agent orchestration.
pub mod provider;
pub mod tools;
pub mod workflow;

pub use agent::{AgentOrchestrator, AgentResponse, AgentTask};
#[cfg(feature = "llm-anthropic")]
pub use anthropic::AnthropicProvider;
pub use ollama::{OllamaConfig, OllamaProvider};
#[cfg(feature = "llm-openai")]
pub use openai::OpenAIProvider;
pub use provider::{
    FunctionCall, LLMConfig, LLMError, LLMProvider, LLMResponse, LLMResult, Message, Role,
    TokenUsage, ToolCall,
};
pub use tools::{
    FunctionDefinition, ParametersSchema, ToolDefinition, ToolParameter, ToolRegistry,
};
pub use workflow::{
    Condition, WebWorkflow, WebWorkflowBuilder, WorkflowError, WorkflowExecutor, WorkflowResult,
    WorkflowState, WorkflowStatus, WorkflowStep,
};

#[cfg(feature = "browser-automation")]
pub use browser_executor::BrowserExecutor;
