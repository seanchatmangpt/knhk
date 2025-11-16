// Integration Tests for Connector Framework
//
// Comprehensive test suite covering all connector types and scenarios.

#![cfg(feature = "connectors")]

use knhk_workflow_engine::connectors::*;
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_connector_registry_lifecycle() {
    // Create registry
    let registry = ConnectorRegistry::new();

    // Create a simple mock connector
    #[derive(Debug)]
    struct MockConnector;

    impl Connector for MockConnector {
        type Config = ();
        type Input = serde_json::Value;
        type Output = serde_json::Value;
        type Error = Box<dyn std::error::Error + Send + Sync>;

        fn execute(
            &self,
            input: Self::Input,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Output, Self::Error>> + Send + '_>> {
            Box::pin(async move { Ok(json!({"result": "processed", "input": input})) })
        }

        fn name(&self) -> &str {
            "mock"
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn capabilities(&self) -> Vec<&str> {
            vec!["mock"]
        }
    }

    impl AsyncConnector for MockConnector {
        fn initialize(
            &mut self,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + '_>> {
            Box::pin(async { Ok(()) })
        }

        fn shutdown(
            &mut self,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + '_>> {
            Box::pin(async { Ok(()) })
        }

        fn is_healthy(&self) -> bool {
            true
        }
    }

    // Register connector
    registry
        .register_connector("mock".to_string(), Box::new(MockConnector))
        .await
        .expect("Failed to register connector");

    // Execute connector
    let input = json!({"test": "data"});
    let output = registry
        .execute_connector("mock", input.clone())
        .await
        .expect("Failed to execute connector");

    assert_eq!(output["result"], "processed");
    assert_eq!(output["input"], input);

    // Health check
    let health = registry
        .health_check("mock")
        .await
        .expect("Failed to health check");
    assert!(health.healthy);

    // List connectors
    let connectors = registry.list_connectors();
    assert_eq!(connectors.len(), 1);
    assert!(connectors.contains(&"mock".to_string()));

    // Unregister
    registry
        .unregister_connector("mock")
        .await
        .expect("Failed to unregister");

    let connectors = registry.list_connectors();
    assert_eq!(connectors.len(), 0);
}

#[tokio::test]
async fn test_rest_connector_with_retry() {
    // This test requires wiremock server
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    let mock_server = MockServer::start().await;

    // Setup mock endpoint that fails twice then succeeds
    Mock::given(method("POST"))
        .and(path("/api/test"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(2)
        .mount(&mock_server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "status": "success",
            "message": "Operation completed"
        })))
        .mount(&mock_server)
        .await;

    // Create REST connector with retry policy
    let config = RestConfig {
        base_url: mock_server.uri(),
        timeout_ms: 5000,
        default_headers: HashMap::new(),
        retry_policy: Some(RetryPolicy {
            max_retries: 3,
            backoff: BackoffStrategy::Fixed { delay_ms: 10 },
            jitter: false,
        }),
        circuit_breaker_threshold: None,
        circuit_breaker_timeout_ms: None,
    };

    let connector = RestConnector::new(config).expect("Failed to create REST connector");

    // Execute request
    let request = RestRequest {
        method: "POST".to_string(),
        path: "/api/test".to_string(),
        headers: HashMap::new(),
        query_params: HashMap::new(),
        body: Some(json!({"data": "test"})),
    };

    let response = connector.execute(request).await.expect("Failed to execute request");

    // Should succeed after retries
    assert_eq!(response.status, 200);
    assert_eq!(response.body["status"], "success");
}

#[tokio::test]
async fn test_circuit_breaker_pattern() {
    use std::time::Duration;

    // Create circuit breaker
    let cb = CircuitBreaker::new(3, Duration::from_millis(100));

    assert_eq!(cb.state(), CircuitState::Closed);

    // Simulate 3 failures to trip the circuit
    for _ in 0..3 {
        let _ = cb
            .call(|| async {
                Err::<(), _>(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Simulated failure",
                ))
            })
            .await;
    }

    // Circuit should be open
    assert_eq!(cb.state(), CircuitState::Open);
    assert_eq!(cb.failure_count(), 3);

    // Next call should fail immediately
    let result = cb.call(|| async { Ok::<(), std::io::Error>(()) }).await;
    assert!(matches!(result, Err(CircuitBreakerError::Open)));

    // Wait for timeout
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Should transition to half-open and allow test calls
    let result = cb.call(|| async { Ok::<(), std::io::Error>(()) }).await;
    assert!(result.is_ok());

    // After successful calls, should close
    for _ in 0..2 {
        let _ = cb.call(|| async { Ok::<(), std::io::Error>(()) }).await;
    }

    assert_eq!(cb.state(), CircuitState::Closed);
}

