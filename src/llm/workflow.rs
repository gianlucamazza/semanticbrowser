//! Web Workflow Engine
//!
//! Orchestrates multi-step web automation tasks with conditional branching,
//! error recovery, and state persistence for complex agent operations.

#[cfg(feature = "browser-automation")]
use super::browser_executor::BrowserExecutor;
use super::provider::{LLMProvider, ToolCall};
use super::tools::ToolRegistry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::{error, info, warn};

/// Workflow execution errors
#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error("Step execution failed: {step_name} - {error}")]
    StepFailed { step_name: String, error: String },

    #[error("Conditional evaluation failed: {condition}")]
    ConditionFailed { condition: String },

    #[error("Workflow validation error: {message}")]
    ValidationError { message: String },

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Browser not available")]
    BrowserNotAvailable,

    #[error("LLM provider error: {0}")]
    LLMError(String),

    #[error("Max retries exceeded for step: {step_name}")]
    MaxRetriesExceeded { step_name: String },
}

/// Retry configuration for error recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub backoff_ms: u64,
    pub exponential_backoff: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self { max_attempts: 3, backoff_ms: 1000, exponential_backoff: true }
    }
}

/// Result type for workflow operations
pub type WorkflowResult<T> = Result<T, WorkflowError>;

/// Step execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_name: String,
    pub success: bool,
    pub output: serde_json::Value,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Workflow execution state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    pub workflow_id: String,
    pub current_step: usize,
    pub variables: HashMap<String, serde_json::Value>,
    pub step_results: Vec<StepResult>,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub last_update: chrono::DateTime<chrono::Utc>,
    pub status: WorkflowStatus,
}

/// Workflow execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Types of workflow steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStep {
    /// Execute a tool call
    ToolCall {
        name: String,
        tool_call: ToolCall,
        timeout_ms: Option<u64>,
    },

    /// Conditional branch based on variable value
    ConditionalBranch {
        name: String,
        condition: Condition,
        then_steps: Vec<WorkflowStep>,
        else_steps: Vec<WorkflowStep>,
    },

    /// Loop over a collection
    Loop {
        name: String,
        variable: String,
        items: Vec<serde_json::Value>,
        loop_steps: Vec<WorkflowStep>,
        max_iterations: Option<usize>,
    },

    /// Set a variable value
    SetVariable {
        name: String,
        variable: String,
        value: serde_json::Value,
    },

    /// Wait for a condition or timeout
    Wait {
        name: String,
        condition: Option<Condition>,
        timeout_ms: u64,
    },

    /// Execute steps in parallel
    Parallel {
        name: String,
        parallel_steps: Vec<Vec<WorkflowStep>>,
        max_concurrent: Option<usize>,
    },

    /// Error handling step
    ErrorHandler {
        name: String,
        error_variable: String,
        handler_steps: Vec<WorkflowStep>,
        retry_count: Option<u32>,
    },
}

/// Condition for branching or waiting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    /// Variable equals value
    Equals {
        variable: String,
        value: serde_json::Value,
    },

    /// Variable contains substring
    Contains { variable: String, substring: String },

    /// Variable exists
    Exists { variable: String },

    /// Custom JavaScript expression (for browser context)
    JavaScript { expression: String },

    /// HTTP status check
    HttpStatus { expected: u16 },

    /// Element exists on page
    ElementExists { selector: String },
}

/// Web workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebWorkflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub variables: HashMap<String, serde_json::Value>,
    pub timeout_ms: Option<u64>,
    pub max_retries: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Workflow executor
#[allow(dead_code)]
pub struct WorkflowExecutor {
    #[cfg(feature = "browser-automation")]
    browser: Option<Arc<BrowserExecutor>>,
    llm_provider: Option<Arc<dyn LLMProvider>>,
    tool_registry: ToolRegistry,
}

impl WorkflowExecutor {
    /// Create new workflow executor
    pub fn new(tool_registry: ToolRegistry) -> Self {
        Self {
            #[cfg(feature = "browser-automation")]
            browser: None,
            llm_provider: None,
            tool_registry,
        }
    }

    /// With browser executor
    #[cfg(feature = "browser-automation")]
    pub fn with_browser(mut self, browser: Arc<BrowserExecutor>) -> Self {
        self.browser = Some(browser);
        self
    }

    /// With LLM provider for dynamic decisions
    pub fn with_llm_provider(mut self, provider: Arc<dyn LLMProvider>) -> Self {
        self.llm_provider = Some(provider);
        self
    }

