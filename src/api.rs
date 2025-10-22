// Agent API module

use crate::kg_integration::insert_snapshot_to_kg;
use crate::models::SemanticSnapshot;
use axum::http::HeaderMap;
use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    routing::{get, post},
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

/// Request for LangGraph workflow execution
#[derive(serde::Deserialize)]
pub struct LangGraphRequest {
    pub graph_definition: String,
    pub input: String,
}

/// Response with parsed data
#[derive(serde::Serialize)]
pub struct ParseResponse {
    pub title: Option<String>,
    pub entities: Vec<String>,
}

/// Response for LangGraph workflow execution
#[derive(serde::Serialize)]
pub struct LangGraphResponse {
    pub result: String,
    pub workflow_state: serde_json::Value,
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

/// Response listing KG items
#[derive(serde::Serialize)]
pub struct GraphItemsResponse {
    pub items: Vec<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<SemanticSnapshot>,
}

/// Browse and insert into KG request (2025 best practice)
#[derive(Debug, serde::Deserialize)]
pub struct BrowseKGRequest {
    pub url: String,
}

/// Browse and insert into KG response
#[derive(serde::Serialize)]
pub struct BrowseKGResponse {
    pub data: String,
    pub triples_inserted: usize,
    pub final_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<SemanticSnapshot>,
}

/// Token generation request
#[derive(serde::Deserialize)]
pub struct TokenRequest {
    pub username: String,
    #[serde(default)]
    pub role: Option<String>,
}

/// Token response
#[derive(serde::Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub expires_in: i64,
}

/// Token revocation request
#[derive(serde::Deserialize)]
pub struct RevokeTokenRequest {
    pub token: String,
}

/// Token revocation response
#[derive(serde::Serialize)]
pub struct RevokeTokenResponse {
    pub revoked: bool,
    pub message: String,
}

/// Start the agent API server
pub async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize KG with persistence if KG_PERSIST_PATH is set
    #[allow(clippy::disallowed_methods)]
    let kg = if let Ok(persist_path) = std::env::var("KG_PERSIST_PATH") {
        tracing::info!("Initializing Knowledge Graph with persistence at: {}", persist_path);
        crate::kg::KnowledgeGraph::with_persistence(std::path::Path::new(&persist_path))?
    } else {
        tracing::info!("Initializing in-memory Knowledge Graph");
        crate::kg::KnowledgeGraph::new()
    };

    // Initialize Redis token revocation store if feature is enabled
    #[cfg(feature = "redis-integration")]
    {
        #[allow(clippy::disallowed_methods)]
        if let Ok(redis_url) = std::env::var("REDIS_URL") {
            tracing::info!("Initializing Redis token revocation store");
            match crate::auth::TokenRevocationStore::new(&redis_url).await {
                Ok(store) => {
                    crate::auth::TokenRevocationStore::init_global(store);
                    tracing::info!("Redis token revocation store initialized successfully");
                }
                Err(e) => {
                    tracing::error!("Failed to initialize Redis token revocation store: {}", e);
                    tracing::warn!("Token revocation will be disabled");
                }
            }
        } else {
            tracing::warn!("REDIS_URL not set - token revocation disabled");
        }
    }

    let state = AppState {
        kg: Arc::new(Mutex::new(kg)),
        rate_limits: Arc::new(Mutex::new(HashMap::new())),
    };
    let app = {
        let router = Router::new()
            .route("/parse", post(parse_html))
            .route("/query", post(query_kg))
            .route("/browse", post(browse_url))
            .route("/langgraph", post(run_langgraph));
        #[cfg(feature = "browser-automation")]
        let router = router.route("/browse_kg", post(browse_url_kg));
        router
            .route("/kg/entities", get(list_entities))
            .route("/kg/relations", get(list_relations))
            .route("/auth/token", post(generate_token_endpoint))
            .route("/auth/revoke", post(revoke_token_endpoint))
            .route("/metrics", get(metrics_endpoint))
            .with_state(state)
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Server running on http://{}", addr);
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}

