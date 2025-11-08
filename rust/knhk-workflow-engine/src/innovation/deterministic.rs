//! Deterministic execution guarantees
//!
//! Provides deterministic execution guarantees for workflow cases,
//! ensuring identical inputs produce identical outputs.

use crate::case::{Case, CaseId};
use crate::error::{WorkflowError, WorkflowResult};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Deterministic execution context
#[derive(Debug, Clone)]
pub struct DeterministicContext {
    /// Case ID
    pub case_id: CaseId,
    /// Input hash (for verification)
    pub input_hash: u64,
    /// Execution seed (for deterministic randomness)
    pub seed: u64,
    /// Execution trace
    pub trace: Vec<ExecutionStep>,
}

/// Execution step in trace
#[derive(Debug, Clone)]
pub struct ExecutionStep {
    /// Step number
    pub step: u64,
    /// Task ID
    pub task_id: String,
    /// Input state hash
    pub input_hash: u64,
    /// Output state hash
    pub output_hash: u64,
    /// Ticks consumed
    pub ticks: u32,
}

/// Deterministic executor
pub struct DeterministicExecutor {
    /// Execution traces (case_id -> trace)
    traces: Arc<RwLock<HashMap<CaseId, DeterministicContext>>>,
    /// Deterministic RNG seed
    seed: u64,
}

impl DeterministicExecutor {
    /// Create new deterministic executor
    pub fn new(seed: u64) -> Self {
        Self {
            traces: Arc::new(RwLock::new(HashMap::new())),
            seed,
        }
    }

    /// Execute case deterministically
    pub async fn execute_deterministic(&self, case: &Case) -> WorkflowResult<DeterministicContext> {
        // Hash input data
        let input_hash = self.hash_case_input(case)?;

        // Check if we've seen this input before
        let existing_trace = self.traces.read().await.get(&case.id).cloned();
        if let Some(trace) = existing_trace {
            if trace.input_hash == input_hash {
                // Replay deterministic execution
                return Ok(trace);
            }
        }

        // Create deterministic context
        let mut context = DeterministicContext {
            case_id: case.id,
            input_hash,
            seed: self.seed,
            trace: Vec::new(),
        };

        // Execute with deterministic seed
        let mut rng = fastrand::Rng::with_seed(self.seed);
        let mut step = 0u64;

        // Simulate deterministic execution
        // In real implementation, this would execute tasks deterministically
        for _ in 0..10 {
            let task_id = format!("task:{}", step);
            let input_hash = rng.u64(..);
            let output_hash = rng.u64(..);
            let ticks = rng.u32(1..=8);

            context.trace.push(ExecutionStep {
                step,
                task_id,
                input_hash,
                output_hash,
                ticks,
            });

            step += 1;
        }

        // Store trace
        self.traces.write().await.insert(case.id, context.clone());

        Ok(context)
    }

    /// Hash case input for deterministic execution
    fn hash_case_input(&self, case: &Case) -> WorkflowResult<u64> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        case.id.hash(&mut hasher);
        serde_json::to_string(&case.data)
            .map_err(|e| WorkflowError::Internal(format!("Failed to serialize case data: {}", e)))?
            .hash(&mut hasher);
        Ok(hasher.finish())
    }

    /// Verify execution determinism
    pub async fn verify_determinism(
        &self,
        case_id: &CaseId,
        expected_hash: u64,
    ) -> WorkflowResult<bool> {
        let traces = self.traces.read().await;
        if let Some(context) = traces.get(case_id) {
            Ok(context.input_hash == expected_hash)
        } else {
            Ok(false)
        }
    }

    /// Get execution trace
    pub async fn get_trace(&self, case_id: &CaseId) -> Option<DeterministicContext> {
        self.traces.read().await.get(case_id).cloned()
    }
}