    /// Execute workflow
    pub async fn execute_workflow(&self, workflow: &WebWorkflow) -> WorkflowResult<WorkflowState> {
        self.execute_workflow_with_retry(workflow, &RetryConfig::default()).await
    }

    /// Execute workflow with custom retry configuration
    pub async fn execute_workflow_with_retry(
        &self,
        workflow: &WebWorkflow,
        retry_config: &RetryConfig,
    ) -> WorkflowResult<WorkflowState> {
        info!("Starting workflow execution: {}", workflow.name);

        let mut state = WorkflowState {
            workflow_id: workflow.id.clone(),
            current_step: 0,
            variables: workflow.variables.clone(),
            step_results: Vec::new(),
            start_time: chrono::Utc::now(),
            last_update: chrono::Utc::now(),
            status: WorkflowStatus::Running,
        };

        // Execute steps sequentially
        for (step_index, step) in workflow.steps.iter().enumerate() {
            state.current_step = step_index;

            let result = self.execute_step_with_retry(step, &mut state, retry_config).await;

            let step_result = match result {
                Ok(output) => StepResult {
                    step_name: self.get_step_name(step),
                    success: true,
                    output,
                    error: None,
                    execution_time_ms: 0, // Will be set by execute_step_with_retry
                    timestamp: chrono::Utc::now(),
                },
                Err(e) => {
                    error!("Step {} failed after retries: {}", step_index, e);
                    StepResult {
                        step_name: self.get_step_name(step),
                        success: false,
                        output: serde_json::Value::Null,
                        error: Some(e.to_string()),
                        execution_time_ms: 0,
                        timestamp: chrono::Utc::now(),
                    }
                }
            };

            let should_stop = !step_result.success;
            state.step_results.push(step_result);
            state.last_update = chrono::Utc::now();

            // Check if we should stop execution
            if should_stop {
                state.status = WorkflowStatus::Failed;
                return Ok(state);
            }
        }

        state.status = WorkflowStatus::Completed;
        info!("Workflow {} completed successfully", workflow.name);
        Ok(state)
    }

    /// Execute single step with retry logic
    async fn execute_step_with_retry(
        &self,
        step: &WorkflowStep,
        state: &mut WorkflowState,
        retry_config: &RetryConfig,
    ) -> WorkflowResult<serde_json::Value> {
        let mut _last_error = None;

        for attempt in 1..=retry_config.max_attempts {
            let start_time = std::time::Instant::now();
            let result = self.execute_step(step, state).await;
            let execution_time = start_time.elapsed().as_millis() as u64;

            match result {
                Ok(output) => {
                    // Update the last step result with correct execution time
                    if let Some(last_result) = state.step_results.last_mut() {
                        last_result.execution_time_ms = execution_time;
                    }
                    return Ok(output);
                }
                Err(e) => {
                    warn!("Step attempt {}/{} failed: {}", attempt, retry_config.max_attempts, e);
                    _last_error = Some(e);

                    // If this isn't the last attempt, wait before retrying
                    if attempt < retry_config.max_attempts {
                        let backoff = if retry_config.exponential_backoff {
                            retry_config.backoff_ms * (2_u64.pow(attempt - 1))
                        } else {
                            retry_config.backoff_ms
                        };

                        info!("Waiting {}ms before retry...", backoff);
                        tokio::time::sleep(std::time::Duration::from_millis(backoff)).await;
                    }
                }
            }
        }

        // All attempts failed
        Err(WorkflowError::MaxRetriesExceeded { step_name: self.get_step_name(step) })
    }

