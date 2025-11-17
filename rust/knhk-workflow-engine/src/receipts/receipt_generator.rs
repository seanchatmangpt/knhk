//! Receipt Generation System
//!
//! Generates cryptographic receipts for all hook executions.
//! Receipt structure: {receipt_id, sigma_id, o_in_hash, a_out_hash, guards_checked, guards_failed, ticks_used, timestamp}

use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Receipt structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Receipt {
    /// Unique receipt ID
    pub receipt_id: String,
    /// Sigma (snapshot) ID
    pub sigma_id: String,
    /// Input observation hash (O_in)
    pub o_in_hash: String,
    /// Output action hash (A_out)
    pub a_out_hash: String,
    /// Guards checked
    pub guards_checked: Vec<String>,
    /// Guards failed
    pub guards_failed: Vec<String>,
    /// Ticks used for execution
    pub ticks_used: u32,
    /// Timestamp (milliseconds since epoch)
    pub timestamp_ms: u64,
    /// Receipt signature (cryptographic hash of entire receipt)
    pub signature: String,
}

impl Receipt {
    /// Create new receipt
    pub fn new(
        sigma_id: String,
        o_in: &serde_json::Value,
        a_out: &serde_json::Value,
        guards_checked: Vec<String>,
        guards_failed: Vec<String>,
        ticks_used: u32,
    ) -> Self {
        let receipt_id = Self::generate_receipt_id();
        let timestamp_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let o_in_hash = Self::hash_data(o_in);
        let a_out_hash = Self::hash_data(a_out);

        let mut receipt = Self {
            receipt_id,
            sigma_id,
            o_in_hash,
            a_out_hash,
            guards_checked,
            guards_failed,
            ticks_used,
            timestamp_ms,
            signature: String::new(), // Will be computed
        };

        // Compute signature
        receipt.signature = receipt.compute_signature();
        receipt
    }

    /// Generate unique receipt ID
    fn generate_receipt_id() -> String {
        use uuid::Uuid;
        Uuid::new_v4().to_string()
    }

