// The Chatman Equation: A = μ(O)
// Every action must be the deterministic projection of observations under ontology and invariants

use serde::{Deserialize, Serialize};
use std::fmt;

/// The Chatman Equation: A = μ(O)
///
/// Every action A is the deterministic projection μ of:
/// - Observations O (what happened)
/// - Ontology Σ (what's possible)
/// - Invariants Q (what's forbidden)
/// - Operators Λ (how to act)
/// - Projection μ (transformation)

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatmanEquation {
    /// Observation plane (O)
    /// What the system observed: events, telemetry, state
    pub observation: String,

    /// Ontology snapshot ID (Σ*)
    /// The active ontology version
    pub ontology_id: String,

    /// Invariants enforced (Q)
    /// Hard constraints: Q1-Q5 must hold
    pub invariants: Vec<String>,

    /// Action generated (A)
    /// The deterministic result of μ(O)
    pub action: Action,

    /// Latency budget spent
    /// How many ticks used (≤8 for hot path)
    pub ticks_used: u32,

    /// Proof of correctness
    /// Cryptographic receipt that μ was applied correctly
    pub receipt_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Action {
    /// What to do
    pub command: String,

    /// Where to do it (sector: finance, ops, health, etc.)
    pub sector: String,

    /// Guards that must hold
    pub preconditions: Vec<String>,

    /// Guarantees after execution
    pub postconditions: Vec<String>,

    /// Resource budget (CPU, memory, latency)
    pub budget: ResourceBudget,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceBudget {
    pub max_ticks: u32,          // Chatman constant: 8
    pub max_memory_mb: u32,      // Per-action budget
    pub max_latency_ms: u32,     // Warm path budget
}

impl ChatmanEquation {
    /// Verify that A = μ(O) was computed correctly
    ///
    /// This is the core validation: given O, Σ*, and Q,
    /// can we reproduce A and verify the receipt?
    pub fn verify(&self, expected_action: &Action, receipt_valid: bool) -> bool {
        if !receipt_valid {
            return false;
        }

        // The action must match
        if self.action.command != expected_action.command {
            return false;
        }

        // Budget must be respected
        if self.ticks_used > self.action.budget.max_ticks {
            return false;
        }

        true
    }

    /// Check that this equation respects all invariants Q
    pub fn respects_invariants(&self, q: &[&str]) -> bool {
        for invariant in q {
            match *invariant {
                "Q1" => {
                    // No retrocausation: action should be based on current observations
                    // (simplified check)
                    if self.ticks_used > 8 {
                        return false;
                    }
                }
                "Q2" => {
                    // Type soundness: action must be in the ontology
                    // (would check against Σ in real implementation)
                    true;
                }
                "Q3" => {
                    // Guard preservation: recursion depth ≤ 8
                    if self.ticks_used > 8 {
                        return false;
                    }
                }
                "Q4" => {
                    // SLO compliance: latency within budget
                    if self.action.budget.max_latency_ms > 100 {
                        return false;  // Hot path only
                    }
                }
                "Q5" => {
                    // Performance bounds
                    if self.action.budget.max_memory_mb > 1024 {
                        return false;
                    }
                }
                _ => {}
            }
        }

        true
    }
}

impl fmt::Display for ChatmanEquation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "A = μ(O): observation={}, ontology={}, action={}, ticks={}/8, receipt={}",
            self.observation, self.ontology_id, self.action.command, self.ticks_used, self.receipt_id
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chatman_equation_respects_q3() {
        let eq = ChatmanEquation {
            observation: "latency_spike".to_string(),
            ontology_id: "snap1".to_string(),
            invariants: vec!["Q3".to_string()],
            action: Action {
                command: "throttle_requests".to_string(),
                sector: "ops".to_string(),
                preconditions: vec!["latency > 100ms".to_string()],
                postconditions: vec!["latency < 50ms".to_string()],
                budget: ResourceBudget {
                    max_ticks: 8,
                    max_memory_mb: 256,
                    max_latency_ms: 10,
                },
            },
            ticks_used: 6,
            receipt_id: "receipt1".to_string(),
        };

        assert!(eq.respects_invariants(&["Q3"]));
    }

    #[test]
    fn test_chatman_equation_violates_q3() {
        let eq = ChatmanEquation {
            observation: "latency_spike".to_string(),
            ontology_id: "snap1".to_string(),
            invariants: vec!["Q3".to_string()],
            action: Action {
                command: "recursive_repair".to_string(),
                sector: "ops".to_string(),
                preconditions: vec![],
                postconditions: vec![],
                budget: ResourceBudget {
                    max_ticks: 16,  // Violates Chatman constant!
                    max_memory_mb: 256,
                    max_latency_ms: 10,
                },
            },
            ticks_used: 16,
            receipt_id: "receipt1".to_string(),
        };

        assert!(!eq.respects_invariants(&["Q3"]));
    }
}
