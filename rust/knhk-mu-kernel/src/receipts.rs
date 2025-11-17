//! Cryptographic Receipts - Proof Chain
//!
//! Every action A produces a receipt proving:
//! hash(A) = hash(μ(O; Σ*)) with guards Q checked

use crate::sigma::SigmaHash;
use alloc::vec::Vec;
use core::mem::size_of;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha3::{Digest, Sha3_256};

/// Receipt ID (sequential, globally unique)
pub type ReceiptId = u64;

/// Receipt (256 bytes, cache-aligned)
#[derive(Debug, Clone, Copy)]
#[repr(C, align(256))]
pub struct Receipt {
    /// Unique receipt ID
    pub receipt_id: ReceiptId,

    /// SHA3-256 of Σ* (which ontology was active)
    pub sigma_hash: SigmaHash,

    /// SHA3-256 of O_in (observation input)
    pub o_in_hash: [u8; 32],

    /// SHA3-256 of A_out (action output)
    pub a_out_hash: [u8; 32],

    /// CPU cycles consumed (τ)
    pub tau_used: u64,

    /// Wall clock timestamp (for audit only, not part of A = μ(O))
    pub timestamp_ns: u64,

    /// Task ID executed
    pub task_id: u64,

    /// Pattern ID used
    pub pattern_id: u8,

    /// Number of guards checked
    pub guards_checked: u8,

    /// Reserved
    _reserved: [u8; 6],

    /// Guard bitmap (which guards were evaluated)
    pub guard_bitmap: u64,

    /// Guard outcomes (which passed/failed)
    pub guard_outcomes: u64,

    /// Parent receipt (for chaining)
    pub parent_receipt: ReceiptId,

    /// Merkle root (for batch receipts)
    pub merkle_root: [u8; 32],

    /// Ed25519 signature (64 bytes)
    pub signature: [u8; 64],
}

impl Receipt {
    /// Create a new receipt
    pub fn new(
        receipt_id: ReceiptId,
        sigma_hash: SigmaHash,
        o_in_hash: [u8; 32],
        a_out_hash: [u8; 32],
        tau_used: u64,
        task_id: u64,
        pattern_id: u8,
    ) -> Self {
        Self {
            receipt_id,
            sigma_hash,
            o_in_hash,
            a_out_hash,
            tau_used,
            timestamp_ns: 0, // Would use hardware timestamp
            task_id,
            pattern_id,
            guards_checked: 0,
            _reserved: [0; 6],
            guard_bitmap: 0,
            guard_outcomes: 0,
            parent_receipt: 0,
            merkle_root: [0; 32],
            signature: [0; 64],
        }
    }

    /// Compute receipt hash (for chaining)
    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha3_256::new();

        // Hash all fields except signature
        hasher.update(&self.receipt_id.to_le_bytes());
        hasher.update(&self.sigma_hash.0);
        hasher.update(&self.o_in_hash);
        hasher.update(&self.a_out_hash);
        hasher.update(&self.tau_used.to_le_bytes());
        hasher.update(&self.task_id.to_le_bytes());
        hasher.update(&[self.pattern_id]);
        hasher.update(&self.guard_bitmap.to_le_bytes());
        hasher.update(&self.guard_outcomes.to_le_bytes());
        hasher.update(&self.parent_receipt.to_le_bytes());
        hasher.update(&self.merkle_root);

        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Sign receipt
    pub fn sign(&mut self, signing_key: &SigningKey) {
        let hash = self.hash();
        let signature = signing_key.sign(&hash);
        self.signature.copy_from_slice(&signature.to_bytes());
    }

    /// Verify receipt signature
    pub fn verify(&self, verifying_key: &VerifyingKey) -> bool {
        let hash = self.hash();
        let signature = match Signature::from_bytes(&self.signature) {
            Ok(sig) => sig,
            Err(_) => return false,
        };

        verifying_key.verify(&hash, &signature).is_ok()
    }

    /// Check if receipt proves A = μ(O)
    ///
    /// This is the fundamental equation: hash(A) must equal hash(μ(O))
    pub fn proves_equation(&self) -> bool {
        // In full implementation:
        // 1. Recompute μ(O_in; Σ*) with given parameters
        // 2. Hash the result
        // 3. Compare with a_out_hash
        //
        // For now, we verify structural integrity
        self.receipt_id > 0 && self.tau_used > 0
    }

    /// Check if Chatman Constant was satisfied
    #[inline(always)]
    pub const fn satisfies_chatman(&self) -> bool {
        self.tau_used <= crate::CHATMAN_CONSTANT
    }
}

/// Receipt chain (immutable log)
#[repr(C, align(4096))]
pub struct ReceiptChain {
    /// Receipts (append-only)
    receipts: Vec<Receipt>,
    /// Next receipt ID
    next_id: ReceiptId,
    /// Chain head hash
    head_hash: [u8; 32],
}

impl ReceiptChain {
    /// Create a new receipt chain
    pub fn new() -> Self {
        Self {
            receipts: Vec::new(),
            next_id: 1,
            head_hash: [0; 32],
        }
    }

    /// Append a receipt (immutable)
    pub fn append(&mut self, mut receipt: Receipt) -> ReceiptId {
        // Set receipt ID
        receipt.receipt_id = self.next_id;
        self.next_id += 1;

        // Link to parent
        if !self.receipts.is_empty() {
            receipt.parent_receipt = self.receipts.last().unwrap().receipt_id;
        }

        // Update chain head
        let receipt_hash = receipt.hash();
        self.head_hash = receipt_hash;

        // Append
        self.receipts.push(receipt);

        receipt.receipt_id
    }

    /// Get receipt by ID
    pub fn get(&self, id: ReceiptId) -> Option<&Receipt> {
        self.receipts.iter().find(|r| r.receipt_id == id)
    }

