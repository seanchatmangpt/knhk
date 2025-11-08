// Tests for hot-path receipt processing kernels
// Tests Δ-Composer, Receipt-Hasher, Verifier, Pruner as described in yawl.txt

use knhk_hot::receipt_kernels::{
    DeltaComposer, Pruner, ReceiptDelta, ReceiptHasher, ReceiptPipeline, Verifier,
};

#[test]
fn test_delta_composer_creates_fold() {
    let mut composer = DeltaComposer::new(8); // 2³ = 8 deltas per fold

    // Add 8 deltas
    for i in 0..8 {
        let delta = ReceiptDelta {
            hash: [i as u64, 0, 0, 0],
            timestamp: i * 1000,
            tick: i,
        };
        let complete = composer.compose_delta(&delta);
        if i == 7 {
            assert!(complete, "Fold should be complete after 8 deltas");
        } else {
            assert!(!complete, "Fold should not be complete before 8 deltas");
        }
    }

    // Take fold
    let fold = composer.take_fold();
    assert_eq!(fold.fold_count, 8);
    assert_eq!(fold.first_tick, 0);
    assert_eq!(fold.last_tick, 7);
}

#[test]
fn test_receipt_hasher_creates_deterministic_hash() {
    let hasher = ReceiptHasher::new(0);
    let fold = knhk_hot::receipt_kernels::ReceiptFold {
        root_hash: [1, 2, 3, 4],
        fold_count: 8,
        first_tick: 0,
        last_tick: 7,
    };

    let hash1 = hasher.hash_fold(&fold);
    let hash2 = hasher.hash_fold(&fold);

    // Hash should be deterministic
    assert_eq!(hash1, hash2);
}

#[test]
fn test_verifier_validates_fold() {
    let mut verifier = Verifier::new(10); // max 10 folds

    let fold = knhk_hot::receipt_kernels::ReceiptFold {
        root_hash: [1, 2, 3, 4],
        fold_count: 8,
        first_tick: 0,
        last_tick: 7,
    };

    let hasher = ReceiptHasher::new(0);
    let hash = hasher.hash_fold(&fold);

    // Verify fold
    assert!(verifier.verify_fold(&fold, &hash));

    // Add fold to table
    let result = verifier.add_fold(fold, hash);
    assert!(result.is_some());
    assert_eq!(verifier.fold_table_size(), 1);
}

#[test]
fn test_verifier_compacts_folds() {
    let mut verifier = Verifier::new(2); // max 2 folds (triggers compaction)

    let hasher = ReceiptHasher::new(0);

    // Add 3 folds (should trigger compaction)
    for i in 0..3 {
        let fold = knhk_hot::receipt_kernels::ReceiptFold {
            root_hash: [i as u64, 0, 0, 0],
            fold_count: 8,
            first_tick: i * 8,
            last_tick: (i + 1) * 8 - 1,
        };
        let hash = hasher.hash_fold(&fold);
        verifier.add_fold(fold, hash);
    }

    // After compaction, should have <= 2 folds
    assert!(verifier.fold_table_size() <= 2);
}

#[test]
fn test_pruner_discards_null_deltas() {
    let mut pruner = Pruner::new(100);

    // Null delta (all zeros)
    let null_delta = ReceiptDelta {
        hash: [0; 4],
        timestamp: 1000,
        tick: 0,
    };

    assert!(pruner.prune_delta(&null_delta), "Should discard null delta");
}

#[test]
fn test_pruner_discards_idempotent_deltas() {
    let mut pruner = Pruner::new(100);

    let delta = ReceiptDelta {
        hash: [1, 2, 3, 4],
        timestamp: 1000,
        tick: 0,
    };

    // First time: should not be pruned
    assert!(
        !pruner.prune_delta(&delta),
        "Should not prune first occurrence"
    );

    // Second time: should be pruned (idempotent)
    assert!(pruner.prune_delta(&delta), "Should prune idempotent delta");
}

#[test]
fn test_receipt_pipeline_processes_deltas() {
    let mut pipeline = ReceiptPipeline::new(8, 10, 100); // fold_size=8, max_folds=10, max_seen=100

    // Process 8 deltas (should complete fold)
    for i in 0..8 {
        let delta = ReceiptDelta {
            hash: [i as u64, 0, 0, 0],
            timestamp: i * 1000,
            tick: i,
        };
        let result = pipeline.process_delta(delta);
        if i == 7 {
            assert!(result.is_some(), "Should complete fold after 8 deltas");
        } else {
            assert!(result.is_none(), "Should not complete fold before 8 deltas");
        }
    }

    // Fold table should have 1 entry
    assert_eq!(pipeline.fold_table_size(), 1);
}

#[test]
fn test_receipt_pipeline_prunes_null_deltas() {
    let mut pipeline = ReceiptPipeline::new(8, 10, 100);

    // Process null delta (should be pruned)
    let null_delta = ReceiptDelta {
        hash: [0; 4],
        timestamp: 1000,
        tick: 0,
    };

    let result = pipeline.process_delta(null_delta);
    assert!(result.is_none(), "Should prune null delta");
}

#[test]
fn test_receipt_pipeline_fold_table_size_limit() {
    let mut pipeline = ReceiptPipeline::new(8, 2, 100); // max_folds=2 (triggers compaction)

    // Process 24 deltas (should create 3 folds, trigger compaction)
    for i in 0..24 {
        let delta = ReceiptDelta {
            hash: [i as u64, 0, 0, 0],
            timestamp: i * 1000,
            tick: i,
        };
        pipeline.process_delta(delta);
    }

    // After compaction, should have <= 2 folds
    assert!(pipeline.fold_table_size() <= 2);
}
