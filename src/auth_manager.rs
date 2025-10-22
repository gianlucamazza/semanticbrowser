//! Authentication Manager Module
//!
//! Provides high-level abstractions for managing authentication sessions across web services.
//! Supports form-based login, OAuth2 flows, token management, and session persistence.
//!
//! # Features
//! - Form-based authentication
//! - OAuth2 authorization code flow
//! - Session data management (cookies, tokens)
//! - Token refresh logic
//! - Session persistence to disk
//!
//! # Best Practices 2025
//! - Secure credential handling
//! - Async-first design
//! - Type-safe configuration
//! - Comprehensive error types

#[cfg(feature = "browser-automation")]
use chromiumoxide::Page;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Session data containing authentication tokens and cookies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    /// HTTP cookies (name -> value)
    pub cookies: HashMap<String, String>,
    /// Authentication tokens (type -> token)
    pub tokens: HashMap<String, String>,
    /// Session expiration time (Unix timestamp)
    pub expires_at: Option<u64>,
    /// Session metadata
    pub metadata: HashMap<String, String>,
}

impl SessionData {
    /// Create new empty session
    pub fn new() -> Self {
        Self {
            cookies: HashMap::new(),
            tokens: HashMap::new(),
            expires_at: None,
            metadata: HashMap::new(),
        }
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs();
            now >= expires_at
        } else {
            false
        }
    }

    /// Set expiration time from duration
    pub fn set_expiration(&mut self, duration: Duration) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        self.expires_at = Some(now + duration.as_secs());
    }

    /// Add bearer token
    pub fn with_bearer_token(mut self, token: &str) -> Self {
        self.tokens.insert("bearer".to_string(), token.to_string());
        self
    }

    /// Add cookie
    pub fn with_cookie(mut self, name: &str, value: &str) -> Self {
        self.cookies.insert(name.to_string(), value.to_string());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

impl Default for SessionData {
    fn default() -> Self {
        Self::new()
    }
}

/// Form-based login configuration
#[derive(Debug, Clone)]
pub struct FormLoginConfig {
    /// URL of login page
    pub login_url: String,
    /// CSS selector for username field
    pub username_selector: String,
    /// CSS selector for password field
    pub password_selector: String,
    /// CSS selector for submit button
    pub submit_selector: String,
    /// Optional: selector to wait for after login
    pub success_selector: Option<String>,
    /// Optional: selector indicating login failure
    pub error_selector: Option<String>,
}

/// OAuth2 configuration
#[derive(Debug, Clone)]
pub struct OAuth2Config {
    /// Provider name (e.g., "github", "google")
    pub provider: String,
    /// Client ID
    pub client_id: String,
    /// Client secret
    pub client_secret: String,
    /// Authorization endpoint
    pub auth_endpoint: String,
    /// Token endpoint
    pub token_endpoint: String,
    /// Redirect URI
    pub redirect_uri: String,
    /// Scopes to request
    pub scopes: Vec<String>,
    /// Optional: selector for consent button
    pub consent_button_selector: Option<String>,
}

/// OAuth2 token response
#[derive(Debug, Deserialize)]
pub struct OAuth2TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

/// Authentication manager
#[derive(Debug)]
pub struct AuthenticationManager {
    /// Active sessions by identifier
    sessions: HashMap<String, SessionData>,
    /// Session storage path (for persistence)
    storage_path: Option<std::path::PathBuf>,
}

impl AuthenticationManager {
    /// Create new authentication manager
    pub fn new() -> Self {
        Self { sessions: HashMap::new(), storage_path: None }
    }

    /// Create with session persistence
    pub fn with_persistence(storage_path: std::path::PathBuf) -> Self {
        Self { sessions: HashMap::new(), storage_path: Some(storage_path) }
    }

    /// Store session
    pub fn store_session(&mut self, id: &str, session: SessionData) {
        self.sessions.insert(id.to_string(), session);

        // Persist if configured
        if let Some(ref path) = self.storage_path {
            if let Err(e) = self.save_sessions(path) {
                tracing::warn!("Failed to persist sessions: {}", e);
            }
        }
    }

    /// Get session
    pub fn get_session(&self, id: &str) -> Option<&SessionData> {
        self.sessions.get(id)
    }

    /// Remove session
    pub fn remove_session(&mut self, id: &str) -> Option<SessionData> {
        let session = self.sessions.remove(id);

        // Update persistence
        if let Some(ref path) = self.storage_path {
            if let Err(e) = self.save_sessions(path) {
                tracing::warn!("Failed to persist sessions: {}", e);
            }
        }

        session
    }

    /// Load sessions from disk
    pub fn load_sessions(
        &mut self,
        path: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let data = std::fs::read_to_string(path)?;
        self.sessions = serde_json::from_str(&data)?;
        tracing::info!("Loaded {} sessions from {:?}", self.sessions.len(), path);
        Ok(())
    }

    /// Save sessions to disk
    pub fn save_sessions(
        &self,
        path: &std::path::Path,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create parent directories
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let data = serde_json::to_string_pretty(&self.sessions)?;
        std::fs::write(path, data)?;
        tracing::debug!("Saved {} sessions to {:?}", self.sessions.len(), path);
        Ok(())
    }

    /// Form-based login (browser automation required)
    #[cfg(feature = "browser-automation")]
    pub async fn login_form(
        &mut self,
        page: &Page,
        username: &str,
        password: &str,
        config: &FormLoginConfig,
    ) -> Result<SessionData, Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Starting form-based login to {}", config.login_url);

        // Navigate to login page
        page.goto(&config.login_url).await?;
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Fill credentials
        let username_field = page.find_element(&config.username_selector).await?;
        username_field.click().await?;
        username_field.type_str(username).await?;

        let password_field = page.find_element(&config.password_selector).await?;
        password_field.click().await?;
        password_field.type_str(password).await?;

        // Submit
        let submit_btn = page.find_element(&config.submit_selector).await?;
        submit_btn.click().await?;

        // Wait for navigation
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Check for success/error
        if let Some(ref error_selector) = config.error_selector {
            if page.find_element(error_selector).await.is_ok() {
                return Err("Login failed: error indicator found".into());
            }
        }

        if let Some(ref success_selector) = config.success_selector {
            // Wait for success indicator
            let start = Instant::now();
            while start.elapsed() < Duration::from_secs(10) {
                if page.find_element(success_selector).await.is_ok() {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }

        // Extract session from cookies
        let cookies = page.get_cookies().await?;
        let mut session = SessionData::new();
        for cookie in cookies {
            session.cookies.insert(cookie.name, cookie.value);
        }

        session.metadata.insert("login_url".to_string(), config.login_url.clone());
        session.metadata.insert("username".to_string(), username.to_string());

        tracing::info!("Form login successful, extracted {} cookies", session.cookies.len());
        Ok(session)
    }

    /// OAuth2 authorization code flow (simplified)
    #[cfg(feature = "browser-automation")]
    pub async fn oauth2_flow(
        &mut self,
        page: &Page,
        config: &OAuth2Config,
    ) -> Result<SessionData, Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("Starting OAuth2 flow for provider '{}'", config.provider);

        // Build authorization URL
        let scopes = config.scopes.join(" ");
        let auth_url = format!(
            "{}?client_id={}&redirect_uri={}&response_type=code&scope={}",
            config.auth_endpoint,
            urlencoding::encode(&config.client_id),
            urlencoding::encode(&config.redirect_uri),
            urlencoding::encode(&scopes)
        );

        // Navigate to authorization page
        page.goto(&auth_url).await?;
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Handle consent if needed
        if let Some(ref consent_selector) = config.consent_button_selector {
            if let Ok(consent_btn) = page.find_element(consent_selector).await {
                tracing::debug!("Clicking consent button");
                consent_btn.click().await?;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }

        // Wait for redirect to callback URL
        let start = Instant::now();
        let mut auth_code: Option<String> = None;

        while start.elapsed() < Duration::from_secs(30) {
            if let Ok(Some(current_url)) = page.url().await {
                if current_url.starts_with(&config.redirect_uri) {
                    // Extract authorization code from URL
                    if let Some(code) = Self::extract_url_param(&current_url, "code") {
                        auth_code = Some(code);
                        break;
                    }
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        let code = auth_code.ok_or("Failed to obtain authorization code")?;
        tracing::debug!("Obtained authorization code");

        // Exchange code for token
        let token_response = self.exchange_code_for_token(config, &code).await?;

        // Build session
        let mut session = SessionData::new();
        session.tokens.insert("access_token".to_string(), token_response.access_token);
        session.tokens.insert("token_type".to_string(), token_response.token_type);

        if let Some(refresh_token) = token_response.refresh_token {
            session.tokens.insert("refresh_token".to_string(), refresh_token);
        }

        if let Some(expires_in) = token_response.expires_in {
            session.set_expiration(Duration::from_secs(expires_in));
        }

        session.metadata.insert("provider".to_string(), config.provider.clone());
        session.metadata.insert("scopes".to_string(), scopes);

        tracing::info!("OAuth2 flow successful");
        Ok(session)
    }

    /// Extract URL parameter
    fn extract_url_param(url: &str, param: &str) -> Option<String> {
        url::Url::parse(url)
            .ok()?
            .query_pairs()
            .find(|(key, _)| key == param)
            .map(|(_, value)| value.to_string())
    }

    async fn exchange_code_for_token(
        &self,
        config: &OAuth2Config,
        code: &str,
    ) -> Result<OAuth2TokenResponse, Box<dyn std::error::Error + Send + Sync>> {
        let client = reqwest::Client::new();

        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("client_id", &config.client_id),
            ("client_secret", &config.client_secret),
            ("redirect_uri", &config.redirect_uri),
        ];

        let response = client.post(&config.token_endpoint).form(&params).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("Token exchange failed: {}", error_text).into());
        }

        let token_response: OAuth2TokenResponse = response.json().await?;
        Ok(token_response)
    }

    /// Apply session to HTTP headers
    pub fn apply_to_headers(
        &self,
        session_id: &str,
        headers: &mut reqwest::header::HeaderMap,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let session = self.get_session(session_id).ok_or("Session not found")?;

        // Add bearer token if present
        if let Some(access_token) = session.tokens.get("access_token") {
            let value =
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", access_token))?;
            headers.insert(reqwest::header::AUTHORIZATION, value);
        }

        Ok(())
    }
}

impl Default for AuthenticationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_data_builder() {
        let session = SessionData::new()
            .with_bearer_token("token123")
            .with_cookie("session_id", "abc")
            .with_metadata("user", "admin");

        assert_eq!(session.tokens.get("bearer"), Some(&"token123".to_string()));
        assert_eq!(session.cookies.get("session_id"), Some(&"abc".to_string()));
        assert_eq!(session.metadata.get("user"), Some(&"admin".to_string()));
    }

    #[test]
    fn test_session_expiration() {
        let mut session = SessionData::new();
        session.set_expiration(Duration::from_secs(3600));

        assert!(!session.is_expired());

        // Test expired session
        let mut expired_session = SessionData::new();
        expired_session.expires_at = Some(0);
        assert!(expired_session.is_expired());
    }

    #[test]
    fn test_auth_manager_session_storage() {
        let mut manager = AuthenticationManager::new();
        let session = SessionData::new().with_bearer_token("test_token");

        manager.store_session("test_session", session.clone());
        assert!(manager.get_session("test_session").is_some());

        manager.remove_session("test_session");
        assert!(manager.get_session("test_session").is_none());
    }
}
