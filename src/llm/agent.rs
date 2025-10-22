#[cfg(feature = "browser-automation")]
use super::browser_executor::BrowserExecutor;
use super::provider::{LLMConfig, LLMProvider, LLMResult, Message};
use super::tools::ToolRegistry;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use std::sync::{Arc, RwLock};
use tracing::{info, warn};

#[cfg(feature = "onnx-integration")]
use crate::kg::KnowledgeGraph;
#[cfg(feature = "onnx-integration")]
use crate::ml::inference::LinkPredictor;

/// Task to be executed by the agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub goal: String,
    pub context: Option<String>,
    pub max_iterations: usize,
}

impl AgentTask {
    pub fn new(goal: impl Into<String>) -> Self {
        Self { goal: goal.into(), context: None, max_iterations: 10 }
    }

    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }
}

/// Response from the agent after task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub success: bool,
    pub result: String,
    pub iterations: usize,
    pub error: Option<String>,
}

/// Agent execution step
#[derive(Debug, Clone)]
struct AgentStep {
    thought: String,
    action: Option<String>,
    action_input: Option<serde_json::Value>,
}

/// Agent Orchestrator
///
/// Orchestrates LLM-based autonomous agents that can use tools to accomplish tasks.
/// Implements a ReAct-style (Reasoning + Acting) loop.
pub struct AgentOrchestrator {
    provider: Arc<dyn LLMProvider>,
    config: LLMConfig,
    tools: ToolRegistry,
    #[cfg(feature = "browser-automation")]
    browser: Option<Arc<BrowserExecutor>>,
    #[cfg(feature = "onnx-integration")]
    kg: Option<Arc<RwLock<KnowledgeGraph>>>,
    #[cfg(feature = "onnx-integration")]
    predictor: Option<Arc<RwLock<LinkPredictor>>>,
    system_prompt: String,
}

impl AgentOrchestrator {
    pub fn new(provider: Arc<dyn LLMProvider>, config: LLMConfig, tools: ToolRegistry) -> Self {
        let system_prompt = Self::default_system_prompt();

        Self {
            provider,
            config,
            tools,
            #[cfg(feature = "browser-automation")]
            browser: None,
            #[cfg(feature = "onnx-integration")]
            kg: None,
            #[cfg(feature = "onnx-integration")]
            predictor: None,
            system_prompt,
        }
    }

    #[cfg(feature = "browser-automation")]
    pub fn with_browser(mut self, browser: Arc<BrowserExecutor>) -> Self {
        self.browser = Some(browser);
        self
    }

    #[cfg(feature = "onnx-integration")]
    pub fn with_kg(mut self, kg: KnowledgeGraph) -> Self {
        self.kg = Some(Arc::new(RwLock::new(kg)));
        self
    }

    #[cfg(feature = "onnx-integration")]
    pub fn with_predictor(mut self, predictor: LinkPredictor) -> Self {
        self.predictor = Some(Arc::new(RwLock::new(predictor)));
        self
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    fn default_system_prompt() -> String {
        r#"You are an autonomous web browsing agent with memory capabilities. Your goal is to accomplish tasks by using the available tools.

Follow this thought process for each step:
1. THOUGHT: Analyze the current situation, recall relevant information from memory, and decide what to do next
2. ACTION: Choose a tool to use (or FINISH if task is complete)
3. ACTION INPUT: Provide the parameters for the tool
4. OBSERVATION: Observe the result of the action

Available format:
THOUGHT: <your reasoning>
ACTION: <tool_name or FINISH>
ACTION INPUT: <JSON parameters or final answer>

Memory capabilities:
- Use store_memory to save important information for future reference
- Use query_kg to retrieve stored information
- Use predict_link to find related entities in the knowledge graph

When you have completed the task, use ACTION: FINISH with your final answer in ACTION INPUT.

Be concise but thorough. Think step by step."#.to_string()
    }

    /// Execute a task with the agent
    pub async fn execute(&self, task: AgentTask) -> LLMResult<AgentResponse> {
        info!("Agent starting task: {}", task.goal);

        let mut messages = vec![Message::system(&self.system_prompt)];

        // Add task and context
        let mut task_description = format!("TASK: {}", task.goal);
        if let Some(context) = &task.context {
            task_description.push_str(&format!("\n\nCONTEXT: {}", context));
        }
        messages.push(Message::user(task_description));

        let mut iterations = 0;
        let tools_json = self.tools.get_tools_json();

        loop {
            if iterations >= task.max_iterations {
                warn!("Agent reached max iterations ({})", task.max_iterations);
                return Ok(AgentResponse {
                    success: false,
                    result: "Maximum iterations reached".to_string(),
                    iterations,
                    error: Some("Max iterations exceeded".to_string()),
                });
            }

            iterations += 1;
            info!("Agent iteration {}/{}", iterations, task.max_iterations);

            // Get LLM response
            let response = self
                .provider
                .chat_completion_with_tools(messages.clone(), tools_json.clone(), &self.config)
                .await?;

            // Parse the response
            let step = self.parse_response(&response.content)?;

            info!("Thought: {}", step.thought);

            if let Some(action) = &step.action {
                info!("Action: {}", action);

                // Check if agent wants to finish
                if action.to_uppercase() == "FINISH" {
                    let result = step
                        .action_input
                        .as_ref()
                        .and_then(|v| v.as_str())
                        .unwrap_or(&step.thought)
                        .to_string();

                    info!("Agent finished successfully");
                    return Ok(AgentResponse { success: true, result, iterations, error: None });
                }

                // Execute tool (simulated for now)
                let observation = self.execute_tool(action, step.action_input.as_ref()).await?;
                info!("Observation: {}", observation);

                // Add observation to conversation
                messages.push(Message::assistant(&response.content));
                messages.push(Message::user(format!("OBSERVATION: {}", observation)));
            } else {
                // No action specified, add response and continue
                messages.push(Message::assistant(&response.content));
            }
        }
    }

    fn parse_response(&self, content: &str) -> LLMResult<AgentStep> {
        let mut thought = String::new();
        let mut action: Option<String> = None;
        let mut action_input: Option<serde_json::Value> = None;

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("THOUGHT:") {
                thought = line.strip_prefix("THOUGHT:").unwrap().trim().to_string();
            } else if line.starts_with("ACTION:") {
                action = Some(line.strip_prefix("ACTION:").unwrap().trim().to_string());
            } else if line.starts_with("ACTION INPUT:") {
                let input_str = line.strip_prefix("ACTION INPUT:").unwrap().trim();
                // Try to parse as JSON, otherwise use as string
                action_input = Some(
                    serde_json::from_str(input_str)
                        .unwrap_or_else(|_| serde_json::Value::String(input_str.to_string())),
                );
            }
        }

        if thought.is_empty() {
            thought = content.to_string();
        }

        Ok(AgentStep { thought, action, action_input })
    }

