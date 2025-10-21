use std::collections::HashMap;
use std::sync::Arc;

use semantic_browser::kg::KnowledgeGraph;
use semantic_browser::kg_integration::insert_snapshot_to_kg;
use semantic_browser::security;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::sync::Mutex;

const JSON_RPC_VERSION: &str = "2.0";
const MCP_PROTOCOL_VERSION: &str = "2025-06-18";
const SERVER_NAME: &str = "semantic-browser-mcp";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let state = ServerState::new()?;
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let mut reader = BufReader::new(stdin).lines();
    let mut writer = BufWriter::new(stdout);
    let mut initialized = false;

    while let Some(line) = reader.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let message: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(err) => {
                send_error(
                    &mut writer,
                    None,
                    JsonRpcError::new(
                        -32700,
                        "Parse error",
                        Some(json!({ "details": err.to_string() })),
                    ),
                )
                .await?;
                continue;
            }
        };

        match extract_method(&message) {
            Some((method, id, params)) => {
                if let Some(request_id) = id {
                    let response = match method {
                        "initialize" => match handle_initialize(params, &mut initialized) {
                            Ok(result) => Response::Success { id: request_id, result },
                            Err(err) => Response::Error { id: Some(request_id), error: err },
                        },
                        "ping" => Response::Success { id: request_id, result: json!({}) },
                        "tools/list" => {
                            if !initialized {
                                Response::Error {
                                    id: Some(request_id),
                                    error: JsonRpcError::new(
                                        -32002,
                                        "Server not initialized",
                                        None,
                                    ),
                                }
                            } else {
                                Response::Success { id: request_id, result: list_tools_result() }
                            }
                        }
                        "tools/call" => {
                            if !initialized {
                                Response::Error {
                                    id: Some(request_id),
                                    error: JsonRpcError::new(
                                        -32002,
                                        "Server not initialized",
                                        None,
                                    ),
                                }
                            } else {
                                match handle_call_tool(params, &state).await {
                                    Ok(result) => Response::Success { id: request_id, result },
                                    Err(err) => {
                                        Response::Error { id: Some(request_id), error: err }
                                    }
                                }
                            }
                        }
                        _ => Response::Error {
                            id: Some(request_id),
                            error: JsonRpcError::new(
                                -32601,
                                "Method not found",
                                Some(json!({ "method": method })),
                            ),
                        },
                    };
                    response.write(&mut writer).await?;
                } else {
                    // Notification (no id)
                    handle_notification(method, params, &mut initialized);
                }
            }
            None => {
                // Ignore responses from the client or malformed messages without a method
                tracing::debug!("Ignoring message without method: {}", message);
            }
        }
    }

    writer.flush().await?;
    Ok(())
}

fn init_tracing() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .without_time()
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);
}

struct ServerState {
    kg: Arc<Mutex<KnowledgeGraph>>,
}

impl ServerState {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        #[allow(clippy::disallowed_methods)]
        let kg = if let Ok(path) = std::env::var("KG_PERSIST_PATH") {
            tracing::info!("Initializing Knowledge Graph with persistence at {}", path);
            KnowledgeGraph::with_persistence(std::path::Path::new(&path))?
        } else {
            tracing::info!("Initializing in-memory Knowledge Graph");
            KnowledgeGraph::new()
        };

        Ok(Self { kg: Arc::new(Mutex::new(kg)) })
    }
}

enum Response {
    Success {
        id: Value,
        result: Value,
    },
    Error {
        id: Option<Value>,
        error: JsonRpcError,
    },
}

impl Response {
    async fn write(self, writer: &mut BufWriter<tokio::io::Stdout>) -> io::Result<()> {
        match self {
            Response::Success { id, result } => {
                let payload = json!({
                    "jsonrpc": JSON_RPC_VERSION,
                    "id": id,
                    "result": result,
                });
                write_json(writer, &payload).await
            }
            Response::Error { id, error } => send_error(writer, id, error).await,
        }
    }
}

#[derive(Clone)]
struct JsonRpcError {
    code: i64,
    message: String,
    data: Option<Value>,
}

impl JsonRpcError {
    fn new(code: i64, message: impl Into<String>, data: Option<Value>) -> Self {
        Self { code, message: message.into(), data }
    }
}

