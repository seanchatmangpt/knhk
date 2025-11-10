//! Best practices integration
//!
//! Demonstrates how to use the best features from each integration
//! in a unified, production-ready way.

use crate::error::WorkflowResult;
#[cfg(feature = "connectors")]
use crate::integration::connectors::ConnectorIntegration;
use crate::integration::{
    check::IntegrationHealthChecker, fortune5::Fortune5Integration,
    lockchain::LockchainIntegration, otel::OtelIntegration,
};
use crate::parser::WorkflowSpecId;
use std::sync::Arc;

/// Best practices example: Using all integrations together
pub struct BestPracticesIntegration {
    fortune5: Option<Arc<Fortune5Integration>>,
    lockchain: Option<Arc<LockchainIntegration>>,
    otel: Option<Arc<OtelIntegration>>,
    #[cfg(feature = "connectors")]
    connectors: Option<Arc<ConnectorIntegration>>,
    #[cfg(not(feature = "connectors"))]
    connectors: Option<()>,
    health_checker: IntegrationHealthChecker,
}

impl BestPracticesIntegration {
    /// Create unified integration with best features from all packages
    ///
    /// Initializes all available integrations:
    /// - Fortune 5: SLO tracking, promotion gates
    /// - Lockchain: Provenance tracking
    /// - OTEL: Observability
    /// - Connectors: External systems
    pub async fn create_unified_integration(
        fortune5_config: Option<crate::integration::fortune5::Fortune5Config>,
        lockchain_path: Option<String>,
        otel_endpoint: Option<String>,
    ) -> WorkflowResult<Self> {
        // Initialize Fortune 5 integration if config provided
        let fortune5 = fortune5_config.map(|config| Arc::new(Fortune5Integration::new(config)));

        // Initialize Lockchain integration if path provided
        let lockchain = if let Some(ref path) = lockchain_path {
            match LockchainIntegration::new(path) {
                Ok(lock) => Some(Arc::new(lock)),
                Err(e) => {
                    tracing::warn!("Failed to initialize Lockchain integration: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Initialize OTEL integration if endpoint provided
        let otel = otel_endpoint.as_ref().map(|endpoint| {
            let integration = OtelIntegration::new(Some(endpoint.clone()));
            Arc::new(integration)
        });

        // Initialize Connector integration
        #[cfg(feature = "connectors")]
        let connectors = Some(Arc::new(ConnectorIntegration::new()));
        #[cfg(not(feature = "connectors"))]
        let connectors = None;

        // Create health checker with integration instances
        let health_checker = IntegrationHealthChecker::with_integrations(
            fortune5.clone(),
            lockchain_path.clone(),
            otel_endpoint.clone(),
        );

        Ok(Self {
            fortune5,
            lockchain,
            otel,
            connectors,
            health_checker,
        })
    }

    /// Example: Execute workflow with all best features
    ///
    /// Executes a workflow with:
    /// - SLO tracking (Fortune 5)
    /// - Provenance recording (Lockchain)
    /// - Observability (OTEL)
    /// - External system integration (Connectors)
    pub async fn execute_with_best_features(
        &self,
        _workflow_name: &str,
        data: serde_json::Value,
    ) -> WorkflowResult<serde_json::Value> {
        // Start OTEL span for workflow execution
        let span_ctx = if let Some(ref otel) = self.otel {
            // Use start_register_workflow_span instead of deprecated start_workflow_span
            otel.start_register_workflow_span(&WorkflowSpecId::new())
                .await
                .ok()
                .flatten()
        } else {
            None
        };

        // Check Fortune 5 promotion gate
        if let Some(ref fortune5) = self.fortune5 {
            match fortune5.check_promotion_gate().await {
                Ok(allowed) => {
                    if !allowed {
                        if let Some(ref ctx) = span_ctx {
                            if let Some(ref otel) = self.otel {
                                let _ = otel
                                    .end_span(ctx.clone(), knhk_otel::SpanStatus::Error)
                                    .await;
                            }
                        }
                        return Err(crate::error::WorkflowError::Validation(
                            "Promotion gate blocked workflow execution".to_string(),
                        ));
                    }
                }
                Err(e) => {
                    tracing::warn!("Promotion gate check failed: {}", e);
                }
            }
        }

        // Record workflow start in Lockchain
        if let Some(ref lockchain) = self.lockchain {
            let case_id = crate::case::CaseId::new();
            let spec_id = WorkflowSpecId::new();
            if let Err(e) = lockchain.record_case_created(&case_id, &spec_id).await {
                tracing::warn!("Failed to record case creation in Lockchain: {}", e);
            }
        }

        // Execute workflow (placeholder - actual execution would be done by workflow engine)
        let result = data.clone();

        // Record workflow completion in Lockchain
        if let Some(ref lockchain) = self.lockchain {
            let case_id = crate::case::CaseId::new();
            if let Err(e) = lockchain.record_case_executed(&case_id, true).await {
                tracing::warn!("Failed to record case execution in Lockchain: {}", e);
            }
        }

        // End OTEL span
        if let Some(ref ctx) = span_ctx {
            if let Some(ref otel) = self.otel {
                let _ = otel.end_span(ctx.clone(), knhk_otel::SpanStatus::Ok).await;
            }
        }

        Ok(result)
    }

    /// Example: Health check with all integrations
    ///
    /// Verifies all integrations are healthy:
    /// - Fortune 5: SLO compliance, SPIFFE connectivity
    /// - Lockchain: Storage connectivity
    /// - OTEL: OTLP exporter connectivity
    /// - Connectors: Connector registry
    pub async fn health_check_all(&self) -> WorkflowResult<()> {
        // Use the health checker to check all integrations
        let mut checker = IntegrationHealthChecker::with_integrations(
            self.fortune5.clone(),
            self.lockchain
                .as_ref()
                .map(|_| "/tmp/lockchain".to_string()), // Placeholder path
            self.otel
                .as_ref()
                .map(|_| "http://localhost:4317".to_string()), // Placeholder endpoint
        );

        let results = checker.check_all().await?;

        // Check if all integrations are healthy
        let all_healthy = results
            .iter()
            .all(|r| matches!(r.status, crate::integration::check::HealthStatus::Healthy));

        if all_healthy {
            Ok(())
        } else {
            let unhealthy: Vec<String> = results
                .iter()
                .filter(|r| !matches!(r.status, crate::integration::check::HealthStatus::Healthy))
                .map(|r| format!("{}: {:?}", r.integration, r.status))
                .collect();
            Err(crate::error::WorkflowError::ResourceUnavailable(format!(
                "Some integrations are unhealthy: {}",
                unhealthy.join(", ")
            )))
        }
    }

    /// Example: Get SLO metrics (best from Fortune 5)
    pub async fn get_slo_metrics(&self) -> Option<(u64, u64, u64)> {
        if let Some(ref fortune5) = self.fortune5 {
            fortune5.get_slo_metrics().await
        } else {
            None
        }
    }

    /// Example: List all available integrations (best from Registry)
    pub async fn list_integrations(
        &self,
    ) -> Vec<crate::integration::registry::IntegrationMetadata> {
        let registry = crate::integration::registry::IntegrationRegistry::new();
        registry.list_available().await
    }
}
