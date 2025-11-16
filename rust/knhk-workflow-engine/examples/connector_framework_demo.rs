//! Connector Framework Demo
//!
//! Demonstrates the usage of the connector framework with various connector types.

#![cfg(feature = "connectors")]

use knhk_workflow_engine::connectors::*;
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("=== Connector Framework Demo ===\n");

    // Demo 1: Connector Registry
    demo_connector_registry().await?;

    // Demo 2: REST Connector with Retry
    demo_rest_connector().await?;

    // Demo 3: Circuit Breaker Pattern
    demo_circuit_breaker().await?;

    // Demo 4: Connection Pooling
    demo_connection_pooling().await?;

    // Demo 5: Configuration from YAML
    demo_yaml_configuration().await?;

    println!("\n=== Demo Complete ===");

    Ok(())
}

async fn demo_connector_registry() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Demo 1: Connector Registry ---");

    // Create a simple test connector
    #[derive(Debug)]
    struct EchoConnector;

    impl Connector for EchoConnector {
        type Config = ();
        type Input = serde_json::Value;
        type Output = serde_json::Value;
        type Error = Box<dyn std::error::Error + Send + Sync>;

        fn execute(
            &self,
            input: Self::Input,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Output, Self::Error>> + Send + '_>> {
            Box::pin(async move {
                Ok(json!({
                    "echo": input,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }))
            })
        }

        fn name(&self) -> &str {
            "echo"
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn capabilities(&self) -> Vec<&str> {
            vec!["echo", "test"]
        }
    }

    impl AsyncConnector for EchoConnector {
        fn initialize(
            &mut self,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + '_>> {
            Box::pin(async {
                println!("  Echo connector initialized");
                Ok(())
            })
        }

        fn shutdown(
            &mut self,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + '_>> {
            Box::pin(async {
                println!("  Echo connector shut down");
                Ok(())
            })
        }

        fn is_healthy(&self) -> bool {
            true
        }
    }

    // Create registry and register connector
    let registry = ConnectorRegistry::new();
    registry.register_connector("echo".to_string(), Box::new(EchoConnector)).await?;

    println!("  Registered connectors: {:?}", registry.list_connectors());

    // Execute connector
    let input = json!({"message": "Hello, Connector Framework!"});
    let output = registry.execute_connector("echo", input).await?;

    println!("  Output: {}", serde_json::to_string_pretty(&output)?);

    // Health check
    let health = registry.health_check_all().await;
    println!("  Health: {} connectors, overall healthy: {}",
             health.connectors.len(), health.overall_healthy);

    println!();
    Ok(())
}

async fn demo_rest_connector() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Demo 2: REST Connector with Retry ---");

    // Note: This demo uses httpbin.org for testing
    // In production, you would use your own API endpoints

    let config = RestConfig {
        base_url: "https://httpbin.org".to_string(),
        timeout_ms: 5000,
        default_headers: {
            let mut headers = HashMap::new();
            headers.insert("User-Agent".to_string(), "KNHK-Connector/1.0".to_string());
            headers
        },
        retry_policy: Some(RetryPolicy {
            max_retries: 3,
            backoff: BackoffStrategy::Exponential {
                base_ms: 100,
                multiplier: 2.0,
                max_delay_ms: 5000,
            },
            jitter: true,
        }),
        circuit_breaker_threshold: Some(5),
        circuit_breaker_timeout_ms: Some(10000),
    };

    let connector = RestConnector::new(config)?;

    // Make a GET request
    let request = RestRequest {
        method: "GET".to_string(),
        path: "/status/200".to_string(),
        headers: HashMap::new(),
        query_params: HashMap::new(),
        body: None,
    };

    println!("  Making GET request to https://httpbin.org/status/200");

    let response = connector.execute(request).await?;

    println!("  Status: {}", response.status);
    println!("  Headers: {} headers received", response.headers.len());

    println!();
    Ok(())
}

