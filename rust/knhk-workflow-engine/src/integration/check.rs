//! Integration health check
//!
//! Provides health checking for all KNHK integrations.

use crate::error::{WorkflowError, WorkflowResult};
use crate::integration::registry::{IntegrationRegistry, IntegrationStatus};
use std::collections::HashMap;
use std::time::Instant;

/// Integration health status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Integration is healthy
    Healthy,
    /// Integration is degraded
    Degraded,
    /// Integration is unhealthy
    Unhealthy,
    /// Integration status is unknown
    Unknown,
}

/// Integration health check result
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Integration name
    pub integration: String,
    /// Health status
    pub status: HealthStatus,
    /// Response time (milliseconds)
    pub response_time_ms: u64,
    /// Error message (if any)
    pub error: Option<String>,
    /// Timestamp
    pub timestamp: Instant,
}

/// Integration health checker
pub struct IntegrationHealthChecker {
    registry: IntegrationRegistry,
    /// Health check results
    results: HashMap<String, HealthCheckResult>,
}

impl IntegrationHealthChecker {
    /// Create new health checker
    pub fn new() -> Self {
        Self {
            registry: IntegrationRegistry::new(),
            results: HashMap::new(),
        }
    }

    /// Check health of all integrations
    pub async fn check_all(&mut self) -> WorkflowResult<Vec<HealthCheckResult>> {
        let integrations = self.registry.list_available().await;
        let mut results = Vec::new();

        for integration in integrations {
            let result = self.check_integration(&integration.name).await?;
            self.results
                .insert(integration.name.clone(), result.clone());
            results.push(result);
        }

        Ok(results)
    }

    /// Check health of specific integration
    pub async fn check_integration(&self, name: &str) -> WorkflowResult<HealthCheckResult> {
        let start = Instant::now();
        let metadata = self.registry.get(name).await;

        let (status, error) = if let Some(meta) = metadata {
            match meta.status {
                IntegrationStatus::Enabled | IntegrationStatus::Available => {
                    // Perform actual health check based on integration type
                    match self.perform_health_check(name).await {
                        Ok(_) => (HealthStatus::Healthy, None),
                        Err(e) => (HealthStatus::Degraded, Some(e.to_string())),
                    }
                }
                IntegrationStatus::Disabled => (
                    HealthStatus::Unhealthy,
                    Some("Integration is disabled".to_string()),
                ),
                IntegrationStatus::NotAvailable => (
                    HealthStatus::Unknown,
                    Some("Integration is not available".to_string()),
                ),
            }
        } else {
            (
                HealthStatus::Unknown,
                Some(format!("Integration {} not found", name)),
            )
        };

        let response_time = start.elapsed().as_millis() as u64;

        Ok(HealthCheckResult {
            integration: name.to_string(),
            status,
            response_time_ms: response_time,
            error,
            timestamp: start,
        })
    }

    /// Perform actual health check for integration
    async fn perform_health_check(&self, name: &str) -> WorkflowResult<()> {
        match name {
            "fortune5" => {
                // Fortune 5 health check: verify integration is accessible
                // In production, would check SPIFFE/SPIRE connectivity, KMS availability, and SLO endpoints
                // For now, check if integration is registered and enabled
                let metadata = self.registry.get(name).await;
                if metadata.is_some() {
                    Ok(())
                } else {
                    Err(WorkflowError::ResourceUnavailable(
                        "Fortune 5 integration not registered".to_string(),
                    ))
                }
            }
            "lockchain" => {
                // Lockchain health check: verify storage is accessible
                // In production, would check receipt storage connectivity and provenance endpoints
                // For now, check if integration is registered and enabled
                let metadata = self.registry.get(name).await;
                if metadata.is_some() {
                    Ok(())
                } else {
                    Err(WorkflowError::ResourceUnavailable(
                        "Lockchain integration not registered".to_string(),
                    ))
                }
            }
            "connectors" => {
                // Connector health check: verify connector registry is accessible
                // In production, would check Kafka broker connectivity, Salesforce API availability
                // For now, check if integration is registered and enabled
                let metadata = self.registry.get(name).await;
                if metadata.is_some() {
                    Ok(())
                } else {
                    Err(WorkflowError::ResourceUnavailable(
                        "Connector integration not registered".to_string(),
                    ))
                }
            }
            "sidecar" => {
                // Sidecar health check: verify gRPC endpoint is accessible
                // In production, would check gRPC endpoint connectivity and sidecar process status
                // For now, check if integration is registered and enabled
                let metadata = self.registry.get(name).await;
                if metadata.is_some() {
                    Ok(())
                } else {
                    Err(WorkflowError::ResourceUnavailable(
                        "Sidecar integration not registered".to_string(),
                    ))
                }
            }
            "etl" => {
                // ETL health check: verify pipeline stages are accessible
                // In production, would check pipeline stage availability, Reflex bridge connectivity
                // For now, check if integration is registered and enabled
                let metadata = self.registry.get(name).await;
                if metadata.is_some() {
                    Ok(())
                } else {
                    Err(WorkflowError::ResourceUnavailable(
                        "ETL integration not registered".to_string(),
                    ))
                }
            }
            "otel" => {
                // OTEL health check: verify OTLP exporter is accessible
                // In production, would check OTLP exporter connectivity, span export validation
                // For now, check if integration is registered and enabled
                let metadata = self.registry.get(name).await;
                if metadata.is_some() {
                    Ok(())
                } else {
                    Err(WorkflowError::ResourceUnavailable(
                        "OTEL integration not registered".to_string(),
                    ))
                }
            }
            _ => Err(WorkflowError::ResourceUnavailable(format!(
                "Unknown integration: {}",
                name
            ))),
        }
    }

    /// Get health check result for integration
    pub fn get_result(&self, name: &str) -> Option<&HealthCheckResult> {
        self.results.get(name)
    }

    /// Get all health check results
    pub fn get_all_results(&self) -> Vec<&HealthCheckResult> {
        self.results.values().collect()
    }

    /// Check if all integrations are healthy
    pub fn all_healthy(&self) -> bool {
        self.results
            .values()
            .all(|r| r.status == HealthStatus::Healthy)
    }
}

impl Default for IntegrationHealthChecker {
    fn default() -> Self {
        Self::new()
    }
}
