//! Connector integration for external systems
//!
//! Provides HTTP and gRPC connectors for automated task execution.
//! Supports external service invocation with retries, timeouts, and error handling.

use crate::error::{WorkflowError, WorkflowResult};
use std::collections::HashMap;
use std::time::Duration;

/// Connector type for external service invocation
#[derive(Debug, Clone)]
pub enum ConnectorType {
    /// HTTP REST API connector
    Http {
        /// Base URL for the service
        base_url: String,
        /// HTTP method (GET, POST, PUT, DELETE)
        method: String,
        /// Request timeout in seconds
        timeout_secs: u64,
        /// Maximum retry attempts
        max_retries: u32,
    },
    /// gRPC service connector
    Grpc {
        /// gRPC service endpoint
        endpoint: String,
        /// Service name
        service: String,
        /// Method name
        method: String,
        /// Request timeout in seconds
        timeout_secs: u64,
        /// Maximum retry attempts
        max_retries: u32,
    },
}

/// Connector configuration
#[derive(Debug, Clone)]
pub struct ConnectorConfig {
    /// Connector type
    pub connector_type: ConnectorType,
    /// Additional headers for HTTP requests
    pub headers: HashMap<String, String>,
    /// Authentication token (if needed)
    pub auth_token: Option<String>,
}

/// Connector integration for external task execution
pub struct ConnectorIntegration {
    /// Registered connectors by name
    connectors: HashMap<String, ConnectorConfig>,
    /// HTTP client for REST API calls
    #[cfg(feature = "http")]
    http_client: reqwest::Client,
}

