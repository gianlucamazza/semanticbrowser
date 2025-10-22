//! Enhanced API Client Module
//!
//! Provides high-level HTTP client for REST APIs, GraphQL, and modern web services.
//! Designed for AI agents to interact with web APIs programmatically.
//!
//! # Features
//! - RESTful HTTP operations (GET, POST, PUT, PATCH, DELETE)
//! - GraphQL query execution
//! - Multipart form data uploads
//! - JSON serialization/deserialization
//! - Custom headers and authentication
//! - Retry logic with exponential backoff
//!
//! # Best Practices 2025
//! - Type-safe request/response handling
//! - Async-first with Tokio
//! - Builder pattern for configuration
//! - Comprehensive error handling

use reqwest::{header::HeaderMap, header::HeaderValue, Client, Method, Response};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::time::Duration;

/// API client configuration
#[derive(Debug, Clone)]
pub struct ApiClientConfig {
    /// Base URL for all requests
    pub base_url: String,
    /// Default headers to include in all requests
    pub default_headers: HeaderMap,
    /// Request timeout (seconds)
    pub timeout: u64,
    /// Enable retry on failure
    pub retry_enabled: bool,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// User agent string
    pub user_agent: String,
}

impl Default for ApiClientConfig {
    fn default() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(reqwest::header::CONTENT_TYPE, HeaderValue::from_static("application/json"));

        Self {
            base_url: String::new(),
            default_headers: headers,
            timeout: 30,
            retry_enabled: true,
            max_retries: 3,
            user_agent: format!("SemanticBrowser/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

/// Enhanced API client
#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
    config: ApiClientConfig,
}

impl ApiClient {
    /// Create new API client with base URL
    pub fn new(base_url: &str) -> Self {
        let config = ApiClientConfig { base_url: base_url.to_string(), ..Default::default() };

        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .user_agent(&config.user_agent)
            .build()
            .expect("Failed to build HTTP client");

        Self { client, config }
    }

    /// Create with custom configuration
    pub fn with_config(config: ApiClientConfig) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout))
            .user_agent(&config.user_agent)
            .build()
            .expect("Failed to build HTTP client");

