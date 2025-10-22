use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Definition of a tool/function that can be called by the LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: FunctionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: ParametersSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParametersSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    pub properties: HashMap<String, ToolParameter>,
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
}

/// Registry for managing available tools
pub struct ToolRegistry {
    tools: HashMap<String, ToolDefinition>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    /// Register a new tool
    pub fn register(&mut self, tool: ToolDefinition) {
        self.tools.insert(tool.function.name.clone(), tool);
    }

    /// Get all registered tools as JSON values for LLM API
    pub fn get_tools_json(&self) -> Vec<serde_json::Value> {
        self.tools.values().map(|tool| serde_json::to_value(tool).unwrap()).collect()
    }

    /// Get a specific tool by name
    pub fn get_tool(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.get(name)
    }

    /// Create a default registry with browser automation tools
    pub fn with_browser_tools() -> Self {
        let mut registry = Self::new();

        // navigate_to tool
        registry.register(ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "navigate_to".to_string(),
                description: "Navigate to a URL in the browser".to_string(),
                parameters: ParametersSchema {
                    schema_type: "object".to_string(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "url".to_string(),
                            ToolParameter {
                                param_type: "string".to_string(),
                                description: "The URL to navigate to".to_string(),
                                enum_values: None,
                            },
                        );
                        props
                    },
                    required: vec!["url".to_string()],
                },
            },
        });

        // fill_form_field tool
        registry.register(ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "fill_form_field".to_string(),
                description: "Fill a specific form field with a value".to_string(),
                parameters: ParametersSchema {
                    schema_type: "object".to_string(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "field_name".to_string(),
                            ToolParameter {
                                param_type: "string".to_string(),
                                description: "Name or hint for the form field to fill".to_string(),
                                enum_values: None,
                            },
                        );
                        props.insert(
                            "value".to_string(),
                            ToolParameter {
                                param_type: "string".to_string(),
                                description: "Value to fill in the field".to_string(),
                                enum_values: None,
                            },
                        );
                        props
                    },
                    required: vec!["field_name".to_string(), "value".to_string()],
                },
            },
        });

        // click_element tool
        registry.register(ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "click_element".to_string(),
                description: "Click on an element matching the given selector".to_string(),
                parameters: ParametersSchema {
                    schema_type: "object".to_string(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "selector".to_string(),
                            ToolParameter {
                                param_type: "string".to_string(),
                                description: "CSS selector for the element to click".to_string(),
                                enum_values: None,
                            },
                        );
                        props
                    },
                    required: vec!["selector".to_string()],
                },
            },
        });

        // extract_text tool
        registry.register(ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "extract_text".to_string(),
                description: "Extract text content from an element using CSS selector".to_string(),
                parameters: ParametersSchema {
                    schema_type: "object".to_string(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "selector".to_string(),
                            ToolParameter {
                                param_type: "string".to_string(),
                                description: "CSS selector for the element to extract text from"
                                    .to_string(),
                                enum_values: None,
                            },
                        );
                        props
                    },
                    required: vec!["selector".to_string()],
                },
            },
        });

        // get_page_content tool
        registry.register(ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "get_page_content".to_string(),
                description: "Get the current page HTML content or text".to_string(),
                parameters: ParametersSchema {
                    schema_type: "object".to_string(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "format".to_string(),
                            ToolParameter {
                                param_type: "string".to_string(),
                                description: "Format of content to retrieve: 'html' or 'text'"
                                    .to_string(),
                                enum_values: Some(vec!["html".to_string(), "text".to_string()]),
                            },
                        );
                        props
                    },
                    required: vec!["format".to_string()],
                },
            },
        });

        // wait_for_element tool
        registry.register(ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "wait_for_element".to_string(),
                description: "Wait for an element to appear on the page".to_string(),
                parameters: ParametersSchema {
                    schema_type: "object".to_string(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "selector".to_string(),
                            ToolParameter {
                                param_type: "string".to_string(),
                                description: "CSS selector for the element to wait for".to_string(),
                                enum_values: None,
                            },
                        );
                        props.insert(
                            "timeout_ms".to_string(),
                            ToolParameter {
                                param_type: "integer".to_string(),
                                description: "Maximum time to wait in milliseconds (default: 5000)"
                                    .to_string(),
                                enum_values: None,
                            },
                        );
                        props
                    },
                    required: vec!["selector".to_string()],
                },
            },
        });

        // get_current_url tool
        registry.register(ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "get_current_url".to_string(),
                description: "Get the current page URL".to_string(),
                parameters: ParametersSchema {
                    schema_type: "object".to_string(),
                    properties: HashMap::new(),
                    required: vec![],
                },
            },
        });

        // get_page_title tool
        registry.register(ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "get_page_title".to_string(),
                description: "Get the current page title".to_string(),
                parameters: ParametersSchema {
                    schema_type: "object".to_string(),
                    properties: HashMap::new(),
                    required: vec![],
                },
            },
        });

        // analyze_form tool
        registry.register(ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "analyze_form".to_string(),
                description: "Analyze forms on the current page and return detailed information about fields, types, and purposes".to_string(),
                parameters: ParametersSchema {
                    schema_type: "object".to_string(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "form_index".to_string(),
                            ToolParameter {
                                param_type: "integer".to_string(),
                                description: "Index of the form to analyze (0-based, optional - analyzes all if not specified)".to_string(),
                                enum_values: None,
                            },
                        );
                        props
                    },
                    required: vec![],
                },
            },
        });

        // auto_fill_form tool
        registry.register(ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "auto_fill_form".to_string(),
                description: "Automatically fill an entire form using semantic field matching".to_string(),
                parameters: ParametersSchema {
                    schema_type: "object".to_string(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "form_data".to_string(),
                            ToolParameter {
                                param_type: "object".to_string(),
                                description: "Key-value pairs of field names/hints and their values to fill".to_string(),
                                enum_values: None,
                            },
                        );
                        props.insert(
                            "form_index".to_string(),
                            ToolParameter {
                                param_type: "integer".to_string(),
                                description: "Index of the form to fill (0-based, optional - uses first form if not specified)".to_string(),
                                enum_values: None,
                            },
                        );
                        props
                    },
                    required: vec!["form_data".to_string()],
                },
            },
        });

        // submit_form tool
        registry.register(ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "submit_form".to_string(),
                description: "Submit a form by clicking its submit button or using form submission".to_string(),
                parameters: ParametersSchema {
                    schema_type: "object".to_string(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "form_selector".to_string(),
                            ToolParameter {
                                param_type: "string".to_string(),
                                description: "CSS selector for the form element or submit button (optional - auto-detects if not specified)".to_string(),
                                enum_values: None,
                            },
                        );
                        props.insert(
                            "form_index".to_string(),
                            ToolParameter {
                                param_type: "integer".to_string(),
                                description: "Index of the form to submit (0-based, optional - uses first form if not specified)".to_string(),
                                enum_values: None,
                            },
                        );
                        props
                    },
                    required: vec![],
                },
            },
        });

        // get_form_fields tool
        registry.register(ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "get_form_fields".to_string(),
                description: "Get a list of all form fields on the current page with their metadata".to_string(),
                parameters: ParametersSchema {
                    schema_type: "object".to_string(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "form_index".to_string(),
                            ToolParameter {
                                param_type: "integer".to_string(),
                                description: "Index of the form to analyze (0-based, optional - analyzes all forms if not specified)".to_string(),
                                enum_values: None,
                            },
                        );
                        props
                    },
                    required: vec![],
                },
            },
        });

        // extract_data tool
        registry.register(ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "extract_data".to_string(),
                description: "Extract structured data from the current page using CSS selectors"
                    .to_string(),
                parameters: ParametersSchema {
                    schema_type: "object".to_string(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "selectors".to_string(),
                            ToolParameter {
                                param_type: "object".to_string(),
                                description: "Key-value pairs of field names and CSS selectors"
                                    .to_string(),
                                enum_values: None,
                            },
                        );
                        props
                    },
                    required: vec!["selectors".to_string()],
                },
            },
        });

        // query_kg tool
        registry.register(ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "query_kg".to_string(),
                description: "Execute a SPARQL query against the Knowledge Graph".to_string(),
                parameters: ParametersSchema {
                    schema_type: "object".to_string(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "query".to_string(),
                            ToolParameter {
                                param_type: "string".to_string(),
                                description:
                                    "SPARQL query string (SELECT, ASK, CONSTRUCT, DESCRIBE)"
                                        .to_string(),
                                enum_values: None,
                            },
                        );
                        props
                    },
                    required: vec!["query".to_string()],
                },
            },
        });

        // predict_link tool
        registry.register(ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "predict_link".to_string(),
                description: "Predict missing links in the Knowledge Graph using ML embeddings".to_string(),
                parameters: ParametersSchema {
                    schema_type: "object".to_string(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "prediction_type".to_string(),
                            ToolParameter {
                                param_type: "string".to_string(),
                                description: "Type of prediction: 'tail' (h,r,?t), 'head' (?,r,t), 'relation' (h,?,t)".to_string(),
                                enum_values: Some(vec!["tail".to_string(), "head".to_string(), "relation".to_string()]),
                            },
                        );
                        props.insert(
                            "head".to_string(),
                            ToolParameter {
                                param_type: "string".to_string(),
                                description: "Head entity URI (required for tail and relation prediction)".to_string(),
                                enum_values: None,
                            },
                        );
                        props.insert(
                            "relation".to_string(),
                            ToolParameter {
                                param_type: "string".to_string(),
                                description: "Relation URI (required for head and tail prediction)".to_string(),
                                enum_values: None,
                            },
                        );
                        props.insert(
                            "tail".to_string(),
                            ToolParameter {
                                param_type: "string".to_string(),
                                description: "Tail entity URI (required for head and relation prediction)".to_string(),
                                enum_values: None,
                            },
                        );
                        props.insert(
                            "k".to_string(),
                            ToolParameter {
                                param_type: "integer".to_string(),
                                description: "Number of top predictions to return (default: 5)".to_string(),
                                enum_values: None,
                            },
                        );
                        props
                    },
                    required: vec!["prediction_type".to_string()],
                },
            },
        });

        // store_memory tool
        registry.register(ToolDefinition {
            tool_type: "function".to_string(),
            function: FunctionDefinition {
                name: "store_memory".to_string(),
                description: "Store information in the Knowledge Graph for future reference"
                    .to_string(),
                parameters: ParametersSchema {
                    schema_type: "object".to_string(),
                    properties: {
                        let mut props = HashMap::new();
                        props.insert(
                            "subject".to_string(),
                            ToolParameter {
                                param_type: "string".to_string(),
                                description: "Subject URI for the triple".to_string(),
                                enum_values: None,
                            },
                        );
                        props.insert(
                            "predicate".to_string(),
                            ToolParameter {
                                param_type: "string".to_string(),
                                description: "Predicate URI for the triple".to_string(),
                                enum_values: None,
                            },
                        );
                        props.insert(
                            "object".to_string(),
                            ToolParameter {
                                param_type: "string".to_string(),
                                description: "Object URI or literal for the triple".to_string(),
                                enum_values: None,
                            },
                        );
                        props
                    },
                    required: vec![
                        "subject".to_string(),
                        "predicate".to_string(),
                        "object".to_string(),
                    ],
                },
            },
        });

        registry
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_creation() {
        let registry = ToolRegistry::with_browser_tools();
        let tools = registry.get_tools_json();

        assert!(!tools.is_empty());
        assert!(registry.get_tool("navigate_to").is_some());
        assert!(registry.get_tool("fill_form_field").is_some());
        assert!(registry.get_tool("click_element").is_some());
    }

    #[test]
    fn test_tool_serialization() {
        let registry = ToolRegistry::with_browser_tools();
        let tools = registry.get_tools_json();

        // Ensure tools can be serialized to JSON
        let json_str = serde_json::to_string(&tools).unwrap();
        assert!(!json_str.is_empty());
    }
}
