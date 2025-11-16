//! KNHK Ontology Runtime (Σ Runtime)
//!
//! Manages immutable ontology snapshots with atomic promotion.
//!
//! ## Core Concepts
//!
//! - **Snapshots**: Immutable, versioned, hash-identified ontology states
//! - **Overlays**: Staged changes without modifying base snapshots
//! - **Promotion**: Atomic pointer swap to change active snapshot (≤10 ticks)
//! - **Receipts**: Cryptographic proofs of validation and correctness
//!
//! ## Guarantees
//!
//! 1. **Immutability**: Snapshots are write-once, never modified
//! 2. **Atomicity**: Promotion is single atomic operation
//! 3. **Performance**: Hot path ≤8 ticks, promotion ≤10 ticks
//! 4. **Correctness**: All changes validated against 5 invariants (Q)
//! 5. **Provenance**: Complete audit trail of all changes
//!
//! ## Example
//!
//! ```rust
//! use knhk_ontology::{
//!     SnapshotStore, SigmaSnapshot, TripleStore, Triple,
//!     SnapshotMetadata, InvariantValidator, SigmaOverlay,
//! };
//!
//! // Create store
//! let store = SnapshotStore::new();
//!
//! // Create initial snapshot
//! let mut triple_store = TripleStore::new();
//! triple_store.add(Triple::new("company1", "sector", "Technology"));
//!
//! let snapshot = SigmaSnapshot::new(
//!     None,
//!     triple_store,
//!     SnapshotMetadata::default(),
//! ).expect("Failed to create snapshot");
//!
//! // Validate and add receipt
//! let validator = InvariantValidator::new();
//! let results = validator.validate(&snapshot);
//!
//! // Store snapshot
//! let snapshot_id = store.add_snapshot(snapshot);
//!
//! // Create overlay for changes
//! let mut overlay = SigmaOverlay::new(snapshot_id, "Add new company".to_string());
//! overlay.add_triple(Triple::new("company2", "sector", "Healthcare"));
//!
//! // Validate overlay
//! let base = store.get_snapshot(&snapshot_id).unwrap();
//! let receipt = overlay.validate(&base, &validator).unwrap();
//!
//! // Commit overlay as new snapshot
//! let new_snapshot = overlay.commit(&base, receipt).unwrap();
//! let new_id = store.add_snapshot(new_snapshot);
//!
//! // Atomically promote to production
//! store.promote_snapshot(new_id).expect("Failed to promote");
//! ```

pub mod overlay;
pub mod promotion;
pub mod receipt;
pub mod snapshot;
pub mod validator;

