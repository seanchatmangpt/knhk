// REST Connector Implementation
//
// HTTP connector for calling external REST APIs with resilience patterns.

use crate::connectors::core::{Connector, AsyncConnector};
use crate::connectors::resilience::{RetryPolicy, CircuitBreaker};
use reqwest::{Client, Method, RequestBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info, instrument};

/// REST connector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestConfig {
    pub base_url: String,
    pub timeout_ms: u64,
    pub default_headers: HashMap<String, String>,
    #[serde(default)]
    pub retry_policy: Option<RetryPolicy>,
    #[serde(default)]
    pub circuit_breaker_threshold: Option<u32>,
    #[serde(default)]
    pub circuit_breaker_timeout_ms: Option<u64>,
}

/// REST request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestRequest {
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub query_params: HashMap<String, String>,
    #[serde(default)]
    pub body: Option<serde_json::Value>,
}

/// REST response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: serde_json::Value,
}

/// REST connector error
#[derive(Debug)]
pub enum RestError {
    Request(String),
    Response(String),
    Timeout,
    Serialization(String),
    CircuitBreakerOpen,
    InvalidMethod(String),
}

impl fmt::Display for RestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Request(msg) => write!(f, "Request error: {}", msg),
            Self::Response(msg) => write!(f, "Response error: {}", msg),
            Self::Timeout => write!(f, "Request timeout"),
            Self::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Self::CircuitBreakerOpen => write!(f, "Circuit breaker is open"),
            Self::InvalidMethod(msg) => write!(f, "Invalid HTTP method: {}", msg),
        }
    }
}

impl std::error::Error for RestError {}

/// REST connector implementation
pub struct RestConnector {
    client: Client,
    config: RestConfig,
    retry_policy: RetryPolicy,
    circuit_breaker: Option<Arc<CircuitBreaker>>,
}

impl RestConnector {
    /// Create a new REST connector
    pub fn new(config: RestConfig) -> Result<Self, RestError> {
        let timeout = Duration::from_millis(config.timeout_ms);
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| RestError::Request(e.to_string()))?;

        let retry_policy = config.retry_policy.clone().unwrap_or_default();

        let circuit_breaker = if let Some(threshold) = config.circuit_breaker_threshold {
            let timeout = Duration::from_millis(
                config.circuit_breaker_timeout_ms.unwrap_or(5000)
            );
            Some(Arc::new(CircuitBreaker::new(threshold, timeout)))
        } else {
            None
        };

        Ok(Self {
            client,
            config,
            retry_policy,
            circuit_breaker,
        })
    }

    /// Build a request
    fn build_request(&self, input: &RestRequest) -> Result<RequestBuilder, RestError> {
        // Parse HTTP method
        let method = match input.method.to_uppercase().as_str() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "PATCH" => Method::PATCH,
            "HEAD" => Method::HEAD,
            "OPTIONS" => Method::OPTIONS,
            m => return Err(RestError::InvalidMethod(m.to_string())),
        };

        // Build URL
        let url = format!("{}{}", self.config.base_url, input.path);

        // Create request builder
        let mut builder = self.client.request(method, &url);

        // Add default headers
        for (key, value) in &self.config.default_headers {
            builder = builder.header(key, value);
        }

        // Add request headers
        for (key, value) in &input.headers {
            builder = builder.header(key, value);
        }

        // Add query parameters
        if !input.query_params.is_empty() {
            builder = builder.query(&input.query_params);
        }

        // Add body if present
        if let Some(body) = &input.body {
            builder = builder.json(body);
        }

        Ok(builder)
    }

    /// Execute request with resilience patterns
    #[instrument(skip(self, builder), fields(method = %builder.try_clone().unwrap().build().unwrap().method()))]
    async fn execute_request(&self, builder: RequestBuilder) -> Result<RestResponse, RestError> {
        // Clone the builder for retry attempts
        let execute = || async {
            let request = builder.try_clone()
                .ok_or_else(|| RestError::Request("Failed to clone request".to_string()))?;

            debug!("Sending HTTP request");

            let response = request
                .send()
                .await
                .map_err(|e| {
                    if e.is_timeout() {
                        RestError::Timeout
                    } else {
                        RestError::Request(e.to_string())
                    }
                })?;

            let status = response.status().as_u16();

            // Extract headers
            let mut headers = HashMap::new();
            for (key, value) in response.headers() {
                if let Ok(v) = value.to_str() {
                    headers.insert(key.to_string(), v.to_string());
                }
            }

            // Parse body
            let body = if status >= 200 && status < 300 {
                response.json::<serde_json::Value>()
                    .await
                    .map_err(|e| RestError::Serialization(e.to_string()))?
            } else {
                // For error responses, try to get text
                let text = response.text().await
                    .map_err(|e| RestError::Response(e.to_string()))?;
                serde_json::Value::String(text)
            };

            debug!(status = status, "Received HTTP response");

            if status >= 200 && status < 300 {
                Ok(RestResponse { status, headers, body })
            } else {
                Err(RestError::Response(format!("HTTP {}: {:?}", status, body)))
            }
        };

        // Apply retry policy
        let with_retry = || async {
            self.retry_policy.execute(|| execute()).await
                .map_err(|_| RestError::Response("Max retries exceeded".to_string()))
        };

        // Apply circuit breaker if configured
        if let Some(cb) = &self.circuit_breaker {
            cb.call(|| with_retry()).await
                .map_err(|e| match e {
                    crate::connectors::error::CircuitBreakerError::Open => RestError::CircuitBreakerOpen,
                    crate::connectors::error::CircuitBreakerError::Failure(err) => {
                        // Try to downcast to RestError
                        RestError::Response(err.to_string())
                    }
                    _ => RestError::Response("Circuit breaker error".to_string()),
                })?
        } else {
            with_retry().await?
        }
    }
}