async fn send_error(
    writer: &mut BufWriter<tokio::io::Stdout>,
    id: Option<Value>,
    error: JsonRpcError,
) -> io::Result<()> {
    let payload = json!({
        "jsonrpc": JSON_RPC_VERSION,
        "id": id.unwrap_or(Value::Null),
        "error": {
            "code": error.code,
            "message": error.message,
            "data": error.data,
        }
    });
    write_json(writer, &payload).await
}

async fn write_json(writer: &mut BufWriter<tokio::io::Stdout>, payload: &Value) -> io::Result<()> {
    let encoded = serde_json::to_string(payload).expect("serialize JSON response");
    writer.write_all(encoded.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await
}

fn extract_method(message: &Value) -> Option<(&str, Option<Value>, Option<Value>)> {
    let method = message.get("method")?.as_str()?;
    let id = message.get("id").cloned();
    let params = message.get("params").cloned();
    Some((method, id, params))
}

fn handle_initialize(params: Option<Value>, initialized: &mut bool) -> Result<Value, JsonRpcError> {
    let params = params.unwrap_or_else(|| Value::Object(Map::new()));
    let params_obj = params.as_object().ok_or_else(|| {
        JsonRpcError::new(
            -32602,
            "Invalid params for initialize",
            Some(json!({ "received": params })),
        )
    })?;

    let _protocol_version =
        params_obj.get("protocolVersion").and_then(Value::as_str).ok_or_else(|| {
            JsonRpcError::new(
                -32602,
                "Missing protocolVersion",
                Some(json!({ "received": params_obj })),
            )
        })?;

    let client_info = params_obj.get("clientInfo").and_then(Value::as_object);
    if let Some(info) = client_info {
        let name = info.get("name").and_then(Value::as_str).unwrap_or("unknown");
        let version = info.get("version").and_then(Value::as_str).unwrap_or("unknown");
        tracing::info!("Client connected: {} {}", name, version);
    }

    *initialized = true;

    let result = json!({
        "protocolVersion": MCP_PROTOCOL_VERSION,
        "capabilities": {
            "tools": {
                "listChanged": false
            }
        },
        "serverInfo": {
            "name": SERVER_NAME,
            "title": "Semantic Browser MCP",
            "version": env!("CARGO_PKG_VERSION"),
        },
        "instructions": "Provides HTML parsing, knowledge graph querying, and browsing tools backed by the Semantic Browser."
    });

    // Ensure we return a success response with the same id
    tracing::info!("Initialization complete");
    Ok(result)
}

async fn handle_call_tool(
    params: Option<Value>,
    state: &ServerState,
) -> Result<Value, JsonRpcError> {
    let argument_value = params.unwrap_or_else(|| Value::Object(Map::new()));
    let param_obj = argument_value.as_object().ok_or_else(|| {
        JsonRpcError::new(
            -32602,
            "Invalid params for tools/call",
            Some(json!({ "received": argument_value })),
        )
    })?;

    let name = param_obj.get("name").and_then(Value::as_str).ok_or_else(|| {
        JsonRpcError::new(-32602, "Missing tool name", Some(json!({ "received": param_obj })))
    })?;

    let arguments = param_obj.get("arguments");

    let result = match name {
        TOOL_PARSE_HTML => {
            let args: ParseHtmlArgs =
                parse_arguments(arguments).map_err(|msg| JsonRpcError::new(-32602, msg, None))?;
            execute_parse_html(args, state).await
        }
        TOOL_QUERY_KG => {
            let args: QueryKgArgs =
                parse_arguments(arguments).map_err(|msg| JsonRpcError::new(-32602, msg, None))?;
            execute_query_kg(args, state).await
        }
        TOOL_BROWSE_URL => {
            let args: BrowseArgs =
                parse_arguments(arguments).map_err(|msg| JsonRpcError::new(-32602, msg, None))?;
            execute_browse(args, state).await
        }
        _ => {
            return Err(JsonRpcError::new(-32601, "Unknown tool", Some(json!({ "tool": name }))));
        }
    };

    match result {
        Ok(call_result) => serde_json::to_value(call_result).map_err(|err| {
            JsonRpcError::new(
                -32603,
                "Failed to encode tool result",
                Some(json!({ "error": err.to_string() })),
            )
        }),
        Err(call_error) => serde_json::to_value(call_error).map_err(|err| {
            JsonRpcError::new(
                -32603,
                "Failed to encode tool error",
                Some(json!({ "error": err.to_string() })),
            )
        }),
    }
}

fn handle_notification(method: &str, params: Option<Value>, initialized: &mut bool) {
    match method {
        "notifications/initialized" => {
            *initialized = true;
            tracing::debug!("Client sent notifications/initialized");
        }
        "notifications/cancelled" => {
            tracing::debug!("Received cancelled notification: {:?}", params);
        }
        _ => {
            tracing::debug!("Ignoring notification: {}", method);
        }
    }
}

fn list_tools_result() -> Value {
    json!({
        "tools": [
            json!({
                "name": TOOL_PARSE_HTML,
                "description": "Parse HTML content and extract semantic annotations.",
                "annotations": {
                    "title": "Parse HTML",
                    "readOnlyHint": true,
                    "openWorldHint": false
                },
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "html": {
                            "type": "string",
                            "description": "Raw HTML content to parse."
                        }
                    },
                    "required": ["html"]
                },
                "outputSchema": {
                    "type": "object",
                    "properties": {
                        "title": { "type": ["string", "null"] },
                        "microdata": { "type": "array", "items": { "type": "object" } },
                        "jsonLd": { "type": "array", "items": { "type": "object" } }
                    }
                }
            }),
            json!({
                "name": TOOL_QUERY_KG,
                "description": "Execute read or write operations against the Semantic Browser knowledge graph.",
                "annotations": {
                    "title": "Knowledge Graph Query",
                    "readOnlyHint": false,
                    "destructiveHint": false,
                    "idempotentHint": false,
                    "openWorldHint": false
                },
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "SPARQL query or update statement."
                        }
                    },
                    "required": ["query"]
                },
                "outputSchema": {
                    "type": "object",
                    "properties": {
                        "results": { "type": "array", "items": { "type": "string" } },
                        "status": { "type": "string" }
                    }
                }
            }),
            json!({
                "name": TOOL_BROWSE_URL,
                "description": "Fetch a URL and summarize semantic signals relevant to a query.",
                "annotations": {
                    "title": "Browse URL",
                    "readOnlyHint": true,
                    "openWorldHint": true
                },
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "format": "uri",
                            "description": "Target URL to browse."
                        },
                        "query": {
                            "type": "string",
                            "description": "Optional focus or extraction instruction.",
                            "default": ""
                        }
                    },
                    "required": ["url"]
                },
                "outputSchema": {
                    "type": "object",
                    "properties": {
                        "url": { "type": "string" },
                        "summary": { "type": "string" }
                    }
                }
            })
        ]
    })
}

