//! Integration tests for projection compiler

use knhk_ontology::{SigmaSnapshot, SnapshotMetadata, Triple, TripleStore};
use knhk_projections::ProjectionCompiler;
use std::sync::Arc;

fn create_sample_snapshot() -> SigmaSnapshot {
    let mut store = TripleStore::new();

    // Add sample Fortune 500 company data
    store.add(Triple::new("company1", "rdf:type", "Company"));
    store.add(Triple::new("company1", "name", "Apple Inc."));
    store.add(Triple::new("company1", "sector", "Technology"));
    store.add(Triple::new("company1", "revenue", "394328000000"));
    store.add(Triple::new("company1", "employees", "164000"));

    store.add(Triple::new("company2", "rdf:type", "Company"));
    store.add(Triple::new("company2", "name", "Microsoft"));
    store.add(Triple::new("company2", "sector", "Technology"));
    store.add(Triple::new("company2", "revenue", "211915000000"));
    store.add(Triple::new("company2", "employees", "221000"));

    SigmaSnapshot::new(
        None,
        store,
        SnapshotMetadata {
            created_by: "test".to_string(),
            description: "Fortune 500 sample data".to_string(),
            sector: Some("Technology".to_string()),
            ..Default::default()
        },
    )
    .expect("Failed to create snapshot")
}

#[tokio::test]
async fn test_compile_all_projections() {
    let compiler = ProjectionCompiler::new();
    let snapshot = Arc::new(create_sample_snapshot());

    let compiled = compiler
        .compile_all(snapshot.clone())
        .await
        .expect("Compilation failed");

    // Verify all projections generated
    assert!(compiled.is_complete());
    assert_eq!(compiled.snapshot_id, snapshot.id);

    // Verify Rust models
    assert!(!compiled.rust_models.models_code.is_empty());
    assert!(compiled.rust_models.models_code.contains("struct Company"));

    // Verify OpenAPI spec
    assert!(!compiled.openapi_spec.openapi_spec.is_empty());
    assert!(compiled.openapi_spec.openapi_spec.contains("openapi: 3.0.0"));

    // Verify hooks config
    assert!(!compiled.hooks_config.hooks_config.is_empty());
    assert!(compiled.hooks_config.hooks_config.contains("[hooks]"));

    // Verify markdown docs
    assert!(!compiled.markdown_docs.markdown.is_empty());
    assert!(compiled.markdown_docs.markdown.contains("# KNHK Ontology Documentation"));

    // Verify OTEL schema
    assert!(!compiled.otel_schema.otel_schema.is_empty());
    assert!(compiled.otel_schema.otel_schema.contains("schema_url"));
}

#[tokio::test]
async fn test_parallel_compilation_performance() {
    let compiler = ProjectionCompiler::new();
    let snapshot = Arc::new(create_sample_snapshot());

    let start = std::time::Instant::now();
    let _compiled = compiler
        .compile_all(snapshot.clone())
        .await
        .expect("Compilation failed");
    let duration = start.elapsed();

    // Compilation should be fast (< 100ms for small snapshot)
    assert!(duration.as_millis() < 100, "Compilation took too long: {:?}", duration);
}

#[tokio::test]
async fn test_cache_effectiveness() {
    let compiler = ProjectionCompiler::new();
    let snapshot = Arc::new(create_sample_snapshot());

    // First compilation (cache miss)
    let first = compiler
        .compile_all(snapshot.clone())
        .await
        .expect("First compilation failed");

    // Second compilation (should hit cache)
    let second = compiler
        .compile_all(snapshot.clone())
        .await
        .expect("Second compilation failed");

    // Should be identical (same instance from cache)
    assert_eq!(first.snapshot_hash, second.snapshot_hash);
    assert_eq!(first.compiled_at, second.compiled_at);

    // Check cache stats
    let (hits, misses) = compiler.cache_stats();
    assert_eq!(hits, 1, "Expected 1 cache hit");
    assert_eq!(misses, 1, "Expected 1 cache miss");
}

#[tokio::test]
async fn test_deterministic_output() {
    let compiler = ProjectionCompiler::new();
    let snapshot = Arc::new(create_sample_snapshot());

    // Clear cache to force recompilation
    compiler.clear_cache();

    // First compilation
    let first = compiler
        .compile_all(snapshot.clone())
        .await
        .expect("First compilation failed");

    // Clear cache again
    compiler.clear_cache();

    // Second compilation (should produce identical output)
    let second = compiler
        .compile_all(snapshot.clone())
        .await
        .expect("Second compilation failed");

    // Compare hashes (should be identical)
    assert_eq!(first.rust_models.hash, second.rust_models.hash);
    assert_eq!(first.openapi_spec.hash, second.openapi_spec.hash);
    assert_eq!(first.hooks_config.hash, second.hooks_config.hash);
    assert_eq!(first.markdown_docs.hash, second.markdown_docs.hash);
    assert_eq!(first.otel_schema.hash, second.otel_schema.hash);
}

#[tokio::test]
async fn test_empty_snapshot() {
    let compiler = ProjectionCompiler::new();
    let store = TripleStore::new(); // Empty store

    let snapshot = Arc::new(
        SigmaSnapshot::new(None, store, SnapshotMetadata::default())
            .expect("Failed to create empty snapshot"),
    );

    let compiled = compiler
        .compile_all(snapshot)
        .await
        .expect("Compilation of empty snapshot failed");

    // Should still generate valid (albeit minimal) output
    assert!(!compiled.rust_models.models_code.is_empty());
    assert!(!compiled.openapi_spec.openapi_spec.is_empty());
}

#[tokio::test]
async fn test_large_snapshot_compilation() {
    let compiler = ProjectionCompiler::new();
    let mut store = TripleStore::new();

    // Create snapshot with many companies
    for i in 1..=100 {
        let company_id = format!("company{}", i);
        store.add(Triple::new(&company_id, "rdf:type", "Company"));
        store.add(Triple::new(&company_id, "name", &format!("Company {}", i)));
        store.add(Triple::new(&company_id, "sector", "Technology"));
        store.add(Triple::new(&company_id, "revenue", "1000000000"));
    }

    let snapshot = Arc::new(
        SigmaSnapshot::new(None, store, SnapshotMetadata::default())
            .expect("Failed to create large snapshot"),
    );

    let start = std::time::Instant::now();
    let compiled = compiler
        .compile_all(snapshot)
        .await
        .expect("Large snapshot compilation failed");
    let duration = start.elapsed();

    assert!(compiled.is_complete());

    // Should still be reasonably fast even with 100 companies
    assert!(duration.as_millis() < 500, "Large compilation took too long: {:?}", duration);
}
