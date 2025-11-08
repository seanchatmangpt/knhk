//! Integration registry
//!
//! Provides a centralized registry for all KNHK integrations,
//! allowing discovery and management of integration capabilities.

use crate::error::{WorkflowError, WorkflowResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Integration metadata
#[derive(Debug, Clone)]
pub struct IntegrationMetadata {
    /// Integration name
    pub name: String,
    /// Integration description
    pub description: String,
    /// Integration version
    pub version: String,
    /// Integration capabilities
    pub capabilities: Vec<String>,
    /// Integration status
    pub status: IntegrationStatus,
}

/// Integration status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrationStatus {
    /// Integration is available
    Available,
    /// Integration is enabled
    Enabled,
    /// Integration is disabled
    Disabled,
    /// Integration is not available
    NotAvailable,
}

/// Integration registry
pub struct IntegrationRegistry {
    /// Registered integrations
    integrations: Arc<RwLock<HashMap<String, IntegrationMetadata>>>,
}

impl IntegrationRegistry {
    /// Create new integration registry
    pub fn new() -> Self {
        let mut registry = Self {
            integrations: Arc::new(RwLock::new(HashMap::new())),
        };
        registry.register_all_integrations();
        registry
    }

    /// Register an integration
    pub async fn register(&self, metadata: IntegrationMetadata) {
        let mut integrations = self.integrations.write().await;
        integrations.insert(metadata.name.clone(), metadata);
    }

    /// Get integration metadata
    pub async fn get(&self, name: &str) -> Option<IntegrationMetadata> {
        let integrations = self.integrations.read().await;
        integrations.get(name).cloned()
    }

    /// List all integrations
    pub async fn list(&self) -> Vec<IntegrationMetadata> {
        let integrations = self.integrations.read().await;
        integrations.values().cloned().collect()
    }

    /// List available integrations
    pub async fn list_available(&self) -> Vec<IntegrationMetadata> {
        let integrations = self.integrations.read().await;
        integrations
            .values()
            .filter(|i| {
                i.status == IntegrationStatus::Available || i.status == IntegrationStatus::Enabled
            })
            .cloned()
            .collect()
    }

    /// Enable integration
    pub async fn enable(&self, name: &str) -> WorkflowResult<()> {
        let mut integrations = self.integrations.write().await;
        if let Some(integration) = integrations.get_mut(name) {
            if integration.status == IntegrationStatus::Available {
                integration.status = IntegrationStatus::Enabled;
                Ok(())
            } else {
                Err(WorkflowError::Validation(format!(
                    "Integration {} is not available",
                    name
                )))
            }
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Integration {} not found",
                name
            )))
        }
    }

    /// Disable integration
    pub async fn disable(&self, name: &str) -> WorkflowResult<()> {
        let mut integrations = self.integrations.write().await;
        if let Some(integration) = integrations.get_mut(name) {
            integration.status = IntegrationStatus::Disabled;
            Ok(())
        } else {
            Err(WorkflowError::ResourceUnavailable(format!(
                "Integration {} not found",
                name
            )))
        }
    }

    /// Register all KNHK integrations
    fn register_all_integrations(&mut self) {
        // Fortune 5 integration
        self.register_sync(IntegrationMetadata {
            name: "fortune5".to_string(),
            description: "Fortune 5 enterprise integration (SPIFFE/SPIRE, KMS, SLO)".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![
                "spiffe".to_string(),
                "kms".to_string(),
                "slo".to_string(),
                "multi_region".to_string(),
                "promotion_gates".to_string(),
            ],
            status: IntegrationStatus::Available,
        });

        // Lockchain integration
        self.register_sync(IntegrationMetadata {
            name: "lockchain".to_string(),
            description: "Lockchain integration for receipt storage".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["receipt_storage".to_string(), "provenance".to_string()],
            status: IntegrationStatus::Available,
        });

        // Connector integration
        self.register_sync(IntegrationMetadata {
            name: "connectors".to_string(),
            description: "External connector integration (Kafka, Salesforce, etc.)".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![
                "kafka".to_string(),
                "salesforce".to_string(),
                "custom".to_string(),
            ],
            status: IntegrationStatus::Available,
        });

        // Sidecar integration
        self.register_sync(IntegrationMetadata {
            name: "sidecar".to_string(),
            description: "KNHK sidecar integration for gRPC communication".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![
                "grpc".to_string(),
                "json_parsing".to_string(),
                "simdjson".to_string(),
            ],
            status: IntegrationStatus::Available,
        });

        // ETL integration
        self.register_sync(IntegrationMetadata {
            name: "etl".to_string(),
            description: "ETL pipeline integration (Ingest, Transform, Load, Reflex, Emit)"
                .to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![
                "ingest".to_string(),
                "transform".to_string(),
                "load".to_string(),
                "reflex".to_string(),
                "emit".to_string(),
            ],
            status: IntegrationStatus::Available,
        });

        // OTEL integration
        self.register_sync(IntegrationMetadata {
            name: "otel".to_string(),
            description: "OpenTelemetry integration for observability".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![
                "tracing".to_string(),
                "metrics".to_string(),
                "logging".to_string(),
            ],
            status: IntegrationStatus::Available,
        });
    }

    /// Register integration synchronously (for initialization)
    fn register_sync(&mut self, metadata: IntegrationMetadata) {
        // This is called during initialization, so we can use blocking
        let integrations = Arc::try_unwrap(self.integrations.clone()).unwrap_or_else(|_arc| {
            // If we can't unwrap, create a new one
            RwLock::new(HashMap::new())
        });
        let mut integrations = integrations.into_inner();
        integrations.insert(metadata.name.clone(), metadata);
        self.integrations = Arc::new(RwLock::new(integrations));
    }
}

impl Default for IntegrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}