fn parse_arguments<T: DeserializeOwned>(value: Option<&Value>) -> Result<T, String> {
    let payload = value.cloned().unwrap_or_else(|| Value::Object(Map::new()));
    serde_json::from_value(payload).map_err(|err| err.to_string())
}

#[derive(Deserialize)]
struct ParseHtmlArgs {
    html: String,
}

#[derive(Deserialize)]
struct QueryKgArgs {
    query: String,
}

#[derive(Deserialize)]
struct BrowseArgs {
    url: String,
    #[serde(default)]
    query: String,
}

const TOOL_PARSE_HTML: &str = "semanticbrowser.parse_html";
const TOOL_QUERY_KG: &str = "semanticbrowser.query_kg";
const TOOL_BROWSE_URL: &str = "semanticbrowser.browse_url";

#[derive(Serialize)]
struct CallToolSuccess {
    content: Vec<TextContent>,
    #[serde(rename = "structuredContent", skip_serializing_if = "Option::is_none")]
    structured_content: Option<Value>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    is_error: Option<bool>,
}

#[derive(Serialize)]
struct TextContent {
    #[serde(rename = "type")]
    content_type: &'static str,
    text: String,
}

fn success_result(text: String, structured: Option<Value>) -> CallToolSuccess {
    CallToolSuccess {
        content: vec![TextContent { content_type: "text", text }],
        structured_content: structured,
        is_error: None,
    }
}

fn error_result(message: String) -> CallToolSuccess {
    CallToolSuccess {
        content: vec![TextContent { content_type: "text", text: message }],
        structured_content: None,
        is_error: Some(true),
    }
}