        Self { client, config }
    }

    /// Add bearer token authentication
    pub fn with_bearer_token(mut self, token: &str) -> Self {
        let value =
            HeaderValue::from_str(&format!("Bearer {}", token)).expect("Invalid bearer token");
        self.config.default_headers.insert(reqwest::header::AUTHORIZATION, value);
        self
    }

    /// Add API key header
    pub fn with_api_key(mut self, header_name: &str, api_key: &str) -> Self {
        let value = HeaderValue::from_str(api_key).expect("Invalid API key");
        let key = reqwest::header::HeaderName::from_bytes(header_name.as_bytes())
            .expect("Invalid header name");
        self.config.default_headers.insert(key, value);
        self
    }

    /// Add custom header
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        let header_value = HeaderValue::from_str(value).expect("Invalid header value");
        let header_name =
            reqwest::header::HeaderName::from_bytes(name.as_bytes()).expect("Invalid header name");
        self.config.default_headers.insert(header_name, header_value);
        self
    }

    /// Build full URL from endpoint
    fn build_url(&self, endpoint: &str) -> String {
        if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            endpoint.to_string()
        } else {
            format!("{}{}", self.config.base_url.trim_end_matches('/'), endpoint)
        }
    }

    /// Execute HTTP request with retry logic
    async fn execute_with_retry(
        &self,
        method: Method,
        url: &str,
        headers: Option<HeaderMap>,
        body: Option<String>,
    ) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
        let mut last_error = None;
        let max_attempts = if self.config.retry_enabled {
            self.config.max_retries + 1
        } else {
            1
        };

        for attempt in 0..max_attempts {
            if attempt > 0 {
                let delay = Duration::from_secs(2u64.pow(attempt - 1));
                tracing::debug!("Retry attempt {} after {:?}", attempt, delay);
                tokio::time::sleep(delay).await;
            }

            // Build request
            let mut request = self.client.request(method.clone(), url);

            // Add headers
            request = request.headers(self.config.default_headers.clone());
            if let Some(ref custom_headers) = headers {
                for (key, value) in custom_headers {
                    request = request.header(key, value);
                }
            }

            // Add body
            if let Some(ref body_data) = body {
                request = request.body(body_data.clone());
            }

            // Execute
            match request.send().await {
                Ok(response) => {
                    tracing::debug!("{} {} -> {}", method, url, response.status());
                    return Ok(response);
                }
                Err(e) => {
                    tracing::warn!("Request failed (attempt {}): {}", attempt + 1, e);
                    last_error = Some(e);
                }
            }
        }

        Err(format!("Request failed after {} attempts: {}", max_attempts, last_error.unwrap())
            .into())
    }

    /// GET request returning JSON
    pub async fn get<T: DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.build_url(endpoint);
        tracing::info!("GET {}", url);

        let response = self.execute_with_retry(Method::GET, &url, None, None).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(format!("GET failed with status {}: {}", status, error_text).into());
        }

        let data: T = response.json().await?;
        Ok(data)
    }

    /// POST request with JSON body
    pub async fn post_json<T: Serialize, R: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<R, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.build_url(endpoint);
        tracing::info!("POST {}", url);

        let body_str = serde_json::to_string(body)?;
        let response = self.execute_with_retry(Method::POST, &url, None, Some(body_str)).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(format!("POST failed with status {}: {}", status, error_text).into());
        }

        let data: R = response.json().await?;
        Ok(data)
    }

    /// PUT request with JSON body
    pub async fn put_json<T: Serialize, R: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<R, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.build_url(endpoint);
        tracing::info!("PUT {}", url);

        let body_str = serde_json::to_string(body)?;
        let response = self.execute_with_retry(Method::PUT, &url, None, Some(body_str)).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(format!("PUT failed with status {}: {}", status, error_text).into());
        }

        let data: R = response.json().await?;
        Ok(data)
    }

    /// PATCH request with JSON body
    pub async fn patch_json<T: Serialize, R: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<R, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.build_url(endpoint);
        tracing::info!("PATCH {}", url);

        let body_str = serde_json::to_string(body)?;
        let response = self.execute_with_retry(Method::PATCH, &url, None, Some(body_str)).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(format!("PATCH failed with status {}: {}", status, error_text).into());
        }

        let data: R = response.json().await?;
        Ok(data)
    }

    /// DELETE request
    pub async fn delete(
        &self,
        endpoint: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = self.build_url(endpoint);
        tracing::info!("DELETE {}", url);

        let response = self.execute_with_retry(Method::DELETE, &url, None, None).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(format!("DELETE failed with status {}: {}", status, error_text).into());
        }

        Ok(())
    }

    /// GraphQL query
    pub async fn graphql_query(
        &self,
        query: &str,
        variables: &HashMap<String, JsonValue>,
    ) -> Result<JsonValue, Box<dyn std::error::Error + Send + Sync>> {
        tracing::info!("GraphQL query");

        let body = serde_json::json!({
            "query": query,
            "variables": variables
        });

        let url = self.build_url("/graphql");
        let body_str = serde_json::to_string(&body)?;

        let response = self.execute_with_retry(Method::POST, &url, None, Some(body_str)).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(format!("GraphQL failed with status {}: {}", status, error_text).into());
        }

        let result: JsonValue = response.json().await?;

        // Check for GraphQL errors
        if let Some(errors) = result.get("errors") {
            return Err(format!("GraphQL errors: {}", errors).into());
        }

        Ok(result)
    }

    /// Upload multipart form data
    pub async fn upload_multipart(
        &self,
        endpoint: &str,
        files: Vec<(&str, Vec<u8>)>,
        fields: HashMap<String, String>,
    ) -> Result<JsonValue, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.build_url(endpoint);
        tracing::info!("POST multipart {}", url);

        let mut form = reqwest::multipart::Form::new();

        // Add files
        for (name, data) in files {
            let part = reqwest::multipart::Part::bytes(data).file_name(name.to_string());
            form = form.part(name.to_string(), part);
        }

        // Add text fields
        for (key, value) in fields {
            form = form.text(key, value);
        }

        let response = self
            .client
            .post(&url)
            .headers(self.config.default_headers.clone())
            .multipart(form)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(format!("Upload failed with status {}: {}", status, error_text).into());
        }

        let result: JsonValue = response.json().await?;
        Ok(result)
    }

    /// Download file as bytes
    pub async fn download_file(
        &self,
        endpoint: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.build_url(endpoint);
        tracing::info!("Downloading file from {}", url);

        let response = self.execute_with_retry(Method::GET, &url, None, None).await?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(format!("Download failed with status {}", status).into());
        }

        let bytes = response.bytes().await?.to_vec();
        tracing::info!("Downloaded {} bytes", bytes.len());
        Ok(bytes)
    }

    /// Custom request with full control
    pub async fn request(
        &self,
        method: Method,
        endpoint: &str,
        headers: Option<HeaderMap>,
        body: Option<JsonValue>,
    ) -> Result<JsonValue, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.build_url(endpoint);
        tracing::info!("{} {}", method, url);

        let body_str = if let Some(b) = body {
            Some(serde_json::to_string(&b)?)
        } else {
            None
        };

        let response = self.execute_with_retry(method, &url, headers, body_str).await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(format!("Request failed with status {}: {}", status, error_text).into());
        }

        let result: JsonValue = response.json().await?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_builder() {
        let client = ApiClient::new("https://api.example.com")
            .with_bearer_token("token123")
            .with_api_key("X-API-Key", "key456");

        assert_eq!(client.config.base_url, "https://api.example.com");
        assert!(client.config.default_headers.contains_key(reqwest::header::AUTHORIZATION));
    }

    #[test]
    fn test_build_url() {
        let client = ApiClient::new("https://api.example.com");

        assert_eq!(client.build_url("/users"), "https://api.example.com/users");
        assert_eq!(client.build_url("https://other.com/data"), "https://other.com/data");
    }

    #[test]
    fn test_api_client_config_default() {
        let config = ApiClientConfig::default();
        assert_eq!(config.timeout, 30);
        assert!(config.retry_enabled);
        assert_eq!(config.max_retries, 3);
    }
}