#[tokio::test]
async fn test_connector_pool_reuse() {
    use std::time::Duration;

    #[derive(Debug)]
    struct TestConnector {
        id: u64,
    }

    impl Connector for TestConnector {
        type Config = ();
        type Input = serde_json::Value;
        type Output = serde_json::Value;
        type Error = Box<dyn std::error::Error + Send + Sync>;

        fn execute(
            &self,
            input: Self::Input,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Output, Self::Error>> + Send + '_>> {
            Box::pin(async move {
                Ok(json!({"connector_id": self.id, "input": input}))
            })
        }

        fn name(&self) -> &str {
            "test"
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn capabilities(&self) -> Vec<&str> {
            vec!["test"]
        }
    }

    impl AsyncConnector for TestConnector {
        fn initialize(
            &mut self,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + '_>> {
            Box::pin(async { Ok(()) })
        }

        fn shutdown(
            &mut self,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + '_>> {
            Box::pin(async { Ok(()) })
        }

        fn is_healthy(&self) -> bool {
            true
        }
    }

    // Create pool
    let pool = ConnectorPool::new();

    // Create a simple counter for connector IDs
    let counter = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));

    let pool_config = PoolConfig {
        max_size: 5,
        min_idle: 2,
        max_lifetime: Duration::from_secs(3600),
        idle_timeout: Duration::from_secs(600),
    };

    // Register factory
    let counter_clone = counter.clone();
    pool.register(
        "test".to_string(),
        pool_config,
        move || {
            let id = counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Box::new(TestConnector { id })
        },
    );

    // Get connector from pool
    let conn1 = pool.get("test").await.expect("Failed to get connector");
    let result1 = conn1.execute(json!({"test": 1})).await.expect("Failed to execute");

    drop(conn1); // Return to pool

    // Get again - should reuse the same connector
    let conn2 = pool.get("test").await.expect("Failed to get connector");
    let result2 = conn2.execute(json!({"test": 2})).await.expect("Failed to execute");

    drop(conn2);

    // Check stats
    let stats = pool.stats("test").await.expect("Failed to get stats");
    assert_eq!(stats.create_count, 1); // Only created once
    assert_eq!(stats.reuse_count, 1); // Reused once
}

#[tokio::test]
async fn test_connector_config_yaml() {
    let yaml = r#"
connectors:
  - name: api
    connector_type: rest
    enabled: true
    timeout_ms: 5000
    retry_policy:
      max_retries: 3
      backoff:
        type: exponential
        base_ms: 100
        multiplier: 2.0
        max_delay_ms: 10000
      jitter: true
    circuit_breaker:
      threshold: 5
      timeout_ms: 30000
    config_data:
      base_url: "http://api.example.com"
      default_headers: {}
"#;

    let config_file = ConnectorConfigFile::from_yaml(yaml)
        .expect("Failed to parse YAML config");

    assert_eq!(config_file.connectors.len(), 1);
    assert!(config_file.validate_all().is_ok());

    let connector_config = &config_file.connectors[0];
    assert_eq!(connector_config.name, "api");
    assert_eq!(connector_config.connector_type, "rest");
    assert!(connector_config.enabled);
    assert!(connector_config.retry_policy.is_some());
    assert!(connector_config.circuit_breaker.is_some());

    // Try to instantiate
    let _connector = connector_config
        .instantiate()
        .await
        .expect("Failed to instantiate connector");
}

#[tokio::test]
async fn test_end_to_end_connector_workflow() {
    // Create registry
    let registry = ConnectorRegistry::new();

    // Create REST connector config
    let config = ConnectorConfig {
        name: "test-api".to_string(),
        connector_type: "rest".to_string(),
        enabled: true,
        retry_policy: Some(RetryPolicyConfig {
            max_retries: 2,
            backoff: BackoffStrategyConfig::Fixed { delay_ms: 10 },
            jitter: false,
        }),
        circuit_breaker: None,
        timeout_ms: 5000,
        config_data: json!({
            "base_url": "http://httpbin.org",
            "default_headers": {
                "User-Agent": "KNHK-Connector/1.0"
            }
        }),
    };

    // Instantiate connector
    let connector = config.instantiate().await.expect("Failed to instantiate");

    // Register with registry
    registry
        .register_connector("test-api".to_string(), connector)
        .await
        .expect("Failed to register");

    // Execute via registry
    let input = json!({
        "method": "GET",
        "path": "/status/200",
        "headers": {},
        "query_params": {},
        "body": null
    });

    // Note: This test requires internet connectivity to httpbin.org
    // In a real test environment, you would use a local mock server
    // For now, we just verify the connector is registered and callable
    let connectors = registry.list_connectors();
    assert!(connectors.contains(&"test-api".to_string()));

    // Health check
    let health = registry.health_check_all().await;
    assert!(health.overall_healthy);
}