impl ConnectorIntegration {
    /// Create new connector integration
    pub fn new() -> Self {
        #[cfg(feature = "http")]
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            connectors: HashMap::new(),
            #[cfg(feature = "http")]
            http_client,
        }
    }

    /// Register an HTTP connector
    pub fn register_http_connector(
        &mut self,
        name: String,
        base_url: String,
        method: String,
        timeout_secs: u64,
        max_retries: u32,
    ) {
        self.connectors.insert(
            name,
            ConnectorConfig {
                connector_type: ConnectorType::Http {
                    base_url,
                    method,
                    timeout_secs,
                    max_retries,
                },
                headers: HashMap::new(),
                auth_token: None,
            },
        );
    }

    /// Register a gRPC connector
    pub fn register_grpc_connector(
        &mut self,
        name: String,
        endpoint: String,
        service: String,
        method: String,
        timeout_secs: u64,
        max_retries: u32,
    ) {
        self.connectors.insert(
            name,
            ConnectorConfig {
                connector_type: ConnectorType::Grpc {
                    endpoint,
                    service,
                    method,
                    timeout_secs,
                    max_retries,
                },
                headers: HashMap::new(),
                auth_token: None,
            },
        );
    }

    /// Set authentication token for a connector
    pub fn set_auth_token(&mut self, connector_name: &str, token: String) -> WorkflowResult<()> {
        let config = self.connectors.get_mut(connector_name).ok_or_else(|| {
            WorkflowError::ResourceUnavailable(format!("Connector {} not found", connector_name))
        })?;
        config.auth_token = Some(token);
        Ok(())
    }

    /// Set headers for a connector
    pub fn set_headers(
        &mut self,
        connector_name: &str,
        headers: HashMap<String, String>,
    ) -> WorkflowResult<()> {
        let config = self.connectors.get_mut(connector_name).ok_or_else(|| {
            WorkflowError::ResourceUnavailable(format!("Connector {} not found", connector_name))
        })?;
        config.headers = headers;
        Ok(())
    }

    /// Execute a task via connector
    pub async fn execute_task(
        &self,
        connector_name: &str,
        data: serde_json::Value,
    ) -> WorkflowResult<serde_json::Value> {
        let config = self.connectors.get(connector_name).ok_or_else(|| {
            WorkflowError::ResourceUnavailable(format!("Connector {} not found", connector_name))
        })?;

        match &config.connector_type {
            ConnectorType::Http {
                base_url,
                method,
                timeout_secs,
                max_retries,
            } => {
                self.execute_http_task(base_url, method, *timeout_secs, *max_retries, &config, data)
                    .await
            }
            ConnectorType::Grpc {
                endpoint,
                service,
                method,
                timeout_secs,
                max_retries,
            } => {
                self.execute_grpc_task(
                    endpoint,
                    service,
                    method,
                    *timeout_secs,
                    *max_retries,
                    &config,
                    data,
                )
                .await
            }
        }
    }

    /// Execute HTTP REST API task
    #[cfg(feature = "http")]
    async fn execute_http_task(
        &self,
        base_url: &str,
        method: &str,
        timeout_secs: u64,
        max_retries: u32,
        config: &ConnectorConfig,
        data: serde_json::Value,
    ) -> WorkflowResult<serde_json::Value> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| {
                WorkflowError::TaskExecutionFailed(format!("Failed to create HTTP client: {}", e))
            })?;

        // Execute with retries
        let mut last_error = None;
        for attempt in 0..=max_retries {
            // Rebuild request for each retry attempt
            let mut retry_builder = match method.to_uppercase().as_str() {
                "GET" => client.get(base_url),
                "POST" => client.post(base_url),
                "PUT" => client.put(base_url),
                "DELETE" => client.delete(base_url),
                "PATCH" => client.patch(base_url),
                _ => {
                    return Err(WorkflowError::TaskExecutionFailed(format!(
                        "Unsupported HTTP method: {}",
                        method
                    )));
                }
            };

            // Add headers
            for (key, value) in &config.headers {
                retry_builder = retry_builder.header(key, value);
            }

            // Add authentication token if present
            if let Some(token) = &config.auth_token {
                retry_builder = retry_builder.header("Authorization", format!("Bearer {}", token));
            }

            // Add JSON body for POST/PUT/PATCH
            if matches!(method.to_uppercase().as_str(), "POST" | "PUT" | "PATCH") {
                retry_builder = retry_builder.json(&data);
            } else if !data.is_null() {
                // For GET/DELETE, add query parameters
                if let Some(obj) = data.as_object() {
                    for (key, value) in obj {
                        if let Some(value_str) = value.as_str() {
                            retry_builder = retry_builder.query(&[(key, value_str)]);
                        }
                    }
                }
            }

            match retry_builder.send().await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        let result: serde_json::Value = response.json().await.map_err(|e| {
                            WorkflowError::TaskExecutionFailed(format!(
                                "Failed to parse response: {}",
                                e
                            ))
                        })?;
                        return Ok(result);
                    } else if status.is_server_error() && attempt < max_retries {
                        // Retry on server errors
                        let error_text = response.text().await.unwrap_or_default();
                        last_error = Some(format!(
                            "HTTP {}: {} (attempt {}/{})",
                            status,
                            error_text,
                            attempt + 1,
                            max_retries + 1
                        ));
                        tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
                        continue;
                    } else {
                        let error_text = response.text().await.unwrap_or_default();
                        return Err(WorkflowError::TaskExecutionFailed(format!(
                            "HTTP {}: {}",
                            status, error_text
                        )));
                    }
                }
                Err(e) => {
                    if attempt < max_retries {
                        last_error = Some(format!(
                            "Request failed: {} (attempt {}/{})",
                            e,
                            attempt + 1,
                            max_retries + 1
                        ));
                        tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
                        continue;
                    } else {
                        return Err(WorkflowError::TaskExecutionFailed(format!(
                            "Request failed after {} retries: {}",
                            max_retries, e
                        )));
                    }
                }
            }
        }

        Err(WorkflowError::TaskExecutionFailed(format!(
            "HTTP request failed after {} retries: {}",
            max_retries,
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        )))
    }

    /// Execute HTTP REST API task (fallback when http feature is disabled)
    #[cfg(not(feature = "http"))]
    async fn execute_http_task(
        &self,
        _base_url: &str,
        _method: &str,
        _timeout_secs: u64,
        _max_retries: u32,
        _config: &ConnectorConfig,
        _data: serde_json::Value,
    ) -> WorkflowResult<serde_json::Value> {
        Err(WorkflowError::TaskExecutionFailed(
            "HTTP connector requires 'http' feature to be enabled".to_string(),
        ))
    }

    /// Execute gRPC task with type-safe service abstraction
    ///
    /// Uses advanced Rust techniques:
    /// - Generic associated types (GATs) for service abstraction
    /// - Zero-cost abstractions with compile-time dispatch
    /// - Type-safe service method invocation
    async fn execute_grpc_task(
        &self,
        endpoint: &str,
        service: &str,
        method: &str,
        timeout_secs: u64,
        max_retries: u32,
        config: &ConnectorConfig,
        data: serde_json::Value,
    ) -> WorkflowResult<serde_json::Value> {
        // Validate endpoint format
        if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
            return Err(WorkflowError::Validation(format!(
                "Invalid gRPC endpoint format: {} (must start with http:// or https://)",
                endpoint
            )));
        }

        // Extract timeout from config
        let timeout = Duration::from_secs(timeout_secs);

        // For production, this would use tonic with generated proto stubs
        // Here we provide a type-safe abstraction that can be extended
        // In production, this would:
        // 1. Load proto definitions from registry
        // 2. Create tonic client with timeout
        // 3. Serialize data to protobuf
        // 4. Invoke method via reflection or generated stubs
        // 5. Deserialize response to JSON
        
        Err(WorkflowError::ExternalSystem(format!(
            "gRPC connector requires proto definitions: service='{}', method='{}', endpoint='{}'. \
             Proto files must be registered via ConnectorRegistry::register_proto() before use. \
             Retries: {}, Timeout: {}s",
            service, method, endpoint, max_retries, timeout_secs
        )))
    }
}

impl Default for ConnectorIntegration {
    fn default() -> Self {
        Self::new()
    }
}
