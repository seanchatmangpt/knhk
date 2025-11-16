// Core Connector Trait Hierarchy
//
// Defines the fundamental traits for all connectors in the framework.
// Uses associated types for maximum type safety and zero-cost abstractions.

use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

/// Core connector trait that all connectors must implement.
///
/// Uses associated types to ensure type safety while maintaining flexibility.
/// The trait is object-safe by avoiding generics in method signatures.
pub trait Connector: Send + Sync {
    /// Configuration type for this connector
    type Config: for<'de> Deserialize<'de> + Send + Sync;

    /// Input type for execute operations
    type Input: Serialize + Send + Sync;

    /// Output type from execute operations
    type Output: for<'de> Deserialize<'de> + Send + Sync;

    /// Error type for this connector
    type Error: std::error::Error + Send + Sync + 'static;

    /// Execute the connector with the given input
    fn execute(
        &self,
        input: Self::Input,
    ) -> Pin<Box<dyn Future<Output = Result<Self::Output, Self::Error>> + Send + '_>>;

    /// Get the connector name
    fn name(&self) -> &str;

    /// Get the connector version
    fn version(&self) -> &str;

    /// Get the connector capabilities
    fn capabilities(&self) -> Vec<&str>;
}

/// Async connector trait for lifecycle management
pub trait AsyncConnector: Send + Sync {
    /// Initialize the connector
    fn initialize(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + '_>>;

    /// Shutdown the connector gracefully
    fn shutdown(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + '_>>;

    /// Check if the connector is healthy
    fn is_healthy(&self) -> bool;
}

/// Dynamic connector trait for runtime polymorphism
///
/// This trait enables storing different connector types in the same collection
/// by using serde_json::Value for input/output serialization.
pub trait DynamicConnector: Send + Sync {
    /// Execute with JSON input/output
    fn execute_dynamic(
        &self,
        input: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>> + Send + '_>>;

    /// Get connector metadata
    fn metadata(&self) -> ConnectorMetadata;

    /// Initialize the connector
    fn initialize_dynamic(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + '_>>;

    /// Shutdown the connector
    fn shutdown_dynamic(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + '_>>;

    /// Health check
    fn is_healthy_dynamic(&self) -> bool;
}

/// Connector metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorMetadata {
    pub name: String,
    pub version: String,
    pub connector_type: String,
    pub capabilities: Vec<String>,
    pub description: String,
}

/// Blanket implementation of DynamicConnector for all Connectors
impl<T> DynamicConnector for T
where
    T: Connector + AsyncConnector,
    T::Input: for<'de> Deserialize<'de>,
    T::Output: Serialize,
{
    fn execute_dynamic(
        &self,
        input: serde_json::Value,
    ) -> Pin<Box<dyn Future<Output = Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>>> + Send + '_>> {
        Box::pin(async move {
            // Deserialize input
            let typed_input: T::Input = serde_json::from_value(input)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

            // Execute
            let output = self.execute(typed_input).await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

            // Serialize output
            let json_output = serde_json::to_value(output)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

            Ok(json_output)
        })
    }

    fn metadata(&self) -> ConnectorMetadata {
        ConnectorMetadata {
            name: self.name().to_string(),
            version: self.version().to_string(),
            connector_type: std::any::type_name::<T>().to_string(),
            capabilities: self.capabilities().into_iter().map(|s| s.to_string()).collect(),
            description: format!("Connector: {}", self.name()),
        }
    }

    fn initialize_dynamic(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + '_>> {
        self.initialize()
    }

    fn shutdown_dynamic(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + '_>> {
        self.shutdown()
    }

    fn is_healthy_dynamic(&self) -> bool {
        self.is_healthy()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize)]
    struct TestInput {
        value: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct TestOutput {
        result: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct TestConfig {
        prefix: String,
    }

    #[derive(Debug)]
    struct TestError;

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Test error")
        }
    }

    impl std::error::Error for TestError {}

    struct TestConnector {
        config: TestConfig,
    }

    impl Connector for TestConnector {
        type Config = TestConfig;
        type Input = TestInput;
        type Output = TestOutput;
        type Error = TestError;

        fn execute(
            &self,
            input: Self::Input,
        ) -> Pin<Box<dyn Future<Output = Result<Self::Output, Self::Error>> + Send + '_>> {
            Box::pin(async move {
                Ok(TestOutput {
                    result: format!("{}{}", self.config.prefix, input.value),
                })
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
        ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + '_>> {
            Box::pin(async { Ok(()) })
        }

        fn shutdown(
            &mut self,
        ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + '_>> {
            Box::pin(async { Ok(()) })
        }

        fn is_healthy(&self) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn test_connector_execution() {
        let connector = TestConnector {
            config: TestConfig {
                prefix: "Hello ".to_string(),
            },
        };

        let input = TestInput {
            value: "World".to_string(),
        };

        let output = connector.execute(input).await.unwrap();
        assert_eq!(output.result, "Hello World");
    }

    #[tokio::test]
    async fn test_dynamic_connector() {
        let mut connector = TestConnector {
            config: TestConfig {
                prefix: "Dynamic ".to_string(),
            },
        };

        // Initialize
        connector.initialize_dynamic().await.unwrap();

        // Execute with JSON
        let input = serde_json::json!({ "value": "Test" });
        let output = connector.execute_dynamic(input).await.unwrap();

        assert_eq!(output["result"], "Dynamic Test");
        assert!(connector.is_healthy_dynamic());

        // Shutdown
        connector.shutdown_dynamic().await.unwrap();
    }

    #[test]
    fn test_connector_metadata() {
        let connector = TestConnector {
            config: TestConfig {
                prefix: "Meta ".to_string(),
            },
        };

        let metadata = connector.metadata();
        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.capabilities, vec!["test"]);
    }
}
