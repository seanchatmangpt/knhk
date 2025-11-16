//! Monitor Phase - Collects observations from receipts and telemetry

use super::{Observation, KnowledgeBase};
use crate::receipts::{Receipt, ReceiptStore};
use crate::error::WorkflowResult;
use std::sync::Arc;
use parking_lot::RwLock;
use std::collections::HashMap;

/// Monitor phase collects observations from the observation plane (O)
pub struct MonitorPhase {
    receipt_store: Arc<ReceiptStore>,
    knowledge: Arc<RwLock<KnowledgeBase>>,
}

impl MonitorPhase {
    pub fn new(
        receipt_store: Arc<ReceiptStore>,
        knowledge: Arc<RwLock<KnowledgeBase>>,
    ) -> Self {
        Self {
            receipt_store,
            knowledge,
        }
    }

    /// Collect observations from recent receipts
    ///
    /// This implements the "Monitor" phase of MAPE-K, gathering data
    /// from the observation plane (O).
    pub async fn collect_observations(&self) -> WorkflowResult<Vec<Observation>> {
        // Query recent receipts (last 100)
        let receipts = self.receipt_store.query_recent(100)?;

        let mut observations = Vec::new();

        for receipt in receipts {
            // Convert receipt to observation
            let observation = Observation {
                receipt_id: receipt.receipt_id.clone(),
                sigma_id: receipt.sigma_id.clone(),
                ticks_used: receipt.ticks_used,
                guards_checked: receipt.guards_checked.clone(),
                guards_failed: receipt.guards_failed.clone(),
                timestamp: receipt.timestamp,
                metrics: self.extract_metrics(&receipt),
            };

            observations.push(observation);
        }

        // Update knowledge base with latest observations
        self.knowledge.write().update_observation_stats(&observations);

        Ok(observations)
    }

    /// Extract metrics from receipt
    fn extract_metrics(&self, receipt: &Receipt) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();

        // Calculate guard failure rate
        if !receipt.guards_checked.is_empty() {
            let failure_rate = receipt.guards_failed.len() as f64 / receipt.guards_checked.len() as f64;
            metrics.insert("guard_failure_rate".to_string(), failure_rate);
        }

        // Tick utilization (percentage of Chatman constant used)
        let tick_utilization = (receipt.ticks_used as f64 / 8.0) * 100.0;
        metrics.insert("tick_utilization_pct".to_string(), tick_utilization);

        // Near-miss indicator (approaching tick limit)
        let near_miss = if receipt.ticks_used >= 7 { 1.0 } else { 0.0 };
        metrics.insert("chatman_near_miss".to_string(), near_miss);

        metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[tokio::test]
    async fn test_monitor_collects_observations() {
        let receipt_store = Arc::new(ReceiptStore::in_memory());
        let knowledge = Arc::new(RwLock::new(KnowledgeBase::new()));
        let monitor = MonitorPhase::new(receipt_store.clone(), knowledge);

        // Create test receipt
        let receipt = Receipt {
            receipt_id: "test-001".to_string(),
            sigma_id: "sigma-v1".to_string(),
            o_in_hash: "input-hash".to_string(),
            a_out_hash: "output-hash".to_string(),
            guards_checked: vec!["G-001".to_string(), "G-002".to_string()],
            guards_failed: vec![],
            ticks_used: 5,
            timestamp: Utc::now(),
        };

        receipt_store.store(receipt).unwrap();

        // Monitor should collect observations
        let observations = monitor.collect_observations().await.unwrap();
        assert_eq!(observations.len(), 1);
        assert_eq!(observations[0].ticks_used, 5);
    }
}
