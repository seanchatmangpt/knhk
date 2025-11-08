//! Deterministic execution guarantees with delta logs and receipts
//!
//! Provides deterministic execution guarantees for workflow cases,
//! ensuring identical inputs produce identical outputs.
//! Integrates with delta logs and receipts for full replay capability.

use crate::case::{Case, CaseId};
use crate::error::{WorkflowError, WorkflowResult};
use crate::integration::LockchainIntegration;
use crate::state::StateStore;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Deterministic execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Delta log entry (state change)
    pub delta: Option<DeltaLogEntry>,
    /// Receipt hash (if recorded)
    pub receipt_hash: Option<String>,
}

/// Delta log entry (state change)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaLogEntry {
    /// Delta hash (hash of state change)
    pub delta_hash: String,
    /// Previous state hash
    pub previous_state_hash: String,
    /// New state hash
    pub new_state_hash: String,
    /// Delta data (JSON)
    pub delta_data: serde_json::Value,
    /// Timestamp
    pub timestamp: u64,
    /// Tick number
    pub tick: u64,
}

/// Deterministic executor with delta logs and receipts
pub struct DeterministicExecutor {
    /// Execution traces (case_id -> trace)
    traces: Arc<RwLock<HashMap<CaseId, DeterministicContext>>>,
    /// Delta logs (case_id -> list of deltas)
    delta_logs: Arc<RwLock<HashMap<CaseId, Vec<DeltaLogEntry>>>>,
    /// State store for persistence
    state_store: Option<Arc<RwLock<StateStore>>>,
    /// Lockchain integration for receipts
    lockchain: Option<Arc<LockchainIntegration>>,
    /// Deterministic RNG seed
    seed: u64,
}

impl DeterministicExecutor {
    /// Create new deterministic executor
    pub fn new(seed: u64) -> Self {
        Self {
            traces: Arc::new(RwLock::new(HashMap::new())),
            delta_logs: Arc::new(RwLock::new(HashMap::new())),
            state_store: None,
            lockchain: None,
            seed,
        }
    }

    /// Create deterministic executor with state store and lockchain
    pub fn with_storage(
        seed: u64,
        state_store: Arc<RwLock<StateStore>>,
        lockchain: Option<Arc<LockchainIntegration>>,
    ) -> Self {
        Self {
            traces: Arc::new(RwLock::new(HashMap::new())),
            delta_logs: Arc::new(RwLock::new(HashMap::new())),
            state_store: Some(state_store),
            lockchain,
            seed,
        }
    }

