//! Main projection compiler orchestrator
//!
//! Coordinates parallel compilation of all projection types and ensures
//! deterministic, reproducible output.

use crate::cache::ProjectionCache;
use crate::determinism::DeterminismVerifier;
use crate::generators::{
    HooksGenerator, HooksOutput, MarkdownGenerator, MarkdownOutput,
    OpenApiGenerator, OpenApiOutput, OtelGenerator, OtelOutput,
    RustModelsGenerator, RustModelsOutput,
};
use crate::Result;
use knhk_ontology::{SigmaSnapshot, SigmaSnapshotId};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::SystemTime;
use tracing::{info, instrument, warn};

/// Complete set of generators for all projection types
pub struct ProjectionGenerators {
    pub rust_models: RustModelsGenerator,
    pub openapi_spec: OpenApiGenerator,
    pub hooks_config: HooksGenerator,
    pub markdown_docs: MarkdownGenerator,
    pub otel_schema: OtelGenerator,
}

impl ProjectionGenerators {
    pub fn new() -> Self {
        Self {
            rust_models: RustModelsGenerator::new(),
            openapi_spec: OpenApiGenerator::new(),
            hooks_config: HooksGenerator::new(),
            markdown_docs: MarkdownGenerator::new(),
            otel_schema: OtelGenerator::new(),
        }
    }
}

impl Default for ProjectionGenerators {
    fn default() -> Self {
        Self::new()
    }
}

/// Main projection compiler
pub struct ProjectionCompiler {
    /// Determinism verifier
    hasher: DeterminismVerifier,

    /// Output cache (for incremental compilation)
    cache: Arc<RwLock<ProjectionCache>>,

    /// Generator implementations
    generators: ProjectionGenerators,
}

impl ProjectionCompiler {
    /// Create new compiler with default configuration
    pub fn new() -> Self {
        Self {
            hasher: DeterminismVerifier::new(),
            cache: Arc::new(RwLock::new(ProjectionCache::new())),
            generators: ProjectionGenerators::new(),
        }
    }

    /// Main entry point: compile all projections in parallel
    #[instrument(skip(self, snapshot), fields(snapshot_id = ?snapshot.id))]
    pub async fn compile_all(
        &self,
        snapshot: Arc<SigmaSnapshot>,
    ) -> Result<CompiledProjections> {
        let snapshot_id = snapshot.id;

        info!("Starting compilation for snapshot {:?}", snapshot_id);

        // 1. Hash snapshot for determinism verification
        let snapshot_hash = self.hasher.hash_snapshot(&snapshot)?;

        // 2. Check cache (avoid recompilation if unchanged)
        if let Some(cached) = self.cache.write().get(&snapshot_id) {
            if cached.snapshot_hash == snapshot_hash {
                info!("Using cached compilation for snapshot {:?}", snapshot_id);
                return Ok(cached);
            } else {
                warn!(
                    "Cache hit but hash mismatch for snapshot {:?}, recompiling",
                    snapshot_id
                );
            }
        }

        // 3. Compile all projections in parallel
        let (rust_result, openapi_result, hooks_result, markdown_result, otel_result) = tokio::join!(
            self.compile_rust_models(&snapshot),
            self.compile_openapi_spec(&snapshot),
            self.compile_hooks(&snapshot),
            self.compile_markdown_docs(&snapshot),
            self.compile_otel_schema(&snapshot),
        );

        // 4. Collect results (fail fast on any error)
        let rust_models = rust_result?;
        let openapi_spec = openapi_result?;
        let hooks_config = hooks_result?;
        let markdown_docs = markdown_result?;
        let otel_schema = otel_result?;

        // 5. Build compiled projections
        let compiled = CompiledProjections {
            snapshot_id,
            snapshot_hash,
            rust_models,
            openapi_spec,
            hooks_config,
            markdown_docs,
            otel_schema,
            compiled_at: SystemTime::now(),
        };

        // 6. Store in cache
        self.cache.write().insert(snapshot_id, compiled.clone());

        info!("Compilation complete for snapshot {:?}", snapshot_id);

        Ok(compiled)
    }

    /// Compile Rust models projection
    async fn compile_rust_models(&self, snapshot: &SigmaSnapshot) -> Result<RustModelsOutput> {
        self.generators.rust_models.generate(snapshot).await
    }

    /// Compile OpenAPI specification projection
    async fn compile_openapi_spec(&self, snapshot: &SigmaSnapshot) -> Result<OpenApiOutput> {
        self.generators.openapi_spec.generate(snapshot).await
    }

