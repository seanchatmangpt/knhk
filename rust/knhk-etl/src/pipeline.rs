// rust/knhk-etl/src/pipeline.rs
// Complete ETL pipeline orchestrator

extern crate alloc;

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::emit::{EmitResult, EmitStage};
use crate::error::PipelineError;
use crate::ingest::IngestStage;
use crate::load::LoadStage;
use crate::reflex::ReflexStage;
use crate::transform::TransformStage;

/// Complete ETL pipeline orchestrator
///
/// Orchestrates the full ETL pipeline: Ingest → Transform → Load → Reflex → Emit
///
/// # Example
///
/// ```rust
/// use knhk_etl::Pipeline;
///
/// let mut pipeline = Pipeline::new(
///     vec!["file://data.nt".to_string()],
///     "http://example.org/schema".to_string(),
///     false, // lockchain disabled
///     vec!["http://localhost:8080".to_string()],
/// );
///
/// let result = pipeline.execute();
/// ```
pub struct Pipeline {
    /// Ingest stage (public for tests)
    pub ingest: IngestStage,
    /// Transform stage (public for tests)
    pub transform: TransformStage,
    /// Load stage (public for tests)
    pub load: LoadStage,
    /// Reflex stage (public for tests)
    pub reflex: ReflexStage,
    /// Emit stage (public for tests)
    pub emit: EmitStage,
}

impl Pipeline {
    /// Creates a new KNHK ETL pipeline instance.
    ///
    /// # Purpose
    /// Initializes the complete ETL pipeline for deterministic data processing
    /// with the 8-beat epoch system. The pipeline orchestrates five stages:
    /// Ingest → Transform → Load → Reflex → Emit.
    ///
    /// # Arguments
    /// * `connectors` - List of connector URIs for data ingestion:
    ///   - `file://path/to/data.nt` - Local file connector
    ///   - `http://api.example.com` - HTTP/REST connector
    ///   - `kafka://topic-name` - Kafka stream connector
    /// * `schema_iri` - IRI of the schema for validation (e.g., `http://example.org/schema`)
    /// * `lockchain_enabled` - Enable lockchain for cryptographic receipt provenance
    /// * `downstream_endpoints` - List of endpoints for action emission (HTTP webhooks)
    ///
    /// # Returns
    /// * A new `Pipeline` instance ready for execution
    ///
    /// # Performance
    /// * Initialization: <1ms (cold path, tick budget: 64 ticks)
    /// * Hot path execution: ≤8 ticks per operation (Chatman Constant)
    ///
    /// # Example
    /// ```rust
    /// use knhk_etl::Pipeline;
    ///
    /// // Basic pipeline with file connector
    /// let pipeline = Pipeline::new(
    ///     vec!["file://data/triples.nt".to_string()],
    ///     "http://example.org/schema".to_string(),
    ///     false, // lockchain disabled for dev
    ///     vec!["http://localhost:8080/actions".to_string()],
    /// );
    ///
    /// // Production pipeline with lockchain
    /// let prod_pipeline = Pipeline::new(
    ///     vec![
    ///         "kafka://events".to_string(),
    ///         "http://api.example.com/data".to_string(),
    ///     ],
    ///     "http://prod.example.org/schema".to_string(),
    ///     true, // lockchain enabled for provenance
    ///     vec![
    ///         "http://webhook1.example.com".to_string(),
    ///         "http://webhook2.example.com".to_string(),
    ///     ],
    /// );
    /// ```
    ///
    /// # See Also
    /// * [`Pipeline::execute`] - Execute the full ETL pipeline
    /// * [`BeatScheduler`] - 8-beat epoch scheduling system
    /// * [`HookRegistry`] - Register validation hooks for predicates
    pub fn new(
        connectors: Vec<String>,
        schema_iri: String,
        lockchain_enabled: bool,
        downstream_endpoints: Vec<String>,
    ) -> Self {
        Self {
            ingest: IngestStage::new(connectors, "rdf/turtle".to_string()),
            transform: TransformStage::new(schema_iri, true),
            load: LoadStage::new(),
            reflex: ReflexStage::new(),
            emit: EmitStage::new(lockchain_enabled, downstream_endpoints),
        }
    }

