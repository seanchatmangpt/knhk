//! Integration Tests for Receipt Chain
//!
//! Tests cryptographic provenance and Γ queries

use ed25519_dalek::SigningKey;
use knhk_mu_kernel::receipts::{Receipt, ReceiptChain, ReceiptQuery};
use knhk_mu_kernel::sigma::SigmaHash;
use rand::rngs::OsRng;

#[test]
fn test_receipt_creation() {
    let sigma_hash = SigmaHash([1; 32]);
    let receipt = Receipt::new(1, sigma_hash, [2; 32], [3; 32], 5, 100, 0);

    assert_eq!(receipt.receipt_id, 1);
    assert_eq!(receipt.tau_used, 5);
    assert_eq!(receipt.task_id, 100);
    assert!(receipt.satisfies_chatman());
}

#[test]
fn test_receipt_hashing() {
    let sigma_hash = SigmaHash([1; 32]);
    let receipt = Receipt::new(1, sigma_hash, [2; 32], [3; 32], 5, 100, 0);

    let hash1 = receipt.hash();
    let hash2 = receipt.hash();

    assert_eq!(hash1, hash2, "Receipt hash should be deterministic");
}

#[test]
fn test_receipt_signing() {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();

    let sigma_hash = SigmaHash([1; 32]);
    let mut receipt = Receipt::new(1, sigma_hash, [2; 32], [3; 32], 5, 100, 0);

    receipt.sign(&signing_key);
    assert!(receipt.verify(&verifying_key), "Signature should verify");
}

#[test]
fn test_receipt_chain_append() {
    let mut chain = ReceiptChain::new();
    let sigma_hash = SigmaHash([1; 32]);

    let receipt1 = Receipt::new(0, sigma_hash, [1; 32], [1; 32], 5, 1, 0);
    let id1 = chain.append(receipt1);

    assert_eq!(id1, 1);
    assert_eq!(chain.len(), 1);

    let receipt2 = Receipt::new(0, sigma_hash, [2; 32], [2; 32], 6, 2, 0);
    let id2 = chain.append(receipt2);

    assert_eq!(id2, 2);
    assert_eq!(chain.len(), 2);
}

#[test]
fn test_receipt_chain_linking() {
    let mut chain = ReceiptChain::new();
    let sigma_hash = SigmaHash([1; 32]);

    // Add multiple receipts
    for i in 0..10 {
        let receipt = Receipt::new(0, sigma_hash, [i; 32], [i; 32], 5, i as u64, 0);
        chain.append(receipt);
    }

    // Verify chain links
    assert!(chain.verify_chain(), "Chain should be valid");

    // Verify parent relationships
    let receipt_5 = chain.get(5).unwrap();
    let receipt_6 = chain.get(6).unwrap();
    assert_eq!(receipt_6.parent_receipt, receipt_5.receipt_id);
}

#[test]
fn test_receipt_chain_integrity() {
    let mut chain = ReceiptChain::new();
    let sigma_hash = SigmaHash([1; 32]);

    for i in 0..100 {
        let receipt = Receipt::new(0, sigma_hash, [i; 32], [i; 32], 5, i as u64, 0);
        chain.append(receipt);
    }

    assert!(
        chain.verify_chain(),
        "Large chain should maintain integrity"
    );
    assert_eq!(chain.len(), 100);
}

#[test]
fn test_chatman_violations_query() {
    let mut chain = ReceiptChain::new();
    let sigma_hash = SigmaHash([1; 32]);

    // Add compliant receipts
    for i in 0..5 {
        let receipt = Receipt::new(0, sigma_hash, [i; 32], [i; 32], 5, i as u64, 0);
        chain.append(receipt);
    }

    // Add violating receipts
    for i in 5..10 {
        let receipt = Receipt::new(0, sigma_hash, [i; 32], [i; 32], 10, i as u64, 0);
        chain.append(receipt);
    }

    let violations = chain.chatman_violations();
    assert_eq!(violations.len(), 5, "Should detect 5 violations");

    for v in violations {
        assert_eq!(v.tau_used, 10);
    }
}

#[test]
fn test_query_by_task() {
    let mut chain = ReceiptChain::new();
    let sigma_hash = SigmaHash([1; 32]);

    // Add receipts for different tasks
    for task_id in 0..10 {
        for _ in 0..3 {
            let receipt = Receipt::new(0, sigma_hash, [0; 32], [0; 32], 5, task_id, 0);
            chain.append(receipt);
        }
    }

    let task_5_receipts = chain.query_by_task(5);
    assert_eq!(
        task_5_receipts.len(),
        3,
        "Should find 3 receipts for task 5"
    );

    for r in task_5_receipts {
        assert_eq!(r.task_id, 5);
    }
}

#[test]
fn test_query_by_pattern() {
    let mut chain = ReceiptChain::new();
    let sigma_hash = SigmaHash([1; 32]);

    // Add receipts with different patterns
    for i in 0..30 {
        let pattern_id = (i % 3) as u8;
        let receipt = Receipt::new(0, sigma_hash, [i; 32], [i; 32], 5, i as u64, pattern_id);
        chain.append(receipt);
    }

    let pattern_0 = chain.query_by_pattern(0);
    assert_eq!(pattern_0.len(), 10, "Should find 10 receipts for pattern 0");

    let pattern_1 = chain.query_by_pattern(1);
    assert_eq!(pattern_1.len(), 10, "Should find 10 receipts for pattern 1");
}

#[test]
fn test_query_guard_failures() {
    let mut chain = ReceiptChain::new();
    let sigma_hash = SigmaHash([1; 32]);

    for i in 0..10 {
        let mut receipt = Receipt::new(0, sigma_hash, [i; 32], [i; 32], 5, i as u64, 0);

        if i % 2 == 0 {
            // Simulate guard failures
            receipt.guard_bitmap = 0b1111; // 4 guards checked
            receipt.guard_outcomes = 0b1100; // 2 passed, 2 failed
        }

        chain.append(receipt);
    }

    let failures = chain.query_guard_failures();
    assert_eq!(
        failures.len(),
        5,
        "Should find 5 receipts with guard failures"
    );
}

#[test]
fn test_avg_tau_calculation() {
    let mut chain = ReceiptChain::new();
    let sigma_hash = SigmaHash([1; 32]);

    // Add receipts with known tau values
    for i in 0..10 {
        let receipt = Receipt::new(0, sigma_hash, [i; 32], [i; 32], i as u64 + 1, i as u64, 0);
        chain.append(receipt);
    }

    let avg = chain.avg_tau();
    // Average of 1,2,3,4,5,6,7,8,9,10 = 5.5
    assert!((avg - 5.5).abs() < 0.01, "Average tau should be 5.5");
}

#[test]
fn test_receipt_proves_equation() {
    let sigma_hash = SigmaHash([1; 32]);
    let receipt = Receipt::new(1, sigma_hash, [2; 32], [3; 32], 5, 100, 0);

    // In full implementation, this would recompute μ(O) and verify hash
    assert!(receipt.proves_equation(), "Receipt should prove A = μ(O)");
}

#[test]
fn test_receipt_immutability() {
    let mut chain = ReceiptChain::new();
    let sigma_hash = SigmaHash([1; 32]);

    let receipt = Receipt::new(0, sigma_hash, [1; 32], [1; 32], 5, 1, 0);
    let hash_before = receipt.hash();

    chain.append(receipt);

    let stored_receipt = chain.get(1).unwrap();
    let hash_after = stored_receipt.hash();

    assert_eq!(hash_before, hash_after, "Receipt should be immutable");
}
