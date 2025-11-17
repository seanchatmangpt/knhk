//! Cryptographic Receipts for Workflow Execution
//!
//! Receipts provide immutable, auditable proof of workflow execution.
//! Each receipt captures: Σ* used, O_in hash, A_out hash, guards checked, latency.
//!
//! # Doctrine Compliance
//!
//! - **A = μ(O)**: Receipt proves A was computed from O using specific μ
//! - **Γ(O)**: Receipts form the global queryable history
//! - **Q enforcement**: All guards checked are listed in receipt

use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::snapshot::SnapshotId;

/// Unique receipt identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ReceiptId(pub String);

impl ReceiptId {
    pub fn new() -> Self {
        #[cfg(feature = "std")]
        {
            use uuid::Uuid;
            // Use UUID v4 for guaranteed uniqueness
            let uuid = Uuid::new_v4();
            return Self(format!("receipt_{}", uuid));
        }

        #[cfg(not(feature = "std"))]
        {
            // Fallback for no_std: use counter only
            use std::sync::atomic::{AtomicU64, Ordering};
            static COUNTER: AtomicU64 = AtomicU64::new(0);
            let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
            Self(format!("receipt_{}", counter))
        }
    }

    pub fn from_string(s: String) -> Self {
        Self(s)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ReceiptId {
    fn default() -> Self {
        Self::new()
    }
}

/// Execution receipt
///
/// Immutable proof of workflow execution conforming to A = μ(O)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    pub receipt_id: ReceiptId,
    pub sigma_id: SnapshotId,
    pub o_in_hash: String,
    pub a_out_hash: String,
    pub guards_checked: Vec<String>,
    pub guards_failed: Vec<String>,
    pub ticks_used: u32,
    pub timestamp: u64, // Unix timestamp in microseconds
    pub workflow_instance_id: String,
    pub success: bool,
}

impl Receipt {
    /// Create a new receipt
    pub fn new(
        sigma_id: SnapshotId,
        o_in: &[u8],
        a_out: &[u8],
        workflow_instance_id: String,
    ) -> Self {
        let mut hasher_in = Sha3_256::new();
        hasher_in.update(o_in);
        let o_in_hash = format!("sha3-256:{:x}", hasher_in.finalize());

        let mut hasher_out = Sha3_256::new();
        hasher_out.update(a_out);
        let a_out_hash = format!("sha3-256:{:x}", hasher_out.finalize());

        #[cfg(feature = "std")]
        let timestamp = {
            use std::time::SystemTime;
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64
        };

        #[cfg(not(feature = "std"))]
        let timestamp = 0;

        Self {
            receipt_id: ReceiptId::new(),
            sigma_id,
            o_in_hash,
            a_out_hash,
            guards_checked: Vec::new(),
            guards_failed: Vec::new(),
            ticks_used: 0,
            timestamp,
            workflow_instance_id,
            success: true,
        }
    }

    /// Add a guard check result
    pub fn add_guard_check(&mut self, guard_name: String, passed: bool) {
        if passed {
            self.guards_checked.push(guard_name);
        } else {
            self.guards_failed.push(guard_name);
            self.success = false;
        }
    }

    /// Set execution latency in ticks
    pub fn set_ticks(&mut self, ticks: u32) {
        self.ticks_used = ticks;

        // Chatman constant violation check
        if ticks > 8 {
            self.add_guard_check("CHATMAN_CONSTANT".to_string(), false);
        }
    }

    /// Verify receipt consistency
    ///
    /// Checks that the receipt state is internally consistent.
    /// Does NOT enforce the Chatman constant - violations are allowed
    /// but should be marked as failures via guards_failed.
    pub fn verify(&self) -> bool {
        // Success requires no failed guards
        if self.success && !self.guards_failed.is_empty() {
            return false;
        }

        // If ticks > 8, CHATMAN_CONSTANT guard should be in guards_failed
        if self.ticks_used > 8 && self.success {
            return false; // Inconsistent: violation not marked as failure
        }

        true
    }

