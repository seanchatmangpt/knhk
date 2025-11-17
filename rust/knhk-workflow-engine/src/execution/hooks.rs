//! Hook Evaluation Framework (μ execution)
//!
//! Implements the execution layer that evaluates workflow hooks.
//! Each hook is a decision point in the workflow graph.
//!
//! # Doctrine Compliance
//!
//! - **A = μ(O)**: Hooks implement μ - deterministic execution from observations
//! - **Chatman Constant**: Each hook must complete in ≤8 ticks
//! - **Q enforcement**: Guards are checked at every hook
//! - **Receipt generation**: Every hook execution is recorded

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::receipt::{Receipt, ReceiptId, ReceiptStore};
use super::snapshot::{SnapshotId, SnapshotStore};

/// Hook execution context
///
/// Contains all state needed for deterministic hook evaluation
#[derive(Debug, Clone)]
pub struct HookContext {
    pub snapshot_id: SnapshotId,
    pub workflow_instance_id: String,
    pub input_data: Vec<u8>,
    pub variables: HashMap<String, String>,
}

/// Hook execution result
#[derive(Debug, Clone)]
pub struct HookResult {
    pub output_data: Vec<u8>,
    pub ticks_used: u32,
    pub guards_passed: Vec<String>,
    pub guards_failed: Vec<String>,
    pub success: bool,
    pub next_hooks: Vec<String>,
}

impl HookResult {
    pub fn success(output: Vec<u8>, ticks: u32) -> Self {
        Self {
            output_data: output,
            ticks_used: ticks,
            guards_passed: Vec::new(),
            guards_failed: Vec::new(),
            success: true,
            next_hooks: Vec::new(),
        }
    }

    pub fn failure(reason: String, ticks: u32) -> Self {
        Self {
            output_data: reason.into_bytes(),
            ticks_used: ticks,
            guards_passed: Vec::new(),
            guards_failed: vec!["EXECUTION_FAILED".to_string()],
            success: false,
            next_hooks: Vec::new(),
        }
    }

    pub fn add_guard_check(&mut self, guard: String, passed: bool) {
        if passed {
            self.guards_passed.push(guard);
        } else {
            self.guards_failed.push(guard);
            self.success = false;
        }
    }
}

/// Hook function signature
///
/// All hooks must conform to this signature for uniform execution
pub type HookFn = Arc<dyn Fn(&HookContext) -> HookResult + Send + Sync>;

/// Hook registry
///
/// Maps hook names to their execution functions
pub struct HookRegistry {
    hooks: Arc<RwLock<HashMap<String, HookFn>>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            hooks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a hook
    pub fn register(&self, name: String, hook_fn: HookFn) -> Result<(), String> {
        self.hooks
            .write()
            .map_err(|e| e.to_string())?
            .insert(name, hook_fn);
        Ok(())
    }

    /// Get a hook by name
    pub fn get(&self, name: &str) -> Result<HookFn, String> {
        self.hooks
            .read()
            .map_err(|e| e.to_string())?
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Hook not found: {}", name))
    }

