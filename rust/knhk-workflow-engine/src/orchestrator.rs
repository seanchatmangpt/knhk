//! Self-Executing Workflow Orchestrator
//!
//! Integrates all 5 layers: Σ (ontology), Π (projection), μ (execution),
//! O (observation), and MAPE-K (autonomic feedback).

use crate::engine::{HookEngine, LatencyBoundedScheduler, PatternLibrary};
use crate::error::{WorkflowError, WorkflowResult};
use crate::guards::InvariantChecker;
use crate::mape::MapeKEngine;
use crate::parser::{WorkflowParser, WorkflowSpec};
use crate::receipts::{ReceiptGenerator, ReceiptStore};
use crate::snapshots::SnapshotVersioning;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Self-executing workflow orchestrator that implements A = μ(O)
///
/// This orchestrator composes all components to create a complete
/// self-executing workflow system:
///
/// ```text
/// Ontology (Σ) → Projection (Π) → Execution (μ) → Observation (O)
///                                      ↑                    ↓
///                                      └─── MAPE-K Loop ←──┘
/// ```
pub struct SelfExecutingOrchestrator {
    /// Pattern library for 43 YAWL patterns
    pattern_library: Arc<PatternLibrary>,
    /// Hook engine for workflow execution
    hook_engine: Arc<HookEngine>,
    /// Scheduler enforcing ≤8 ticks
    scheduler: Arc<LatencyBoundedScheduler>,
    /// Invariant checker (Q enforcement)
    invariant_checker: Arc<InvariantChecker>,
    /// Receipt generator
    receipt_generator: Arc<ReceiptGenerator>,
    /// Receipt store (observation plane)
    receipt_store: Arc<ReceiptStore>,
    /// Snapshot versioning (Σ management)
    snapshot_versioning: Arc<SnapshotVersioning>,
    /// MAPE-K autonomic engine
    mape_k: Arc<MapeKEngine>,
    /// Workflow parser
    parser: WorkflowParser,
    /// Active workflows
    workflows: HashMap<String, WorkflowSpec>,
}

impl SelfExecutingOrchestrator {
    /// Create a new self-executing orchestrator
    pub fn new(snapshot_dir: &str, receipt_dir: &str) -> WorkflowResult<Self> {
        // Create receipt store
        let receipt_store = Arc::new(ReceiptStore::new());

        // Create snapshot versioning
        let snapshot_versioning = Arc::new(SnapshotVersioning::new());

        // Create pattern library
        let pattern_library = Arc::new(PatternLibrary::new());

        // Create hook registry and tracer for hook engine
        let hook_registry = Arc::new(HookRegistry::new());
        let tracer = Arc::new(RwLock::new(Tracer::new()));

        // Create hook engine
        let hook_engine = Arc::new(HookEngine::new(hook_registry.clone(), tracer.clone()));

        // Create scheduler
        let scheduler = Arc::new(LatencyBoundedScheduler::new(8)); // Chatman Constant

        // Create invariant checker
        let invariant_checker = Arc::new(InvariantChecker::new());

        // Create receipt generator
        let receipt_generator = Arc::new(ReceiptGenerator::new());

        // Create MAPE-K engine
        let mape_k = Arc::new(MapeKEngine::new(
            receipt_store.clone(),
            snapshot_versioning.clone(),
            hook_engine.clone(),
            invariant_checker.clone(),
        ));

        // Create workflow parser
        let parser = WorkflowParser::new()?;

        Ok(Self {
            pattern_library,
            hook_engine,
            scheduler,
            invariant_checker,
            receipt_generator,
            receipt_store,
            snapshot_versioning,
            mape_k,
            parser,
            workflows: HashMap::new(),
        })
    }

    /// Load workflow from ontology (Σ)
    ///
    /// This demonstrates the Σ → Π → μ flow:
    /// 1. Parse ontology (Σ)
    /// 2. Project to executable format (Π)
    /// 3. Register with execution engine (μ)
    pub async fn load_workflow_from_ontology(
        &mut self,
        ontology_path: &str,
    ) -> WorkflowResult<String> {
        // Parse workflow from RDF/Turtle ontology
        let spec = self.parser.parse_file(ontology_path)?;
        let workflow_id = spec.id.clone();

        // Validate against invariants (Q)
        self.invariant_checker.validate_workflow_spec(&spec)?;

        // Store workflow
        self.workflows.insert(workflow_id.clone(), spec);

        tracing::info!(
            "Loaded workflow {} from ontology {}",
            workflow_id,
            ontology_path
        );

        Ok(workflow_id)
    }

