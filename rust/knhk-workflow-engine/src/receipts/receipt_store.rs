//! Receipt Store
//!
//! Immutable log storage for execution receipts.
//! Provides query API for receipt retrieval and audit trails.

use crate::error::{WorkflowError, WorkflowResult};
use crate::receipts::receipt_generator::Receipt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Receipt query filter
#[derive(Debug, Clone, Default)]
pub struct ReceiptQuery {
    /// Filter by sigma ID
    pub sigma_id: Option<String>,
    /// Filter by minimum timestamp
    pub min_timestamp_ms: Option<u64>,
    /// Filter by maximum timestamp
    pub max_timestamp_ms: Option<u64>,
    /// Filter by validity (only valid/invalid receipts)
    pub only_valid: Option<bool>,
    /// Maximum number of results
    pub limit: Option<usize>,
}

/// Receipt statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptStats {
    /// Total receipts
    pub total_receipts: usize,
    /// Valid receipts
    pub valid_receipts: usize,
    /// Invalid receipts (with guard failures)
    pub invalid_receipts: usize,
    /// Total ticks consumed
    pub total_ticks: u64,
    /// Average ticks per receipt
    pub avg_ticks: f64,
}

/// Receipt store (immutable log)
pub struct ReceiptStore {
    /// Receipt log (append-only)
    log: Arc<RwLock<Vec<Receipt>>>,
    /// Receipt index by ID
    index_by_id: Arc<RwLock<HashMap<String, usize>>>,
    /// Receipt index by sigma ID
    index_by_sigma: Arc<RwLock<HashMap<String, Vec<usize>>>>,
}