async fn demo_circuit_breaker() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Demo 3: Circuit Breaker Pattern ---");

    use std::time::Duration;

    let cb = CircuitBreaker::new(3, Duration::from_millis(500));

    println!("  Initial state: {:?}", cb.state());

    // Simulate failures
    println!("  Simulating 3 failures...");
    for i in 0..3 {
        let result = cb.call(|| async {
            Err::<(), _>(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failure {}", i + 1),
            ))
        }).await;

        match result {
            Ok(_) => println!("    Attempt {}: Success", i + 1),
            Err(CircuitBreakerError::Failure(_)) => println!("    Attempt {}: Failed", i + 1),
            Err(CircuitBreakerError::Open) => println!("    Attempt {}: Circuit open", i + 1),
            _ => {}
        }
    }

    println!("  State after failures: {:?}", cb.state());
    println!("  Failure count: {}", cb.failure_count());

    // Try to call - should fail immediately
    let result = cb.call(|| async { Ok::<(), std::io::Error>(()) }).await;
    match result {
        Err(CircuitBreakerError::Open) => println!("  Circuit is open - request rejected"),
        _ => println!("  Unexpected result"),
    }

    // Wait for timeout
    println!("  Waiting for circuit breaker timeout...");
    tokio::time::sleep(Duration::from_millis(600)).await;

    // Should transition to half-open
    println!("  Attempting recovery...");
    for i in 0..3 {
        let result = cb.call(|| async { Ok::<(), std::io::Error>(()) }).await;
        match result {
            Ok(_) => println!("    Recovery attempt {}: Success", i + 1),
            Err(CircuitBreakerError::Open) => println!("    Recovery attempt {}: Circuit open", i + 1),
            _ => {}
        }
    }

    println!("  Final state: {:?}", cb.state());

    println!();
    Ok(())
}

async fn demo_connection_pooling() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Demo 4: Connection Pooling ---");

    use std::time::Duration;

    #[derive(Debug)]
    struct MockDbConnector {
        id: u64,
    }

    impl Connector for MockDbConnector {
        type Config = ();
        type Input = serde_json::Value;
        type Output = serde_json::Value;
        type Error = Box<dyn std::error::Error + Send + Sync>;

        fn execute(
            &self,
            input: Self::Input,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Output, Self::Error>> + Send + '_>> {
            Box::pin(async move {
                // Simulate query execution
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok(json!({
                    "connector_id": self.id,
                    "query_result": input
                }))
            })
        }

        fn name(&self) -> &str {
            "mock_db"
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn capabilities(&self) -> Vec<&str> {
            vec!["database", "mock"]
        }
    }

    impl AsyncConnector for MockDbConnector {
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

    let counter = std::sync::Arc::new(std::sync::atomic::AtomicU64::new(0));

    let config = PoolConfig {
        max_size: 5,
        min_idle: 2,
        max_lifetime: Duration::from_secs(3600),
        idle_timeout: Duration::from_secs(600),
    };

    let counter_clone = counter.clone();
    pool.register(
        "mock_db".to_string(),
        config,
        move || {
            let id = counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            println!("    Creating new connection #{}", id);
            Box::new(MockDbConnector { id })
        },
    );

    // Execute multiple queries
    println!("  Executing queries...");
    for i in 0..3 {
        let conn = pool.get("mock_db").await?;
        let result = conn.execute(json!({"query": format!("SELECT * FROM table_{}", i)})).await?;
        println!("    Query {}: {:?}", i + 1, result);
        // Connection is returned to pool when dropped
    }

    // Check stats
    let stats = pool.stats("mock_db").await?;
    println!("\n  Pool Statistics:");
    println!("    Total connections: {}", stats.total_connections);
    println!("    Idle connections: {}", stats.idle_connections);
    println!("    Active connections: {}", stats.active_connections);
    println!("    Connections created: {}", stats.create_count);
    println!("    Connections reused: {}", stats.reuse_count);

    println!();
    Ok(())
}

async fn demo_yaml_configuration() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Demo 5: Configuration from YAML ---");

    let yaml_config = r#"
connectors:
  - name: production-api
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
      base_url: "https://api.example.com"
      default_headers:
        Authorization: "Bearer token"
        Content-Type: "application/json"
  - name: analytics-db
    connector_type: database
    enabled: true
    timeout_ms: 10000
    config_data:
      connection_string: "postgres://localhost/analytics"
      max_connections: 20
      min_connections: 5
      connection_timeout_ms: 5000
      idle_timeout_ms: 60000
"#;

    let config_file = ConnectorConfigFile::from_yaml(yaml_config)?;

    println!("  Loaded {} connector configurations", config_file.connectors.len());

    for connector_config in &config_file.connectors {
        println!("\n  Connector: {}", connector_config.name);
        println!("    Type: {}", connector_config.connector_type);
        println!("    Enabled: {}", connector_config.enabled);
        println!("    Timeout: {}ms", connector_config.timeout_ms);
        println!("    Has retry policy: {}", connector_config.retry_policy.is_some());
        println!("    Has circuit breaker: {}", connector_config.circuit_breaker.is_some());
    }

    // Validate all configurations
    config_file.validate_all()?;
    println!("\n  All configurations validated successfully");

    println!();
    Ok(())
}
