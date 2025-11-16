// Connector Registry
//
// Manages lifecycle and dispatch of connectors.

use crate::connectors::core::{ConnectorMetadata, DynamicConnector};
use crate::connectors::error::{ConnectorError, RegistryError};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

/// Health status for a connector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub healthy: bool,
    pub details: String,
}

/// Overall health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub connectors: Vec<ConnectorHealth>,
    pub overall_healthy: bool,
}

/// Individual connector health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorHealth {
    pub name: String,
    pub healthy: bool,
    pub details: String,
}

/// Connector registry for managing lifecycle and dispatch
pub struct ConnectorRegistry {
    connectors: DashMap<String, Arc<tokio::sync::RwLock<Box<dyn DynamicConnector>>>>,
}

impl ConnectorRegistry {
    /// Create a new connector registry
    pub fn new() -> Self {
        Self {
            connectors: DashMap::new(),
        }
    }

    /// Register a connector
    #[instrument(skip(self, connector), fields(name = %name))]
    pub async fn register_connector(
        &self,
        name: String,
        connector: Box<dyn DynamicConnector>,
    ) -> Result<(), RegistryError> {
        if self.connectors.contains_key(&name) {
            warn!(name = %name, "Connector already registered");
            return Err(RegistryError::AlreadyRegistered(name));
        }

        info!(name = %name, "Registering connector");

        // Initialize the connector
        let mut connector_box = connector;
        connector_box.initialize_dynamic().await
            .map_err(|e| RegistryError::InitializationFailed(e.to_string()))?;

        self.connectors.insert(name.clone(), Arc::new(tokio::sync::RwLock::new(connector_box)));

        info!(name = %name, "Connector registered successfully");

        Ok(())
    }

    /// Unregister a connector
    #[instrument(skip(self), fields(name = %name))]
    pub async fn unregister_connector(&self, name: &str) -> Result<(), RegistryError> {
        info!(name = %name, "Unregistering connector");

        let connector = self.connectors.remove(name)
            .ok_or_else(|| RegistryError::NotFound(name.to_string()))?;

        // Shutdown the connector
        let mut connector_guard = connector.1.write().await;
        connector_guard.shutdown_dynamic().await
            .map_err(|e| RegistryError::InitializationFailed(e.to_string()))?;

        info!(name = %name, "Connector unregistered successfully");

        Ok(())
    }

    /// Execute a connector
    #[instrument(skip(self, input), fields(name = %name))]
    pub async fn execute_connector(
        &self,
        name: &str,
        input: serde_json::Value,
    ) -> Result<serde_json::Value, ConnectorError> {
        debug!(name = %name, "Looking up connector");

        let connector = self.connectors.get(name)
            .ok_or_else(|| ConnectorError::NotFound(name.to_string()))?;

        let connector_clone = Arc::clone(connector.value());
        drop(connector); // Release the DashMap lock

        info!(name = %name, "Executing connector");

        let connector_guard = connector_clone.read().await;

        // Check health before execution
        if !connector_guard.is_healthy_dynamic() {
            warn!(name = %name, "Connector is unhealthy");
            return Err(ConnectorError::HealthCheckFailed(name.to_string()));
        }

        let result = connector_guard.execute_dynamic(input).await
            .map_err(|e| ConnectorError::Execution(e.to_string()))?;

        info!(name = %name, "Connector execution completed");

        Ok(result)
    }

    /// Get connector metadata
    pub async fn get_metadata(&self, name: &str) -> Result<ConnectorMetadata, RegistryError> {
        let connector = self.connectors.get(name)
            .ok_or_else(|| RegistryError::NotFound(name.to_string()))?;

        let connector_guard = connector.value().read().await;
        Ok(connector_guard.metadata())
    }

    /// List all registered connectors
    pub fn list_connectors(&self) -> Vec<String> {
        self.connectors.iter()
            .map(|entry| entry.key().clone())
            .collect()
    }

    /// Health check for a specific connector
    #[instrument(skip(self), fields(name = %name))]
    pub async fn health_check(&self, name: &str) -> Result<HealthStatus, RegistryError> {
        let connector = self.connectors.get(name)
            .ok_or_else(|| RegistryError::NotFound(name.to_string()))?;

        let connector_guard = connector.value().read().await;
        let healthy = connector_guard.is_healthy_dynamic();

        Ok(HealthStatus {
            healthy,
            details: if healthy {
                "Connector is healthy".to_string()
            } else {
                "Connector is unhealthy".to_string()
            },
        })
    }

    /// Health check for all connectors
    #[instrument(skip(self))]
    pub async fn health_check_all(&self) -> HealthReport {
        let mut connectors = Vec::new();
        let mut all_healthy = true;

        for entry in self.connectors.iter() {
            let name = entry.key().clone();
            let connector = entry.value();

            let connector_guard = connector.read().await;
            let healthy = connector_guard.is_healthy_dynamic();

            if !healthy {
                all_healthy = false;
            }

            connectors.push(ConnectorHealth {
                name,
                healthy,
                details: if healthy {
                    "OK".to_string()
                } else {
                    "UNHEALTHY".to_string()
                },
            });
        }

        HealthReport {
            connectors,
            overall_healthy: all_healthy,
        }
    }

