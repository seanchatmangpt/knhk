// rust/knhk-etl/src/fiber.rs
// Cooperative fibers for 8-beat epoch system
// Per-shard execution units with tick budget enforcement

use alloc::vec::Vec;
use alloc::string::String;
use crate::ingest::RawTriple;
use crate::reflex::{Action, Receipt};
use crate::park::{ParkCause, ExecutionResult};

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
    pub fn execute_tick(
        &mut self,
        tick: u64,
        delta: &[RawTriple],
    ) -> ExecutionResult {
        self.current_tick = tick;
        self.state = FiberState::Running;
        self.accumulated_ticks = 0;

        // Simulate tick counting (in production, this would use PMU or actual timing)
        // For now, we estimate ticks based on delta size
        let estimated_ticks = self.estimate_ticks(delta);

        if estimated_ticks > self.tick_budget {
            // Exceeded budget, must park
            self.state = FiberState::Parked;
            ExecutionResult::Parked {
                delta: delta.to_vec(),
                receipt: self.generate_receipt(tick, estimated_ticks),
                cause: ParkCause::TickBudgetExceeded,
            }
        } else {
            // Within budget, execute
            let action = self.run_mu(delta);
            self.accumulated_ticks = estimated_ticks;
            self.state = FiberState::Completed;
            
            ExecutionResult::Completed {
                action,
                receipt: self.generate_receipt(tick, estimated_ticks),
            }
        }
    }

    /// Estimate ticks for delta execution
    /// In production, this would use PMU or heatmap prediction
    fn estimate_ticks(&self, delta: &[RawTriple]) -> u32 {
        // Simple heuristic: 1 tick per triple, max 8
        // In production, use MPHF + heatmap for accurate prediction
        let base_ticks = delta.len() as u32;
        if base_ticks > 8 {
            8 // Exceeds budget
        } else {
            base_ticks
        }
    }

    /// Execute μ(Δ) - reconciliation function
    /// In production, this would call hot path kernels
    fn run_mu(&self, delta: &[RawTriple]) -> Action {
        // Placeholder: In production, this would:
        // 1. Call hot path kernels (ASK/COUNT/COMPARE/VALIDATE/SELECT/UNIQUE)
        // 2. Execute hooks via μ ⊣ H guards
        // 3. Generate actions based on reconciliation rules
        
        // For now, return a simple action
        // Serialize delta to payload (in production, would use proper serialization)
        let payload = alloc::vec::Vec::new(); // Placeholder
        Action {
            id: alloc::format!("action_{}_{}", self.shard_id, self.current_tick),
            payload,
            receipt_id: alloc::format!("receipt_{}_{}", self.shard_id, self.current_tick),
        }
    }

    /// Generate receipt for execution
    fn generate_receipt(&self, tick: u64, ticks: u32) -> Receipt {
        Receipt {
            id: alloc::format!("receipt_{}_{}", self.shard_id, tick),
            ticks,
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
                subject: "s1".to_string(),
                predicate: "p1".to_string(),
                object: "o1".to_string(),
            },
        ];

        let result = fiber.execute_tick(0, &delta);
        match result {
            ExecutionResult::Completed { action, receipt } => {
                assert_eq!(action.triples.len(), 1);
                assert_eq!(receipt.shard_id, 0);
            }
            _ => panic!("Expected Completed result"),
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
            })
            .collect();

        let result = fiber.execute_tick(0, &delta);
        match result {
            ExecutionResult::Parked { cause, .. } => {
                assert_eq!(cause, ParkCause::TickBudgetExceeded);
            }
            _ => panic!("Expected Parked result"),
        }
    }
}

