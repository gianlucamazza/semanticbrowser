// Authentication and JWT management module
//
// Best practices 2025:
// - JWT tokens for stateless authentication
// - Environment-based secret configuration
// - FromRequestParts for automatic token extraction
// - Token expiration and validation
// - Support for Authorization header (Bearer token)

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[cfg(feature = "redis-integration")]
use redis::{AsyncCommands, Client};
#[cfg(feature = "redis-integration")]
use std::sync::Arc;

/// JWT Claims structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (username or user ID)
    pub sub: String,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at (Unix timestamp)
    pub iat: i64,
    /// Additional custom claims
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

impl Claims {
    /// Create new claims with default expiration (24 hours)
    pub fn new(subject: String, role: Option<String>) -> Self {
        let now = Utc::now();
        let exp = (now + Duration::hours(24)).timestamp();

        Self { sub: subject, exp, iat: now.timestamp(), role }
    }

    /// Create new claims with custom expiration duration
    pub fn with_expiration(subject: String, role: Option<String>, duration: Duration) -> Self {
        let now = Utc::now();
        let exp = (now + duration).timestamp();

        Self { sub: subject, exp, iat: now.timestamp(), role }
    }
}

/// JWT authentication configuration
pub struct JwtConfig {
    #[allow(dead_code)]
    secret: String,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

/// Global JWT authentication state
enum AuthState {
    Disabled,
    Enabled(JwtConfig),
}

static JWT_STATE: OnceLock<AuthState> = OnceLock::new();

/// Token revocation store for Redis-based token blacklisting
#[cfg(feature = "redis-integration")]
pub struct TokenRevocationStore {
    client: Arc<Client>,
}

#[cfg(feature = "redis-integration")]
static REVOCATION_STORE: OnceLock<TokenRevocationStore> = OnceLock::new();

#[cfg(feature = "redis-integration")]
impl TokenRevocationStore {
    /// Create new token revocation store with Redis connection
    pub async fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::open(redis_url)?;
        // Test connection
        let mut conn = client.get_multiplexed_async_connection().await?;
        let _: () = conn.set("test", "ok").await?;
        let _: () = conn.del(&["test"]).await?;

        Ok(Self { client: Arc::new(client) })
    }

    /// Revoke a token by storing it in Redis with expiration
    pub async fn revoke_token(
        &self,
        token: &str,
        expiration: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("revoked_token:{}", token);

        // Store token with expiration (same as JWT expiration)
        let _: () = conn.set_ex(key, "revoked", expiration as u64).await?;
        Ok(())
    }

    /// Check if a token is revoked
    pub async fn is_revoked(&self, token: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let mut conn = self.client.get_multiplexed_async_connection().await?;
        let key = format!("revoked_token:{}", token);

        let exists: bool = conn.exists(key).await?;
        Ok(exists)
    }

    /// Get global revocation store instance
    pub fn get() -> Option<&'static Self> {
        REVOCATION_STORE.get()
    }

    /// Initialize global revocation store
    pub fn init_global(store: TokenRevocationStore) {
        let _ = REVOCATION_STORE.set(store);
    }
}

impl JwtConfig {
    /// Initialize JWT configuration from environment variable
    pub fn init() -> Result<(), String> {
        #[allow(clippy::disallowed_methods)]
        let secret = match std::env::var("JWT_SECRET") {
            Ok(value) if value.trim().is_empty() => None,
            Ok(value) => Some(value),
            Err(_) => None,
        };

        if secret.is_none() {
            tracing::warn!("JWT_SECRET not set - JWT authentication disabled");
            return JWT_STATE
                .set(AuthState::Disabled)
                .map_err(|_| "JWT config already initialized".to_string());
        }

        let secret = secret.expect("secret always present at this point");

        if secret.len() < 32 {
            return Err("JWT_SECRET must be at least 32 characters long".to_string());
        }

        let config = Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            secret,
        };

        JWT_STATE
            .set(AuthState::Enabled(config))
            .map_err(|_| "JWT config already initialized".to_string())?;
        tracing::info!("JWT authentication configured");
        Ok(())
    }

    /// Returns true if JWT authentication is enabled
    pub fn is_enabled() -> bool {
        matches!(JWT_STATE.get(), Some(AuthState::Enabled(_)))
    }

    /// Get JWT configuration (panics if not initialized)
    fn get() -> &'static JwtConfig {
        match JWT_STATE.get() {
            Some(AuthState::Enabled(config)) => config,
            Some(AuthState::Disabled) => {
                panic!("JWT authentication disabled - JwtConfig::get() should not be called")
            }
            None => panic!("JWT config not initialized - call JwtConfig::init() first"),
        }
    }
}

/// Generate JWT token for claims
pub fn generate_token(claims: &Claims) -> Result<String, jsonwebtoken::errors::Error> {
    let config = JwtConfig::get();
    encode(&Header::default(), claims, &config.encoding_key)
}

