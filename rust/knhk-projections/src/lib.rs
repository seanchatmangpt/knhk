//! KNHK Projection Compiler
//!
//! Generates deterministic artifacts from Σ (ontology) snapshots:
//! - Π_models: Rust structs/enums from RDF classes
//! - Π_apis: OpenAPI 3.0 specifications
//! - Π_hooks: KNHK hook configurations
//! - Π_docs: Markdown documentation
//! - Π_telemetry: OpenTelemetry schemas
//!
//! ## Key Guarantees
//!
//! 1. **Deterministic**: Same Σ_id + same O → same output bits (bit-for-bit)
//! 2. **Comprehensive**: All projections generated in parallel
//! 3. **Fast**: Pre-compiled before promotion (not on hot path)
//! 4. **Cached**: Recompilation avoided if Σ unchanged
//! 5. **Integrated**: Works with SnapshotStore promotion pipeline
//!
//! ## Example
//!
//! ```rust
//! use knhk_projections::{ProjectionCompiler, CompiledProjections};
//! use knhk_ontology::{SigmaSnapshot, SnapshotStore};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create compiler
//! let compiler = ProjectionCompiler::new();
//!
//! // Get snapshot
//! let store = SnapshotStore::new();
//! let snapshot = store.current_snapshot().unwrap();
//!
//! // Compile all projections in parallel
//! let compiled = compiler.compile_all(Arc::new(snapshot)).await?;
//!
//! // Access generated artifacts
//! println!("Rust models:\n{}", compiled.rust_models.models_code);
//! println!("OpenAPI spec:\n{}", compiled.openapi_spec.openapi_spec);
//! # Ok(())
//! # }
//! ```

pub mod cache;
pub mod compiler;
pub mod determinism;
pub mod generators;
pub mod integration;

// Re-export main types
pub use cache::ProjectionCache;
pub use compiler::{CompiledProjections, ProjectionCompiler};
pub use determinism::DeterminismVerifier;
pub use generators::{
    HooksGenerator, HooksOutput, MarkdownGenerator, MarkdownOutput,
    OpenApiGenerator, OpenApiOutput, OtelGenerator, OtelOutput,
    RustModelsGenerator, RustModelsOutput,
};
pub use integration::ProjectionSnapshotStore;

/// Projection compiler errors
#[derive(thiserror::Error, Debug)]
pub enum ProjectionError {
    #[error("Snapshot error: {0}")]
    Snapshot(#[from] knhk_ontology::SnapshotError),

    #[error("Determinism violation: {0}")]
    DeterminismViolation(String),

    #[error("Generation failed: {0}")]
    GenerationFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Cache error: {0}")]
    Cache(String),
}

pub type Result<T> = std::result::Result<T, ProjectionError>;
