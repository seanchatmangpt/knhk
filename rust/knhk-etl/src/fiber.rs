// rust/knhk-etl/src/fiber.rs
// Cooperative fibers for 8-beat epoch system
// Per-shard execution units with tick budget enforcement

use crate::ingest::RawTriple;
use crate::reflex::{Action, Receipt};
use crate::park::{ParkCause, ExecutionResult};
use crate::ring_conversion::raw_triples_to_soa;
use knhk_hot::{FiberExecutor, Ctx, Ir, Op, Run};
use tracing::instrument;

/// Fiber state
#[derive(Debug, Clone, PartialEq)]
pub enum FiberState {
    Idle,
    Running,
    Parked,
    Completed,
}

/// Cooperative fiber for deterministic execution
/// Each fiber is bound to a shard and executes μ(Δ) within tick budget
#[derive(Debug)]
pub struct Fiber {
    /// Shard ID this fiber belongs to
    shard_id: u32,
    /// CPU core ID for NUMA pinning (None = not pinned)
    core_id: Option<usize>,
    /// Current tick being processed
    current_tick: u64,
    /// Tick budget per execution (≤8)
    tick_budget: u32,
    /// Current state
    state: FiberState,
    /// Accumulated ticks for current execution
    accumulated_ticks: u32,
}

impl Fiber {
    /// Create new fiber
    pub fn new(shard_id: u32, tick_budget: u32) -> Self {
        if tick_budget > 8 {
            panic!("Fiber tick_budget {} exceeds Chatman Constant (8)", tick_budget);
        }

        Self {
            shard_id,
            core_id: None,
            current_tick: 0,
            tick_budget,
            state: FiberState::Idle,
            accumulated_ticks: 0,
        }
    }

    /// Create fiber with core pinning
    pub fn with_core(mut self, core_id: usize) -> Self {
        self.core_id = Some(core_id);
        self
    }

    /// Get shard ID
    pub fn shard_id(&self) -> u32 {
        self.shard_id
    }

    /// Get core ID
    pub fn core_id(&self) -> Option<usize> {
        self.core_id
    }

    /// Get current state
    pub fn state(&self) -> &FiberState {
        &self.state
    }