/// Handler for revoking JWT tokens
#[axum::debug_handler]
#[tracing::instrument(skip(req), fields(token_prefix = %req.token.chars().take(10).collect::<String>()))]
async fn revoke_token_endpoint(
    _user: crate::auth::AuthenticatedUser, // Require authentication to revoke tokens
    Json(req): Json<RevokeTokenRequest>,
) -> Result<Json<RevokeTokenResponse>, (StatusCode, String)> {
    #[cfg(not(feature = "redis-integration"))]
    {
        return Err((
            StatusCode::NOT_IMPLEMENTED,
            "Token revocation requires Redis integration (enable redis-integration feature)"
                .to_string(),
        ));
    }

    #[cfg(feature = "redis-integration")]
    {
        let store = match crate::auth::TokenRevocationStore::get() {
            Some(store) => store,
            None => {
                return Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Token revocation store not initialized".to_string(),
                ));
            }
        };

        // First validate the token to get its expiration
        let claims = match crate::auth::validate_token_async(&req.token).await {
            Ok(claims) => claims,
            Err(_) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Invalid token provided for revocation".to_string(),
                ));
            }
        };

        // Revoke the token
        match store.revoke_token(&req.token, claims.exp).await {
            Ok(_) => {
                crate::security::log_action(
                    "revoke_token",
                    &format!("Revoked token for user: {}", claims.sub),
                );
                Ok(Json(RevokeTokenResponse {
                    revoked: true,
                    message: "Token successfully revoked".to_string(),
                }))
            }
            Err(e) => {
                tracing::error!("Failed to revoke token: {}", e);
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to revoke token".to_string()))
            }
        }
    }
}

/// Execute LangGraph workflow
async fn run_langgraph(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    _user: crate::auth::AuthenticatedUser,
    Json(req): Json<LangGraphRequest>,
) -> Json<LangGraphResponse> {
    // Authentication handled by AuthenticatedUser extractor

    // Check rate limit - extract real IP
    let ip = extract_ip(&headers, &addr);
    tracing::debug!("Processing LangGraph request from IP: {} for input: {}", ip, req.input);
    {
        let mut rate_limits = state.rate_limits.lock().await;
        if !check_rate_limit(&mut rate_limits, &ip) {
            crate::security::log_action("langgraph", &format!("Rate limit exceeded for {}", ip));
            return Json(LangGraphResponse {
                result: "Rate limit exceeded".to_string(),
                workflow_state: serde_json::json!({"error": "rate_limit_exceeded"}),
            });
        }
    }

    // Execute LangGraph workflow
    // Create a new KG instance for the workflow (we'll need to refactor this for shared access)
    let kg_for_workflow = crate::kg::KnowledgeGraph::new();
    let kg_arc = std::sync::Arc::new(tokio::sync::Mutex::new(kg_for_workflow));
    match crate::external::run_langgraph_workflow(&req.graph_definition, &req.input, kg_arc).await {
        Ok(result) => {
            crate::security::log_action(
                "langgraph",
                &format!("Workflow completed successfully for input: {}", req.input),
            );
            Json(LangGraphResponse {
                result,
                workflow_state: serde_json::json!({"status": "completed"}),
            })
        }
        Err(e) => {
            crate::security::log_action("langgraph", &format!("Workflow failed: {}", e));
            Json(LangGraphResponse {
                result: format!("Workflow execution failed: {}", e),
                workflow_state: serde_json::json!({"error": e.to_string()}),
            })
        }
    }
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

// Authentication is now handled by AuthenticatedUser extractor (src/auth.rs)
// This provides JWT-based authentication with configurable secrets

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
#[tracing::instrument(skip(state, addr, headers, _user, req), fields(html_size = req.html.len()))]
async fn parse_html(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    _user: crate::auth::AuthenticatedUser,
    Json(req): Json<ParseRequest>,
) -> Json<ParseResponse> {
    let start_time = Instant::now();

    // Authentication handled by AuthenticatedUser extractor

    // Check rate limit - extract real IP
    let ip = extract_ip(&headers, &addr);
    tracing::debug!("Processing parse request from IP: {}", ip);
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
        return Json(ParseResponse { title: None, entities: vec![e.to_string()] });
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
            tracing::debug!("Parse duration: {:?}", start_time.elapsed());
            Json(ParseResponse { title: data.title, entities })
        }
        Err(e) => {
            crate::security::log_action("parse_html", &format!("Parse error: {}", e));
            tracing::debug!("Parse duration: {:?}", start_time.elapsed());
            Json(ParseResponse { title: None, entities: vec![] })
        }
    }
}

