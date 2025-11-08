//! Best practices integration
//!
//! Demonstrates how to use the best features from each integration
//! in a unified, production-ready way.

use crate::error::WorkflowResult;
use crate::integration::fortune5::{Fortune5Config, SloConfig};
use crate::integration::unified::{UnifiedIntegration, UnifiedIntegrationBuilder};
use std::collections::HashMap;

/// Best practices example: Using all integrations together
pub struct BestPracticesIntegration;

impl BestPracticesIntegration {
    /// Create unified integration with best features from all packages
    pub async fn create_unified_integration() -> WorkflowResult<UnifiedIntegration> {
        // Build unified integration with best features from each package
        let integration = UnifiedIntegrationBuilder::new()
            // Best from Fortune 5: SLO tracking
            .with_fortune5(Fortune5Config {
                spiffe: None,
                kms: None,
                multi_region: None,
                slo: Some(SloConfig {
                    r1_p99_max_ns: 2,   // Hot path: ≤2ns P99
                    w1_p99_max_ms: 1,   // Warm path: ≤1ms P99
                    c1_p99_max_ms: 500, // Cold path: ≤500ms P99
                    admission_strategy:
                        crate::integration::fortune5::config::AdmissionStrategy::Strict,
                }),
                promotion: None,
            })
            // Best from Lockchain: Provenance tracking
            .with_lockchain("/tmp/knhk-lockchain")
            // Best from OTEL: Observability
            .with_otel("http://localhost:4317")
            // Best from Connectors: External systems
            .with_connector("kafka", "localhost:9092")
            .with_connector("salesforce", "api.salesforce.com")
            // Enable all best features
            .with_slo(true)
            .with_provenance(true)
            .with_observability(true)
            .build()
            .await?;

        Ok(integration)
    }

    /// Example: Execute workflow with all best features
    pub async fn execute_with_best_features(
        integration: &UnifiedIntegration,
        workflow_name: &str,
        data: serde_json::Value,
    ) -> WorkflowResult<serde_json::Value> {
        // 1. Start trace (best from OTEL)
        let mut attributes = HashMap::new();
        attributes.insert("workflow.name".to_string(), workflow_name.to_string());
        let _span = integration.start_trace_span("workflow.execute", attributes);

        // 2. Check SLO compliance (best from Fortune 5)
        let slo_compliant = integration.check_slo_compliance().await?;
        if !slo_compliant {
            return Err(crate::error::WorkflowError::Validation(
                "SLO compliance check failed".to_string(),
            ));
        }

        // 3. Check promotion gate (best from Fortune 5)
        let promotion_allowed = integration.check_promotion_gate().await?;
        if !promotion_allowed {
            return Err(crate::error::WorkflowError::Validation(
                "Promotion gate denied".to_string(),
            ));
        }

        // 4. Execute connector task (best from Connectors)
        let result = integration.execute_connector_task("kafka", data).await?;

        // 5. Store receipt with provenance (best from Lockchain)
        let receipt = knhk_lockchain::Receipt {
            hash: "example_hash".to_string(),
            timestamp: chrono::Utc::now(),
            metadata: HashMap::new(),
        };
        integration.store_receipt(receipt).await?;

        // 6. Record SLO metric (best from Fortune 5)
        integration
            .record_slo_metric(
                crate::integration::fortune5::slo::RuntimeClass::W1,
                100_000, // 100μs
            )
            .await;

        // 7. Record metric (best from OTEL)
        let mut labels = HashMap::new();
        labels.insert("workflow".to_string(), workflow_name.to_string());
        integration.record_metric("workflow.execution.time", 0.1, labels);

        // 8. Log event (best from OTEL)
        let mut fields = HashMap::new();
        fields.insert("workflow".to_string(), workflow_name.to_string());
        fields.insert("status".to_string(), "success".to_string());
        integration.log_event("info", "Workflow executed successfully", fields);

        Ok(result)
    }

    /// Example: Health check with all integrations
    pub async fn health_check_all(integration: &UnifiedIntegration) -> WorkflowResult<()> {
        // Check health of all integrations
        let health_results = integration.check_health().await?;

        // Verify all are healthy
        for result in health_results {
            if result.status != crate::integration::check::HealthStatus::Healthy {
                return Err(crate::error::WorkflowError::ResourceUnavailable(format!(
                    "Integration {} is not healthy: {:?}",
                    result.integration, result.status
                )));
            }
        }

        Ok(())
    }

    /// Example: Get SLO metrics (best from Fortune 5)
    pub async fn get_slo_metrics(integration: &UnifiedIntegration) -> Option<(u64, u64, u64)> {
        integration.get_slo_metrics().await
    }

    /// Example: List all available integrations (best from Registry)
    pub async fn list_integrations(
        integration: &UnifiedIntegration,
    ) -> Vec<crate::integration::registry::IntegrationMetadata> {
        integration.list_available().await
    }
}