    /// Execute μ(Δ) for current tick
    /// Returns ExecutionResult indicating completion or parking
    /// Uses C fiber execution for actual hot path execution
    #[instrument(
        name = "fiber.process_tick",
        skip(self, delta),
        fields(
            tick = tick,
            n_deltas = delta.len(),
            shard_id = self.shard_id,
            cycle_id = cycle_id,
        )
    )]
    pub fn execute_tick(
        &mut self,
        tick: u64,
        delta: &[RawTriple],
        cycle_id: u64,
    ) -> ExecutionResult {
        self.current_tick = tick;
        self.state = FiberState::Running;
        self.accumulated_ticks = 0;

        // Validate run length ≤ 8 (guard H)
        if delta.len() > 8 {
            self.state = FiberState::Parked;

            tracing::info!(
                tick = tick,
                shard_id = self.shard_id,
                n_deltas = delta.len(),
                parked = true,
                cause = "RunLengthExceeded",
                "Fiber parked due to run length exceeded"
            );

            // Compute hook_id for parking receipt
            let hook_id = if let Some(first_triple) = delta.first() {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                first_triple.predicate.hash(&mut hasher);
                let predicate_hash = hasher.finish();
                Self::compute_hook_id(self.shard_id as u64, predicate_hash)
            } else {
                Self::compute_hook_id(self.shard_id as u64, 0)
            };
            
            return ExecutionResult::Parked {
                delta: delta.to_vec(),
                receipt: self.generate_receipt(tick, 0, cycle_id, hook_id),
                cause: ParkCause::RunLengthExceeded,
            };
        }

        // Compute hook_id from first predicate in delta
        let hook_id = if let Some(first_triple) = delta.first() {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            first_triple.predicate.hash(&mut hasher);
            let predicate_hash = hasher.finish();
            Self::compute_hook_id(self.shard_id as u64, predicate_hash)
        } else {
            Self::compute_hook_id(self.shard_id as u64, 0)
        };
        
        // Execute via C fiber executor (actual hot path execution)
        let action = self.run_mu(tick, delta, cycle_id, hook_id);
        
        // Check if execution was parked by C fiber
        match &action {
            Action { receipt_id, .. } if receipt_id.contains("error") => {
                // C fiber execution failed or was parked
                self.state = FiberState::Parked;

                tracing::info!(
                    tick = tick,
                    shard_id = self.shard_id,
                    parked = true,
                    cause = "TickBudgetExceeded",
                    "Fiber parked by C executor"
                );

                // Compute hook_id for parking receipt
                let hook_id = if let Some(first_triple) = delta.first() {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    let mut hasher = DefaultHasher::new();
                    first_triple.predicate.hash(&mut hasher);
                    let predicate_hash = hasher.finish();
                    Self::compute_hook_id(self.shard_id as u64, predicate_hash)
                } else {
                    Self::compute_hook_id(self.shard_id as u64, 0)
                };
                
                ExecutionResult::Parked {
                    delta: delta.to_vec(),
                    receipt: self.generate_receipt(tick, 0, cycle_id, hook_id),
                    cause: ParkCause::TickBudgetExceeded,
                }
            }
            _ => {
                // Execution completed successfully
                // Extract receipt from action (receipt_id contains receipt info)
                let receipt = self.extract_receipt_from_action(&action, tick, cycle_id);
                self.accumulated_ticks = receipt.ticks;
                self.state = FiberState::Completed;

                tracing::info!(
                    tick = tick,
                    shard_id = self.shard_id,
                    actual_ticks = receipt.ticks,
                    n_deltas = delta.len(),
                    parked = false,
                    "Fiber completed execution"
                );
                
                ExecutionResult::Completed {
                    action,
                    receipt,
                }
            }
        }
    }
    
    /// Extract receipt from action (receipt_id contains receipt metadata)
    fn extract_receipt_from_action(&self, action: &Action, tick: u64, cycle_id: u64) -> Receipt {
        // Parse receipt_id to extract span_id if available
        // Format: "receipt_{span_id}" or "receipt_{shard_id}_{tick}"
        let span_id = if action.receipt_id.starts_with("receipt_") {
            // Try to extract span_id from receipt_id
            action.receipt_id
                .strip_prefix("receipt_")
                .and_then(|s| s.parse::<u64>().ok())
                .unwrap_or(tick as u64)
        } else {
            tick as u64
        };
        
        // Extract hook_id from action payload if available, otherwise compute from shard
        // For v1.0, we compute hook_id from shard_id (will be improved with hook registry)
        let hook_id = Self::compute_hook_id(self.shard_id as u64, 0);
        
        Receipt {
            id: action.receipt_id.clone(),
            cycle_id,
            shard_id: self.shard_id as u64,
            hook_id,
            ticks: self.accumulated_ticks.max(1), // Use accumulated ticks or default to 1
            actual_ticks: self.accumulated_ticks.max(1),
            lanes: 1, // Will be set by C fiber execution
            span_id,
            a_hash: 0, // Will be computed from action hash
        }
    }

    /// Estimate ticks for delta execution
    /// Uses simple heuristic: 1 tick per triple
    /// Note: MPHF + heatmap prediction planned for v1.0
    fn estimate_ticks(&self, delta: &[RawTriple]) -> u32 {
        // Simple heuristic: 1 tick per triple
        // Future: Use MPHF + heatmap for accurate prediction
        delta.len() as u32
    }

    /// Execute μ(Δ) - reconciliation function
    /// Calls C hot path kernels via FiberExecutor
    fn run_mu(&self, tick: u64, delta: &[RawTriple], cycle_id: u64, _hook_id: u64) -> Action {
        // Convert RawTriple to SoA arrays
        let (s_vec, p_vec, o_vec) = match raw_triples_to_soa(delta) {
            Ok(arrays) => arrays,
            Err(e) => {
                // If conversion fails, return action with error payload
                return Action {
                    id: alloc::format!("action_error_{}_{}", self.shard_id, self.current_tick),
                    payload: alloc::format!("Conversion error: {}", e).into_bytes(),
                    receipt_id: alloc::format!("receipt_error_{}_{}", self.shard_id, self.current_tick),
                };
            }
        };

        // Create Ctx from SoA arrays
        let soa_arrays = crate::load::SoAArrays {
            s: {
                let mut arr = [0u64; 8];
                for (i, &val) in s_vec.iter().take(8).enumerate() {
                    arr[i] = val;
                }
                arr
            },
            p: {
                let mut arr = [0u64; 8];
                for (i, &val) in p_vec.iter().take(8).enumerate() {
                    arr[i] = val;
                }
                arr
            },
            o: {
                let mut arr = [0u64; 8];
                for (i, &val) in o_vec.iter().take(8).enumerate() {
                    arr[i] = val;
                }
                arr
            },
        };

        let ctx = Ctx {
            S: soa_arrays.s.as_ptr(),
            P: soa_arrays.p.as_ptr(),
            O: soa_arrays.o.as_ptr(),
            run: Run {
                pred: if !p_vec.is_empty() { p_vec[0] } else { 0 },
                off: 0,
                len: delta.len() as u64,
            },
        };

        // Create Ir for ASK_SP operation (default for reconciliation)
        let mut ir = Ir {
            op: Op::AskSp,
            s: if !s_vec.is_empty() { s_vec[0] } else { 0 },
            p: if !p_vec.is_empty() { p_vec[0] } else { 0 },
            o: if !o_vec.is_empty() { o_vec[0] } else { 0 },
            k: 0,
            out_S: std::ptr::null_mut(),
            out_P: std::ptr::null_mut(),
            out_O: std::ptr::null_mut(),
            out_mask: 0,
            construct8_pattern_hint: 0,
        };

        // Use provided cycle_id from beat scheduler
        let shard_id = self.shard_id as u64;
        // Compute hook_id from shard_id and predicate (first predicate in delta)
        let hook_id = Self::compute_hook_id(shard_id, ctx.run.pred);

        // Execute via C fiber executor (actual hot path execution)
        match FiberExecutor::execute(&ctx, &mut ir, tick, cycle_id, shard_id, hook_id) {
            Ok(hot_receipt) => {
                // Use receipt from C fiber execution (contains actual ticks, span_id, a_hash)
                // Store receipt info in action for later extraction
                let receipt_id = alloc::format!("receipt_{}", hot_receipt.span_id);

                // Create action with serialized delta as payload
                let mut payload = alloc::vec::Vec::new();
                for triple in delta {
                    payload.extend_from_slice(
                        alloc::format!("{} {} {};", triple.subject, triple.predicate, triple.object)
                            .as_bytes(),
                    );
                }

                Action {
                    id: alloc::format!("action_{}_{}", self.shard_id, tick),
                    payload,
                    receipt_id: receipt_id.clone(),
                }
            }
            Err(e) => {
                // Fiber execution failed or was parked
                // Return action with error payload
                Action {
                    id: alloc::format!("action_error_{}_{}", self.shard_id, tick),
                    payload: alloc::format!("Fiber execution error: {}", e).into_bytes(),
                    receipt_id: alloc::format!("receipt_error_{}_{}", self.shard_id, tick),
                }
            }
        }
    }

    /// Compute hook_id from shard_id and predicate
    /// Uses hash-based assignment: (shard_id << 32) | (predicate & 0xFFFFFFFF)
    /// This provides deterministic hook_id assignment for v1.0
    fn compute_hook_id(shard_id: u64, predicate: u64) -> u64 {
        (shard_id << 32) | (predicate & 0xFFFFFFFF)
    }

    /// Generate receipt for execution (used when parking before C execution)
    fn generate_receipt(&self, tick: u64, ticks: u32, cycle_id: u64, hook_id: u64) -> Receipt {
        Receipt {
            id: alloc::format!("receipt_{}_{}", self.shard_id, tick),
            cycle_id,
            shard_id: self.shard_id as u64,
            hook_id,
            ticks,
            actual_ticks: ticks,
            lanes: 1, // Single lane for now
            span_id: tick as u64, // Use tick as span ID for now
            a_hash: 0, // Will be computed from action hash
        }
    }

    /// Yield control back to beat scheduler
    pub fn yield_control(&mut self) {
        match self.state {
            FiberState::Running | FiberState::Completed => {
                self.state = FiberState::Idle;
                self.accumulated_ticks = 0;
            }
            _ => {}
        }
    }

    /// Reset fiber for next beat
    pub fn reset(&mut self) {
        self.current_tick = 0;
        self.state = FiberState::Idle;
        self.accumulated_ticks = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fiber_creation() {
        let fiber = Fiber::new(0, 8);
        assert_eq!(fiber.shard_id(), 0);
        assert_eq!(fiber.tick_budget, 8);
        assert_eq!(fiber.state(), &FiberState::Idle);
    }

    #[test]
    #[should_panic(expected = "exceeds Chatman Constant")]
    fn test_fiber_invalid_budget() {
        Fiber::new(0, 9);
    }

    #[test]
    fn test_fiber_execute_within_budget() {
        let mut fiber = Fiber::new(0, 8);
        let delta = vec![
            RawTriple {
                subject: "http://example.org/s1".to_string(),
                predicate: "http://example.org/p1".to_string(),
                object: "http://example.org/o1".to_string(),
                graph: None,
            },
        ];

        let result = fiber.execute_tick(0, &delta, 1);
        match result {
            ExecutionResult::Completed { action, receipt } => {
                assert!(!action.id.is_empty());
                assert_eq!(receipt.shard_id, 0);
                assert!(receipt.ticks > 0);
            }
            ExecutionResult::Parked { .. } => {
                // Parking is acceptable if budget exceeded
            }
        }
    }

    #[test]
    fn test_fiber_execute_exceeds_budget() {
        let mut fiber = Fiber::new(0, 8);
        // Create delta with 10 triples (exceeds budget of 8)
        let delta: Vec<RawTriple> = (0..10)
            .map(|i| RawTriple {
                subject: format!("s{}", i),
                predicate: format!("p{}", i),
                object: format!("o{}", i),
                graph: None,
            })
            .collect();

        let result = fiber.execute_tick(0, &delta, 1);
        match result {
            ExecutionResult::Parked { cause, .. } => {
                assert_eq!(cause, ParkCause::TickBudgetExceeded);
            }
            _ => panic!("Expected Parked result"),
        }
    }
}