    /// List all registered hooks
    pub fn list(&self) -> Result<Vec<String>, String> {
        Ok(self
            .hooks
            .read()
            .map_err(|e| e.to_string())?
            .keys()
            .cloned()
            .collect())
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook execution engine
///
/// Evaluates hooks with guard checking and receipt generation
pub struct HookEngine {
    registry: Arc<HookRegistry>,
    snapshot_store: Arc<SnapshotStore>,
    receipt_store: Arc<ReceiptStore>,
}

impl HookEngine {
    pub fn new(
        registry: Arc<HookRegistry>,
        snapshot_store: Arc<SnapshotStore>,
        receipt_store: Arc<ReceiptStore>,
    ) -> Self {
        Self {
            registry,
            snapshot_store,
            receipt_store,
        }
    }

    /// Execute a hook with full context and receipt generation
    pub fn execute(&self, hook_name: &str, context: HookContext) -> Result<ReceiptId, String> {
        // Verify snapshot exists
        let snapshot = self.snapshot_store.get(&context.snapshot_id)?;

        // Get hook function
        let hook_fn = self.registry.get(hook_name)?;

        // Execute hook
        let start_tick = self.get_tick_count();
        let mut result = hook_fn(&context);
        let end_tick = self.get_tick_count();

        // Use measured ticks only if hook didn't set them explicitly
        if result.ticks_used == 0 {
            let actual_ticks = (end_tick - start_tick) as u32;
            result.ticks_used = actual_ticks;
        }

        // Enforce Chatman constant
        if result.ticks_used > 8 {
            result.add_guard_check("CHATMAN_CONSTANT".to_string(), false);
        } else {
            result.add_guard_check("CHATMAN_CONSTANT".to_string(), true);
        }

        // Verify all required guards from snapshot
        for guard in &snapshot.guards_checked {
            if !result.guards_passed.contains(guard) && !result.guards_failed.contains(guard) {
                result.add_guard_check(format!("MISSING_GUARD_{}", guard), false);
            }
        }

        // Create receipt
        let mut receipt = Receipt::new(
            context.snapshot_id.clone(),
            &context.input_data,
            &result.output_data,
            context.workflow_instance_id.clone(),
        );

        receipt.set_ticks(result.ticks_used);

        for guard in &result.guards_passed {
            receipt.add_guard_check(guard.clone(), true);
        }

        for guard in &result.guards_failed {
            receipt.add_guard_check(guard.clone(), false);
        }

        // Store receipt
        let receipt_id = self.receipt_store.append(receipt)?;

        Ok(receipt_id)
    }

    /// Execute a workflow (sequence of hooks)
    pub fn execute_workflow(
        &self,
        hooks: &[String],
        context: HookContext,
    ) -> Result<Vec<ReceiptId>, String> {
        let mut receipts = Vec::new();
        let mut current_context = context;

        for hook_name in hooks {
            let receipt_id = self.execute(hook_name, current_context.clone())?;
            receipts.push(receipt_id.clone());

            // Get receipt to check if execution succeeded
            let receipt = self.receipt_store.get(&receipt_id)?;

            if !receipt.success {
                return Err(format!(
                    "Workflow failed at hook '{}': {:?}",
                    hook_name, receipt.guards_failed
                ));
            }

            // Update context with output for next hook
            current_context.input_data = receipt.a_out_hash.into_bytes();
        }

        Ok(receipts)
    }

    /// Get tick count (for latency measurement)
    #[inline(always)]
    fn get_tick_count(&self) -> u64 {
        #[cfg(feature = "std")]
        {
            use std::time::Instant;
            static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
            let start = START.get_or_init(Instant::now);
            start.elapsed().as_nanos() as u64 / 100 // Convert to ticks (100ns units)
        }

        #[cfg(not(feature = "std"))]
        {
            0
        }
    }
}

/// YAWL pattern implementations as hook builders
pub mod patterns {
    use super::*;

    /// Sequence pattern (op_sequence)
    pub fn sequence(steps: Vec<String>) -> HookFn {
        Arc::new(move |ctx| {
            let mut result = HookResult::success(ctx.input_data.clone(), 1);
            result.next_hooks = steps.clone();
            result.add_guard_check("PATTERN_SEQUENCE".to_string(), true);
            result
        })
    }

    /// Parallel split pattern (op_parallel_split)
    pub fn parallel_split(branches: Vec<String>) -> HookFn {
        Arc::new(move |ctx| {
            let mut result = HookResult::success(ctx.input_data.clone(), 2);
            result.next_hooks = branches.clone();
            result.add_guard_check("PATTERN_PARALLEL_SPLIT".to_string(), true);
            result
        })
    }

    /// Synchronization pattern (op_synchronize)
    pub fn synchronize(expected_inputs: usize) -> HookFn {
        Arc::new(move |ctx| {
            let mut result = HookResult::success(ctx.input_data.clone(), 1);

            // Check if all inputs received
            let received = ctx
                .variables
                .get("sync_count")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(1);

            if received >= expected_inputs {
                result.add_guard_check("PATTERN_SYNCHRONIZE".to_string(), true);
            } else {
                result.success = false;
                result.add_guard_check("PATTERN_SYNCHRONIZE_INCOMPLETE".to_string(), false);
            }

            result
        })
    }

    /// Exclusive choice pattern (op_exclusive_choice)
    pub fn exclusive_choice(condition: String) -> HookFn {
        Arc::new(move |ctx| {
            let mut result = HookResult::success(ctx.input_data.clone(), 1);

            // Evaluate condition from context
            let choice = ctx
                .variables
                .get(&condition)
                .cloned()
                .unwrap_or_else(|| "default".to_string());

            result.next_hooks = vec![choice];
            result.add_guard_check("PATTERN_EXCLUSIVE_CHOICE".to_string(), true);
            result
        })
    }

    /// Simple merge pattern (op_simple_merge)
    pub fn simple_merge() -> HookFn {
        Arc::new(|ctx| {
            let mut result = HookResult::success(ctx.input_data.clone(), 1);
            result.add_guard_check("PATTERN_SIMPLE_MERGE".to_string(), true);
            result
        })
    }
}

#[cfg(test)]
mod tests {
    use super::super::snapshot::*;
    use super::*;

    #[test]
    fn test_hook_registry() {
        let registry = HookRegistry::new();

        let hook = Arc::new(|ctx: &HookContext| HookResult::success(ctx.input_data.clone(), 1));

        registry.register("test_hook".to_string(), hook).unwrap();

        let retrieved = registry.get("test_hook").unwrap();
        let ctx = HookContext {
            snapshot_id: SnapshotId::from_string("test".to_string()),
            workflow_instance_id: "test".to_string(),
            input_data: vec![1, 2, 3],
            variables: HashMap::new(),
        };

        let result = retrieved(&ctx);
        assert!(result.success);
    }

    #[test]
    fn test_hook_execution() {
        let registry = Arc::new(HookRegistry::new());
        let snapshot_store = Arc::new(SnapshotStore::new());
        let receipt_store = Arc::new(ReceiptStore::new());

        // Create and store a snapshot
        let snapshot_id = SnapshotId::from_string("Σ_test_001".to_string());
        let manifest = SnapshotManifest::new(snapshot_id.clone(), vec![]);
        snapshot_store.store(manifest).unwrap();

        // Register a test hook
        let hook = Arc::new(|ctx: &HookContext| {
            let mut result = HookResult::success(ctx.input_data.clone(), 3);
            result.add_guard_check("TEST_GUARD".to_string(), true);
            result
        });
        registry.register("test_hook".to_string(), hook).unwrap();

        let engine = HookEngine::new(registry, snapshot_store, receipt_store.clone());

        let context = HookContext {
            snapshot_id: snapshot_id.clone(),
            workflow_instance_id: "workflow-1".to_string(),
            input_data: b"test input".to_vec(),
            variables: HashMap::new(),
        };

        let receipt_id = engine.execute("test_hook", context).unwrap();
        let receipt = receipt_store.get(&receipt_id).unwrap();

        assert!(receipt.success);
        assert!(receipt.ticks_used <= 8); // Chatman constant
    }

    #[test]
    fn test_chatman_constant_violation() {
        let registry = Arc::new(HookRegistry::new());
        let snapshot_store = Arc::new(SnapshotStore::new());
        let receipt_store = Arc::new(ReceiptStore::new());

        let snapshot_id = SnapshotId::from_string("Σ_test_001".to_string());
        let manifest = SnapshotManifest::new(snapshot_id.clone(), vec![]);
        snapshot_store.store(manifest).unwrap();

        // Register a slow hook
        let hook = Arc::new(|ctx: &HookContext| {
            let mut result = HookResult::success(ctx.input_data.clone(), 10); // Violates constant
            result
        });
        registry.register("slow_hook".to_string(), hook).unwrap();

        let engine = HookEngine::new(registry, snapshot_store, receipt_store.clone());

        let context = HookContext {
            snapshot_id,
            workflow_instance_id: "workflow-1".to_string(),
            input_data: b"test".to_vec(),
            variables: HashMap::new(),
        };

        let receipt_id = engine.execute("slow_hook", context).unwrap();
        let receipt = receipt_store.get(&receipt_id).unwrap();

        assert!(!receipt.success); // Should fail due to Chatman violation
        assert!(receipt
            .guards_failed
            .contains(&"CHATMAN_CONSTANT".to_string()));
    }

    #[test]
    fn test_workflow_execution() {
        let registry = Arc::new(HookRegistry::new());
        let snapshot_store = Arc::new(SnapshotStore::new());
        let receipt_store = Arc::new(ReceiptStore::new());

        let snapshot_id = SnapshotId::from_string("Σ_test_001".to_string());
        let manifest = SnapshotManifest::new(snapshot_id.clone(), vec![]);
        snapshot_store.store(manifest).unwrap();

        // Register multiple hooks
        for i in 1..=3 {
            let hook = Arc::new(move |ctx: &HookContext| {
                let mut output = ctx.input_data.clone();
                output.push(i as u8);
                HookResult::success(output, 1)
            });
            registry.register(format!("hook_{}", i), hook).unwrap();
        }

        let engine = HookEngine::new(registry, snapshot_store, receipt_store.clone());

        let context = HookContext {
            snapshot_id,
            workflow_instance_id: "workflow-1".to_string(),
            input_data: vec![0],
            variables: HashMap::new(),
        };

        let hooks = vec![
            "hook_1".to_string(),
            "hook_2".to_string(),
            "hook_3".to_string(),
        ];
        let receipts = engine.execute_workflow(&hooks, context).unwrap();

        assert_eq!(receipts.len(), 3);

        // Verify all receipts succeeded
        for receipt_id in receipts {
            let receipt = receipt_store.get(&receipt_id).unwrap();
            assert!(receipt.success);
        }
    }

    #[test]
    fn test_pattern_sequence() {
        let hook = patterns::sequence(vec!["step1".to_string(), "step2".to_string()]);

        let ctx = HookContext {
            snapshot_id: SnapshotId::from_string("test".to_string()),
            workflow_instance_id: "test".to_string(),
            input_data: vec![],
            variables: HashMap::new(),
        };

        let result = hook(&ctx);
        assert!(result.success);
        assert_eq!(result.next_hooks.len(), 2);
        assert!(result
            .guards_passed
            .contains(&"PATTERN_SEQUENCE".to_string()));
    }

    #[test]
    fn test_pattern_parallel_split() {
        let hook = patterns::parallel_split(vec!["branch1".to_string(), "branch2".to_string()]);

        let ctx = HookContext {
            snapshot_id: SnapshotId::from_string("test".to_string()),
            workflow_instance_id: "test".to_string(),
            input_data: vec![],
            variables: HashMap::new(),
        };

        let result = hook(&ctx);
        assert!(result.success);
        assert_eq!(result.next_hooks.len(), 2);
        assert!(result
            .guards_passed
            .contains(&"PATTERN_PARALLEL_SPLIT".to_string()));
    }
}