    /// Executes the full ETL pipeline with performance guarantees.
    ///
    /// # Purpose
    /// Orchestrates all five pipeline stages sequentially with deterministic
    /// execution guarantees. Each stage adheres to the ≤8 tick hot path budget
    /// (Chatman Constant) for predictable performance.
    ///
    /// # Pipeline Stages
    /// 1. **Ingest**: Read raw RDF triples from connectors
    /// 2. **Transform**: Hash URIs to u64, validate against schema
    /// 3. **Load**: Group triples into SoA (Structure-of-Arrays) format
    /// 4. **Reflex**: Execute validation hooks, generate receipts
    /// 5. **Emit**: Send actions to downstream endpoints, store receipts
    ///
    /// # Returns
    /// * `Ok(EmitResult)` - Pipeline completed successfully with metrics:
    ///   - `receipts_written`: Number of receipts persisted
    ///   - `actions_sent`: Number of actions emitted to downstream
    ///   - `lockchain_hashes`: Merkle roots if lockchain enabled
    /// * `Err(PipelineError)` - Pipeline stage failed (see Errors)
    ///
    /// # Errors
    /// Returns errors from any stage that fails:
    /// - `IngestError` - Failed to read data from connectors
    /// - `TransformError` - Schema validation failed
    /// - `LoadError` - SoA grouping failed (run length exceeded)
    /// - `ReflexError` - Hook execution exceeded tick budget
    /// - `EmitError` - Failed to emit actions or write receipts
    ///
    /// # Performance Guarantees
    /// * Hot path operations: ≤8 ticks per predicate run
    /// * Load stage: Enforces max run length of 8 triples
    /// * Reflex stage: Tick budget of 8 ticks per hook
    /// * Over-budget work: Parked to warm path (W1) for later processing
    ///
    /// # Example
    /// ```rust
    /// use knhk_etl::Pipeline;
    ///
    /// let mut pipeline = Pipeline::new(
    ///     vec!["file://data/events.nt".to_string()],
    ///     "http://example.org/schema".to_string(),
    ///     false,
    ///     vec!["http://localhost:8080/actions".to_string()],
    /// );
    ///
    /// // Execute pipeline
    /// match pipeline.execute() {
    ///     Ok(result) => {
    ///         println!("Pipeline completed successfully!");
    ///         println!("Receipts written: {}", result.receipts_written);
    ///         println!("Actions sent: {}", result.actions_sent);
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Pipeline failed: {:?}", e);
    ///     }
    /// }
    /// ```
    ///
    /// # See Also
    /// * [`IngestStage::ingest`] - Stage 1: Data ingestion
    /// * [`TransformStage::transform`] - Stage 2: Hashing and validation
    /// * [`LoadStage::load`] - Stage 3: SoA grouping
    /// * [`ReflexStage::reflex`] - Stage 4: Hook execution
    /// * [`EmitStage::emit`] - Stage 5: Action emission
    pub fn execute(&mut self) -> Result<EmitResult, PipelineError> {
        // Stage 1: Ingest
        let ingest_result = self.ingest.ingest()?;

        // Stage 2: Transform
        let transform_result = self.transform.transform(ingest_result)?;

        // Stage 3: Load
        let load_result = self.load.load(transform_result)?;

        // Stage 4: Reflex
        let reflex_result = self.reflex.reflex(load_result)?;

        // Stage 5: Emit
        let emit_result = self.emit.emit(reflex_result)?;

        Ok(emit_result)
    }

    /// Execute pipeline up to Load stage (for hook orchestration)
    ///
    /// Returns LoadResult that can be used for pattern-based hook execution
    pub fn execute_to_load(&mut self) -> Result<crate::load::LoadResult, PipelineError> {
        // Stage 1: Ingest
        let ingest_result = self.ingest.ingest()?;

        // Stage 2: Transform
        let transform_result = self.transform.transform(ingest_result)?;

        // Stage 3: Load
        let load_result = self.load.load(transform_result)?;

        Ok(load_result)
    }
}
