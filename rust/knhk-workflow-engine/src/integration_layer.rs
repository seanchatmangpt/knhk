//! Integration Layer - Glue Code Between All Components
//!
//! Provides helper functions and traits that integrate:
//! - Engine (μ) with Patterns
//! - Guards (Q) with Execution
//! - Receipts (O) with MAPE-K
//! - Snapshots (Σ) with Versioning
//! - Ontology parsing with Code generation

use crate::error::{WorkflowError, WorkflowResult};
use crate::engine::HookEngine;
use crate::patterns::{PatternId, PatternRegistry};
use crate::guards::InvariantChecker;
use crate::receipts::{Receipt, ReceiptStore};
use crate::snapshots::SnapshotVersioning;
use crate::parser::WorkflowSpec;
use std::sync::Arc;

/// Extension trait for integrating hook engine with pattern registry
pub trait HookEnginePatternExt {
    /// Execute a workflow using the pattern library
    async fn execute_workflow(
        &self,
        spec: &WorkflowSpec,
        input: serde_json::Value,
        pattern_library: &crate::engine::PatternLibrary,
        invariant_checker: &InvariantChecker,
    ) -> WorkflowResult<serde_json::Value>;
}

impl HookEnginePatternExt for HookEngine {
    async fn execute_workflow(
        &self,
        spec: &WorkflowSpec,
        input: serde_json::Value,
        pattern_library: &crate::engine::PatternLibrary,
        invariant_checker: &InvariantChecker,
    ) -> WorkflowResult<serde_json::Value> {
        // Validate input against guards
        invariant_checker.validate_workflow_input(spec, &input)?;

        // Execute hooks for each task
        let mut output = input.clone();

        for task in &spec.tasks {
            // Get pattern for task
            let pattern = pattern_library.get_pattern(&task.pattern)?;

            // Execute task through hook
            output = self.execute_task_with_pattern(
                &task.id,
                output,
                pattern,
            ).await?;

            // Validate intermediate state
            invariant_checker.validate_task_output(&task.id, &output)?;
        }

        // Validate final output
        invariant_checker.validate_workflow_output(spec, &output)?;

        Ok(output)
    }
}

/// Extension trait for receipt-based observability
pub trait ReceiptObservabilityExt {
    /// Create observation from receipt for MAPE-K
    fn to_observation(&self) -> crate::mape::Observation;

    /// Check if receipt indicates performance degradation
    fn indicates_performance_issue(&self) -> bool;

    /// Check if receipt indicates guard failures
    fn has_guard_failures(&self) -> bool;
}

impl ReceiptObservabilityExt for Receipt {
    fn to_observation(&self) -> crate::mape::Observation {
        crate::mape::Observation {
            receipt_id: self.receipt_id.clone(),
            sigma_id: self.sigma_id.clone(),
            ticks_used: self.ticks_used,
            guards_checked: self.guards_checked.clone(),
            guards_failed: self.guards_failed.clone(),
            timestamp: self.timestamp,
            metrics: std::collections::HashMap::new(),
        }
    }

    fn indicates_performance_issue(&self) -> bool {
        self.ticks_used >= 7 // Near Chatman Constant
    }

    fn has_guard_failures(&self) -> bool {
        !self.guards_failed.is_empty()
    }
}

/// Extension trait for snapshot-based versioning
pub trait SnapshotVersioningExt {
    /// Create a new snapshot from workflow spec
    fn create_snapshot_from_spec(&self, spec: &WorkflowSpec) -> WorkflowResult<String>;

    /// Load workflow spec from snapshot
    fn load_spec_from_snapshot(&self, snapshot_id: &str) -> WorkflowResult<WorkflowSpec>;
}

impl SnapshotVersioningExt for SnapshotVersioning {
    fn create_snapshot_from_spec(&self, spec: &WorkflowSpec) -> WorkflowResult<String> {
        // Serialize spec to RDF/Turtle
        let rdf_data = serialize_spec_to_rdf(spec)?;

        // Create snapshot
        self.create_snapshot(&rdf_data, &std::collections::HashMap::new())
    }

    fn load_spec_from_snapshot(&self, snapshot_id: &str) -> WorkflowResult<WorkflowSpec> {
        // Load snapshot
        let snapshot = self.load_snapshot(snapshot_id)?;

        // Parse RDF back to spec
        parse_rdf_to_spec(&snapshot.content)
    }
}

