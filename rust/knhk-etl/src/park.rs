// rust/knhk-etl/src/park.rs
// Park/escalate mechanism for 8-beat epoch system
// Handles demotion of over-budget work to W1

use alloc::vec::Vec;
use alloc::string::String;
use crate::ingest::RawTriple;
use crate::reflex::Receipt;

/// Reason for parking a delta
#[derive(Debug, Clone, PartialEq)]
pub enum ParkCause {
    /// Tick budget exceeded (ticks > 8)
    TickBudgetExceeded,
    /// L1 cache miss predicted
    L1MissPredicted,
    /// Run length exceeds limit (run_len > 8)
    RunLengthExceeded,
    /// Heat below threshold (not hot enough for R1)
    HeatBelowThreshold,
}

impl ParkCause {
    /// Get human-readable description
    pub fn description(&self) -> &str {
        match self {
            ParkCause::TickBudgetExceeded => "Tick budget exceeded (ticks > 8)",
            ParkCause::L1MissPredicted => "L1 cache miss predicted",
            ParkCause::RunLengthExceeded => "Run length exceeds limit (run_len > 8)",
            ParkCause::HeatBelowThreshold => "Heat below threshold (not hot enough for R1)",
        }
    }
}

/// Execution result from fiber
#[derive(Debug, Clone)]
pub enum ExecutionResult {
    /// Execution completed successfully
    Completed {
        action: crate::reflex::Action,
        receipt: Receipt,
    },
    /// Execution parked to W1
    Parked {
        delta: Vec<RawTriple>,
        receipt: Receipt,
        cause: ParkCause,
    },
}

impl ExecutionResult {
    /// Check if result is completed
    pub fn is_completed(&self) -> bool {
        matches!(self, ExecutionResult::Completed { .. })
    }

    /// Check if result is parked
    pub fn is_parked(&self) -> bool {
        matches!(self, ExecutionResult::Parked { .. })
    }

    /// Get receipt (works for both Completed and Parked)
    pub fn receipt(&self) -> &Receipt {
        match self {
            ExecutionResult::Completed { receipt, .. } => receipt,
            ExecutionResult::Parked { receipt, .. } => receipt,
        }
    }
}

/// Park manager for handling parked deltas
pub struct ParkManager {
    /// Parked deltas waiting for W1 processing
    parked_deltas: Vec<ParkedDelta>,
}

/// Parked delta with metadata
#[derive(Clone)]
pub struct ParkedDelta {
    /// Delta triples
    pub delta: Vec<RawTriple>,
    /// Receipt generated at park time
    pub receipt: Receipt,
    /// Reason for parking
    pub cause: ParkCause,
    /// Cycle ID when parked
    pub cycle_id: u64,
    /// Tick when parked
    pub tick: u64,
}

impl ParkManager {
    /// Create new park manager
    pub fn new() -> Self {
        Self {
            parked_deltas: Vec::new(),
        }
    }

    /// Park a delta (demote to W1)
    pub fn park(&mut self, delta: Vec<RawTriple>, receipt: Receipt, cause: ParkCause, cycle_id: u64, tick: u64) {
        self.parked_deltas.push(ParkedDelta {
            delta,
            receipt,
            cause,
            cycle_id,
            tick,
        });
    }

    /// Get parked deltas for W1 consumption
    pub fn get_parked(&mut self) -> Vec<ParkedDelta> {
        let result = self.parked_deltas.clone();
        self.parked_deltas.clear();
        result
    }

    /// Get count of parked deltas
    pub fn parked_count(&self) -> usize {
        self.parked_deltas.len()
    }
}

impl Default for ParkManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_park_cause_descriptions() {
        assert_eq!(
            ParkCause::TickBudgetExceeded.description(),
            "Tick budget exceeded (ticks > 8)"
        );
        assert_eq!(
            ParkCause::L1MissPredicted.description(),
            "L1 cache miss predicted"
        );
    }

    #[test]
    fn test_execution_result_completed() {
        let receipt = Receipt {
            id: "test".to_string(),
            ticks: 5,
            lanes: 1,
            span_id: 0,
            a_hash: 0,
        };
        let action = crate::reflex::Action {
            id: "action1".to_string(),
            payload: Vec::new(),
            receipt_id: "test".to_string(),
        };

        let result = ExecutionResult::Completed { action, receipt };
        assert!(result.is_completed());
        assert!(!result.is_parked());
    }

    #[test]
    fn test_execution_result_parked() {
        let receipt = Receipt {
            id: "test".to_string(),
            ticks: 10,
            lanes: 1,
            span_id: 0,
            a_hash: 0,
        };
        let delta = vec![];

        let result = ExecutionResult::Parked {
            delta,
            receipt,
            cause: ParkCause::TickBudgetExceeded,
        };
        assert!(!result.is_completed());
        assert!(result.is_parked());
    }

    #[test]
    fn test_park_manager() {
        let mut manager = ParkManager::new();
        assert_eq!(manager.parked_count(), 0);

        let receipt = Receipt {
            id: "test".to_string(),
            ticks: 10,
            lanes: 1,
            span_id: 0,
            a_hash: 0,
        };
        let delta = vec![];

        manager.park(delta, receipt, ParkCause::TickBudgetExceeded, 0, 0);
        assert_eq!(manager.parked_count(), 1);

        let parked = manager.get_parked();
        assert_eq!(parked.len(), 1);
        assert_eq!(manager.parked_count(), 0);
    }
}

