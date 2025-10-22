# Authentication Guide

This guide explains how to use the JWT authentication system in Semantic Browser.

## Overview

Semantic Browser uses JWT (JSON Web Tokens) for stateless authentication, following industry best practices for 2025:

- **Configurable secrets** via environment variables
- **Token expiration** for enhanced security
- **Role-based access control** (RBAC) support
- **Axum extractors** for clean, type-safe authentication

## Quick Start

### 1. Configure JWT Secret

Create a `.env` file with a strong secret (minimum 32 characters):

```bash
# Generate a secure random secret
openssl rand -base64 48 > /tmp/jwt_secret.txt

# Add to .env
echo "JWT_SECRET=$(cat /tmp/jwt_secret.txt)" > .env
```

**Warning**: Never use the default secret in production!

**Development fallback**: If `JWT_SECRET` is unset the server logs a warning and disables JWT validation entirely. This is handy for local debugging only—do not rely on it beyond your laptop.

### 2. Start the Server

```bash
cargo run
```

The server will initialize JWT authentication on startup.

### 3. Generate a Token

```bash
curl -X POST http://localhost:3000/auth/token \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "role": "admin"
  }'
```

Response:
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "expires_in": 86400
}
```

### 4. Use the Token

Include the token in the `Authorization` header for protected endpoints:

```bash
export TOKEN="eyJ0eXAiOiJKV1QiLCJhbGc..."

curl -X POST http://localhost:3000/parse \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"html": "<html><body>Test</body></html>"}'
```

## Token Structure

JWT tokens contain three parts (header.payload.signature):

```json
{
  "sub": "admin",           // Subject (username/user ID)
  "exp": 1735689600,        // Expiration time (Unix timestamp)
  "iat": 1735603200,        // Issued at (Unix timestamp)
  "role": "admin"           // Optional role for RBAC
}
```

### Token Lifetime

- **Default**: 24 hours from issue time
- **Custom duration**: Use `Claims::with_expiration()`

## Protected Endpoints

All API endpoints require authentication:

- `POST /parse` - Parse HTML and extract semantic data
- `POST /query` - Query Knowledge Graph with SPARQL
- `POST /browse` - Browse URL and extract semantic information

### Unprotected Endpoints

- `POST /auth/token` - Generate JWT token (for development/testing)

**Note**: In production, token generation should be protected by additional authentication (e.g., username/password, API key).

## Role-Based Access Control

Tokens can include optional roles for fine-grained access control:

```rust
use semantic_browser::auth::{Claims, require_role};

// Create admin token
let claims = Claims::new("admin".to_string(), Some("admin".to_string()));

// Verify role
require_role(&claims, "admin")?; // Ok
require_role(&claims, "user")?;  // Error
```

### Common Roles

- `admin` - Full access to all operations
- `user` - Standard read/write access
- `readonly` - Read-only access to queries
- `service` - For inter-service communication

## Security Best Practices

### Secret Management

1. **Never hardcode secrets** in source code
2. **Use environment variables** for configuration
3. **Rotate secrets periodically** (recommended: quarterly)
4. **Use different secrets** for dev/staging/production

### Token Security

1. **Short expiration times** reduce risk if token is compromised
2. **HTTPS only** in production to prevent token interception
3. **Secure storage** on client side (avoid localStorage for sensitive data)
4. **Token revocation** - implemented with Redis for immediate invalidation

### Token Revocation with Redis

The Semantic Browser supports immediate token revocation using Redis for state management. This allows invalidating tokens before their natural expiration.

#### Setup

1. **Enable Redis Integration**:
   ```bash
   cargo build --features redis-integration
   ```

2. **Configure Redis**:
   ```bash
   # Add to .env
   REDIS_URL=redis://127.0.0.1:6379
   # Or with authentication
   REDIS_URL=redis://username:password@host:6379/0
   ```

3. **Start Redis Server**:
   ```bash
   # Using Docker
   docker run -d -p 6379:6379 redis:alpine

   # Or install locally
   redis-server
   ```

#### Usage

**Revoke a Token**:
```bash
curl -X POST http://localhost:3000/auth/revoke \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <admin-token>" \
  -d '{"token": "token-to-revoke"}'