/// Handler for querying KG
#[axum::debug_handler]
#[tracing::instrument(skip(state, addr, headers, _user))]
async fn list_entities(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    _user: crate::auth::AuthenticatedUser,
) -> Json<GraphItemsResponse> {
    let ip = extract_ip(&headers, &addr);
    tracing::debug!("Listing entities for IP: {}", ip);
    {
        let mut rate_limits = state.rate_limits.lock().await;
        if !check_rate_limit(&mut rate_limits, &ip) {
            crate::security::log_action(
                "list_entities",
                &format!("Rate limit exceeded for {}", ip),
            );
            return Json(GraphItemsResponse { items: vec!["Rate limit exceeded".to_string()] });
        }
    }

    let kg = state.kg.lock().await;
    match kg.get_all_entities() {
        Ok(items) => {
            crate::security::log_action(
                "list_entities",
                &format!("Returned {} entities", items.len()),
            );
            Json(GraphItemsResponse { items })
        }
        Err(e) => {
            crate::security::log_action("list_entities", &format!("KG error: {}", e));
            Json(GraphItemsResponse { items: vec![format!("Error retrieving entities: {}", e)] })
        }
    }
}

#[axum::debug_handler]
#[tracing::instrument(skip(state, addr, headers, _user))]
async fn list_relations(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    _user: crate::auth::AuthenticatedUser,
) -> Json<GraphItemsResponse> {
    let ip = extract_ip(&headers, &addr);
    tracing::debug!("Listing relations for IP: {}", ip);
    {
        let mut rate_limits = state.rate_limits.lock().await;
        if !check_rate_limit(&mut rate_limits, &ip) {
            crate::security::log_action(
                "list_relations",
                &format!("Rate limit exceeded for {}", ip),
            );
            return Json(GraphItemsResponse { items: vec!["Rate limit exceeded".to_string()] });
        }
    }

    let kg = state.kg.lock().await;
    match kg.get_all_relations() {
        Ok(items) => {
            crate::security::log_action(
                "list_relations",
                &format!("Returned {} relations", items.len()),
            );
            Json(GraphItemsResponse { items })
        }
        Err(e) => {
            crate::security::log_action("list_relations", &format!("KG error: {}", e));
            Json(GraphItemsResponse { items: vec![format!("Error retrieving relations: {}", e)] })
        }
    }
}

/// Handler for querying KG
#[axum::debug_handler]
#[tracing::instrument(skip(state, addr, headers, _user, req), fields(query_length = req.query.len()))]
async fn query_kg(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    _user: crate::auth::AuthenticatedUser,
    Json(req): Json<QueryRequest>,
) -> Json<QueryResponse> {
    // Authentication handled by AuthenticatedUser extractor

    // Check rate limit - extract real IP
    let ip = extract_ip(&headers, &addr);
    tracing::debug!("Processing KG query from IP: {}", ip);
    {
        let mut rate_limits = state.rate_limits.lock().await;
        if !check_rate_limit(&mut rate_limits, &ip) {
            crate::security::log_action("query_kg", &format!("Rate limit exceeded for {}", ip));
            return Json(QueryResponse { results: vec!["Rate limit exceeded".to_string()] });
        }
    }

    // Validate query
    if let Err(e) = crate::security::validate_sparql_query(&req.query) {
        crate::security::log_action("query_kg", &format!("Validation failed: {}", e));
        return Json(QueryResponse { results: vec![e.to_string()] });
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
                Json(QueryResponse { results: vec!["Update successful".to_string()] })
            }
            Err(e) => {
                crate::security::log_action("query_kg", &format!("Update error: {}", e));
                Json(QueryResponse { results: vec![format!("Update error: {}", e)] })
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
                Json(QueryResponse { results: vec![format!("Query error: {}", e)] })
            }
        }
    }
}