    async fn execute_tool(
        &self,
        tool_name: &str,
        input: Option<&serde_json::Value>,
    ) -> LLMResult<String> {
        info!("Executing tool: {} with input: {:?}", tool_name, input);

        // Handle KG and ML tools with real execution if available
        if tool_name == "query_kg" {
            #[cfg(feature = "onnx-integration")]
            if let Some(kg_lock) = &self.kg {
                let kg = kg_lock.read().unwrap();
                let query = input
                    .and_then(|v| v.get("query"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        super::provider::LLMError::Config("Missing 'query' parameter".to_string())
                    })?;
                let results =
                    kg.query(query).map_err(|e| super::provider::LLMError::Api(e.to_string()))?;
                return Ok(format!("KG Query Results:\n{}", results.join("\n")));
            }
            return Ok("KG not available - enable onnx-integration feature".to_string());
        } else if tool_name == "store_memory" {
            #[cfg(feature = "onnx-integration")]
            if let Some(kg_lock) = &self.kg {
                let mut kg = kg_lock.write().unwrap();
                let subject = input
                    .and_then(|v| v.get("subject"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        super::provider::LLMError::Config("Missing 'subject' parameter".to_string())
                    })?;
                let predicate = input
                    .and_then(|v| v.get("predicate"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        super::provider::LLMError::Config(
                            "Missing 'predicate' parameter".to_string(),
                        )
                    })?;
                let object = input
                    .and_then(|v| v.get("object"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        super::provider::LLMError::Config("Missing 'object' parameter".to_string())
                    })?;
                kg.insert(subject, predicate, object)
                    .map_err(|e| super::provider::LLMError::Api(e.to_string()))?;
                return Ok(format!("Memory stored: {} {} {}", subject, predicate, object));
            }
            return Ok("KG not available for memory storage".to_string());
        } else if tool_name == "predict_link" {
            #[cfg(feature = "onnx-integration")]
            if let Some(predictor_lock) = &self.predictor {
                let predictor = predictor_lock.read().unwrap();
                let prediction_type = input
                    .and_then(|v| v.get("prediction_type"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        super::provider::LLMError::Config(
                            "Missing 'prediction_type' parameter".to_string(),
                        )
                    })?;

                match prediction_type {
                    "tail" => {
                        let head = input
                            .and_then(|v| v.get("head"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("http://ex.org/Alice");
                        let relation = input
                            .and_then(|v| v.get("relation"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("http://ex.org/knows");
                        let k = input.and_then(|v| v.get("k")).and_then(|v| v.as_u64()).unwrap_or(5)
                            as usize;
                        let predictions = predictor
                            .predict_tail(head, relation, k, false)
                            .map_err(|e| super::provider::LLMError::Api(e.to_string()))?;
                        let results: Vec<String> = predictions
                            .iter()
                            .map(|p| format!("{} (score: {:.3})", p.uri, p.score))
                            .collect();
                        return Ok(format!("Tail Predictions:\n{}", results.join("\n")));
                    }
                    "head" => {
                        let relation = input
                            .and_then(|v| v.get("relation"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("http://ex.org/knows");
                        let tail = input
                            .and_then(|v| v.get("tail"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("http://ex.org/Bob");
                        let k = input.and_then(|v| v.get("k")).and_then(|v| v.as_u64()).unwrap_or(5)
                            as usize;
                        let predictions = predictor
                            .predict_head(relation, tail, k, false)
                            .map_err(|e| super::provider::LLMError::Api(e.to_string()))?;
                        let results: Vec<String> = predictions
                            .iter()
                            .map(|p| format!("{} (score: {:.3})", p.uri, p.score))
                            .collect();
                        return Ok(format!("Head Predictions:\n{}", results.join("\n")));
                    }
                    "relation" => {
                        let head = input
                            .and_then(|v| v.get("head"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("http://ex.org/Alice");
                        let tail = input
                            .and_then(|v| v.get("tail"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("http://ex.org/Bob");
                        let k = input.and_then(|v| v.get("k")).and_then(|v| v.as_u64()).unwrap_or(5)
                            as usize;
                        let predictions = predictor
                            .predict_relation(head, tail, k, false)
                            .map_err(|e| super::provider::LLMError::Api(e.to_string()))?;
                        let results: Vec<String> = predictions
                            .iter()
                            .map(|p| format!("{} (score: {:.3})", p.uri, p.score))
                            .collect();
                        return Ok(format!("Relation Predictions:\n{}", results.join("\n")));
                    }
                    _ => {
                        return Err(super::provider::LLMError::Config(format!(
                            "Invalid prediction_type: {}",
                            prediction_type
                        )))
                    }
                }
            }
            return Ok("Link predictor not available - enable onnx-integration feature".to_string());
        }

        // If browser is available, use real execution
        #[cfg(feature = "browser-automation")]
        if let Some(browser) = &self.browser {
            return self.execute_tool_real(browser, tool_name, input).await;
        }

        // Otherwise, use mock execution for browser tools
        self.execute_tool_mock(tool_name, input).await
    }

    #[cfg(feature = "browser-automation")]
    async fn execute_tool_real(
        &self,
        browser: &BrowserExecutor,
        tool_name: &str,
        input: Option<&Value>,
    ) -> LLMResult<String> {
        use super::provider::ToolCall;

        // Convert the input to a ToolCall format that BrowserExecutor expects
        let tool_call = ToolCall {
            id: "agent-tool-call".to_string(),
            tool_type: "function".to_string(),
            function: super::provider::FunctionCall {
                name: tool_name.to_string(),
                arguments: input
                    .map(|v| serde_json::to_string(v).unwrap_or_else(|_| "{}".to_string()))
                    .unwrap_or_else(|| "{}".to_string()),
            },
        };

        browser.execute_tool(&tool_call).await
    }

    async fn execute_tool_mock(
        &self,
        tool_name: &str,
        input: Option<&serde_json::Value>,
    ) -> LLMResult<String> {
        // Mock execution for browser tools when browser not available
        match tool_name {
            "navigate_to" => {
                let url =
                    input.and_then(|v| v.get("url")).and_then(|v| v.as_str()).unwrap_or("unknown");
                Ok(format!("Successfully navigated to: {}", url))
            }
            "fill_form" => Ok("Form filled successfully".to_string()),
            "click_element" => {
                let selector = input
                    .and_then(|v| v.get("selector"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                Ok(format!("Clicked element: {}", selector))
            }
            "get_page_content" => Ok("<html><body><h1>Example Page</h1></body></html>".to_string()),
            "extract_data" => Ok(r#"{"title": "Example", "price": "$99.99"}"#.to_string()),
            _ => {
                warn!("Unknown tool: {}", tool_name);
                Ok(format!("Tool '{}' not implemented", tool_name))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::ollama::{OllamaConfig, OllamaProvider};

    #[tokio::test]
    #[ignore] // Requires Ollama running
    async fn test_agent_simple_task() {
        let provider = Arc::new(OllamaProvider::new(OllamaConfig::default()));
        let config = LLMConfig::default();
        let tools = ToolRegistry::with_browser_tools();

        let agent = AgentOrchestrator::new(provider, config, tools);

        let task = AgentTask::new("Navigate to google.com and search for 'Rust programming'")
            .with_max_iterations(5);

        let response = agent.execute(task).await;
        assert!(response.is_ok());

        let result = response.unwrap();
        println!("Agent result: {:?}", result);
    }

    #[test]
    fn test_tools_include_memory_and_ml() {
        let registry = ToolRegistry::with_browser_tools();
        let tools = registry.get_tools_json();

        // Check that new tools are included
        let tool_names: Vec<String> = tools
            .iter()
            .filter_map(|t| t.get("function").and_then(|f| f.get("name")).and_then(|n| n.as_str()))
            .map(|s| s.to_string())
            .collect();

        assert!(tool_names.contains(&"query_kg".to_string()));
        assert!(tool_names.contains(&"predict_link".to_string()));
        assert!(tool_names.contains(&"store_memory".to_string()));
    }
}