    /// Hash data using SHA-256
    fn hash_data(data: &serde_json::Value) -> String {
        let json_str = serde_json::to_string(data).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(json_str.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Compute cryptographic signature of receipt
    fn compute_signature(&self) -> String {
        let mut hasher = Sha256::new();

        hasher.update(self.receipt_id.as_bytes());
        hasher.update(self.sigma_id.as_bytes());
        hasher.update(self.o_in_hash.as_bytes());
        hasher.update(self.a_out_hash.as_bytes());
        hasher.update(&self.ticks_used.to_le_bytes());
        hasher.update(&self.timestamp_ms.to_le_bytes());

        // Include guards in signature
        for guard in &self.guards_checked {
            hasher.update(guard.as_bytes());
        }
        for guard in &self.guards_failed {
            hasher.update(guard.as_bytes());
        }

        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Verify receipt signature
    pub fn verify_signature(&self) -> bool {
        let computed_signature = self.compute_signature();
        computed_signature == self.signature
    }

    /// Check if receipt is valid (all guards passed)
    pub fn is_valid(&self) -> bool {
        self.guards_failed.is_empty()
    }
}

/// Receipt generator
pub struct ReceiptGenerator {
    /// Receipt counter
    receipt_counter: Arc<AtomicU64>,
}

impl ReceiptGenerator {
    /// Create new receipt generator
    pub fn new() -> Self {
        Self {
            receipt_counter: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Generate receipt (alias for generate_receipt for compatibility)
    pub fn generate(
        &self,
        sigma_id: &str,
        o_in: &serde_json::Value,
        a_out: &serde_json::Value,
        guards_checked: &[String],
        guards_failed: &[String],
        ticks_used: u32,
    ) -> WorkflowResult<Receipt> {
        self.generate_receipt(
            sigma_id.to_string(),
            o_in,
            a_out,
            guards_checked.to_vec(),
            guards_failed.to_vec(),
            ticks_used,
        )
    }

    /// Generate receipt for hook execution
    pub fn generate_receipt(
        &self,
        sigma_id: String,
        o_in: &serde_json::Value,
        a_out: &serde_json::Value,
        guards_checked: Vec<String>,
        guards_failed: Vec<String>,
        ticks_used: u32,
    ) -> WorkflowResult<Receipt> {
        // Increment counter
        self.receipt_counter.fetch_add(1, Ordering::Relaxed);

        let receipt = Receipt::new(
            sigma_id,
            o_in,
            a_out,
            guards_checked,
            guards_failed,
            ticks_used,
        );

        // Verify receipt was generated correctly
        if !receipt.verify_signature() {
            return Err(WorkflowError::ReceiptGenerationFailed(
                "Receipt signature verification failed".to_string(),
            ));
        }

        Ok(receipt)
    }

    /// Get total receipts generated
    pub fn get_total_receipts(&self) -> u64 {
        self.receipt_counter.load(Ordering::Relaxed)
    }

    /// Reset counter
    pub fn reset_counter(&self) {
        self.receipt_counter.store(0, Ordering::Relaxed);
    }
}

impl Default for ReceiptGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_generation() {
        let generator = ReceiptGenerator::new();

        let o_in = serde_json::json!({"input": "data"});
        let a_out = serde_json::json!({"output": "result"});

        let receipt = generator
            .generate_receipt(
                "sigma-123".to_string(),
                &o_in,
                &a_out,
                vec!["guard1".to_string()],
                vec![],
                5,
            )
            .expect("Receipt generation failed");

        assert!(!receipt.receipt_id.is_empty());
        assert_eq!(receipt.sigma_id, "sigma-123");
        assert!(!receipt.o_in_hash.is_empty());
        assert!(!receipt.a_out_hash.is_empty());
        assert_eq!(receipt.ticks_used, 5);
        assert!(receipt.verify_signature());
        assert!(receipt.is_valid());
    }

    #[test]
    fn test_receipt_signature_verification() {
        let o_in = serde_json::json!({"test": 123});
        let a_out = serde_json::json!({"result": 456});

        let receipt = Receipt::new("sigma-1".to_string(), &o_in, &a_out, vec![], vec![], 3);

        assert!(receipt.verify_signature());

        // Tampered receipt should fail verification
        let mut tampered = receipt.clone();
        tampered.ticks_used = 999;
        assert!(!tampered.verify_signature());
    }

    #[test]
    fn test_receipt_validity() {
        let o_in = serde_json::json!({});
        let a_out = serde_json::json!({});

        // Valid receipt (no failures)
        let valid_receipt = Receipt::new(
            "sigma-1".to_string(),
            &o_in,
            &a_out,
            vec!["guard1".to_string()],
            vec![],
            2,
        );
        assert!(valid_receipt.is_valid());

        // Invalid receipt (has failures)
        let invalid_receipt = Receipt::new(
            "sigma-2".to_string(),
            &o_in,
            &a_out,
            vec!["guard1".to_string()],
            vec!["guard2".to_string()],
            2,
        );
        assert!(!invalid_receipt.is_valid());
    }

    #[test]
    fn test_receipt_counter() {
        let generator = ReceiptGenerator::new();

        let o_in = serde_json::json!({});
        let a_out = serde_json::json!({});

        generator
            .generate_receipt("sigma-1".to_string(), &o_in, &a_out, vec![], vec![], 1)
            .expect("Generation failed");

        assert_eq!(generator.get_total_receipts(), 1);

        generator
            .generate_receipt("sigma-2".to_string(), &o_in, &a_out, vec![], vec![], 2)
            .expect("Generation failed");

        assert_eq!(generator.get_total_receipts(), 2);
    }

    #[test]
    fn test_hash_consistency() {
        let data = serde_json::json!({"test": "data"});
        let hash1 = Receipt::hash_data(&data);
        let hash2 = Receipt::hash_data(&data);

        assert_eq!(hash1, hash2, "Same data should produce same hash");

        let different_data = serde_json::json!({"test": "different"});
        let hash3 = Receipt::hash_data(&different_data);

        assert_ne!(hash1, hash3, "Different data should produce different hash");
    }
}