/// Handler for browsing with external tools
#[axum::debug_handler]
#[tracing::instrument(skip(state, addr, headers, _user, req), fields(url = %req.url))]
async fn browse_url(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    _user: crate::auth::AuthenticatedUser,
    Json(req): Json<BrowseRequest>,
) -> Json<BrowseResponse> {
    // Authentication handled by AuthenticatedUser extractor

    // Check rate limit - extract real IP
    let ip = extract_ip(&headers, &addr);
    tracing::debug!("Processing browse request from IP: {} for URL: {}", ip, req.url);
    {
        let mut rate_limits = state.rate_limits.lock().await;
        if !check_rate_limit(&mut rate_limits, &ip) {
            crate::security::log_action("browse_url", &format!("Rate limit exceeded for {}", ip));
            return Json(BrowseResponse {
                data: "Rate limit exceeded".to_string(),
                snapshot: None,
            });
        }
    }

    // Basic URL validation
    if !req.url.starts_with("http") {
        crate::security::log_action("browse_url", "Invalid URL");
        return Json(BrowseResponse { data: "Invalid URL".to_string(), snapshot: None });
    }

    // Use smart browse: chromiumoxide → HTTP fallback (best practice 2025)
    let browse_result = crate::external::browse_with_best_available(&req.url, &req.query).await;

    match browse_result {
        Ok(outcome) => {
            // Insert structured snapshot into KG
            {
                let mut kg = state.kg.lock().await;
                if let Err(err) =
                    insert_snapshot_to_kg(&outcome.snapshot, &mut kg, &req.url, Some(&req.query))
                {
                    tracing::debug!("Failed to persist snapshot into KG: {}", err);
                }
            }

            crate::security::log_action("browse_url", &format!("Browsed {} successfully", req.url));
            Json(BrowseResponse { data: outcome.summary.clone(), snapshot: Some(outcome.snapshot) })
        }
        Err(e) => {
            crate::security::log_action("browse_url", &format!("Browse error: {}", e));
            Json(BrowseResponse { data: format!("Error: {}", e), snapshot: None })
        }
    }
}