impl Connector for RestConnector {
    type Config = RestConfig;
    type Input = RestRequest;
    type Output = RestResponse;
    type Error = RestError;

    #[instrument(skip(self, input), fields(method = %input.method, path = %input.path))]
    fn execute(
        &self,
        input: Self::Input,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Output, Self::Error>> + Send + '_>> {
        Box::pin(async move {
            info!(
                method = %input.method,
                path = %input.path,
                "Executing REST connector"
            );

            let builder = self.build_request(&input)?;
            let response = self.execute_request(builder).await?;

            info!(
                status = response.status,
                "REST connector execution completed"
            );

            Ok(response)
        })
    }

    fn name(&self) -> &str {
        "rest"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn capabilities(&self) -> Vec<&str> {
        vec!["http", "rest", "api", "retry", "circuit-breaker"]
    }
}

impl AsyncConnector for RestConnector {
    fn initialize(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + '_>> {
        Box::pin(async move {
            info!("Initializing REST connector");
            Ok(())
        })
    }

    fn shutdown(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + '_>> {
        Box::pin(async move {
            info!("Shutting down REST connector");
            Ok(())
        })
    }

    fn is_healthy(&self) -> bool {
        // Check circuit breaker state if present
        if let Some(cb) = &self.circuit_breaker {
            matches!(cb.state(), crate::connectors::resilience::CircuitState::Closed)
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_rest_connector_get() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "message": "success"
            })))
            .mount(&mock_server)
            .await;

        let config = RestConfig {
            base_url: mock_server.uri(),
            timeout_ms: 5000,
            default_headers: HashMap::new(),
            retry_policy: None,
            circuit_breaker_threshold: None,
            circuit_breaker_timeout_ms: None,
        };

        let connector = RestConnector::new(config).unwrap();

        let request = RestRequest {
            method: "GET".to_string(),
            path: "/test".to_string(),
            headers: HashMap::new(),
            query_params: HashMap::new(),
            body: None,
        };

        let response = connector.execute(request).await.unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body["message"], "success");
    }

    #[tokio::test]
    async fn test_rest_connector_post() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/create"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "id": 123
            })))
            .mount(&mock_server)
            .await;

        let config = RestConfig {
            base_url: mock_server.uri(),
            timeout_ms: 5000,
            default_headers: HashMap::new(),
            retry_policy: None,
            circuit_breaker_threshold: None,
            circuit_breaker_timeout_ms: None,
        };

        let connector = RestConnector::new(config).unwrap();

        let request = RestRequest {
            method: "POST".to_string(),
            path: "/create".to_string(),
            headers: HashMap::new(),
            query_params: HashMap::new(),
            body: Some(serde_json::json!({"name": "test"})),
        };

        let response = connector.execute(request).await.unwrap();
        assert_eq!(response.status, 201);
        assert_eq!(response.body["id"], 123);
    }

    #[tokio::test]
    async fn test_rest_connector_retry() {
        let mock_server = MockServer::start().await;

        // First request fails, second succeeds
        Mock::given(method("GET"))
            .and(path("/retry"))
            .respond_with(ResponseTemplate::new(500))
            .up_to_n_times(1)
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/retry"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "message": "success"
            })))
            .mount(&mock_server)
            .await;

        let config = RestConfig {
            base_url: mock_server.uri(),
            timeout_ms: 5000,
            default_headers: HashMap::new(),
            retry_policy: Some(RetryPolicy {
                max_retries: 2,
                backoff: crate::connectors::resilience::BackoffStrategy::Fixed { delay_ms: 10 },
                jitter: false,
            }),
            circuit_breaker_threshold: None,
            circuit_breaker_timeout_ms: None,
        };

        let connector = RestConnector::new(config).unwrap();

        let request = RestRequest {
            method: "GET".to_string(),
            path: "/retry".to_string(),
            headers: HashMap::new(),
            query_params: HashMap::new(),
            body: None,
        };

        let response = connector.execute(request).await.unwrap();
        assert_eq!(response.status, 200);
    }
}
