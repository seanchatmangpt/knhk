//! Self-Executing Workflow Coordinator
//!
//! Integrates all execution components (Σ*, receipts, hooks, MAPE-K) into a unified
//! workflow execution system that aligns with DOCTRINE_2027.md and implements
//! A = μ(O) with continuous MAPE-K feedback.
//!
//! # Architecture
//!
//! ```text
//! Observation (O) → Hook Engine (μ) → Action (A) → Receipt
//!                         ↑                              ↓
//!                    Snapshot (Σ*)                    Γ(O)
//!                         ↑                              ↓
//!                      MAPE-K ←─────────────────────────┘
//! ```

use std::sync::Arc;
use std::collections::HashMap;

use super::snapshot::{SnapshotId, SnapshotStore, SnapshotManifest, OntologyFile};
use super::receipt::{Receipt, ReceiptStore, ReceiptId};
use super::hooks::{HookEngine, HookRegistry, HookContext, HookResult, HookFn};
use crate::observability::{MapekManager, DarkMatterDetector};

/// Self-executing workflow coordinator
///
/// Combines all components for autonomous workflow execution
pub struct SelfExecutingWorkflow {
    /// Hook registry (μ functions)
    registry: Arc<HookRegistry>,
    /// Snapshot store (Σ* versions)
    snapshots: Arc<SnapshotStore>,
    /// Receipt store (Γ(O) history)
    receipts: Arc<ReceiptStore>,
    /// Hook execution engine
    engine: Arc<HookEngine>,
    /// MAPE-K autonomic manager
    mape_k: Arc<MapekManager>,
}

impl SelfExecutingWorkflow {
    /// Create a new self-executing workflow coordinator
    pub fn new() -> Self {
        let registry = Arc::new(HookRegistry::new());
        let snapshots = Arc::new(SnapshotStore::new());
        let receipts = Arc::new(ReceiptStore::new());
        let engine = Arc::new(HookEngine::new(
            registry.clone(),
            snapshots.clone(),
            receipts.clone(),
        ));

        // Create dark matter detector and MAPE-K manager
        let detector = Arc::new(DarkMatterDetector::new());
        let mape_k = Arc::new(MapekManager::new(detector));

        Self {
            registry,
            snapshots,
            receipts,
            engine,
            mape_k,
        }
    }

    /// Register a workflow hook
    pub fn register_hook(&self, name: String, hook_fn: HookFn) -> Result<(), String> {
        self.registry.register(name, hook_fn)
    }

    /// Create a new ontology snapshot
    pub fn create_snapshot(&self, ontology_files: Vec<OntologyFile>) -> Result<SnapshotId, String> {
        #[cfg(feature = "std")]
        let id = SnapshotId::new(0);

        #[cfg(not(feature = "std"))]
        let id = SnapshotId::from_string("Σ_default_001".to_string());

        let manifest = SnapshotManifest::new(id.clone(), ontology_files);
        self.snapshots.store(manifest)?;
        Ok(id)
    }

    /// Set active snapshot (atomic pointer update)
    pub fn set_active_snapshot(&self, id: SnapshotId) -> Result<(), String> {
        self.snapshots.set_current(id)
    }

    /// Execute a workflow with observation O → action A
    pub fn execute(
        &self,
        hook_name: &str,
        observation: Vec<u8>,
        workflow_instance_id: String,
    ) -> Result<ReceiptId, String> {
        // Get current snapshot
        let snapshot = self.snapshots.get_current()?;

        // Create execution context
        let context = HookContext {
            snapshot_id: snapshot.id,
            workflow_instance_id,
            input_data: observation,
            variables: HashMap::new(),
        };

        // Execute hook and generate receipt
        let receipt_id = self.engine.execute(hook_name, context)?;

        // Run MAPE-K cycle to adapt system
        let autonomic = self.mape_k.run_cycle();

        if !autonomic {
            eprintln!("WARNING: MAPE-K detected edge case requiring human intervention");
        }

        Ok(receipt_id)
    }

    /// Execute a complete workflow (sequence of hooks)
    pub fn execute_workflow(
        &self,
        hooks: &[String],
        observation: Vec<u8>,
        workflow_instance_id: String,
    ) -> Result<Vec<ReceiptId>, String> {
        let snapshot = self.snapshots.get_current()?;

        let context = HookContext {
            snapshot_id: snapshot.id,
            workflow_instance_id,
            input_data: observation,
            variables: HashMap::new(),
        };

        let receipts = self.engine.execute_workflow(hooks, context)?;

        // Run MAPE-K cycle after workflow
        self.mape_k.run_cycle();

        Ok(receipts)
    }

