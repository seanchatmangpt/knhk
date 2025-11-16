//! Tests for individual generators

use knhk_ontology::{SigmaSnapshot, SnapshotMetadata, Triple, TripleStore};
use knhk_projections::generators::*;

fn create_test_snapshot() -> SigmaSnapshot {
    let mut store = TripleStore::new();
    store.add(Triple::new("company1", "rdf:type", "Company"));
    store.add(Triple::new("company1", "name", "TechCorp"));
    store.add(Triple::new("company1", "sector", "Technology"));
    store.add(Triple::new("company1", "revenue", "1000000000"));
    store.add(Triple::new("company1", "employees", "5000"));

    SigmaSnapshot::new(None, store, SnapshotMetadata::default())
        .expect("Failed to create snapshot")
}

#[tokio::test]
async fn test_rust_models_generator() {
    let generator = RustModelsGenerator::new();
    let snapshot = create_test_snapshot();

    let output = generator.generate(&snapshot).await.unwrap();

    assert!(!output.models_code.is_empty());
    assert!(output.models_code.contains("struct Company"));
    assert!(output.models_code.contains("pub name: String"));
    assert!(output.models_code.contains("pub sector: String"));
    assert!(output.models_code.contains("pub revenue: i64"));
    assert!(output.models_code.contains("pub employees: i64"));
    assert!(output.models_code.contains("use serde::{Deserialize, Serialize}"));
    assert!(output.hash.len() == 32);
}

#[tokio::test]
async fn test_openapi_generator() {
    let generator = OpenApiGenerator::new();
    let snapshot = create_test_snapshot();

    let output = generator.generate(&snapshot).await.unwrap();

    assert!(!output.openapi_spec.is_empty());
    assert!(output.openapi_spec.contains("openapi: 3.0.0"));
    assert!(output.openapi_spec.contains("Company"));
    assert!(output.paths.len() > 0);
    assert!(output.schemas.len() > 0);
    assert!(output.schemas.contains(&"Company".to_string()));
    assert!(output.hash.len() == 32);
}

#[tokio::test]
async fn test_hooks_generator() {
    let generator = HooksGenerator::new();
    let snapshot = create_test_snapshot();

    let output = generator.generate(&snapshot).await.unwrap();

    assert!(!output.hooks_config.is_empty());
    assert!(output.hooks_config.contains("[hooks]"));
    assert!(output.hooks_config.contains("validate"));
    assert!(output.hooks_config.contains("verify-q"));
    assert!(output.hooks_config.contains("perf-check"));
    assert!(output.operators.len() > 0);
    assert!(output.guards.len() > 0);
    assert!(output.hash.len() == 32);
}

#[tokio::test]
async fn test_markdown_generator() {
    let generator = MarkdownGenerator::new();
    let snapshot = create_test_snapshot();

    let output = generator.generate(&snapshot).await.unwrap();

    assert!(!output.markdown.is_empty());
    assert!(output.markdown.contains("# KNHK Ontology Documentation"));
    assert!(output.markdown.contains("## Classes"));
    assert!(output.markdown.contains("### Company"));
    assert!(output.markdown.contains("## Statistics"));
    assert!(output.sections.len() > 0);
    assert!(output.sections.contains(&"Overview".to_string()));
    assert!(output.sections.contains(&"Classes".to_string()));
    assert!(output.hash.len() == 32);
}

#[tokio::test]
async fn test_otel_generator() {
    let generator = OtelGenerator::new();
    let snapshot = create_test_snapshot();

    let output = generator.generate(&snapshot).await.unwrap();

    assert!(!output.otel_schema.is_empty());
    assert!(output.otel_schema.contains("schema_url"));
    assert!(output.otel_schema.contains("resource_spans"));
    assert!(output.otel_schema.contains("company.process"));
    assert!(output.spans.len() > 0);
    assert!(output.metrics.len() > 0);
    assert!(output.hash.len() == 32);
}

#[tokio::test]
async fn test_generator_determinism() {
    let snapshot = create_test_snapshot();

    // Test each generator twice
    let rust_gen = RustModelsGenerator::new();
    let rust1 = rust_gen.generate(&snapshot).await.unwrap();
    let rust2 = rust_gen.generate(&snapshot).await.unwrap();
    assert_eq!(rust1.hash, rust2.hash, "RustModelsGenerator not deterministic");

    let openapi_gen = OpenApiGenerator::new();
    let openapi1 = openapi_gen.generate(&snapshot).await.unwrap();
    let openapi2 = openapi_gen.generate(&snapshot).await.unwrap();
    assert_eq!(openapi1.hash, openapi2.hash, "OpenApiGenerator not deterministic");

    let hooks_gen = HooksGenerator::new();
    let hooks1 = hooks_gen.generate(&snapshot).await.unwrap();
    let hooks2 = hooks_gen.generate(&snapshot).await.unwrap();
    assert_eq!(hooks1.hash, hooks2.hash, "HooksGenerator not deterministic");

    let markdown_gen = MarkdownGenerator::new();
    let markdown1 = markdown_gen.generate(&snapshot).await.unwrap();
    let markdown2 = markdown_gen.generate(&snapshot).await.unwrap();
    assert_eq!(markdown1.hash, markdown2.hash, "MarkdownGenerator not deterministic");

    let otel_gen = OtelGenerator::new();
    let otel1 = otel_gen.generate(&snapshot).await.unwrap();
    let otel2 = otel_gen.generate(&snapshot).await.unwrap();
    assert_eq!(otel1.hash, otel2.hash, "OtelGenerator not deterministic");
}

#[tokio::test]
async fn test_generators_with_empty_snapshot() {
    let store = TripleStore::new();
    let snapshot = SigmaSnapshot::new(None, store, SnapshotMetadata::default()).unwrap();

    // All generators should handle empty snapshots gracefully
    let rust_gen = RustModelsGenerator::new();
    let rust_output = rust_gen.generate(&snapshot).await;
    assert!(rust_output.is_ok());

    let openapi_gen = OpenApiGenerator::new();
    let openapi_output = openapi_gen.generate(&snapshot).await;
    assert!(openapi_output.is_ok());

    let hooks_gen = HooksGenerator::new();
    let hooks_output = hooks_gen.generate(&snapshot).await;
    assert!(hooks_output.is_ok());

    let markdown_gen = MarkdownGenerator::new();
    let markdown_output = markdown_gen.generate(&snapshot).await;
    assert!(markdown_output.is_ok());

    let otel_gen = OtelGenerator::new();
    let otel_output = otel_gen.generate(&snapshot).await;
    assert!(otel_output.is_ok());
}

#[tokio::test]
async fn test_rust_models_multiple_classes() {
    let mut store = TripleStore::new();

    // Add multiple classes
    store.add(Triple::new("company1", "rdf:type", "Company"));
    store.add(Triple::new("company1", "name", "TechCorp"));

    store.add(Triple::new("person1", "rdf:type", "Person"));
    store.add(Triple::new("person1", "name", "John Doe"));
    store.add(Triple::new("person1", "age", "30"));

    let snapshot = SigmaSnapshot::new(None, store, SnapshotMetadata::default()).unwrap();

    let generator = RustModelsGenerator::new();
    let output = generator.generate(&snapshot).await.unwrap();

    // Should generate both structs
    assert!(output.models_code.contains("struct Company"));
    assert!(output.models_code.contains("struct Person"));
}
