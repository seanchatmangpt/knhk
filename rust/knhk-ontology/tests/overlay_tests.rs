//! Overlay tests - isolation, commit, rollback

use knhk_ontology::{
    InvariantValidator, SigmaOverlay, SigmaSnapshot, SnapshotMetadata,
    Triple, TriplePattern, TripleStore,
};

#[test]
fn test_overlay_add_triples() {
    let mut base_store = TripleStore::new();
    base_store.add(Triple::new("base", "data", "original"));

    let base = SigmaSnapshot::new(None, base_store, SnapshotMetadata::default())
        .expect("Failed to create base");

    let mut overlay = SigmaOverlay::new(base.id, "test additions".to_string());
    overlay.add_triple(Triple::new("new", "data", "added"));
    overlay.add_triple(Triple::new("another", "data", "added"));

    let virtual_snap = overlay.apply(&base);

    // Should have base + 2 new triples
    assert_eq!(virtual_snap.triples.len(), 3);
}

#[test]
fn test_overlay_remove_triples() {
    let mut base_store = TripleStore::new();
    base_store.add(Triple::new("alice", "knows", "bob"));
    base_store.add(Triple::new("alice", "age", "30"));
    base_store.add(Triple::new("bob", "age", "25"));

    let base = SigmaSnapshot::new(None, base_store, SnapshotMetadata::default())
        .expect("Failed to create base");

    let mut overlay = SigmaOverlay::new(base.id, "test removals".to_string());

    // Remove all triples where subject is "alice"
    overlay.remove_pattern(TriplePattern::new(
        Some("alice".to_string()),
        None,
        None,
    ));

    let virtual_snap = overlay.apply(&base);

    // Should only have bob's age triple
    assert_eq!(virtual_snap.triples.len(), 1);
    assert_eq!(virtual_snap.triples[0].subject, "bob");
}

#[test]
fn test_overlay_add_and_remove() {
    let mut base_store = TripleStore::new();
    base_store.add(Triple::new("old", "data", "remove_me"));
    base_store.add(Triple::new("keep", "data", "this"));

    let base = SigmaSnapshot::new(None, base_store, SnapshotMetadata::default())
        .expect("Failed to create base");

    let mut overlay = SigmaOverlay::new(base.id, "test both".to_string());

    // Remove old data
    overlay.remove_pattern(TriplePattern::new(
        Some("old".to_string()),
        None,
        None,
    ));

    // Add new data
    overlay.add_triple(Triple::new("new", "data", "added"));

    let virtual_snap = overlay.apply(&base);

    // Should have: keep + new (old removed)
    assert_eq!(virtual_snap.triples.len(), 2);

    let has_keep = virtual_snap.triples.iter().any(|t| t.subject == "keep");
    let has_new = virtual_snap.triples.iter().any(|t| t.subject == "new");
    let has_old = virtual_snap.triples.iter().any(|t| t.subject == "old");

    assert!(has_keep);
    assert!(has_new);
    assert!(!has_old);
}

#[test]
fn test_overlay_isolation_from_base() {
    let mut base_store = TripleStore::new();
    base_store.add(Triple::new("base", "data", "value"));

    let base = SigmaSnapshot::new(None, base_store, SnapshotMetadata::default())
        .expect("Failed to create base");

    let base_count = base.all_triples().len();

    // Create overlay and add lots of data
    let mut overlay = SigmaOverlay::new(base.id, "test isolation".to_string());
    for i in 0..100 {
        overlay.add_triple(Triple::new(format!("new{}", i), "data", "value"));
    }

    let _virtual = overlay.apply(&base);

    // Base snapshot should be completely unchanged
    assert_eq!(base.all_triples().len(), base_count);
}

#[test]
fn test_overlay_commit_creates_new_snapshot() {
    let mut base_store = TripleStore::new();
    base_store.add(Triple::new("base", "data", "value"));
    base_store.add(Triple::new("company1", "sector", "Technology"));

    let base = SigmaSnapshot::new(None, base_store, SnapshotMetadata::default())
        .expect("Failed to create base");

    let mut overlay = SigmaOverlay::new(base.id, "test commit".to_string());
    overlay.add_triple(Triple::new("new", "data", "committed"));
    overlay.add_triple(Triple::new("company2", "sector", "Healthcare")); // Add sector for completeness

    // Validate overlay (use generous performance limit for tests)
    let validator = InvariantValidator::new()
        .with_min_sectors(1)
        .with_max_ticks(100); // Test mode: generous limit
    let receipt = overlay.validate(&base, &validator)
        .expect("Validation failed");

    if !receipt.production_ready {
        eprintln!("Receipt not production ready:");
        eprintln!("  Static: {}", receipt.validation_results.static_checks_passed);
        eprintln!("  Dynamic: {}", receipt.validation_results.dynamic_checks_passed);
        eprintln!("  Performance: {}", receipt.validation_results.performance_checks_passed);
        eprintln!("  Invariants Q: {}", receipt.validation_results.invariants_q_preserved);
        for error in &receipt.validation_results.errors {
            eprintln!("  Error {}: {}", error.code, error.message);
        }
    }

    assert!(receipt.production_ready);

    // Commit overlay
    let new_snapshot = overlay.commit(&base, receipt)
        .expect("Commit failed");

    // New snapshot should have different ID
    assert_ne!(new_snapshot.id, base.id);

    // New snapshot should have parent
    assert_eq!(new_snapshot.parent_id, Some(base.id));

    // New snapshot should have all triples
    // base (2) + overlay (2) = 4 total
    assert_eq!(new_snapshot.all_triples().len(), 4);
}

