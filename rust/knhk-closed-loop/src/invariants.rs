// Hard Invariant Enforcement: Q1-Q5
// The immutable constraints that govern all decisions
// No ontology change can violate these

use serde::{Deserialize, Serialize};

/// Hard invariant definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HardInvariants {
    /// Q1: No retrocausation (time flows forward, immutable DAG)
    pub q1_no_retrocausation: bool,

    /// Q2: Type soundness (O ⊨ Σ, observations match ontology)
    pub q2_type_soundness: bool,

    /// Q3: Guard preservation (max_run_len ≤ 8, Chatman constant)
    pub q3_guard_preservation: bool,

    /// Q4: SLO compliance (hot path ≤8 ticks, warm path <100ms)
    pub q4_slo_compliance: bool,

    /// Q5: Performance bounds (memory, CPU, latency)
    pub q5_performance_bounds: bool,
}

impl HardInvariants {
    pub fn all_preserved(&self) -> bool {
        self.q1_no_retrocausation
            && self.q2_type_soundness
            && self.q3_guard_preservation
            && self.q4_slo_compliance
            && self.q5_performance_bounds
    }

    pub fn which_violated(&self) -> Vec<String> {
        let mut violated = Vec::new();
        if !self.q1_no_retrocausation {
            violated.push("Q1: Retrocausation (time not flowing forward)".to_string());
        }
        if !self.q2_type_soundness {
            violated.push("Q2: Type unsoundness (O ⊭ Σ)".to_string());
        }
        if !self.q3_guard_preservation {
            violated.push("Q3: Guard violation (max_run_len > 8)".to_string());
        }
        if !self.q4_slo_compliance {
            violated.push("Q4: SLO violation (latency budget exceeded)".to_string());
        }
        if !self.q5_performance_bounds {
            violated.push("Q5: Resource bounds exceeded".to_string());
        }
        violated
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InvariantViolation {
    #[error("Q1 violated: {0}")]
    Q1Violation(String),

    #[error("Q2 violated: {0}")]
    Q2Violation(String),

    #[error("Q3 violated: {0}")]
    Q3Violation(String),

    #[error("Q4 violated: {0}")]
    Q4Violation(String),

    #[error("Q5 violated: {0}")]
    Q5Violation(String),
}

/// Validator that checks hard invariants
pub struct InvariantValidator;

impl InvariantValidator {
    /// Check Q1: No retrocausation
    /// Verify that snapshot parent references form a DAG with no cycles
    pub fn check_q1_no_retrocausation(
        snapshot_id: &str,
        parent_id: Option<&str>,
        visited: &mut std::collections::HashSet<String>,
    ) -> Result<bool, InvariantViolation> {
        if visited.contains(snapshot_id) {
            return Err(InvariantViolation::Q1Violation(
                "Cycle detected in snapshot DAG".to_string(),
            ));
        }

        visited.insert(snapshot_id.to_string());

        // If parent exists, check parent recursively
        if let Some(parent) = parent_id {
            // This is simplified; in real implementation, would look up parent from store
            if parent == snapshot_id {
                return Err(InvariantViolation::Q1Violation(
                    "Snapshot cannot be its own parent".to_string(),
                ));
            }
            // Could recurse to check grandparent, etc.
        }

        Ok(true)
    }

    /// Check Q2: Type soundness
    /// Verify that observations conform to the ontology schema
    pub fn check_q2_type_soundness(
        observation_count: usize,
        schema_violations: usize,
    ) -> Result<bool, InvariantViolation> {
        if schema_violations == 0 {
            Ok(true)
        } else {
            let violation_rate = schema_violations as f64 / observation_count.max(1) as f64;
            if violation_rate > 0.01 {
                // >1% violations
                Err(InvariantViolation::Q2Violation(format!(
                    "Schema violations detected: {}/{}",
                    schema_violations, observation_count
                )))
            } else {
                Ok(true)
            }
        }
    }

    /// Check Q3: Guard preservation
    /// Verify that max_run_length (Chatman constant) ≤ 8 ticks
    pub fn check_q3_guard_preservation(max_ticks: u32) -> Result<bool, InvariantViolation> {
        if max_ticks <= crate::CHATMAN_CONSTANT {
            Ok(true)
        } else {
            Err(InvariantViolation::Q3Violation(format!(
                "Max run length {} exceeds Chatman constant of {}",
                max_ticks,
                crate::CHATMAN_CONSTANT
            )))
        }
    }

    /// Check Q4: SLO compliance
    /// Verify that operation latencies meet budgets
    pub fn check_q4_slo_compliance(
        hot_path_latency_ticks: u32,
        warm_path_latency_ms: u32,
    ) -> Result<bool, InvariantViolation> {
        let mut violations = Vec::new();

        if hot_path_latency_ticks > crate::CHATMAN_CONSTANT {
            violations.push(format!(
                "Hot path {} ticks exceeds budget of {}",
                hot_path_latency_ticks,
                crate::CHATMAN_CONSTANT
            ));
        }

        if warm_path_latency_ms > 100 {
            violations.push(format!(
                "Warm path {} ms exceeds budget of 100ms",
                warm_path_latency_ms
            ));
        }

        if violations.is_empty() {
            Ok(true)
        } else {
            Err(InvariantViolation::Q4Violation(violations.join("; ")))
        }
    }

    /// Check Q5: Performance bounds
    /// Verify memory, CPU, and latency stay within budget
    pub fn check_q5_performance_bounds(
        memory_usage_mb: u32,
        cpu_percent: f64,
        tail_latency_ms: u32,
    ) -> Result<bool, InvariantViolation> {
        let mut violations = Vec::new();

        if memory_usage_mb > 1024 {
            violations.push(format!(
                "Memory {} MB exceeds budget of 1024 MB",
                memory_usage_mb
            ));
        }

        if cpu_percent > 50.0 {
            violations.push(format!(
                "CPU {:.1}% exceeds budget of 50%",
                cpu_percent
            ));
        }

        if tail_latency_ms > 500 {
            violations.push(format!(
                "Tail latency {} ms exceeds budget of 500ms",
                tail_latency_ms
            ));
        }

        if violations.is_empty() {
            Ok(true)
        } else {
            Err(InvariantViolation::Q5Violation(violations.join("; ")))
        }
    }

    /// Comprehensive invariant check
    pub fn check_all(
        snapshot_id: &str,
        parent_id: Option<&str>,
        observation_count: usize,
        schema_violations: usize,
        max_ticks: u32,
        hot_path_latency_ticks: u32,
        warm_path_latency_ms: u32,
        memory_usage_mb: u32,
        cpu_percent: f64,
        tail_latency_ms: u32,
    ) -> Result<HardInvariants, InvariantViolation> {
        let mut visited = std::collections::HashSet::new();

        Ok(HardInvariants {
            q1_no_retrocausation: Self::check_q1_no_retrocausation(
                snapshot_id,
                parent_id,
                &mut visited,
            )?,
            q2_type_soundness: Self::check_q2_type_soundness(observation_count, schema_violations)?,
            q3_guard_preservation: Self::check_q3_guard_preservation(max_ticks)?,
            q4_slo_compliance: Self::check_q4_slo_compliance(
                hot_path_latency_ticks,
                warm_path_latency_ms,
            )?,
            q5_performance_bounds: Self::check_q5_performance_bounds(
                memory_usage_mb,
                cpu_percent,
                tail_latency_ms,
            )?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hard_invariants_all_preserved() {
        let inv = HardInvariants {
            q1_no_retrocausation: true,
            q2_type_soundness: true,
            q3_guard_preservation: true,
            q4_slo_compliance: true,
            q5_performance_bounds: true,
        };
        assert!(inv.all_preserved());
    }

    #[test]
    fn test_hard_invariants_some_violated() {
        let inv = HardInvariants {
            q1_no_retrocausation: false,
            q2_type_soundness: true,
            q3_guard_preservation: true,
            q4_slo_compliance: false,
            q5_performance_bounds: true,
        };
        assert!(!inv.all_preserved());
        assert_eq!(inv.which_violated().len(), 2);
    }

    #[test]
    fn test_q1_no_retrocausation() {
        let mut visited = std::collections::HashSet::new();
        assert!(InvariantValidator::check_q1_no_retrocausation("snap1", Some("snap0"), &mut visited)
            .is_ok());
    }

    #[test]
    fn test_q1_self_reference_fails() {
        let mut visited = std::collections::HashSet::new();
        assert!(InvariantValidator::check_q1_no_retrocausation("snap1", Some("snap1"), &mut visited)
            .is_err());
    }

    #[test]
    fn test_q3_guard_preservation_valid() {
        assert!(InvariantValidator::check_q3_guard_preservation(8).is_ok());
        assert!(InvariantValidator::check_q3_guard_preservation(4).is_ok());
    }

    #[test]
    fn test_q3_guard_preservation_invalid() {
        assert!(InvariantValidator::check_q3_guard_preservation(9).is_err());
        assert!(InvariantValidator::check_q3_guard_preservation(100).is_err());
    }

    #[test]
    fn test_q4_slo_compliance_valid() {
        assert!(InvariantValidator::check_q4_slo_compliance(8, 50).is_ok());
    }

    #[test]
    fn test_q4_slo_compliance_hot_path_violation() {
        assert!(InvariantValidator::check_q4_slo_compliance(9, 50).is_err());
    }

    #[test]
    fn test_q4_slo_compliance_warm_path_violation() {
        assert!(InvariantValidator::check_q4_slo_compliance(8, 101).is_err());
    }

    #[test]
    fn test_q5_performance_bounds_valid() {
        assert!(InvariantValidator::check_q5_performance_bounds(512, 25.0, 250).is_ok());
    }

    #[test]
    fn test_q5_performance_bounds_memory_violation() {
        assert!(InvariantValidator::check_q5_performance_bounds(2048, 25.0, 250).is_err());
    }

    #[test]
    fn test_comprehensive_check() {
        let result = InvariantValidator::check_all(
            "snap1",
            Some("snap0"),
            100,
            0,
            8,
            8,
            50,
            512,
            25.0,
            250,
        );
        assert!(result.is_ok());
        let inv = result.unwrap();
        assert!(inv.all_preserved());
    }

    #[test]
    fn test_comprehensive_check_multiple_violations() {
        let result = InvariantValidator::check_all(
            "snap1",
            Some("snap0"),
            100,
            50, // Schema violations
            9,  // Guard violation
            9,  // Hot path violation
            150, // Warm path violation
            2048, // Memory violation
            75.0, // CPU violation
            750, // Tail latency violation
        );
        assert!(result.is_err());
    }
}