    /// Shutdown all connectors
    #[instrument(skip(self))]
    pub async fn shutdown_all(&self) -> Result<(), RegistryError> {
        info!("Shutting down all connectors");

        let names: Vec<String> = self.list_connectors();

        for name in names {
            if let Err(e) = self.unregister_connector(&name).await {
                warn!(name = %name, error = %e, "Failed to shutdown connector");
            }
        }

        info!("All connectors shut down");

        Ok(())
    }
}

impl Default for ConnectorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connectors::core::{Connector, AsyncConnector};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct TestInput {
        value: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct TestOutput {
        result: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct TestConfig;

    #[derive(Debug)]
    struct TestError;

    impl std::fmt::Display for TestError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Test error")
        }
    }

    impl std::error::Error for TestError {}

    struct TestConnector {
        name: String,
    }

    impl Connector for TestConnector {
        type Config = TestConfig;
        type Input = TestInput;
        type Output = TestOutput;
        type Error = TestError;

        fn execute(
            &self,
            input: Self::Input,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Output, Self::Error>> + Send + '_>> {
            Box::pin(async move {
                Ok(TestOutput {
                    result: format!("Processed: {}", input.value),
                })
            })
        }

        fn name(&self) -> &str {
            &self.name
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

    #[tokio::test]
    async fn test_registry_register_and_execute() {
        let registry = ConnectorRegistry::new();

        let connector = Box::new(TestConnector {
            name: "test".to_string(),
        });

        registry.register_connector("test".to_string(), connector).await.unwrap();

        let input = serde_json::json!({"value": "Hello"});
        let output = registry.execute_connector("test", input).await.unwrap();

        assert_eq!(output["result"], "Processed: Hello");
    }

    #[tokio::test]
    async fn test_registry_duplicate_registration() {
        let registry = ConnectorRegistry::new();

        let connector1 = Box::new(TestConnector {
            name: "test".to_string(),
        });

        registry.register_connector("test".to_string(), connector1).await.unwrap();

        let connector2 = Box::new(TestConnector {
            name: "test".to_string(),
        });

        let result = registry.register_connector("test".to_string(), connector2).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_registry_not_found() {
        let registry = ConnectorRegistry::new();

        let input = serde_json::json!({"value": "test"});
        let result = registry.execute_connector("nonexistent", input).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_registry_list_connectors() {
        let registry = ConnectorRegistry::new();

        registry.register_connector("test1".to_string(), Box::new(TestConnector {
            name: "test1".to_string(),
        })).await.unwrap();

        registry.register_connector("test2".to_string(), Box::new(TestConnector {
            name: "test2".to_string(),
        })).await.unwrap();

        let connectors = registry.list_connectors();
        assert_eq!(connectors.len(), 2);
        assert!(connectors.contains(&"test1".to_string()));
        assert!(connectors.contains(&"test2".to_string()));
    }

    #[tokio::test]
    async fn test_registry_health_check() {
        let registry = ConnectorRegistry::new();

        registry.register_connector("test".to_string(), Box::new(TestConnector {
            name: "test".to_string(),
        })).await.unwrap();

        let health = registry.health_check("test").await.unwrap();
        assert!(health.healthy);
    }

    #[tokio::test]
    async fn test_registry_health_check_all() {
        let registry = ConnectorRegistry::new();

        registry.register_connector("test1".to_string(), Box::new(TestConnector {
            name: "test1".to_string(),
        })).await.unwrap();

        registry.register_connector("test2".to_string(), Box::new(TestConnector {
            name: "test2".to_string(),
        })).await.unwrap();

        let report = registry.health_check_all().await;
        assert!(report.overall_healthy);
        assert_eq!(report.connectors.len(), 2);
    }

    #[tokio::test]
    async fn test_registry_unregister() {
        let registry = ConnectorRegistry::new();

        registry.register_connector("test".to_string(), Box::new(TestConnector {
            name: "test".to_string(),
        })).await.unwrap();

        registry.unregister_connector("test").await.unwrap();

        let connectors = registry.list_connectors();
        assert_eq!(connectors.len(), 0);
    }

    #[tokio::test]
    async fn test_registry_shutdown_all() {
        let registry = ConnectorRegistry::new();

        registry.register_connector("test1".to_string(), Box::new(TestConnector {
            name: "test1".to_string(),
        })).await.unwrap();

        registry.register_connector("test2".to_string(), Box::new(TestConnector {
            name: "test2".to_string(),
        })).await.unwrap();

        registry.shutdown_all().await.unwrap();

        let connectors = registry.list_connectors();
        assert_eq!(connectors.len(), 0);
    }
}
