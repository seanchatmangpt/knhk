//! Snapshot tests - immutability, creation, lineage

use knhk_ontology::{
    SigmaSnapshot, SnapshotMetadata, Triple, TripleStore,
};

#[test]
fn test_snapshot_immutability() {
    let mut store = TripleStore::new();
    store.add(Triple::new("alice", "age", "30"));

    let snapshot = SigmaSnapshot::new(None, store, SnapshotMetadata::default())
        .expect("Failed to create snapshot");

    let original_id = snapshot.id;
    let triples_count = snapshot.all_triples().len();

    // Snapshot ID should never change
    assert_eq!(snapshot.id, original_id);

    // Triple count should remain constant
    assert_eq!(snapshot.all_triples().len(), triples_count);
}

#[test]
fn test_snapshot_with_lineage() {
    // Create parent snapshot
    let mut store1 = TripleStore::new();
    store1.add(Triple::new("gen1", "data", "value1"));

    let parent = SigmaSnapshot::new(None, store1, SnapshotMetadata::default())
        .expect("Failed to create parent");

    // Create child snapshot
    let mut store2 = TripleStore::new();
    store2.add(Triple::new("gen2", "data", "value2"));

    let child = SigmaSnapshot::new(Some(parent.id), store2, SnapshotMetadata::default())
        .expect("Failed to create child");

    // Verify lineage
    assert_eq!(child.parent_id, Some(parent.id));
    assert_eq!(child.lineage_depth(), 1);
    assert_eq!(parent.lineage_depth(), 0);
}

#[test]
fn test_snapshot_different_data_different_id() {
    let mut store1 = TripleStore::new();
    store1.add(Triple::new("a", "b", "c"));

    let mut store2 = TripleStore::new();
    store2.add(Triple::new("d", "e", "f"));

    let snap1 = SigmaSnapshot::new(None, store1, SnapshotMetadata::default())
        .expect("Failed to create snap1");

    let snap2 = SigmaSnapshot::new(None, store2, SnapshotMetadata::default())
        .expect("Failed to create snap2");

    // Different data = different IDs
    assert_ne!(snap1.id, snap2.id);
}

#[test]
fn test_snapshot_metadata() {
    let mut store = TripleStore::new();
    store.add(Triple::new("test", "data", "value"));

    let metadata = SnapshotMetadata {
        created_by: "test-user".to_string(),
        description: "Test snapshot with metadata".to_string(),
        sector: Some("Technology".to_string()),
        ..Default::default()
    };

    let snapshot = SigmaSnapshot::new(None, store, metadata)
        .expect("Failed to create snapshot");

    assert_eq!(snapshot.metadata.created_by, "test-user");
    assert_eq!(snapshot.metadata.description, "Test snapshot with metadata");
    assert_eq!(snapshot.metadata.sector, Some("Technology".to_string()));
}

#[test]
fn test_snapshot_query_operations() {
    let mut store = TripleStore::new();
    store.add(Triple::new("alice", "knows", "bob"));
    store.add(Triple::new("alice", "age", "30"));
    store.add(Triple::new("bob", "age", "25"));

    let snapshot = SigmaSnapshot::new(None, store, SnapshotMetadata::default())
        .expect("Failed to create snapshot");

    // Query by subject
    let alice_triples = snapshot.query_subject("alice");
    assert_eq!(alice_triples.len(), 2);

    let bob_triples = snapshot.query_subject("bob");
    assert_eq!(bob_triples.len(), 1);

    // Get all triples
    let all_triples = snapshot.all_triples();
    assert_eq!(all_triples.len(), 3);
}

#[test]
fn test_production_ready_flag() {
    let mut store = TripleStore::new();
    store.add(Triple::new("test", "data", "value"));

    let snapshot = SigmaSnapshot::new(None, store, SnapshotMetadata::default())
        .expect("Failed to create snapshot");

    // Without receipt, not production ready
    assert!(!snapshot.is_production_ready());
}

#[test]
fn test_snapshot_promotion_marking() {
    let mut store = TripleStore::new();
    store.add(Triple::new("test", "data", "value"));

    let snapshot = SigmaSnapshot::new(None, store, SnapshotMetadata::default())
        .expect("Failed to create snapshot");

    assert!(snapshot.promoted_at.is_none());

    let promoted = snapshot.mark_promoted();
    assert!(promoted.promoted_at.is_some());
}