    /// Check if receipt satisfies specific guard
    pub fn satisfies_guard(&self, guard_name: &str) -> bool {
        self.guards_checked.iter().any(|g| g == guard_name)
    }
}

/// Receipt store for Γ(O) - global queryable history
pub struct ReceiptStore {
    receipts: Arc<RwLock<HashMap<ReceiptId, Receipt>>>,
    by_workflow: Arc<RwLock<HashMap<String, Vec<ReceiptId>>>>,
    by_snapshot: Arc<RwLock<HashMap<SnapshotId, Vec<ReceiptId>>>>,
}

impl ReceiptStore {
    pub fn new() -> Self {
        Self {
            receipts: Arc::new(RwLock::new(HashMap::new())),
            by_workflow: Arc::new(RwLock::new(HashMap::new())),
            by_snapshot: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Append a receipt to the immutable log
    pub fn append(&self, receipt: Receipt) -> Result<ReceiptId, String> {
        if !receipt.verify() {
            return Err("Receipt verification failed".to_string());
        }

        let id = receipt.receipt_id.clone();
        let workflow_id = receipt.workflow_instance_id.clone();
        let snapshot_id = receipt.sigma_id.clone();

        // Store receipt
        self.receipts
            .write()
            .map_err(|e| e.to_string())?
            .insert(id.clone(), receipt);

        // Index by workflow
        self.by_workflow
            .write()
            .map_err(|e| e.to_string())?
            .entry(workflow_id)
            .or_insert_with(Vec::new)
            .push(id.clone());

        // Index by snapshot
        self.by_snapshot
            .write()
            .map_err(|e| e.to_string())?
            .entry(snapshot_id)
            .or_insert_with(Vec::new)
            .push(id.clone());

        Ok(id)
    }

    /// Get receipt by ID
    pub fn get(&self, id: &ReceiptId) -> Result<Receipt, String> {
        self.receipts
            .read()
            .map_err(|e| e.to_string())?
            .get(id)
            .cloned()
            .ok_or_else(|| format!("Receipt not found: {}", id.as_str()))
    }

    /// Query receipts by workflow instance
    pub fn get_by_workflow(&self, workflow_id: &str) -> Result<Vec<Receipt>, String> {
        let ids = self
            .by_workflow
            .read()
            .map_err(|e| e.to_string())?
            .get(workflow_id)
            .cloned()
            .unwrap_or_default();

        let receipts = self.receipts.read().map_err(|e| e.to_string())?;
        Ok(ids
            .iter()
            .filter_map(|id| receipts.get(id).cloned())
            .collect())
    }

    /// Query receipts by snapshot (for MAPE-K analysis)
    pub fn get_by_snapshot(&self, snapshot_id: &SnapshotId) -> Result<Vec<Receipt>, String> {
        let ids = self
            .by_snapshot
            .read()
            .map_err(|e| e.to_string())?
            .get(snapshot_id)
            .cloned()
            .unwrap_or_default();

        let receipts = self.receipts.read().map_err(|e| e.to_string())?;
        Ok(ids
            .iter()
            .filter_map(|id| receipts.get(id).cloned())
            .collect())
    }

    /// Get all receipts (for Γ(O) queries)
    pub fn get_all(&self) -> Result<Vec<Receipt>, String> {
        Ok(self
            .receipts
            .read()
            .map_err(|e| e.to_string())?
            .values()
            .cloned()
            .collect())
    }

    /// Query for Chatman constant violations
    pub fn get_violations(&self) -> Result<Vec<Receipt>, String> {
        Ok(self
            .get_all()?
            .into_iter()
            .filter(|r| r.ticks_used > 8)
            .collect())
    }

    /// Query for guard failures
    pub fn get_guard_failures(&self, guard_name: &str) -> Result<Vec<Receipt>, String> {
        Ok(self
            .get_all()?
            .into_iter()
            .filter(|r| r.guards_failed.iter().any(|g| g == guard_name))
            .collect())
    }

    /// Get statistics for MAPE-K monitoring
    pub fn get_statistics(&self) -> Result<ReceiptStatistics, String> {
        let all = self.get_all()?;

        let total = all.len();
        let successful = all.iter().filter(|r| r.success).count();
        let failed = total - successful;

        let avg_ticks = if total > 0 {
            all.iter().map(|r| r.ticks_used as f64).sum::<f64>() / total as f64
        } else {
            0.0
        };

        let violations = all.iter().filter(|r| r.ticks_used > 8).count();

        Ok(ReceiptStatistics {
            total_receipts: total,
            successful_executions: successful,
            failed_executions: failed,
            average_ticks: avg_ticks,
            chatman_violations: violations,
        })
    }
}

impl Default for ReceiptStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for MAPE-K monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiptStatistics {
    pub total_receipts: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub average_ticks: f64,
    pub chatman_violations: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_creation() {
        let snapshot_id = SnapshotId::from_string("Σ_test_001".to_string());
        let o_in = b"input data";
        let a_out = b"output data";

        let receipt = Receipt::new(
            snapshot_id.clone(),
            o_in,
            a_out,
            "workflow-instance-1".to_string(),
        );

        assert_eq!(receipt.sigma_id, snapshot_id);
        assert!(receipt.success);
        assert_eq!(receipt.ticks_used, 0);
    }

    #[test]
    fn test_guard_checks() {
        let snapshot_id = SnapshotId::from_string("Σ_test_001".to_string());
        let mut receipt = Receipt::new(snapshot_id, b"input", b"output", "workflow-1".to_string());

        receipt.add_guard_check("Q1".to_string(), true);
        receipt.add_guard_check("Q2".to_string(), true);
        receipt.add_guard_check("Q3".to_string(), false);

        assert_eq!(receipt.guards_checked.len(), 2);
        assert_eq!(receipt.guards_failed.len(), 1);
        assert!(!receipt.success);
    }

    #[test]
    fn test_chatman_constant_enforcement() {
        let snapshot_id = SnapshotId::from_string("Σ_test_001".to_string());
        let mut receipt = Receipt::new(snapshot_id, b"input", b"output", "workflow-1".to_string());

        receipt.set_ticks(6);
        assert!(receipt.verify());
        assert!(receipt.success);

        receipt.set_ticks(10); // Violates Chatman constant
        assert!(receipt.verify()); // Receipt is internally consistent
        assert!(!receipt.success); // But execution failed
        assert!(receipt
            .guards_failed
            .contains(&"CHATMAN_CONSTANT".to_string()));
    }

    #[test]
    fn test_receipt_store() {
        let store = ReceiptStore::new();
        let snapshot_id = SnapshotId::from_string("Σ_test_001".to_string());

        let mut receipt = Receipt::new(snapshot_id, b"input", b"output", "workflow-1".to_string());
        receipt.set_ticks(5);

        let id = store.append(receipt.clone()).unwrap();
        let retrieved = store.get(&id).unwrap();

        assert_eq!(retrieved.receipt_id, id);
        assert_eq!(retrieved.ticks_used, 5);
    }

    #[test]
    fn test_query_by_workflow() {
        let store = ReceiptStore::new();
        let snapshot_id = SnapshotId::from_string("Σ_test_001".to_string());

        for i in 0..3 {
            let mut receipt = Receipt::new(
                snapshot_id.clone(),
                format!("input-{}", i).as_bytes(),
                format!("output-{}", i).as_bytes(),
                "workflow-1".to_string(),
            );
            receipt.set_ticks(i + 1);
            store.append(receipt).unwrap();
        }

        let receipts = store.get_by_workflow("workflow-1").unwrap();
        assert_eq!(receipts.len(), 3);
    }

    #[test]
    fn test_violation_queries() {
        let store = ReceiptStore::new();
        let snapshot_id = SnapshotId::from_string("Σ_test_001".to_string());

        let mut receipt1 = Receipt::new(
            snapshot_id.clone(),
            b"input1",
            b"output1",
            "workflow-1".to_string(),
        );
        receipt1.set_ticks(5); // OK

        let mut receipt2 =
            Receipt::new(snapshot_id, b"input2", b"output2", "workflow-2".to_string());
        receipt2.set_ticks(10); // Violation

        store.append(receipt1).unwrap();
        store.append(receipt2).unwrap();

        let violations = store.get_violations().unwrap();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].workflow_instance_id, "workflow-2");
    }