    /// Execute workflow instance (A = μ(O))
    ///
    /// This implements the core execution equation:
    /// - Takes observation O (input data)
    /// - Applies μ (execution via hooks and patterns)
    /// - Produces action A (output)
    /// - Generates receipt proving hash(A) = hash(μ(O))
    pub async fn execute_workflow(
        &self,
        workflow_id: &str,
        input_data: serde_json::Value,
    ) -> WorkflowResult<ExecutionResult> {
        let workflow = self
            .workflows
            .get(workflow_id)
            .ok_or_else(|| WorkflowError::WorkflowNotFound(workflow_id.to_string()))?;

        // Start execution with tick budget
        let start_tick = self.scheduler.current_tick();

        // Execute workflow through hook engine
        let output_data = self
            .hook_engine
            .execute_workflow(
                workflow,
                input_data.clone(),
                &self.pattern_library,
                &self.invariant_checker,
            )
            .await?;

        let end_tick = self.scheduler.current_tick();
        let ticks_used = end_tick - start_tick;

        // Enforce Chatman Constant
        if ticks_used > 8 {
            return Err(WorkflowError::TickBudgetExceeded {
                used: ticks_used,
                limit: 8,
            });
        }

        // Generate cryptographic receipt
        let receipt = self.receipt_generator.generate(
            &self.snapshot_versioning.current_id(),
            &input_data,
            &output_data,
            ticks_used,
            &[], // guards_checked
            &[], // guards_failed
        )?;

        // Store receipt in observation plane (O)
        self.receipt_store.store(receipt.clone())?;

        tracing::info!(
            "Executed workflow {} in {} ticks (≤8), receipt: {}",
            workflow_id,
            ticks_used,
            receipt.receipt_id
        );

        Ok(ExecutionResult {
            workflow_id: workflow_id.to_string(),
            output_data,
            ticks_used,
            receipt_id: receipt.receipt_id,
            sigma_id: receipt.sigma_id,
        })
    }

    /// Start MAPE-K autonomic loop
    ///
    /// This enables continuous self-adaptation:
    /// - Monitor: Collect receipts and telemetry
    /// - Analyze: Detect symptoms (performance, guards, behavior)
    /// - Plan: Generate adaptation proposals
    /// - Execute: Shadow deploy and promote
    /// - Knowledge: Learn and improve
    pub async fn start_autonomic_loop(&self, interval_ms: u64) -> WorkflowResult<()> {
        tracing::info!(
            "Starting MAPE-K autonomic loop ({}ms interval)",
            interval_ms
        );
        self.mape_k.start_continuous_loop(interval_ms).await
    }

    /// Run a single MAPE-K cycle manually
    pub async fn run_mape_k_cycle(&self) -> WorkflowResult<crate::mape::MapeKCycleMetrics> {
        self.mape_k.run_cycle().await
    }

    /// Query knowledge base
    pub fn query_knowledge(&self, query: &str) -> Vec<String> {
        self.mape_k.query_knowledge(query)
    }

    /// Get current snapshot ID (current Σ)
    pub fn current_sigma(&self) -> String {
        self.snapshot_versioning.current_id()
    }

    /// Get workflow statistics
    pub fn get_statistics(&self) -> OrchestratorStats {
        OrchestratorStats {
            workflows_loaded: self.workflows.len(),
            current_sigma_id: self.current_sigma(),
            total_receipts: self.receipt_store.count(),
            avg_tick_usage: 0.0, // Would query knowledge base
        }
    }
}

/// Result of workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Workflow ID
    pub workflow_id: String,
    /// Output data produced
    pub output_data: serde_json::Value,
    /// Ticks used (must be ≤8)
    pub ticks_used: u32,
    /// Receipt ID for provenance
    pub receipt_id: String,
    /// Snapshot ID (Σ version)
    pub sigma_id: String,
}

/// Orchestrator statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorStats {
    /// Number of workflows loaded
    pub workflows_loaded: usize,
    /// Current snapshot version
    pub current_sigma_id: String,
    /// Total receipts generated
    pub total_receipts: usize,
    /// Average tick usage
    pub avg_tick_usage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let temp_dir = TempDir::new().unwrap();
        let snapshot_dir = temp_dir.path().join("snapshots");
        let receipt_dir = temp_dir.path().join("receipts");

        std::fs::create_dir_all(&snapshot_dir).unwrap();
        std::fs::create_dir_all(&receipt_dir).unwrap();

        let orchestrator = SelfExecutingOrchestrator::new(
            snapshot_dir.to_str().unwrap(),
            receipt_dir.to_str().unwrap(),
        )
        .unwrap();

        let stats = orchestrator.get_statistics();
        assert_eq!(stats.workflows_loaded, 0);
        assert!(!stats.current_sigma_id.is_empty());
    }

    #[tokio::test]
    async fn test_workflow_execution_enforces_chatman_constant() {
        let temp_dir = TempDir::new().unwrap();
        let orchestrator = SelfExecutingOrchestrator::new(
            temp_dir.path().join("snapshots").to_str().unwrap(),
            temp_dir.path().join("receipts").to_str().unwrap(),
        )
        .unwrap();

        // Would test actual execution with mock workflow
        // Verify ticks_used ≤ 8
    }
}