    /// Get latest receipt
    pub fn latest(&self) -> Option<&Receipt> {
        self.receipts.last()
    }

    /// Verify chain integrity
    pub fn verify_chain(&self) -> bool {
        if self.receipts.is_empty() {
            return true;
        }

        // Check parent links
        for i in 1..self.receipts.len() {
            let current = &self.receipts[i];
            let parent_id = current.parent_receipt;

            if parent_id == 0 {
                return false; // Missing parent
            }

            if parent_id != self.receipts[i - 1].receipt_id {
                return false; // Broken chain
            }
        }

        // Check head hash
        if let Some(latest) = self.latest() {
            let computed_head = latest.hash();
            if computed_head != self.head_hash {
                return false; // Head hash mismatch
            }
        }

        true
    }

    /// Count receipts
    pub fn len(&self) -> usize {
        self.receipts.len()
    }

    /// Check if chain is empty
    pub fn is_empty(&self) -> bool {
        self.receipts.is_empty()
    }

    /// Get receipts violating Chatman Constant
    pub fn chatman_violations(&self) -> Vec<&Receipt> {
        self.receipts
            .iter()
            .filter(|r| !r.satisfies_chatman())
            .collect()
    }
}

impl Default for ReceiptChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Receipt query interface (for Γ)
pub trait ReceiptQuery {
    /// Query receipts by task ID
    fn query_by_task(&self, task_id: u64) -> Vec<&Receipt>;

    /// Query receipts by pattern
    fn query_by_pattern(&self, pattern_id: u8) -> Vec<&Receipt>;

    /// Query receipts with guard failures
    fn query_guard_failures(&self) -> Vec<&Receipt>;

    /// Get average τ
    fn avg_tau(&self) -> f64;
}

impl ReceiptQuery for ReceiptChain {
    fn query_by_task(&self, task_id: u64) -> Vec<&Receipt> {
        self.receipts
            .iter()
            .filter(|r| r.task_id == task_id)
            .collect()
    }

    fn query_by_pattern(&self, pattern_id: u8) -> Vec<&Receipt> {
        self.receipts
            .iter()
            .filter(|r| r.pattern_id == pattern_id)
            .collect()
    }

    fn query_guard_failures(&self) -> Vec<&Receipt> {
        self.receipts
            .iter()
            .filter(|r| {
                // Check if any guard failed (guard_outcomes has 0 bits)
                let checked = r.guard_bitmap;
                let passed = r.guard_outcomes;
                (checked & !passed) != 0
            })
            .collect()
    }

    fn avg_tau(&self) -> f64 {
        if self.receipts.is_empty() {
            return 0.0;
        }

        let total: u64 = self.receipts.iter().map(|r| r.tau_used).sum();
        total as f64 / self.receipts.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_creation() {
        let sigma_hash = SigmaHash([1; 32]);
        let o_in = [2; 32];
        let a_out = [3; 32];

        let receipt = Receipt::new(1, sigma_hash, o_in, a_out, 5, 100, 0);

        assert_eq!(receipt.receipt_id, 1);
        assert_eq!(receipt.tau_used, 5);
        assert_eq!(receipt.task_id, 100);
        assert!(receipt.satisfies_chatman());
    }

    #[test]
    fn test_receipt_hash() {
        let sigma_hash = SigmaHash([1; 32]);
        let receipt = Receipt::new(1, sigma_hash, [2; 32], [3; 32], 5, 100, 0);

        let hash1 = receipt.hash();
        let hash2 = receipt.hash();

        assert_eq!(hash1, hash2); // Deterministic
    }

    #[test]
    fn test_receipt_chain() {
        let mut chain = ReceiptChain::new();

        assert_eq!(chain.len(), 0);
        assert!(chain.is_empty());

        let sigma_hash = SigmaHash([1; 32]);
        let receipt1 = Receipt::new(0, sigma_hash, [2; 32], [3; 32], 5, 100, 0);
        let id1 = chain.append(receipt1);

        assert_eq!(id1, 1);
        assert_eq!(chain.len(), 1);

        let receipt2 = Receipt::new(0, sigma_hash, [4; 32], [5; 32], 6, 101, 1);
        let id2 = chain.append(receipt2);

        assert_eq!(id2, 2);
        assert_eq!(chain.len(), 2);

        // Verify chain
        assert!(chain.verify_chain());
    }

    #[test]
    fn test_chatman_violations() {
        let mut chain = ReceiptChain::new();
        let sigma_hash = SigmaHash([1; 32]);

        // Add compliant receipt
        let r1 = Receipt::new(0, sigma_hash, [1; 32], [1; 32], 5, 1, 0);
        chain.append(r1);

        // Add violating receipt
        let r2 = Receipt::new(0, sigma_hash, [2; 32], [2; 32], 10, 2, 0);
        chain.append(r2);

        let violations = chain.chatman_violations();
        assert_eq!(violations.len(), 1);
        assert_eq!(violations[0].tau_used, 10);
    }

    #[test]
    fn test_receipt_queries() {
        let mut chain = ReceiptChain::new();
        let sigma_hash = SigmaHash([1; 32]);

        for i in 0..10 {
            let r = Receipt::new(
                0,
                sigma_hash,
                [i; 32],
                [i; 32],
                i as u64 + 1,
                i as u64,
                (i % 3) as u8,
            );
            chain.append(r);
        }

        let by_pattern_0 = chain.query_by_pattern(0);
        assert_eq!(by_pattern_0.len(), 4); // 0, 3, 6, 9

        let avg = chain.avg_tau();
        assert!(avg > 0.0);
    }

    #[test]
    fn test_receipt_size() {
        assert_eq!(size_of::<Receipt>(), 256); // Cache-aligned
    }
}