async fn execute_parse_html(
    args: ParseHtmlArgs,
    state: &ServerState,
) -> Result<CallToolSuccess, CallToolSuccess> {
    if let Err(err) = security::validate_html_input(&args.html) {
        security::log_action("mcp.parse_html", "validation_failed");
        return Err(error_result(format!("HTML validation failed: {}", err)));
    }

    let parsed = security::sandbox_parsing(|| semantic_browser::parser::parse_html(&args.html));
    let data = match parsed {
        Ok(data) => data,
        Err(err) => {
            security::log_action("mcp.parse_html", "parse_error");
            return Err(error_result(format!("Parse error: {}", err)));
        }
    };

    {
        let mut kg = state.kg.lock().await;
        for item in &data.microdata {
            if let Err(err) = kg.insert(&item.item_type, "rdf:type", "schema:Thing") {
                tracing::debug!("Failed to insert microdata into KG: {}", err);
            }
        }
    }

    security::log_action(
        "mcp.parse_html",
        &format!("parsed_title={:?} microdata={}", data.title, data.microdata.len()),
    );

    let microdata: Vec<Value> = data
        .microdata
        .iter()
        .map(|item| {
            let properties: HashMap<_, _> =
                item.properties.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
            json!({
                "itemType": item.item_type,
                "properties": properties,
            })
        })
        .collect();

    let structured = json!({
        "title": data.title,
        "microdata": microdata,
        "jsonLd": data.json_ld,
    });

    let summary = format!(
        "Parsed HTML. Title: {}. Microdata items: {}. JSON-LD blocks: {}.",
        data.title.clone().unwrap_or_else(|| "n/a".to_string()),
        data.microdata.len(),
        data.json_ld.len()
    );

    Ok(success_result(summary, Some(structured)))
}

async fn execute_query_kg(
    args: QueryKgArgs,
    state: &ServerState,
) -> Result<CallToolSuccess, CallToolSuccess> {
    if let Err(err) = security::validate_sparql_query(&args.query) {
        security::log_action("mcp.query_kg", "validation_failed");
        return Err(error_result(format!("SPARQL validation failed: {}", err)));
    }

    let trimmed = args.query.trim().to_uppercase();
    let is_update = trimmed.starts_with("INSERT") || trimmed.starts_with("DELETE");

    if is_update {
        let mut kg = state.kg.lock().await;
        match kg.update(&args.query) {
            Ok(()) => {
                security::log_action("mcp.query_kg", "update_success");
                Ok(success_result(
                    "Knowledge graph update completed.".to_string(),
                    Some(json!({ "status": "updated" })),
                ))
            }
            Err(err) => {
                security::log_action("mcp.query_kg", "update_error");
                Err(error_result(format!("Update error: {}", err)))
            }
        }
    } else {
        let kg = state.kg.lock().await;
        match kg.query(&args.query) {
            Ok(results) => {
                security::log_action(
                    "mcp.query_kg",
                    &format!("query_success results={}", results.len()),
                );
                let summary = if results.is_empty() {
                    "Query returned no results.".to_string()
                } else {
                    format!("Query returned {} results.", results.len())
                };
                Ok(success_result(summary, Some(json!({ "results": results }))))
            }
            Err(err) => {
                security::log_action("mcp.query_kg", "query_error");
                Err(error_result(format!("Query error: {}", err)))
            }
        }
    }
}

async fn execute_browse(
    args: BrowseArgs,
    state: &ServerState,
) -> Result<CallToolSuccess, CallToolSuccess> {
    if !args.url.starts_with("http://") && !args.url.starts_with("https://") {
        security::log_action("mcp.browse_url", "invalid_url");
        return Err(error_result("URL must start with http:// or https://".to_string()));
    }

    match semantic_browser::external::browse_with_best_available(&args.url, &args.query).await {
        Ok(outcome) => {
            security::log_action("mcp.browse_url", "browse_success");
            {
                let mut kg = state.kg.lock().await;
                if let Err(err) =
                    insert_snapshot_to_kg(&outcome.snapshot, &mut kg, &args.url, Some(&args.query))
                {
                    tracing::debug!("Failed to persist browsing snapshot into KG: {}", err);
                }
            }
            let summary_len = outcome.summary.len();
            Ok(success_result(
                format!("Browsed {}. Summary length: {} characters.", args.url, summary_len),
                Some(json!({
                    "url": args.url,
                    "query": args.query,
                    "summary": outcome.summary,
                    "snapshot": outcome.snapshot
                })),
            ))
        }
        Err(err) => {
            security::log_action("mcp.browse_url", "browse_error");
            Err(error_result(format!("Browsing error: {}", err)))
        }
    }
}