impl ReceiptStore {
    /// Create new receipt store
    pub fn new() -> Self {
        Self {
            log: Arc::new(RwLock::new(Vec::new())),
            index_by_id: Arc::new(RwLock::new(HashMap::new())),
            index_by_sigma: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store a receipt (append-only)
    pub async fn store(&self, receipt: Receipt) -> WorkflowResult<()> {
        // Verify receipt signature before storing
        if !receipt.verify_signature() {
            return Err(WorkflowError::ReceiptGenerationFailed(
                "Invalid receipt signature".to_string(),
            ));
        }

        let mut log = self.log.write().await;
        let mut index_by_id = self.index_by_id.write().await;
        let mut index_by_sigma = self.index_by_sigma.write().await;

        // Check for duplicate
        if index_by_id.contains_key(&receipt.receipt_id) {
            return Err(WorkflowError::Validation(format!(
                "Receipt {} already exists",
                receipt.receipt_id
            )));
        }

        // Append to log
        let index = log.len();
        log.push(receipt.clone());

        // Update indices
        index_by_id.insert(receipt.receipt_id.clone(), index);
        index_by_sigma
            .entry(receipt.sigma_id.clone())
            .or_insert_with(Vec::new)
            .push(index);

        Ok(())
    }

    /// Get receipt by ID
    pub async fn get_by_id(&self, receipt_id: &str) -> Option<Receipt> {
        let log = self.log.read().await;
        let index = self.index_by_id.read().await;

        index.get(receipt_id).and_then(|&idx| log.get(idx).cloned())
    }

    /// Get receipts by sigma ID
    pub async fn get_by_sigma(&self, sigma_id: &str) -> Vec<Receipt> {
        let log = self.log.read().await;
        let index = self.index_by_sigma.read().await;

        index
            .get(sigma_id)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&idx| log.get(idx).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Query receipts
    pub async fn query(&self, query: ReceiptQuery) -> Vec<Receipt> {
        let log = self.log.read().await;

        let mut results: Vec<Receipt> = log
            .iter()
            .filter(|r| {
                // Filter by sigma ID
                if let Some(ref sigma_id) = query.sigma_id {
                    if &r.sigma_id != sigma_id {
                        return false;
                    }
                }

                // Filter by timestamp range
                if let Some(min_ts) = query.min_timestamp_ms {
                    if r.timestamp_ms < min_ts {
                        return false;
                    }
                }
                if let Some(max_ts) = query.max_timestamp_ms {
                    if r.timestamp_ms > max_ts {
                        return false;
                    }
                }

                // Filter by validity
                if let Some(only_valid) = query.only_valid {
                    if only_valid && !r.is_valid() {
                        return false;
                    }
                    if !only_valid && r.is_valid() {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Sort by timestamp (newest first)
        results.sort_by(|a, b| b.timestamp_ms.cmp(&a.timestamp_ms));

        // Apply limit
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        results
    }

    /// Get receipt statistics
    pub async fn get_stats(&self) -> ReceiptStats {
        let log = self.log.read().await;

        let total_receipts = log.len();
        let valid_receipts = log.iter().filter(|r| r.is_valid()).count();
        let invalid_receipts = total_receipts - valid_receipts;
        let total_ticks: u64 = log.iter().map(|r| r.ticks_used as u64).sum();
        let avg_ticks = if total_receipts > 0 {
            total_ticks as f64 / total_receipts as f64
        } else {
            0.0
        };

        ReceiptStats {
            total_receipts,
            valid_receipts,
            invalid_receipts,
            total_ticks,
            avg_ticks,
        }
    }

    /// Get total receipts count
    pub async fn count(&self) -> usize {
        let log = self.log.read().await;
        log.len()
    }

    /// Get all receipts
    pub async fn get_all(&self) -> Vec<Receipt> {
        let log = self.log.read().await;
        log.clone()
    }
}

impl Default for ReceiptStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::receipts::receipt_generator::ReceiptGenerator;

    #[tokio::test]
    async fn test_receipt_store() {
        let store = ReceiptStore::new();
        let generator = ReceiptGenerator::new();

        let o_in = serde_json::json!({"input": 1});
        let a_out = serde_json::json!({"output": 1});

        let receipt = generator
            .generate_receipt("sigma-1".to_string(), &o_in, &a_out, vec![], vec![], 5)
            .expect("Generation failed");

        store.store(receipt.clone()).await.expect("Store failed");

        let retrieved = store.get_by_id(&receipt.receipt_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().receipt_id, receipt.receipt_id);
    }

    #[tokio::test]
    async fn test_receipt_query() {
        let store = ReceiptStore::new();
        let generator = ReceiptGenerator::new();

        // Store multiple receipts
        for i in 0..5 {
            let o_in = serde_json::json!({"input": i});
            let a_out = serde_json::json!({"output": i});

            let receipt = generator
                .generate_receipt(
                    "sigma-1".to_string(),
                    &o_in,
                    &a_out,
                    vec![],
                    vec![],
                    i as u32,
                )
                .expect("Generation failed");

            store.store(receipt).await.expect("Store failed");
        }

        // Query by sigma ID
        let query = ReceiptQuery {
            sigma_id: Some("sigma-1".to_string()),
            ..Default::default()
        };

        let results = store.query(query).await;
        assert_eq!(results.len(), 5);
    }

    #[tokio::test]
    async fn test_receipt_stats() {
        let store = ReceiptStore::new();
        let generator = ReceiptGenerator::new();

        let o_in = serde_json::json!({});
        let a_out = serde_json::json!({});

        // Valid receipt
        let receipt1 = generator
            .generate_receipt("sigma-1".to_string(), &o_in, &a_out, vec![], vec![], 10)
            .expect("Generation failed");
        store.store(receipt1).await.expect("Store failed");

        // Invalid receipt
        let receipt2 = generator
            .generate_receipt(
                "sigma-2".to_string(),
                &o_in,
                &a_out,
                vec!["guard1".to_string()],
                vec!["guard1".to_string()],
                5,
            )
            .expect("Generation failed");
        store.store(receipt2).await.expect("Store failed");

        let stats = store.get_stats().await;
        assert_eq!(stats.total_receipts, 2);
        assert_eq!(stats.valid_receipts, 1);
        assert_eq!(stats.invalid_receipts, 1);
        assert_eq!(stats.total_ticks, 15);
        assert_eq!(stats.avg_ticks, 7.5);
    }

    #[tokio::test]
    async fn test_duplicate_prevention() {
        let store = ReceiptStore::new();
        let generator = ReceiptGenerator::new();

        let o_in = serde_json::json!({});
        let a_out = serde_json::json!({});

        let receipt = generator
            .generate_receipt("sigma-1".to_string(), &o_in, &a_out, vec![], vec![], 1)
            .expect("Generation failed");

        store
            .store(receipt.clone())
            .await
            .expect("First store failed");
        let result = store.store(receipt).await;

        assert!(result.is_err(), "Should reject duplicate receipt");
    }
}