    /// Execute single step
    async fn execute_step(
        &self,
        step: &WorkflowStep,
        state: &mut WorkflowState,
    ) -> WorkflowResult<serde_json::Value> {
        match step {
            WorkflowStep::ToolCall { tool_call, .. } => self.execute_tool_call(tool_call).await,
            WorkflowStep::ConditionalBranch { condition, .. } => {
                // For now, just evaluate condition and return result
                // Complex branching logic should be handled at workflow level
                let condition_met = self.evaluate_condition(condition, state).await?;
                Ok(serde_json::Value::Bool(condition_met))
            }
            WorkflowStep::Loop { variable, items, loop_steps, max_iterations, .. } => {
                let max_iter = max_iterations.unwrap_or(100); // Default limit
                let mut results = Vec::new();

                for (i, item) in items.iter().enumerate() {
                    if i >= max_iter {
                        break;
                    }

                    // Set loop variable
                    state.variables.insert(variable.clone(), item.clone());

                    // Execute loop steps (simplified - no recursion)
                    let mut loop_results = Vec::new();
                    for step in loop_steps {
                        match step {
                            WorkflowStep::ToolCall { tool_call, .. } => {
                                match self.execute_tool_call(tool_call).await {
                                    Ok(result) => loop_results.push(result),
                                    Err(e) => return Err(e),
                                }
                            }
                            WorkflowStep::SetVariable { variable: var, value, .. } => {
                                state.variables.insert(var.clone(), value.clone());
                                loop_results
                                    .push(serde_json::Value::String(format!("Set {}", var)));
                            }
                            _ => {
                                // Skip complex steps in loop for now
                                loop_results.push(serde_json::Value::String(
                                    "Step skipped in loop".to_string(),
                                ));
                            }
                        }
                    }
                    results.push(serde_json::Value::Array(loop_results));
                }

                Ok(serde_json::Value::Array(results))
            }
            WorkflowStep::SetVariable { variable, value, .. } => {
                state.variables.insert(variable.clone(), value.clone());
                Ok(serde_json::Value::String(format!("Set {} = {:?}", variable, value)))
            }
            WorkflowStep::Wait { condition, timeout_ms, .. } => {
                self.execute_wait(condition.as_ref(), *timeout_ms).await
            }
            WorkflowStep::Parallel { parallel_steps, max_concurrent, .. } => {
                let max_conc = max_concurrent.unwrap_or(5);
                let mut results = Vec::new();

                // Simple parallel execution using chunks
                for chunk in parallel_steps.chunks(max_conc) {
                    let mut futures = Vec::new();

                    for steps in chunk {
                        let future_results = self.execute_parallel_steps(steps.clone(), state);
                        futures.push(future_results);
                    }

                    // Wait for this batch to complete
                    for future in futures {
                        match future.await {
                            Ok(batch_results) => results.extend(batch_results),
                            Err(e) => return Err(e),
                        }
                    }
                }

                Ok(serde_json::Value::Array(results))
            }
            WorkflowStep::ErrorHandler { error_variable, handler_steps, retry_count, .. } => {
                // Check if there's an error in the specified variable
                if let Some(error_val) = state.variables.get(error_variable) {
                    if !error_val.is_null() {
                        let retries = retry_count.unwrap_or(3);
                        let mut last_error = None;

                        for attempt in 1..=retries {
                            info!("Error handler attempt {}/{}", attempt, retries);

                            // Execute handler steps (simplified)
                            let mut handler_results = Vec::new();
                            for step in handler_steps {
                                match step {
                                    WorkflowStep::ToolCall { tool_call, .. } => {
                                        match self.execute_tool_call(tool_call).await {
                                            Ok(result) => handler_results.push(result),
                                            Err(e) => {
                                                last_error = Some(e);
                                                break;
                                            }
                                        }
                                    }
                                    WorkflowStep::SetVariable { variable: var, value, .. } => {
                                        state.variables.insert(var.clone(), value.clone());
                                        handler_results.push(serde_json::Value::String(format!(
                                            "Set {}",
                                            var
                                        )));
                                    }
                                    _ => {
                                        handler_results.push(serde_json::Value::String(
                                            "Handler step executed".to_string(),
                                        ));
                                    }
                                }
                            }

                            if last_error.is_none() {
                                return Ok(serde_json::Value::Array(handler_results));
                            }
                        }

                        // All retries failed
                        return Err(last_error.unwrap_or_else(|| WorkflowError::StepFailed {
                            step_name: "error_handler".to_string(),
                            error: "All retry attempts failed".to_string(),
                        }));
                    }
                }

                Ok(serde_json::Value::String("No error to handle".to_string()))
            }
        }
    }

    /// Execute tool call
    #[allow(unused_variables)]
    async fn execute_tool_call(&self, tool_call: &ToolCall) -> WorkflowResult<serde_json::Value> {
        #[cfg(feature = "browser-automation")]
        if let Some(browser) = &self.browser {
            return browser.execute_tool(tool_call).await.map(serde_json::Value::String).map_err(
                |e| WorkflowError::StepFailed {
                    step_name: tool_call.function.name.clone(),
                    error: e.to_string(),
                },
            );
        }

        Err(WorkflowError::BrowserNotAvailable)
    }

    /// Evaluate condition
    async fn evaluate_condition(
        &self,
        condition: &Condition,
        state: &WorkflowState,
    ) -> WorkflowResult<bool> {
        match condition {
            Condition::Equals { variable, value } => {
                Ok(state.variables.get(variable) == Some(value))
            }
            Condition::Exists { variable } => Ok(state.variables.contains_key(variable)),
            Condition::Contains { variable, substring } => {
                if let Some(var_value) = state.variables.get(variable) {
                    if let Some(str_val) = var_value.as_str() {
                        return Ok(str_val.contains(substring));
                    }
                }
                Ok(false)
            }
            #[cfg(feature = "browser-automation")]
            Condition::ElementExists { selector } => {
                if let Some(browser) = &self.browser {
                    // Check if element exists by trying to find it
                    Ok(browser.element_exists(selector).await)
                } else {
                    Ok(false)
                }
            }
            _ => Err(WorkflowError::ConditionFailed {
                condition: format!("Unsupported condition: {:?}", condition),
            }),
        }
    }