#[test]
fn test_overlay_commit_requires_valid_receipt() {
    let mut base_store = TripleStore::new();
    base_store.add(Triple::new("base", "data", "value"));

    let base = SigmaSnapshot::new(None, base_store, SnapshotMetadata::default())
        .expect("Failed to create base");

    let mut overlay = SigmaOverlay::new(base.id, "test bad commit".to_string());
    overlay.add_triple(Triple::new("new", "data", "value"));

    // Create invalid receipt (failed validation)
    let bad_results = knhk_ontology::ValidationResults {
        static_checks_passed: false,
        dynamic_checks_passed: false,
        performance_checks_passed: false,
        invariants_q_preserved: false,
        errors: vec![],
        warnings: vec![],
    };

    let bad_receipt = knhk_ontology::SigmaReceipt::new(
        [0u8; 32],
        None,
        "bad".to_string(),
        bad_results,
        100,
    );

    // Commit should fail
    let result = overlay.commit(&base, bad_receipt);
    assert!(result.is_err());
}

#[test]
fn test_overlay_double_commit_fails() {
    let mut base_store = TripleStore::new();
    base_store.add(Triple::new("base", "data", "value"));
    base_store.add(Triple::new("company1", "sector", "Technology"));

    let base = SigmaSnapshot::new(None, base_store, SnapshotMetadata::default())
        .expect("Failed to create base");

    let mut overlay = SigmaOverlay::new(base.id, "test double commit".to_string());
    overlay.add_triple(Triple::new("new", "data", "value"));

    let validator = InvariantValidator::new().with_min_sectors(1);
    let receipt1 = overlay.validate(&base, &validator)
        .expect("Validation failed");

    // First commit succeeds and consumes overlay
    let _snapshot = overlay.commit(&base, receipt1)
        .expect("First commit failed");

    // Cannot commit again because overlay was moved
    // (This test verifies that commit takes ownership)
    // If we try to use overlay again, it won't compile:
    // let result = overlay.commit(&base, receipt2); // ERROR: value used after move

    // Instead, verify that marking an overlay as committed prevents re-commit
    let mut overlay2 = SigmaOverlay::new(base.id, "test already committed".to_string());
    overlay2.add_triple(Triple::new("new", "data", "value"));
    overlay2.committed = true; // Manually mark as committed

    let receipt2 = overlay2.validate(&base, &validator)
        .expect("Validation failed");

    // Second commit should fail (overlay already marked as committed)
    let result = overlay2.commit(&base, receipt2);
    assert!(result.is_err());
}

#[test]
fn test_parallel_overlays_independent() {
    let mut base_store = TripleStore::new();
    base_store.add(Triple::new("base", "data", "original"));

    let base = SigmaSnapshot::new(None, base_store, SnapshotMetadata::default())
        .expect("Failed to create base");

    // Create two independent overlays
    let mut overlay1 = SigmaOverlay::new(base.id, "experiment A".to_string());
    overlay1.add_triple(Triple::new("exp_a", "result", "value_a"));

    let mut overlay2 = SigmaOverlay::new(base.id, "experiment B".to_string());
    overlay2.add_triple(Triple::new("exp_b", "result", "value_b"));

    // Apply both
    let virtual1 = overlay1.apply(&base);
    let virtual2 = overlay2.apply(&base);

    // Both should have base + their own triple
    assert_eq!(virtual1.triples.len(), 2);
    assert_eq!(virtual2.triples.len(), 2);

    // Each has different data
    let has_a_in_1 = virtual1.triples.iter().any(|t| t.subject == "exp_a");
    let has_b_in_2 = virtual2.triples.iter().any(|t| t.subject == "exp_b");
    let has_b_in_1 = virtual1.triples.iter().any(|t| t.subject == "exp_b");
    let has_a_in_2 = virtual2.triples.iter().any(|t| t.subject == "exp_a");

    assert!(has_a_in_1);
    assert!(has_b_in_2);
    assert!(!has_b_in_1);
    assert!(!has_a_in_2);
}
