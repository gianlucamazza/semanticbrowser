// Agent API module

use axum::http::HeaderMap;
use axum::{
    extract::{ConnectInfo, State},
    routing::post,
    Json, Router,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

/// Application state
#[derive(Clone)]
pub struct AppState {
    pub kg: Arc<Mutex<crate::kg::KnowledgeGraph>>,
    pub rate_limits: Arc<Mutex<HashMap<String, (u32, Instant)>>>,
}

/// Request to parse HTML
#[derive(serde::Deserialize)]
pub struct ParseRequest {
    pub html: String,
}

/// Response with parsed data
#[derive(serde::Serialize)]
pub struct ParseResponse {
    pub title: Option<String>,
    pub entities: Vec<String>,
}

/// Query request
#[derive(serde::Deserialize)]
pub struct QueryRequest {
    pub query: String, // Simplified, not full SPARQL
}

/// Query response
#[derive(serde::Serialize)]
pub struct QueryResponse {
    pub results: Vec<String>, // Placeholder
}

/// Browse request
#[derive(serde::Deserialize)]
pub struct BrowseRequest {
    pub url: String,
    pub query: String,
}

/// Browse response
#[derive(serde::Serialize)]
pub struct BrowseResponse {
    pub data: String,
}

/// Start the agent API server
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize KG with persistence if KG_PERSIST_PATH is set
    let kg = if let Ok(persist_path) = std::env::var("KG_PERSIST_PATH") {
        tracing::info!(
            "Initializing Knowledge Graph with persistence at: {}",
            persist_path
        );
        crate::kg::KnowledgeGraph::with_persistence(std::path::Path::new(&persist_path))?
    } else {
        tracing::info!("Initializing in-memory Knowledge Graph");
        crate::kg::KnowledgeGraph::new()
    };

    let state = AppState {
        kg: Arc::new(Mutex::new(kg)),
        rate_limits: Arc::new(Mutex::new(HashMap::new())),
    };
    let app = Router::new()
        .route("/parse", post(parse_html))
        .route("/query", post(query_kg))
        .route("/browse", post(browse_url))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Server running on http://{}", addr);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}

/// Check rate limit for IP
fn check_rate_limit(rate_limits: &mut HashMap<String, (u32, Instant)>, ip: &str) -> bool {
    let now = Instant::now();
    let entry = rate_limits.entry(ip.to_string()).or_insert((0, now));
    if now.duration_since(entry.1) > Duration::from_secs(60) {
        *entry = (1, now);
        true
    } else if entry.0 < 10 {
        // 10 requests per minute
        entry.0 += 1;
        true
    } else {
        false
    }
}

/// Check authentication
fn check_auth(headers: &HeaderMap) -> bool {
    if let Some(auth) = headers.get("authorization") {
        if let Ok(auth_str) = auth.to_str() {
            return auth_str == "Bearer secret"; // Hardcoded for demo
        }
    }
    false
}

/// Extract real IP from request, considering X-Forwarded-For and X-Real-IP headers
fn extract_ip(headers: &HeaderMap, addr: &SocketAddr) -> String {
    // First try X-Forwarded-For (proxy)
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // X-Forwarded-For can be a comma-separated list; take the first one
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }

    // Try X-Real-IP
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }

    // Fall back to connection address
    addr.ip().to_string()
}

/// Handler for parsing HTML
#[axum::debug_handler]
async fn parse_html(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<ParseRequest>,
) -> Json<ParseResponse> {
    // Check auth
    if !check_auth(&headers) {
        crate::security::log_action("parse_html", "Unauthorized access");
        return Json(ParseResponse {
            title: None,
            entities: vec!["Unauthorized".to_string()],
        });
    }

    // Check rate limit - extract real IP
    let ip = extract_ip(&headers, &addr);
    {
        let mut rate_limits = state.rate_limits.lock().await;
        if !check_rate_limit(&mut rate_limits, &ip) {
            crate::security::log_action("parse_html", &format!("Rate limit exceeded for {}", ip));
            return Json(ParseResponse {
                title: None,
                entities: vec!["Rate limit exceeded".to_string()],
            });
        }
    }

    // Validate input
    if let Err(e) = crate::security::validate_html_input(&req.html) {
        crate::security::log_action("parse_html", &format!("Validation failed: {}", e));
        return Json(ParseResponse {
            title: None,
            entities: vec![e.to_string()],
        });
    }

    // Use parser module
    let mut kg = state.kg.lock().await;
    match crate::parser::parse_html(&req.html) {
        Ok(data) => {
            // Insert basic triples to KG
            for micro in &data.microdata {
                let _ = kg.insert(&micro.item_type, "rdf:type", "schema:Thing");
            }
            let entities: Vec<String> = data.microdata.into_iter().map(|m| m.item_type).collect();
            crate::security::log_action(
                "parse_html",
                &format!("Parsed {} entities", entities.len()),
            );
            Json(ParseResponse {
                title: data.title,
                entities,
            })
        }
        Err(e) => {
            crate::security::log_action("parse_html", &format!("Parse error: {}", e));
            Json(ParseResponse {
                title: None,
                entities: vec![],
            })
        }
    }
}