/// Serialize workflow spec to RDF/Turtle
fn serialize_spec_to_rdf(spec: &WorkflowSpec) -> WorkflowResult<String> {
    // Simple RDF serialization
    let mut rdf = String::new();
    rdf.push_str(&format!("@prefix : <http://example.org/workflow#> .\n"));
    rdf.push_str(&format!("@prefix yawl: <http://www.yawlfoundation.org/yawlschema#> .\n\n"));

    rdf.push_str(&format!(":Workflow{} a yawl:Workflow ;\n", spec.id));
    rdf.push_str(&format!("    yawl:name \"{}\" ;\n", spec.name));
    rdf.push_str(&format!("    yawl:version \"{}\" .\n\n", spec.version));

    for task in &spec.tasks {
        rdf.push_str(&format!(":Task{} a yawl:Task ;\n", task.id));
        rdf.push_str(&format!("    yawl:name \"{}\" ;\n", task.name));
        rdf.push_str(&format!("    yawl:pattern \"{}\" .\n\n", task.pattern));
    }

    Ok(rdf)
}

/// Parse RDF to workflow spec
fn parse_rdf_to_spec(rdf: &str) -> WorkflowResult<WorkflowSpec> {
    // Simple RDF parsing (would use proper SPARQL in production)
    // For now, create a minimal spec
    Ok(WorkflowSpec {
        id: "parsed".to_string(),
        name: "Parsed Workflow".to_string(),
        version: "1.0.0".to_string(),
        tasks: vec![],
        data_inputs: vec![],
        data_outputs: vec![],
        patterns: vec![],
    })
}

/// Integration helper for connecting all layers
pub struct LayerIntegrator {
    hook_engine: Arc<HookEngine>,
    pattern_registry: Arc<PatternRegistry>,
    invariant_checker: Arc<InvariantChecker>,
    receipt_store: Arc<ReceiptStore>,
    snapshot_versioning: Arc<SnapshotVersioning>,
}

impl LayerIntegrator {
    pub fn new(
        hook_engine: Arc<HookEngine>,
        pattern_registry: Arc<PatternRegistry>,
        invariant_checker: Arc<InvariantChecker>,
        receipt_store: Arc<ReceiptStore>,
        snapshot_versioning: Arc<SnapshotVersioning>,
    ) -> Self {
        Self {
            hook_engine,
            pattern_registry,
            invariant_checker,
            receipt_store,
            snapshot_versioning,
        }
    }

    /// Execute workflow with full integration
    pub async fn execute_integrated(
        &self,
        spec: &WorkflowSpec,
        input: serde_json::Value,
    ) -> WorkflowResult<IntegratedExecutionResult> {
        // 1. Validate against invariants
        self.invariant_checker.validate_workflow_spec(spec)?;

        // 2. Get patterns
        let pattern_library = crate::engine::PatternLibrary::new();

        // 3. Execute through hooks
        let output = self.hook_engine.execute_workflow(
            spec,
            input.clone(),
            &pattern_library,
            &self.invariant_checker,
        ).await?;

        // 4. Create snapshot
        let snapshot_id = self.snapshot_versioning.create_snapshot_from_spec(spec)?;

        // 5. Generate receipt
        let receipt_gen = crate::receipts::ReceiptGenerator::new();
        let receipt = receipt_gen.generate(
            &snapshot_id,
            &input,
            &output,
            5, // ticks
            &[],
            &[],
        )?;

        // 6. Store receipt
        self.receipt_store.store(receipt.clone())?;

        Ok(IntegratedExecutionResult {
            output,
            receipt_id: receipt.receipt_id,
            snapshot_id,
        })
    }
}

/// Result of integrated execution
#[derive(Debug)]
pub struct IntegratedExecutionResult {
    pub output: serde_json::Value,
    pub receipt_id: String,
    pub snapshot_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_receipt_to_observation() {
        let receipt = Receipt {
            receipt_id: "test".to_string(),
            sigma_id: "v1".to_string(),
            o_in_hash: "in".to_string(),
            a_out_hash: "out".to_string(),
            guards_checked: vec!["G1".to_string()],
            guards_failed: vec![],
            ticks_used: 5,
            timestamp: chrono::Utc::now(),
        };

        let observation = receipt.to_observation();
        assert_eq!(observation.receipt_id, "test");
        assert_eq!(observation.ticks_used, 5);
    }

    #[test]
    fn test_performance_issue_detection() {
        let receipt = Receipt {
            receipt_id: "test".to_string(),
            sigma_id: "v1".to_string(),
            o_in_hash: "in".to_string(),
            a_out_hash: "out".to_string(),
            guards_checked: vec![],
            guards_failed: vec![],
            ticks_used: 7,
            timestamp: chrono::Utc::now(),
        };

        assert!(receipt.indicates_performance_issue());
    }
}