/// Handler for browsing URL and inserting into Knowledge Graph (2025 best practice)
///
/// This endpoint combines web browsing with Knowledge Graph population:
/// - Extracts semantic metadata (Phase 1: meta tags, Open Graph, Twitter Cards)
/// - Inserts RDF triples into the Knowledge Graph
/// - Returns extracted data and triple count
/// - Enables SPARQL queries on browsed content
///
/// # Example Request
/// ```json
/// {
///   "url": "https://example.com"
/// }
/// ```
///
/// # Example Response
/// ```json
/// {
///   "data": "Browsed https://example.com...",
///   "triples_inserted": 15,
///   "final_url": "https://example.com"
/// }
/// ```
#[cfg(feature = "browser-automation")]
#[axum::debug_handler]
#[tracing::instrument(skip(state, headers), fields(url = %req.url))]
async fn browse_url_kg(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    _user: crate::auth::AuthenticatedUser,
    Json(req): Json<BrowseKGRequest>,
) -> Json<BrowseKGResponse> {
    // Authentication handled by AuthenticatedUser extractor

    // Check rate limit - extract real IP
    let ip = extract_ip(&headers, &addr);
    tracing::debug!("Processing browse_kg request from IP: {} for URL: {}", ip, req.url);
    {
        let mut rate_limits = state.rate_limits.lock().await;
        if !check_rate_limit(&mut rate_limits, &ip) {
            crate::security::log_action(
                "browse_url_kg",
                &format!("Rate limit exceeded for {}", ip),
            );
            return Json(BrowseKGResponse {
                data: "Rate limit exceeded".to_string(),
                triples_inserted: 0,
                final_url: req.url.clone(),
                snapshot: None,
            });
        }
    }

    // Basic URL validation
    if !req.url.starts_with("http") {
        crate::security::log_action("browse_url_kg", "Invalid URL");
        return Json(BrowseKGResponse {
            data: "Invalid URL".to_string(),
            triples_inserted: 0,
            final_url: req.url.clone(),
            snapshot: None,
        });
    }

    // Browse and insert into KG
    let mut kg = state.kg.lock().await;
    let options = crate::browser::NavigationOptions::default();

    let browse_result = crate::external::browse_and_insert_kg(&req.url, options, &mut kg).await;

    match browse_result {
        Ok((semantic_data, count)) => {
            let snapshot = crate::kg_integration::semantic_data_to_snapshot(&semantic_data);
            crate::security::log_action(
                "browse_url_kg",
                &format!("Browsed {} and inserted {} triples", req.url, count),
            );

            // Format response data similar to browse_url
            let mut data = format!("Browsed {} and inserted into KG\n", req.url);
            if let Some(title) = &snapshot.title {
                data.push_str(&format!("Title: {}\n", title));
            }
            if let Some(desc) = &snapshot.description {
                data.push_str(&format!("Description: {}\n", desc));
            }
            data.push_str(&format!("JSON-LD objects: {}\n", snapshot.json_ld_count));
            data.push_str(&format!("Microdata items: {}\n", snapshot.microdata.len()));
            data.push_str(&format!("Triples inserted: {}\n", count));

            Json(BrowseKGResponse {
                data,
                triples_inserted: count,
                final_url: snapshot.final_url.clone(),
                snapshot: Some(snapshot),
            })
        }
        Err(e) => {
            crate::security::log_action("browse_url_kg", &format!("Browse error: {}", e));
            Json(BrowseKGResponse {
                data: format!("Error browsing and inserting into KG: {}", e),
                triples_inserted: 0,
                final_url: req.url.clone(),
                snapshot: None,
            })
        }
    }
}

/// Handler for generating JWT tokens
/// Note: In production, this should require authentication or API key
/// This is a simplified version for development/testing
#[axum::debug_handler]
#[tracing::instrument(skip(req), fields(username = %req.username))]
async fn generate_token_endpoint(
    Json(req): Json<TokenRequest>,
) -> Result<Json<TokenResponse>, (StatusCode, String)> {
    // In production, you would validate the user credentials here
    // For now, we just generate a token for any username

    if !crate::auth::JwtConfig::is_enabled() {
        tracing::warn!("JWT authentication disabled - token generation endpoint unavailable");
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "JWT authentication disabled - token generation unavailable".to_string(),
        ));
    }

    tracing::debug!("Generating token for user: {}", req.username);
    let claims = crate::auth::Claims::new(req.username.clone(), req.role);
    let token = crate::auth::generate_token(&claims).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("Token generation failed: {}", e))
    })?;

    let expires_in = claims.exp - claims.iat;

    crate::security::log_action(
        "generate_token",
        &format!("Generated token for user: {}", req.username),
    );

    Ok(Json(TokenResponse { token, expires_in }))
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

    // Authentication tests moved to src/auth.rs module

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
/// Handler for Prometheus metrics endpoint
#[axum::debug_handler]
async fn metrics_endpoint() -> Result<String, (StatusCode, String)> {
    #[cfg(feature = "observability")]
    {
        match crate::observability::get_metrics_handler() {
            Ok(metrics) => Ok(metrics),
            Err(e) => {
                tracing::error!("Failed to generate metrics: {}", e);
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate metrics".to_string()))
            }
        }
    }

    #[cfg(not(feature = "observability"))]
    {
        Err((
            StatusCode::NOT_IMPLEMENTED,
            "Observability feature not enabled. Enable with: cargo build --features observability"
                .to_string(),
        ))
    }
}
