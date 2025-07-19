// File: crates/storm-finalverse/src/rest_api.rs
// REST API client for Finalverse structured API calls
// Handles authentication, world queries, and asset management

use std::collections::HashMap;
use std::time::Duration;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, debug};
use anyhow::Result;
use url::Url;

use crate::{EntityData, EntityUpdate};

/// REST API client for Finalverse services
pub struct FinalverseRestClient {
    /// HTTP client for making requests
    client: Client,
    /// Base URL for the Finalverse API
    base_url: Url,
    /// Authentication token for API requests
    auth_token: Option<String>,
    /// API configuration settings
    config: RestApiConfig,
}

/// Configuration for REST API client
#[derive(Debug, Clone)]
pub struct RestApiConfig {
    /// Request timeout duration
    pub timeout: Duration,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// User agent string for requests
    pub user_agent: String,
    /// Enable gzip compression for requests
    pub enable_compression: bool,
    /// Default headers to include in all requests
    pub default_headers: HashMap<String, String>,
}

impl Default for RestApiConfig {
    fn default() -> Self {
        let mut default_headers = HashMap::new();
        default_headers.insert("Content-Type".to_string(), "application/json".to_string());
        default_headers.insert("Accept".to_string(), "application/json".to_string());

        Self {
            timeout: Duration::from_secs(30),
            max_retries: 3,
            user_agent: format!("StormCore-Finalverse/{}", crate::FINALVERSE_PROTOCOL_VERSION),
            enable_compression: true,
            default_headers,
        }
    }
}

/// Authentication request payload
#[derive(Debug, Serialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
    pub client_version: String,
}

/// World information response
#[derive(Debug, Deserialize)]
pub struct WorldInfo {
    pub world_id: String,
    pub name: String,
    pub description: String,
    pub max_users: u32,
    pub current_users: u32,
    pub status: String,
    pub regions: Vec<RegionInfo>,
}

/// Region information within a world
#[derive(Debug, Deserialize)]
pub struct RegionInfo {
    pub region_id: String,
    pub name: String,
    pub position: [f32; 3],
    pub size: [f32; 3],
    pub entity_count: u32,
}

/// API error response structure
#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub code: i32,
    pub details: Option<String>,
}

impl FinalverseRestClient {
    /// Create a new REST API client
    pub fn new(base_url: impl AsRef<str>) -> Result<Self> {
        Self::with_config(base_url, RestApiConfig::default())
    }