```

**Response**:
```json
{
  "message": "Token revoked successfully"
}
```

#### How It Works

- Revoked tokens are stored in Redis with their expiration timestamp
- Each token validation checks Redis for revocation status
- Revoked tokens are automatically cleaned up when they expire
- Works across multiple server instances for horizontal scaling

#### Security Benefits

- **Immediate invalidation** of compromised tokens
- **No database dependency** for core authentication
- **Automatic cleanup** prevents Redis bloat
- **Distributed revocation** across server instances

### Secret Generation

Generate cryptographically secure secrets:

```bash
# Method 1: OpenSSL
openssl rand -base64 48

# Method 2: Python
python3 -c "import secrets; print(secrets.token_urlsafe(48))"

# Method 3: Rust
cargo install uuid-cli
uuid -v4 | tr -d '-' | fold -w 48 | head -1
```

## Advanced Usage

### Custom Token Duration

```rust
use semantic_browser::auth::Claims;
use chrono::Duration;

// Create token valid for 1 hour
let claims = Claims::with_expiration(
    "user123".to_string(),
    Some("user".to_string()),
    Duration::hours(1)
);
```

### Extracting Claims in Handlers

```rust
use axum::{Json, response::IntoResponse};
use semantic_browser::auth::AuthenticatedUser;

async fn protected_handler(
    user: AuthenticatedUser
) -> impl IntoResponse {
    let username = user.0.sub;
    let role = user.0.role;

    Json(format!("Hello, {}! Role: {:?}", username, role))
}
```

### Custom Authentication Middleware

For more complex scenarios, you can create custom middleware:

```rust
use axum::middleware;
use semantic_browser::auth::validate_token;

async fn auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    // Custom authentication logic
    // ...
}
```

## Error Handling

### Authentication Errors

- `401 Unauthorized` - Missing or invalid token
  - Missing `Authorization` header
  - Invalid token format
  - Expired token
  - Invalid signature

### Error Responses

```json
{
  "error": "Missing or invalid Authorization header"
}
```

or

```json
{
  "error": "Invalid or expired token"
}
```

## Development & Testing

### Development Mode

During local development you can leave `JWT_SECRET` unset; the server will log a warning and disable JWT validation entirely. This is convenient for quick manual testing, but remember that every endpoint becomes publicly accessible. Set a strong secret before exposing the service to anyone else.

### Testing Tokens

Generate test tokens programmatically:

```rust
#[cfg(test)]
mod tests {
    use semantic_browser::auth::{Claims, generate_token, JwtConfig};

    #[test]
    fn test_create_token() {
        std::env::set_var("JWT_SECRET", "test-secret-key-32-chars-long");
        JwtConfig::init().unwrap();

        let claims = Claims::new("testuser".to_string(), None);
        let token = generate_token(&claims).unwrap();

        assert!(!token.is_empty());
    }
}
```

### Disabling Authentication

To disable authentication in integration tests, simply clear the environment variable:

```bash
unset JWT_SECRET
```

The server will confirm the bypass with a warning log message. Remember to restore a valid secret for security-sensitive tests.

## Migration from Legacy Authentication

If migrating from the old hardcoded "Bearer secret" system:

1. **Update environment**: Add `JWT_SECRET` to `.env`
2. **Generate tokens**: Use `/auth/token` endpoint
3. **Update clients**: Replace hardcoded token with JWT
4. **Test thoroughly**: Verify all integrations work

## Troubleshooting

### "Failed to initialize JWT config"

- Ensure `JWT_SECRET` is set
- Verify secret is at least 32 characters
- Check for typos in environment variable name

### "Invalid or expired token"

- Token may have expired (check `exp` claim)
- Token may be corrupted
- Secret may have changed since token was issued
- Generate a new token

### "401 Unauthorized"

- Verify `Authorization` header is present
- Ensure format is `Bearer <token>`
- Check token hasn't expired
- Verify correct secret is configured

## Further Reading

- [JWT.io](https://jwt.io/) - JWT token debugger
- [RFC 7519](https://tools.ietf.org/html/rfc7519) - JWT specification
- [OWASP JWT Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/JSON_Web_Token_for_Java_Cheat_Sheet.html)
- [API Security Best Practices](../api/README.md)