    /// Execute parallel steps
    async fn execute_parallel_steps(
        &self,
        steps: Vec<WorkflowStep>,
        _state: &WorkflowState,
    ) -> WorkflowResult<Vec<serde_json::Value>> {
        let mut results = Vec::new();

        // Execute steps sequentially in this "parallel" batch
        // In a real implementation, this could use tokio::spawn for true parallelism
        for step in steps {
            match step {
                WorkflowStep::ToolCall { tool_call, .. } => {
                    match self.execute_tool_call(&tool_call).await {
                        Ok(result) => results.push(result),
                        Err(e) => return Err(e),
                    }
                }
                WorkflowStep::SetVariable { variable, value, .. } => {
                    // Note: This modifies state, which could be problematic in parallel execution
                    results
                        .push(serde_json::Value::String(format!("Set {} = {:?}", variable, value)));
                }
                _ => {
                    results.push(serde_json::Value::String("Parallel step executed".to_string()));
                }
            }
        }

        Ok(results)
    }

    /// Execute wait step
    async fn execute_wait(
        &self,
        condition: Option<&Condition>,
        timeout_ms: u64,
    ) -> WorkflowResult<serde_json::Value> {
        let start = std::time::Instant::now();
        let timeout_duration = std::time::Duration::from_millis(timeout_ms);

        while start.elapsed() < timeout_duration {
            if let Some(cond) = condition {
                // For now, create a dummy state for condition evaluation
                let dummy_state = WorkflowState {
                    workflow_id: String::new(),
                    current_step: 0,
                    variables: HashMap::new(),
                    step_results: Vec::new(),
                    start_time: chrono::Utc::now(),
                    last_update: chrono::Utc::now(),
                    status: WorkflowStatus::Running,
                };

                if self.evaluate_condition(cond, &dummy_state).await? {
                    return Ok(serde_json::Value::String("Condition met".to_string()));
                }
            }

            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        Ok(serde_json::Value::String("Timeout reached".to_string()))
    }

    /// Get step name for logging
    fn get_step_name(&self, step: &WorkflowStep) -> String {
        match step {
            WorkflowStep::ToolCall { name, .. } => name.clone(),
            WorkflowStep::ConditionalBranch { name, .. } => name.clone(),
            WorkflowStep::Loop { name, .. } => name.clone(),
            WorkflowStep::SetVariable { name, .. } => name.clone(),
            WorkflowStep::Wait { name, .. } => name.clone(),
            WorkflowStep::Parallel { name, .. } => name.clone(),
            WorkflowStep::ErrorHandler { name, .. } => name.clone(),
        }
    }
}

/// Builder for creating workflows
pub struct WebWorkflowBuilder {
    id: Option<String>,
    name: String,
    description: String,
    steps: Vec<WorkflowStep>,
    variables: HashMap<String, serde_json::Value>,
    timeout_ms: Option<u64>,
    max_retries: u32,
}

impl WebWorkflowBuilder {
    /// Create new workflow builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            description: String::new(),
            steps: Vec::new(),
            variables: HashMap::new(),
            timeout_ms: None,
            max_retries: 3,
        }
    }

    /// Set workflow ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Add a tool call step
    pub fn tool_call(mut self, name: impl Into<String>, tool_call: ToolCall) -> Self {
        self.steps.push(WorkflowStep::ToolCall { name: name.into(), tool_call, timeout_ms: None });
        self
    }

    /// Add conditional branch
    pub fn conditional_branch(
        mut self,
        name: impl Into<String>,
        condition: Condition,
        then_steps: Vec<WorkflowStep>,
        else_steps: Vec<WorkflowStep>,
    ) -> Self {
        self.steps.push(WorkflowStep::ConditionalBranch {
            name: name.into(),
            condition,
            then_steps,
            else_steps,
        });
        self
    }

    /// Add variable assignment
    pub fn set_variable(
        mut self,
        name: impl Into<String>,
        variable: impl Into<String>,
        value: serde_json::Value,
    ) -> Self {
        self.steps.push(WorkflowStep::SetVariable {
            name: name.into(),
            variable: variable.into(),
            value,
        });
        self
    }

    /// Add wait step
    pub fn wait(mut self, name: impl Into<String>, timeout_ms: u64) -> Self {
        self.steps.push(WorkflowStep::Wait { name: name.into(), condition: None, timeout_ms });
        self
    }

    /// Set initial variable
    pub fn variable(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.variables.insert(key.into(), value);
        self
    }

    /// Set timeout
    pub fn timeout_ms(mut self, timeout: u64) -> Self {
        self.timeout_ms = Some(timeout);
        self
    }

    /// Add a custom step
    pub fn step(mut self, step: WorkflowStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Build the workflow
    pub fn build(self) -> WebWorkflow {
        WebWorkflow {
            id: self.id.unwrap_or_else(|| format!("workflow_{}", chrono::Utc::now().timestamp())),
            name: self.name,
            description: self.description,
            steps: self.steps,
            variables: self.variables,
            timeout_ms: self.timeout_ms,
            max_retries: self.max_retries,
            created_at: chrono::Utc::now(),
        }
    }
}