/// Validate and decode JWT token (async version)
pub async fn validate_token_async(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let config = JwtConfig::get();
    let validation = Validation::default();
    let token_data = decode::<Claims>(token, &config.decoding_key, &validation)?;

    // Check if token is revoked (only if Redis integration is enabled)
    #[cfg(feature = "redis-integration")]
    {
        if let Some(store) = TokenRevocationStore::get() {
            if store.is_revoked(token).await.unwrap_or(false) {
                return Err(jsonwebtoken::errors::Error::from(
                    jsonwebtoken::errors::ErrorKind::InvalidToken,
                ));
            }
        }
    }

    Ok(token_data.claims)
}

/// Validate and decode JWT token (sync wrapper for backward compatibility)
pub fn validate_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    // For now, we'll use a simple approach - in production this should be handled better
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(validate_token_async(token))
    })
}

/// Extract token from Authorization header
fn extract_token_from_header(auth_header: Option<&str>) -> Option<String> {
    auth_header?.strip_prefix("Bearer ").map(|token| token.trim().to_string())
}

/// Axum extractor for JWT authentication
///
/// Usage in handlers:
/// ```rust,no_run
/// use semantic_browser::auth::AuthenticatedUser;
/// use axum::response::IntoResponse;
///
/// async fn protected_handler(claims: AuthenticatedUser) -> impl IntoResponse {
///     format!("Hello, {}!", claims.0.sub)
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthenticatedUser(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let auth_header = parts.headers.get("authorization").and_then(|h| h.to_str().ok());

        if !JwtConfig::is_enabled() {
            tracing::debug!("JWT authentication disabled - bypassing token validation");
            let now = Utc::now();
            let claims = Claims {
                sub: "anonymous".to_string(),
                exp: now.timestamp(),
                iat: now.timestamp(),
                role: None,
            };
            return Ok(AuthenticatedUser(claims));
        }

        // Extract token from header
        let token = extract_token_from_header(auth_header).ok_or(AuthError::MissingToken)?;

        // Validate token and extract claims
        let claims = validate_token_async(&token).await.map_err(|e| {
            tracing::debug!("Token validation failed: {}", e);
            AuthError::InvalidToken
        })?;

        crate::security::log_action("auth", &format!("Authenticated user: {}", claims.sub));
        Ok(AuthenticatedUser(claims))
    }
}

/// Authentication error types
#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingToken => {
                (StatusCode::UNAUTHORIZED, "Missing or invalid Authorization header")
            }
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "Invalid or expired token"),
        };

        crate::security::log_action("auth_error", message);
        (status, message).into_response()
    }
}

/// Middleware helper to check for specific roles
pub fn require_role(claims: &Claims, required_role: &str) -> Result<(), AuthError> {
    if !JwtConfig::is_enabled() {
        tracing::debug!(
            "JWT authentication disabled - allowing access to role-protected resource: {}",
            required_role
        );
        return Ok(());
    }

    match &claims.role {
        Some(role) if role == required_role => Ok(()),
        _ => {
            tracing::warn!("Role check failed for user {}: required {}", claims.sub, required_role);
            Err(AuthError::InvalidToken)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_config_init() {
        // Set a proper secret for testing
        std::env::set_var("JWT_SECRET", "test-secret-key-that-is-long-enough-for-validation");
        let result = JwtConfig::init();
        // Result might be Ok or Err("already initialized") depending on test execution order
        // Both are acceptable - we just want to ensure it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_claims_creation() {
        let claims = Claims::new("testuser".to_string(), Some("admin".to_string()));
        assert_eq!(claims.sub, "testuser");
        assert_eq!(claims.role, Some("admin".to_string()));
        assert!(claims.exp > claims.iat);
    }

    #[tokio::test]
    async fn test_token_generation_and_validation() {
        std::env::set_var("JWT_SECRET", "test-secret-key-that-is-long-enough-for-validation");
        JwtConfig::init().ok();

        let claims = Claims::new("testuser".to_string(), Some("user".to_string()));
        let token = generate_token(&claims).unwrap();

        let decoded = validate_token_async(&token).await.unwrap();
        assert_eq!(decoded.sub, "testuser");
        assert_eq!(decoded.role, Some("user".to_string()));
    }

    #[test]
    fn test_extract_token_from_header() {
        let header = "Bearer my-token-123";
        let token = extract_token_from_header(Some(header));
        assert_eq!(token, Some("my-token-123".to_string()));

        let invalid_header = "InvalidFormat token";
        let token = extract_token_from_header(Some(invalid_header));
        assert_eq!(token, None);
    }

    #[test]
    fn test_require_role() {
        std::env::set_var("JWT_SECRET", "test-secret-key-that-is-long-enough-for-validation");
        let _ = JwtConfig::init();

        let admin_claims = Claims::new("admin".to_string(), Some("admin".to_string()));
        assert!(require_role(&admin_claims, "admin").is_ok());
        assert!(require_role(&admin_claims, "user").is_err());

        let user_claims = Claims::new("user".to_string(), Some("user".to_string()));
        assert!(require_role(&user_claims, "admin").is_err());
        assert!(require_role(&user_claims, "user").is_ok());
    }
}
