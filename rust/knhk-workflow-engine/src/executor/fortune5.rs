//! Fortune 5 integration methods

use crate::error::WorkflowResult;

use super::WorkflowEngine;

impl WorkflowEngine {
    /// Check SLO compliance (Fortune 5 feature)
    pub async fn check_slo_compliance(&self) -> WorkflowResult<bool> {
        if let Some(ref fortune5) = self.fortune5_integration {
            fortune5.check_slo_compliance().await
        } else {
            Ok(true) // No SLO configured, always compliant
        }
    }

    /// Get SLO metrics (Fortune 5)
    pub async fn get_slo_metrics(&self) -> Option<(u64, u64, u64)> {
        if let Some(ref fortune5) = self.fortune5_integration {
            fortune5.get_slo_metrics().await
        } else {
            None
        }
    }

    /// Check if feature flag is enabled (Fortune 5)
    pub async fn is_feature_enabled(&self, feature: &str) -> bool {
        if let Some(ref fortune5) = self.fortune5_integration {
            fortune5.is_feature_enabled(feature).await
        } else {
            true // No Fortune 5, allow all features
        }
    }
}