    #[test]
    fn test_statistics() {
        let store = ReceiptStore::new();
        let snapshot_id = SnapshotId::from_string("Σ_test_001".to_string());

        // Create receipts with different characteristics
        for i in 0..10 {
            let mut receipt = Receipt::new(
                snapshot_id.clone(),
                format!("input-{}", i).as_bytes(),
                format!("output-{}", i).as_bytes(),
                format!("workflow-{}", i),
            );
            let ticks = (i % 12) as u32;
            receipt.set_ticks(ticks); // Some violations (9, 10, 11)

            // Debug: print ID to see if they're unique
            eprintln!(
                "Created receipt {}: id={}, ticks={}",
                i,
                receipt.receipt_id.as_str(),
                ticks
            );

            let id = store
                .append(receipt)
                .expect(&format!("Failed to append receipt {}", i));
            eprintln!("Appended receipt {} with id={}", i, id.as_str());
        }

        // Check how many are actually stored
        let all = store.get_all().unwrap();
        eprintln!("Total receipts in store: {}", all.len());
        for (idx, r) in all.iter().enumerate() {
            eprintln!(
                "  Receipt {}: id={}, ticks={}",
                idx,
                r.receipt_id.as_str(),
                r.ticks_used
            );
        }

        let stats = store.get_statistics().unwrap();
        assert_eq!(
            stats.total_receipts, 10,
            "Expected 10 receipts but got {}",
            stats.total_receipts
        );
        assert!(stats.average_ticks > 0.0);
        assert!(
            stats.chatman_violations > 0,
            "Expected some Chatman violations"
        ); // Should have violations for i >= 9
    }
}