    /// Create a new REST API client with custom configuration
    pub fn with_config(base_url: impl AsRef<str>, config: RestApiConfig) -> Result<Self> {
        let base_url = Url::parse(base_url.as_ref())
            .map_err(|e| anyhow::anyhow!("Invalid base URL: {}", e))?;

        // Build HTTP client with configuration
        let mut client_builder = Client::builder()
            .timeout(config.timeout)
            .user_agent(&config.user_agent);

        if config.enable_compression {
            client_builder = client_builder.gzip(true);
        }

        let client = client_builder.build()
            .map_err(|e| anyhow::anyhow!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            client,
            base_url,
            auth_token: None,
            config,
        })
    }

    /// Authenticate with the Finalverse server
    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<crate::AuthResponse> {
        info!("Authenticating user: {}", username);

        let auth_request = AuthRequest {
            username: username.to_string(),
            password: password.to_string(),
            client_version: crate::FINALVERSE_PROTOCOL_VERSION.to_string(),
        };

        let url = self.base_url.join("/api/v1/auth/login")?;
        let response = self.make_request_with_retry("POST", &url, Some(&auth_request)).await?;

        let auth_response: crate::AuthResponse = response.json().await
            .map_err(|e| anyhow::anyhow!("Failed to parse auth response: {}", e))?;

        if auth_response.success {
            self.auth_token = auth_response.token.clone();
            info!("Authentication successful for user: {}", username);
        } else {
            warn!("Authentication failed for user: {}", username);
        }

        Ok(auth_response)
    }

    /// Get information about available worlds
    pub async fn get_worlds(&self) -> Result<Vec<WorldInfo>> {
        self.ensure_authenticated()?;

        let url = self.base_url.join("/api/v1/worlds")?;
        let response = self.make_authenticated_request("GET", &url, Option::<&()>::None).await?;

        let worlds: Vec<WorldInfo> = response.json().await
            .map_err(|e| anyhow::anyhow!("Failed to parse worlds response: {}", e))?;

        debug!("Retrieved {} worlds", worlds.len());
        Ok(worlds)
    }

    /// Get detailed information about a specific world
    pub async fn get_world_info(&self, world_id: &str) -> Result<WorldInfo> {
        self.ensure_authenticated()?;

        let url = self.base_url.join(&format!("/api/v1/worlds/{}", world_id))?;
        let response = self.make_authenticated_request("GET", &url, Option::<&()>::None).await?;

        let world_info: WorldInfo = response.json().await
            .map_err(|e| anyhow::anyhow!("Failed to parse world info response: {}", e))?;

        debug!("Retrieved info for world: {}", world_info.name);
        Ok(world_info)
    }

    /// Get entities in a specific region
    pub async fn get_region_entities(&self, world_id: &str, region_id: &str) -> Result<Vec<EntityData>> {
        self.ensure_authenticated()?;

        let url = self.base_url.join(&format!("/api/v1/worlds/{}/regions/{}/entities", world_id, region_id))?;
        let response = self.make_authenticated_request("GET", &url, Option::<&()>::None).await?;

        let entities: Vec<EntityData> = response.json().await
            .map_err(|e| anyhow::anyhow!("Failed to parse entities response: {}", e))?;

        debug!("Retrieved {} entities from region {}", entities.len(), region_id);
        Ok(entities)
    }

    /// Update an entity's properties - fixed to avoid response move issue
    pub async fn update_entity(&self, world_id: &str, entity_update: &EntityUpdate) -> Result<()> {
        self.ensure_authenticated()?;

        let url = self.base_url.join(&format!("/api/v1/worlds/{}/entities/{}", world_id, entity_update.entity_id))?;
        let response = self.make_authenticated_request("PATCH", &url, Some(entity_update)).await?;

        let status = response.status();
        if status.is_success() {
            debug!("Successfully updated entity: {}", entity_update.entity_id);
            Ok(())
        } else {
            let error: ApiError = response.json().await
                .unwrap_or_else(|_| ApiError {
                    error: "Unknown error".to_string(),
                    code: status.as_u16() as i32,
                    details: None,
                });
            Err(anyhow::anyhow!("Failed to update entity: {}", error.error))
        }
    }

    /// Create a new entity in the world - fixed to avoid response move issue
    pub async fn create_entity(&self, world_id: &str, entity_data: &EntityData) -> Result<String> {
        self.ensure_authenticated()?;

        let url = self.base_url.join(&format!("/api/v1/worlds/{}/entities", world_id))?;
        let response = self.make_authenticated_request("POST", &url, Some(entity_data)).await?;

        let status = response.status();
        if status.is_success() {
            #[derive(Deserialize)]
            struct CreateResponse {
                entity_id: String,
            }

            let create_response: CreateResponse = response.json().await
                .map_err(|e| anyhow::anyhow!("Failed to parse create response: {}", e))?;

            debug!("Successfully created entity: {}", create_response.entity_id);
            Ok(create_response.entity_id)
        } else {
            let error: ApiError = response.json().await
                .unwrap_or_else(|_| ApiError {
                    error: "Unknown error".to_string(),
                    code: status.as_u16() as i32,
                    details: None,
                });
            Err(anyhow::anyhow!("Failed to create entity: {}", error.error))
        }
    }

    /// Delete an entity from the world - fixed to avoid response move issue
    pub async fn delete_entity(&self, world_id: &str, entity_id: &str) -> Result<()> {
        self.ensure_authenticated()?;

        let url = self.base_url.join(&format!("/api/v1/worlds/{}/entities/{}", world_id, entity_id))?;
        let response = self.make_authenticated_request("DELETE", &url, Option::<&()>::None).await?;

        let status = response.status();
        if status.is_success() {
            debug!("Successfully deleted entity: {}", entity_id);
            Ok(())
        } else {
            let error: ApiError = response.json().await
                .unwrap_or_else(|_| ApiError {
                    error: "Unknown error".to_string(),
                    code: status.as_u16() as i32,
                    details: None,
                });
            Err(anyhow::anyhow!("Failed to delete entity: {}", error.error))
        }
    }

    /// Make an authenticated HTTP request - fixed Option reference issue
    async fn make_authenticated_request<T: Serialize>(
        &self,
        method: &str,
        url: &Url,
        body: Option<&T>,
    ) -> Result<Response> {
        self.ensure_authenticated()?;

        let mut request = match method {
            "GET" => self.client.get(url.clone()),
            "POST" => self.client.post(url.clone()),
            "PUT" => self.client.put(url.clone()),
            "PATCH" => self.client.patch(url.clone()),
            "DELETE" => self.client.delete(url.clone()),
            _ => return Err(anyhow::anyhow!("Unsupported HTTP method: {}", method)),
        };

        // Add authentication header - fixed Option reference issue
        if let Some(ref token) = self.auth_token {
            request = request.bearer_auth(token);
        }

        // Add default headers
        for (key, value) in &self.config.default_headers {
            request = request.header(key, value);
        }

        // Add body if provided
        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request.send().await
            .map_err(|e| anyhow::anyhow!("Request failed: {}", e))?;

        if response.status().is_success() {
            Ok(response)
        } else {
            Err(anyhow::anyhow!("Request failed with status: {}", response.status()))
        }
    }

    /// Make an HTTP request with retry logic
    async fn make_request_with_retry<T: Serialize>(
        &self,
        method: &str,
        url: &Url,
        body: Option<&T>,
    ) -> Result<Response> {
        let mut last_error = None;

        for attempt in 0..=self.config.max_retries {
            let mut request = match method {
                "GET" => self.client.get(url.clone()),
                "POST" => self.client.post(url.clone()),
                "PUT" => self.client.put(url.clone()),
                "PATCH" => self.client.patch(url.clone()),
                "DELETE" => self.client.delete(url.clone()),
                _ => return Err(anyhow::anyhow!("Unsupported HTTP method: {}", method)),
            };

            // Add default headers
            for (key, value) in &self.config.default_headers {
                request = request.header(key, value);
            }

            // Add body if provided
            if let Some(body) = body {
                request = request.json(body);
            }

            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(response);
                    } else if response.status().is_server_error() && attempt < self.config.max_retries {
                        warn!("Server error on attempt {}, retrying...", attempt + 1);
                        tokio::time::sleep(Duration::from_millis(1000 * (attempt as u64 + 1))).await;
                        continue;
                    } else {
                        return Err(anyhow::anyhow!("Request failed with status: {}", response.status()));
                    }
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.config.max_retries {
                        warn!("Request failed on attempt {}, retrying...", attempt + 1);
                        tokio::time::sleep(Duration::from_millis(1000 * (attempt as u64 + 1))).await;
                    }
                }
            }
        }

        Err(anyhow::anyhow!("Request failed after {} attempts: {:?}",
            self.config.max_retries + 1, last_error))
    }

    /// Ensure the client is authenticated
    fn ensure_authenticated(&self) -> Result<()> {
        if self.auth_token.is_none() {
            return Err(anyhow::anyhow!("Not authenticated - call authenticate() first"));
        }
        Ok(())
    }

    /// Check if the client is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.auth_token.is_some()
    }

    /// Get the current authentication token
    pub fn get_auth_token(&self) -> Option<&String> {
        self.auth_token.as_ref()
    }

    /// Set authentication token manually
    pub fn set_auth_token(&mut self, token: String) {
        self.auth_token = Some(token);
    }

    /// Clear authentication token
    pub fn clear_auth_token(&mut self) {
        self.auth_token = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rest_client_creation() {
        let client = FinalverseRestClient::new("https://api.finalverse.example.com");
        assert!(client.is_ok());
        let client = client.unwrap();
        assert!(!client.is_authenticated());
    }

    #[test]
    fn test_rest_config_default() {
        let config = RestApiConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
        assert!(config.enable_compression);
        assert!(config.default_headers.contains_key("Content-Type"));
    }

    #[tokio::test]
    async fn test_authentication_required() {
        let client = FinalverseRestClient::new("https://api.finalverse.example.com").unwrap();
        let result = client.get_worlds().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not authenticated"));
    }

    #[test]
    fn test_auth_request_serialization() {
        let request = AuthRequest {
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            client_version: "1.0.0".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("testuser"));
        assert!(json.contains("testpass"));
        assert!(json.contains("1.0.0"));
    }

    #[test]
    fn test_api_error_deserialization() {
        let json = r#"{"error": "Not found", "code": 404, "details": "Resource not found"}"#;
        let error: ApiError = serde_json::from_str(json).unwrap();
        assert_eq!(error.error, "Not found");
        assert_eq!(error.code, 404);
        assert_eq!(error.details, Some("Resource not found".to_string()));
    }
}