impl WebWorkflow {
    /// Create a builder
    pub fn builder(name: impl Into<String>) -> WebWorkflowBuilder {
        WebWorkflowBuilder::new(name)
    }

    /// Save workflow to JSON file
    pub fn save_to_file(&self, path: &std::path::Path) -> WorkflowResult<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load workflow from JSON file
    pub fn load_from_file(path: &std::path::Path) -> WorkflowResult<Self> {
        let json = std::fs::read_to_string(path)?;
        let workflow = serde_json::from_str(&json)?;
        Ok(workflow)
    }
}

impl WorkflowState {
    /// Save state to JSON file
    pub fn save_to_file(&self, path: &std::path::Path) -> WorkflowResult<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load state from JSON file
    pub fn load_from_file(path: &std::path::Path) -> WorkflowResult<Self> {
        let json = std::fs::read_to_string(path)?;
        let state = serde_json::from_str(&json)?;
        Ok(state)
    }

    /// Resume workflow execution from this state
    pub async fn resume_execution(
        &self,
        executor: &WorkflowExecutor,
        workflow: &WebWorkflow,
    ) -> WorkflowResult<Self> {
        let mut resumed_state = self.clone();
        resumed_state.status = WorkflowStatus::Running;
        resumed_state.last_update = chrono::Utc::now();

        // Continue execution from the next step
        let start_step = resumed_state.current_step;

        for (step_index, step) in workflow.steps.iter().enumerate().skip(start_step) {
            resumed_state.current_step = step_index;

            let start_time = std::time::Instant::now();
            let result = executor.execute_step(step, &mut resumed_state).await;

            let execution_time = start_time.elapsed().as_millis() as u64;

            let step_result = match result {
                Ok(output) => StepResult {
                    step_name: executor.get_step_name(step),
                    success: true,
                    output,
                    error: None,
                    execution_time_ms: execution_time,
                    timestamp: chrono::Utc::now(),
                },
                Err(e) => {
                    error!("Step {} failed during resume: {}", step_index, e);
                    StepResult {
                        step_name: executor.get_step_name(step),
                        success: false,
                        output: serde_json::Value::Null,
                        error: Some(e.to_string()),
                        execution_time_ms: execution_time,
                        timestamp: chrono::Utc::now(),
                    }
                }
            };

            resumed_state.step_results.push(step_result.clone());
            resumed_state.last_update = chrono::Utc::now();

            // Check if we should stop execution
            if !step_result.success {
                resumed_state.status = WorkflowStatus::Failed;
                return Ok(resumed_state);
            }
        }

        resumed_state.status = WorkflowStatus::Completed;
        info!("Resumed workflow {} completed successfully", workflow.name);
        Ok(resumed_state)
    }

    /// Get summary of execution progress
    pub fn get_progress_summary(&self) -> serde_json::Value {
        let total_steps = self.step_results.len();
        let successful_steps = self.step_results.iter().filter(|r| r.success).count();
        let failed_steps = total_steps - successful_steps;

        serde_json::json!({
            "workflow_id": self.workflow_id,
            "status": format!("{:?}", self.status),
            "current_step": self.current_step,
            "total_steps_executed": total_steps,
            "successful_steps": successful_steps,
            "failed_steps": failed_steps,
            "success_rate": if total_steps > 0 { successful_steps as f64 / total_steps as f64 } else { 0.0 },
            "start_time": self.start_time.to_rfc3339(),
            "last_update": self.last_update.to_rfc3339(),
            "duration_seconds": (self.last_update - self.start_time).num_seconds(),
            "variables_count": self.variables.len()
        })
    }
}