// Re-export main types
pub use overlay::{SigmaOverlay, TriplePattern, VirtualSnapshot};
pub use promotion::{PromotionError, PromotionEvent, SnapshotStore};
pub use receipt::{SigmaReceipt, ValidationError, ValidationResults};
pub use snapshot::{
    SigmaSnapshot, SigmaSnapshotId, SnapshotError, SnapshotMetadata,
    Triple, TripleStore,
};
pub use validator::{InvariantValidator, ValidatorError};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_workflow() {
        // Create store
        let store = SnapshotStore::new();

        // Create initial snapshot
        let mut triple_store = TripleStore::new();
        triple_store.add(Triple::new("alice", "knows", "bob"));
        triple_store.add(Triple::new("company1", "sector", "Technology"));

        let snapshot1 = SigmaSnapshot::new(
            None,
            triple_store,
            SnapshotMetadata {
                created_by: "system".to_string(),
                description: "Initial snapshot".to_string(),
                ..Default::default()
            },
        ).expect("Failed to create snapshot1");

        // Validate (use generous performance limit for tests)
        let validator = InvariantValidator::new()
            .with_min_sectors(1)
            .with_max_ticks(100); // Test mode: generous limit
        let results = validator.validate(&snapshot1);
        assert!(results.passed());

        // Add receipt
        let receipt1 = SigmaReceipt::new(
            snapshot1.id,
            None,
            "Initial creation".to_string(),
            results,
            100,
        );

        let snapshot1 = snapshot1.with_receipt(receipt1);
        let id1 = snapshot1.id;

        // Add to store
        store.add_snapshot(snapshot1);

        // Promote to production
        store.promote_snapshot(id1).expect("Failed to promote snapshot1");

        assert_eq!(store.current_snapshot().unwrap().id, id1);

        // Create overlay for changes
        let mut overlay = SigmaOverlay::new(id1, "Add new data".to_string());
        overlay.add_triple(Triple::new("charlie", "knows", "david"));
        overlay.add_triple(Triple::new("company2", "sector", "Healthcare"));

        // Validate overlay
        let base = store.get_snapshot(&id1).unwrap();
        let receipt2 = overlay.validate(&base, &validator)
            .expect("Overlay validation failed");

        // Debug output if validation failed
        if !receipt2.production_ready {
            eprintln!("Validation failed:");
            eprintln!("  Static: {}", receipt2.validation_results.static_checks_passed);
            eprintln!("  Dynamic: {}", receipt2.validation_results.dynamic_checks_passed);
            eprintln!("  Performance: {}", receipt2.validation_results.performance_checks_passed);
            eprintln!("  Invariants Q: {}", receipt2.validation_results.invariants_q_preserved);
            for error in &receipt2.validation_results.errors {
                eprintln!("  Error {}: {}", error.code, error.message);
            }
        }

        assert!(receipt2.production_ready);

        // Commit overlay
        let snapshot2 = overlay.commit(&base, receipt2)
            .expect("Failed to commit overlay");

        let id2 = snapshot2.id;
        store.add_snapshot(snapshot2);

        // Promote new snapshot
        store.promote_snapshot(id2).expect("Failed to promote snapshot2");

        assert_eq!(store.current_snapshot().unwrap().id, id2);

        // Verify lineage
        let lineage = store.snapshot_lineage(id2);
        assert_eq!(lineage.len(), 2);
        assert_eq!(lineage[0], id2);
        assert_eq!(lineage[1], id1);

        // Verify promotion history
        let history = store.promotion_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].to_snapshot_id, id1);
        assert_eq!(history[1].to_snapshot_id, id2);
    }

    #[test]
    fn test_parallel_overlays() {
        // Create store and base snapshot
        let store = SnapshotStore::new();

        let mut triple_store = TripleStore::new();
        triple_store.add(Triple::new("base", "data", "original"));

        let snapshot = SigmaSnapshot::new(
            None,
            triple_store,
            SnapshotMetadata::default(),
        ).expect("Failed to create snapshot");

        let validator = InvariantValidator::new();
        let results = validator.validate(&snapshot);
        let receipt = SigmaReceipt::new(
            snapshot.id,
            None,
            "Base".to_string(),
            results,
            100,
        );

        let snapshot = snapshot.with_receipt(receipt);
        let base_id = snapshot.id;
        store.add_snapshot(snapshot);

        // Create two parallel overlays
        let mut overlay1 = SigmaOverlay::new(base_id, "Experiment A".to_string());
        overlay1.add_triple(Triple::new("exp", "a", "value1"));

        let mut overlay2 = SigmaOverlay::new(base_id, "Experiment B".to_string());
        overlay2.add_triple(Triple::new("exp", "b", "value2"));
        overlay2.add_triple(Triple::new("exp", "c", "value3")); // Add extra triple to make counts different

        // Both overlays work independently
        let base = store.get_snapshot(&base_id).unwrap();

        let virtual1 = overlay1.apply(&base);
        let virtual2 = overlay2.apply(&base);

        // Each virtual snapshot has different data
        // overlay1: base (1) + new (1) = 2
        // overlay2: base (1) + new (2) = 3
        assert_eq!(virtual1.triples.len(), 2);
        assert_eq!(virtual2.triples.len(), 3);
        assert_ne!(virtual1.triples.len(), virtual2.triples.len());

        // Base snapshot unchanged
        assert_eq!(base.all_triples().len(), 1);
    }
}