/// Handler for querying KG
#[axum::debug_handler]
async fn query_kg(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<QueryRequest>,
) -> Json<QueryResponse> {
    // Check auth
    if !check_auth(&headers) {
        crate::security::log_action("query_kg", "Unauthorized access");
        return Json(QueryResponse {
            results: vec!["Unauthorized".to_string()],
        });
    }

    // Check rate limit - extract real IP
    let ip = extract_ip(&headers, &addr);
    {
        let mut rate_limits = state.rate_limits.lock().await;
        if !check_rate_limit(&mut rate_limits, &ip) {
            crate::security::log_action("query_kg", &format!("Rate limit exceeded for {}", ip));
            return Json(QueryResponse {
                results: vec!["Rate limit exceeded".to_string()],
            });
        }
    }

    // Validate query
    if let Err(e) = crate::security::validate_sparql_query(&req.query) {
        crate::security::log_action("query_kg", &format!("Validation failed: {}", e));
        return Json(QueryResponse {
            results: vec![e.to_string()],
        });
    }

    // Determine if this is a query or update operation
    let query_trimmed = req.query.trim().to_uppercase();
    let is_update = query_trimmed.starts_with("INSERT") || query_trimmed.starts_with("DELETE");

    if is_update {
        // Execute update
        let mut kg = state.kg.lock().await;
        match kg.update(&req.query) {
            Ok(()) => {
                crate::security::log_action("query_kg", "Update executed successfully");
                Json(QueryResponse {
                    results: vec!["Update successful".to_string()],
                })
            }
            Err(e) => {
                crate::security::log_action("query_kg", &format!("Update error: {}", e));
                Json(QueryResponse {
                    results: vec![format!("Update error: {}", e)],
                })
            }
        }
    } else {
        // Execute query
        let kg = state.kg.lock().await;
        match kg.query(&req.query) {
            Ok(results) => {
                crate::security::log_action(
                    "query_kg",
                    &format!("Query returned {} results", results.len()),
                );
                Json(QueryResponse { results })
            }
            Err(e) => {
                crate::security::log_action("query_kg", &format!("Query error: {}", e));
                Json(QueryResponse {
                    results: vec![format!("Query error: {}", e)],
                })
            }
        }
    }
}

/// Handler for browsing with external tools
#[axum::debug_handler]
async fn browse_url(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Json(req): Json<BrowseRequest>,
) -> Json<BrowseResponse> {
    // Check auth
    if !check_auth(&headers) {
        crate::security::log_action("browse_url", "Unauthorized access");
        return Json(BrowseResponse {
            data: "Unauthorized".to_string(),
        });
    }

    // Check rate limit - extract real IP
    let ip = extract_ip(&headers, &addr);
    {
        let mut rate_limits = state.rate_limits.lock().await;
        if !check_rate_limit(&mut rate_limits, &ip) {
            crate::security::log_action("browse_url", &format!("Rate limit exceeded for {}", ip));
            return Json(BrowseResponse {
                data: "Rate limit exceeded".to_string(),
            });
        }
    }

    // Basic URL validation
    if !req.url.starts_with("http") {
        crate::security::log_action("browse_url", "Invalid URL");
        return Json(BrowseResponse {
            data: "Invalid URL".to_string(),
        });
    }

    // Try PyO3-based browser-use first, then fall back to HTTP
    let mut browse_result =
        crate::external::browse_with_python_browser_use(&req.url, &req.query).await;

    if browse_result.is_err() {
        tracing::debug!("PyO3 browse failed, falling back to HTTP");
        browse_result = crate::external::browse_with_browser_use(&req.url, &req.query).await;
    }

    match browse_result {
        Ok(data) => {
            // Optionally insert to KG, e.g., add triple for the URL
            let mut kg = state.kg.lock().await;
            let _ = kg.insert(&req.url, "browsed", &data);
            crate::security::log_action("browse_url", &format!("Browsed {} successfully", req.url));
            Json(BrowseResponse { data })
        }
        Err(e) => {
            crate::security::log_action("browse_url", &format!("Browse error: {}", e));
            Json(BrowseResponse {
                data: format!("Error browsing: {}", e),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_logic() {
        let mut rate_limits = HashMap::new();
        let ip = "127.0.0.1";

        // First 10 requests should succeed
        for _ in 0..10 {
            assert!(check_rate_limit(&mut rate_limits, ip));
        }

        // 11th request should fail
        assert!(!check_rate_limit(&mut rate_limits, ip));
    }

    #[test]
    fn test_check_auth() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer secret".parse().unwrap());
        assert!(check_auth(&headers));

        let mut bad_headers = HeaderMap::new();
        bad_headers.insert("authorization", "Bearer wrong".parse().unwrap());
        assert!(!check_auth(&bad_headers));
    }

    #[test]
    fn test_extract_ip() {
        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

        // Test with X-Forwarded-For
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "192.168.1.1".parse().unwrap());
        assert_eq!(extract_ip(&headers, &addr), "192.168.1.1");

        // Test with X-Real-IP
        let mut headers2 = HeaderMap::new();
        headers2.insert("x-real-ip", "10.0.0.1".parse().unwrap());
        assert_eq!(extract_ip(&headers2, &addr), "10.0.0.1");

        // Test fallback to connection address
        let headers3 = HeaderMap::new();
        assert_eq!(extract_ip(&headers3, &addr), "127.0.0.1");
    }
}