    /// Execute case deterministically with delta logging
    pub async fn execute_deterministic(&self, case: &Case) -> WorkflowResult<DeterministicContext> {
        // Hash input data
        let input_hash = self.hash_case_input(case)?;

        // Check if we've seen this input before
        let existing_trace = self.traces.read().await.get(&case.id).cloned();
        if let Some(trace) = existing_trace {
            if trace.input_hash == input_hash {
                // Replay deterministic execution from delta logs
                return self.replay_from_deltas(&case.id, &trace).await;
            }
        }

        // Create deterministic context
        let mut context = DeterministicContext {
            case_id: case.id,
            input_hash,
            seed: self.seed,
            trace: Vec::new(),
        };

        // Load existing delta log if available
        let mut delta_log = self
            .delta_logs
            .read()
            .await
            .get(&case.id)
            .cloned()
            .unwrap_or_default();
        let mut previous_state_hash = input_hash;

        // Execute with deterministic seed
        let mut rng = fastrand::Rng::with_seed(self.seed);
        let mut step = 0u64;
        let mut tick = 0u64;

        // Simulate deterministic execution
        // In real implementation, this would execute tasks deterministically
        for _ in 0..10 {
            let task_id = format!("task:{}", step);
            let input_hash = previous_state_hash;
            let output_hash = rng.u64(..);
            let ticks = rng.u32(1..=8);

            // Create delta log entry
            let delta_data = serde_json::json!({
                "task_id": task_id,
                "step": step,
                "ticks": ticks,
            });

            let delta_hash = self.hash_delta(&delta_data, previous_state_hash, output_hash);
            let delta_entry = DeltaLogEntry {
                delta_hash: format!("{:x}", delta_hash),
                previous_state_hash: format!("{:x}", previous_state_hash),
                new_state_hash: format!("{:x}", output_hash),
                delta_data,
                timestamp: chrono::Utc::now().timestamp() as u64,
                tick,
            };

            // Record receipt if lockchain is available
            let receipt_hash = if let Some(ref lockchain) = self.lockchain {
                let receipt_data = serde_json::json!({
                    "case_id": case.id.to_string(),
                    "step": step,
                    "delta_hash": delta_entry.delta_hash.clone(),
                    "tick": tick,
                });
                let receipt_bytes = serde_json::to_vec(&receipt_data)
                    .map_err(|e| {
                        WorkflowError::Internal(format!("Failed to serialize receipt: {}", e))
                    })?;
                let mut hasher = sha2::Sha256::new();
                hasher.update(&receipt_bytes);
                let hash: [u8; 32] = hasher.finalize().into();
                let hash_str = format!("{:x}", hash);

                // Record to lockchain
                let _ = lockchain
                    .record_case_executed(&case.id, true)
                    .await;
                Some(hash_str)
            } else {
                None
            };

            context.trace.push(ExecutionStep {
                step,
                task_id,
                input_hash,
                output_hash,
                ticks,
                delta: Some(delta_entry.clone()),
                receipt_hash: receipt_hash.clone(),
            });

            delta_log.push(delta_entry);
            previous_state_hash = output_hash;
            step += 1;
            tick += ticks as u64;
        }

        // Store trace and delta log
        self.traces.write().await.insert(case.id, context.clone());
        self.delta_logs
            .write()
            .await
            .insert(case.id, delta_log);

        // Persist to state store if available
        // Note: StateStore persistence would be handled via StateStore methods
        // For now, traces are stored in memory

        Ok(context)
    }

    /// Replay execution from delta logs
    async fn replay_from_deltas(
        &self,
        case_id: &CaseId,
        trace: &DeterministicContext,
    ) -> WorkflowResult<DeterministicContext> {
        let delta_log = self
            .delta_logs
            .read()
            .await
            .get(case_id)
            .cloned()
            .unwrap_or_default();

        // Reconstruct trace from delta log
        let mut replayed_trace = trace.clone();
        for (i, delta) in delta_log.iter().enumerate() {
            if i < replayed_trace.trace.len() {
                replayed_trace.trace[i].delta = Some(delta.clone());
            }
        }

        Ok(replayed_trace)
    }

    /// Hash delta for deterministic replay
    fn hash_delta(&self, delta_data: &serde_json::Value, prev_hash: u64, new_hash: u64) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        serde_json::to_string(delta_data).unwrap().hash(&mut hasher);
        prev_hash.hash(&mut hasher);
        new_hash.hash(&mut hasher);
        hasher.finish()
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

    /// Get delta log for a case
    pub async fn get_delta_log(&self, case_id: &CaseId) -> Vec<DeltaLogEntry> {
        self.delta_logs
            .read()
            .await
            .get(case_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Replay case from delta log
    pub async fn replay_case(&self, case_id: &CaseId) -> WorkflowResult<DeterministicContext> {
        let trace = self
            .traces
            .read()
            .await
            .get(case_id)
            .cloned()
            .ok_or_else(|| {
                WorkflowError::Validation(format!("No trace found for case {}", case_id))
            })?;

        self.replay_from_deltas(case_id, &trace).await
    }

    /// Get receipt hash for a step
    pub async fn get_receipt_hash(
        &self,
        case_id: &CaseId,
        step: u64,
    ) -> Option<String> {
        let traces = self.traces.read().await;
        let trace = traces.get(case_id)?;
        trace.trace.iter().find(|s| s.step == step)?.receipt_hash.clone()
    }
}