    /// Query receipt history (Γ(O))
    pub fn query_receipts(&self, workflow_id: &str) -> Result<Vec<Receipt>, String> {
        self.receipts.get_by_workflow(workflow_id)
    }

    /// Get receipt statistics (for monitoring)
    pub fn get_statistics(&self) -> Result<crate::execution::ReceiptStatistics, String> {
        self.receipts.get_statistics()
    }

    /// Promote a shadow snapshot to production
    ///
    /// This is the MAPE-K "Execute" phase for ontology updates
    pub fn promote_snapshot(&self, from: &SnapshotId, to: &SnapshotId) -> Result<(), String> {
        self.snapshots.promote(from, to)
    }

    /// Get all snapshots (for MAPE-K knowledge base)
    pub fn list_snapshots(&self) -> Result<Vec<SnapshotId>, String> {
        self.snapshots.list_all()
    }

    /// Get MAPE-K manager reference (for advanced monitoring)
    pub fn mape_k(&self) -> &Arc<MapekManager> {
        &self.mape_k
    }
}

impl Default for SelfExecutingWorkflow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::hooks::patterns;

    #[test]
    fn test_self_executing_workflow_creation() {
        let workflow = SelfExecutingWorkflow::new();
        assert!(workflow.list_snapshots().unwrap().is_empty());
    }

    #[test]
    fn test_snapshot_creation_and_activation() {
        let workflow = SelfExecutingWorkflow::new();

        let files = vec![
            OntologyFile {
                path: "workflow.ttl".to_string(),
                content_hash: "sha3-256:abc123".to_string(),
                size_bytes: 1024,
            },
        ];

        let id = workflow.create_snapshot(files).unwrap();
        workflow.set_active_snapshot(id.clone()).unwrap();

        let snapshots = workflow.list_snapshots().unwrap();
        assert_eq!(snapshots.len(), 1);
    }

    #[test]
    fn test_hook_registration_and_execution() {
        let workflow = SelfExecutingWorkflow::new();

        // Create snapshot
        let files = vec![
            OntologyFile {
                path: "workflow.ttl".to_string(),
                content_hash: "sha3-256:abc123".to_string(),
                size_bytes: 1024,
            },
        ];

        let id = workflow.create_snapshot(files).unwrap();
        workflow.set_active_snapshot(id).unwrap();

        // Register hook
        let hook = Arc::new(|ctx: &HookContext| {
            let mut result = HookResult::success(ctx.input_data.clone(), 3);
            result.add_guard_check("TEST_GUARD".to_string(), true);
            result
        });

        workflow.register_hook("test_hook".to_string(), hook).unwrap();

        // Execute
        let observation = b"test observation".to_vec();
        let receipt_id = workflow.execute(
            "test_hook",
            observation,
            "workflow-instance-1".to_string(),
        ).unwrap();

        // Verify receipt
        let receipt = workflow.receipts.get(&receipt_id).unwrap();
        assert!(receipt.success);
        assert!(receipt.ticks_used <= 8); // Chatman constant
    }

    #[test]
    fn test_full_workflow_execution() {
        let workflow = SelfExecutingWorkflow::new();

        // Create snapshot
        let files = vec![
            OntologyFile {
                path: "workflow.ttl".to_string(),
                content_hash: "sha3-256:abc123".to_string(),
                size_bytes: 1024,
            },
        ];

        let id = workflow.create_snapshot(files).unwrap();
        workflow.set_active_snapshot(id).unwrap();

        // Register multiple hooks
        for i in 1..=3 {
            let hook = Arc::new(move |ctx: &HookContext| {
                let mut output = ctx.input_data.clone();
                output.push(i as u8);
                HookResult::success(output, 2)
            });
            workflow.register_hook(format!("step_{}", i), hook).unwrap();
        }

        // Execute workflow
        let hooks = vec![
            "step_1".to_string(),
            "step_2".to_string(),
            "step_3".to_string(),
        ];

        let observation = b"initial".to_vec();
        let receipts = workflow.execute_workflow(
            &hooks,
            observation,
            "workflow-instance-1".to_string(),
        ).unwrap();

        assert_eq!(receipts.len(), 3);

        // Verify all receipts
        for receipt_id in receipts {
            let receipt = workflow.receipts.get(&receipt_id).unwrap();
            assert!(receipt.success);
        }
    }

    #[test]
    fn test_mape_k_integration() {
        let workflow = SelfExecutingWorkflow::new();

        // Create snapshot
        let files = vec![
            OntologyFile {
                path: "workflow.ttl".to_string(),
                content_hash: "sha3-256:abc123".to_string(),
                size_bytes: 1024,
            },
        ];

        let id = workflow.create_snapshot(files).unwrap();
        workflow.set_active_snapshot(id).unwrap();

        // Register hook
        let hook = Arc::new(|ctx: &HookContext| {
            HookResult::success(ctx.input_data.clone(), 1)
        });
        workflow.register_hook("test_hook".to_string(), hook).unwrap();

        // Execute multiple times to generate data for MAPE-K
        for i in 0..10 {
            let observation = format!("observation-{}", i).into_bytes();
            let _ = workflow.execute(
                "test_hook",
                observation,
                format!("workflow-{}", i),
            );
        }

        // Verify MAPE-K manager is functional
        let mape_k = workflow.mape_k();
        let monitoring_data = mape_k.monitor();

        assert!(monitoring_data.coverage_percentage >= 0.0);
    }

    #[test]
    fn test_receipt_queries() {
        let workflow = SelfExecutingWorkflow::new();

        let files = vec![
            OntologyFile {
                path: "workflow.ttl".to_string(),
                content_hash: "sha3-256:abc123".to_string(),
                size_bytes: 1024,
            },
        ];

        let id = workflow.create_snapshot(files).unwrap();
        workflow.set_active_snapshot(id).unwrap();

        let hook = Arc::new(|ctx: &HookContext| {
            HookResult::success(ctx.input_data.clone(), 2)
        });
        workflow.register_hook("test_hook".to_string(), hook).unwrap();

        // Execute multiple times with same workflow ID
        for i in 0..5 {
            let observation = format!("observation-{}", i).into_bytes();
            let _ = workflow.execute(
                "test_hook",
                observation,
                "workflow-123".to_string(),
            );
        }

        // Query receipts
        let receipts = workflow.query_receipts("workflow-123").unwrap();
        assert_eq!(receipts.len(), 5);

        // Get statistics
        let stats = workflow.get_statistics().unwrap();
        assert_eq!(stats.total_receipts, 5);
        assert!(stats.average_ticks > 0.0);
    }

    #[test]
    fn test_snapshot_promotion() {
        let workflow = SelfExecutingWorkflow::new();

        // Create two snapshots
        let files1 = vec![
            OntologyFile {
                path: "workflow_v1.ttl".to_string(),
                content_hash: "sha3-256:abc123".to_string(),
                size_bytes: 1024,
            },
        ];

        let files2 = vec![
            OntologyFile {
                path: "workflow_v2.ttl".to_string(),
                content_hash: "sha3-256:def456".to_string(),
                size_bytes: 2048,
            },
        ];

        let id1 = workflow.create_snapshot(files1).unwrap();
        let id2 = workflow.create_snapshot(files2).unwrap();

        // Set first as active
        workflow.set_active_snapshot(id1.clone()).unwrap();

        // Promote to second
        workflow.promote_snapshot(&id1, &id2).unwrap();

        // Verify active snapshot changed
        let current = workflow.snapshots.get_current().unwrap();
        assert_eq!(current.id, id2);
    }

    #[test]
    fn test_pattern_integration() {
        let workflow = SelfExecutingWorkflow::new();

        let files = vec![
            OntologyFile {
                path: "workflow.ttl".to_string(),
                content_hash: "sha3-256:abc123".to_string(),
                size_bytes: 1024,
            },
        ];

        let id = workflow.create_snapshot(files).unwrap();
        workflow.set_active_snapshot(id).unwrap();

        // Register YAWL patterns
        workflow.register_hook(
            "sequence".to_string(),
            patterns::sequence(vec!["step1".to_string(), "step2".to_string()]),
        ).unwrap();

        workflow.register_hook(
            "parallel".to_string(),
            patterns::parallel_split(vec!["branch1".to_string(), "branch2".to_string()]),
        ).unwrap();

        // Execute pattern hooks
        let receipt_id = workflow.execute(
            "sequence",
            b"test".to_vec(),
            "workflow-1".to_string(),
        ).unwrap();

        let receipt = workflow.receipts.get(&receipt_id).unwrap();
        assert!(receipt.success);
        assert!(receipt.guards_checked.contains(&"PATTERN_SEQUENCE".to_string()));
    }
}