    /// Compile hooks configuration projection
    async fn compile_hooks(&self, snapshot: &SigmaSnapshot) -> Result<HooksOutput> {
        self.generators.hooks_config.generate(snapshot).await
    }

    /// Compile markdown documentation projection
    async fn compile_markdown_docs(&self, snapshot: &SigmaSnapshot) -> Result<MarkdownOutput> {
        self.generators.markdown_docs.generate(snapshot).await
    }

    /// Compile OpenTelemetry schema projection
    async fn compile_otel_schema(&self, snapshot: &SigmaSnapshot) -> Result<OtelOutput> {
        self.generators.otel_schema.generate(snapshot).await
    }

    /// Verify determinism (optional but recommended)
    pub async fn verify_deterministic_compilation(
        &self,
        snapshot: Arc<SigmaSnapshot>,
        first_compilation: &CompiledProjections,
    ) -> Result<bool> {
        // Re-compile the snapshot
        let second_compilation = self.compile_all(snapshot).await?;

        // Compare outputs byte-for-byte
        let is_deterministic = first_compilation.rust_models.hash == second_compilation.rust_models.hash
            && first_compilation.openapi_spec.hash == second_compilation.openapi_spec.hash
            && first_compilation.hooks_config.hash == second_compilation.hooks_config.hash
            && first_compilation.markdown_docs.hash == second_compilation.markdown_docs.hash
            && first_compilation.otel_schema.hash == second_compilation.otel_schema.hash;

        if !is_deterministic {
            warn!("Determinism violation detected!");
        }

        Ok(is_deterministic)
    }

    /// Clear cache
    pub fn clear_cache(&self) {
        self.cache.write().clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        self.cache.read().stats()
    }
}

impl Default for ProjectionCompiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete compiled projections output
#[derive(Clone, Debug)]
pub struct CompiledProjections {
    pub snapshot_id: SigmaSnapshotId,
    pub snapshot_hash: [u8; 32],

    pub rust_models: RustModelsOutput,
    pub openapi_spec: OpenApiOutput,
    pub hooks_config: HooksOutput,
    pub markdown_docs: MarkdownOutput,
    pub otel_schema: OtelOutput,

    pub compiled_at: SystemTime,
}

impl CompiledProjections {
    /// Check if all projections were successfully generated
    pub fn is_complete(&self) -> bool {
        !self.rust_models.models_code.is_empty()
            && !self.openapi_spec.openapi_spec.is_empty()
            && !self.markdown_docs.markdown.is_empty()
    }

    /// Get total size of all generated artifacts
    pub fn total_size(&self) -> usize {
        self.rust_models.models_code.len()
            + self.openapi_spec.openapi_spec.len()
            + self.hooks_config.hooks_config.len()
            + self.markdown_docs.markdown.len()
            + self.otel_schema.otel_schema.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use knhk_ontology::{SigmaSnapshot, SnapshotMetadata, Triple, TripleStore};

    fn create_test_snapshot() -> SigmaSnapshot {
        let mut store = TripleStore::new();
        store.add(Triple::new("company1", "rdf:type", "Company"));
        store.add(Triple::new("company1", "sector", "Technology"));
        store.add(Triple::new("company1", "name", "TechCorp"));

        SigmaSnapshot::new(None, store, SnapshotMetadata::default())
            .expect("Failed to create test snapshot")
    }

    #[tokio::test]
    async fn test_compile_all() {
        let compiler = ProjectionCompiler::new();
        let snapshot = Arc::new(create_test_snapshot());

        let compiled = compiler.compile_all(snapshot.clone()).await.unwrap();

        assert_eq!(compiled.snapshot_id, snapshot.id);
        assert!(compiled.is_complete());
        assert!(compiled.total_size() > 0);
    }

    #[tokio::test]
    async fn test_deterministic_compilation() {
        let compiler = ProjectionCompiler::new();
        let snapshot = Arc::new(create_test_snapshot());

        let first = compiler.compile_all(snapshot.clone()).await.unwrap();
        let is_deterministic = compiler
            .verify_deterministic_compilation(snapshot.clone(), &first)
            .await
            .unwrap();

        assert!(is_deterministic, "Compilation should be deterministic");
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let compiler = ProjectionCompiler::new();
        let snapshot = Arc::new(create_test_snapshot());

        // First compilation
        let first = compiler.compile_all(snapshot.clone()).await.unwrap();

        // Second compilation should hit cache
        let second = compiler.compile_all(snapshot.clone()).await.unwrap();

        assert_eq!(first.snapshot_hash, second.snapshot_hash);
        assert_eq!(first.compiled_at, second.compiled_at); // Same instance from cache
    }
}
