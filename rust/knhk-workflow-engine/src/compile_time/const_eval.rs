//! Const evaluation utilities for compile-time workflow analysis
//!
//! This module provides const functions for analyzing workflow properties
//! at compile time, enabling compile-time assertions and optimizations.

use std::collections::HashSet;

/// Workflow metrics that can be calculated at compile time
#[derive(Debug, Clone, Copy)]
pub struct WorkflowMetrics {
    /// Number of states in the workflow
    pub state_count: usize,
    /// Number of transitions
    pub transition_count: usize,
    /// Cyclomatic complexity
    pub complexity: usize,
    /// Maximum parallel branches
    pub max_parallelism: usize,
    /// Whether workflow has cycles
    pub has_cycles: bool,
}

impl WorkflowMetrics {
    /// Create new metrics
    pub const fn new(
        state_count: usize,
        transition_count: usize,
        complexity: usize,
        max_parallelism: usize,
        has_cycles: bool,
    ) -> Self {
        Self {
            state_count,
            transition_count,
            complexity,
            max_parallelism,
            has_cycles,
        }
    }

    /// Check if workflow is simple (low complexity)
    pub const fn is_simple(&self) -> bool {
        self.complexity <= 10 && self.max_parallelism <= 2
    }

    /// Check if workflow is complex
    pub const fn is_complex(&self) -> bool {
        self.complexity > 50 || self.max_parallelism > 10
    }

    /// Estimate memory usage (bytes)
    pub const fn estimated_memory_bytes(&self) -> usize {
        // Rough estimate: state object + context map
        self.state_count * 256 + self.transition_count * 64
    }
}

/// Calculate cyclomatic complexity at compile time
///
/// Complexity = E - N + 2P
/// where E = edges, N = nodes, P = connected components
pub const fn calculate_complexity(edges: usize, nodes: usize, components: usize) -> usize {
    // Use saturating arithmetic for const context
    edges.saturating_sub(nodes).saturating_add(2 * components)
}

/// Validate workflow bounds at compile time
pub const fn validate_bounds(
    state_count: usize,
    max_states: usize,
    transition_count: usize,
    max_transitions: usize,
) -> bool {
    state_count <= max_states && transition_count <= max_transitions
}

/// Calculate maximum parallel branches
pub const fn max_parallel_branches(splits: usize, max_branches_per_split: usize) -> usize {
    splits * max_branches_per_split
}

/// Estimate workflow execution time (ticks)
pub const fn estimate_execution_ticks(
    transition_count: usize,
    avg_ticks_per_transition: usize,
) -> usize {
    transition_count * avg_ticks_per_transition
}

/// Check if workflow meets performance requirements
pub const fn meets_performance_requirements(
    estimated_ticks: usize,
    max_ticks: usize,
) -> bool {
    estimated_ticks <= max_ticks
}

/// Compile-time assertion helper
///
/// # Example
///
/// ```rust,ignore
/// const_assert!(WORKFLOW_COMPLEXITY <= 100);
/// const_assert!(ESTIMATED_TICKS <= 8); // Chatman Constant
/// ```
#[macro_export]
macro_rules! const_assert {
    ($condition:expr) => {
        const _: () = {
            if !$condition {
                panic!(concat!("Assertion failed: ", stringify!($condition)));
            }
        };
    };
    ($condition:expr, $message:expr) => {
        const _: () = {
            if !$condition {
                panic!($message);
            }
        };
    };
}

/// Compile-time workflow validation
///
/// This struct holds compile-time workflow properties and provides
/// validation methods that can be called in const contexts.
pub struct CompileTimeWorkflow<const STATES: usize, const TRANSITIONS: usize> {
    _marker: std::marker::PhantomData<()>,
}

impl<const STATES: usize, const TRANSITIONS: usize> CompileTimeWorkflow<STATES, TRANSITIONS> {
    /// Calculate workflow complexity
    pub const fn complexity() -> usize {
        calculate_complexity(TRANSITIONS, STATES, 1)
    }

    /// Check if workflow is within bounds
    pub const fn is_valid(max_states: usize, max_transitions: usize) -> bool {
        validate_bounds(STATES, max_states, TRANSITIONS, max_transitions)
    }

    /// Estimate memory usage
    pub const fn memory_usage() -> usize {
        STATES * 256 + TRANSITIONS * 64
    }

    /// Check if workflow is simple
    pub const fn is_simple() -> bool {
        Self::complexity() <= 10
    }
}

/// Type-level natural numbers for compile-time arithmetic
pub mod nat {
    /// Zero
    pub struct Z;

    /// Successor
    pub struct S<N>(std::marker::PhantomData<N>);

    /// Type-level natural number trait
    pub trait Nat {
        const VALUE: usize;
    }

    impl Nat for Z {
        const VALUE: usize = 0;
    }

    impl<N: Nat> Nat for S<N> {
        const VALUE: usize = N::VALUE + 1;
    }

    // Type aliases for common numbers
    pub type N0 = Z;
    pub type N1 = S<N0>;
    pub type N2 = S<N1>;
    pub type N3 = S<N2>;
    pub type N4 = S<N3>;
    pub type N5 = S<N4>;
    pub type N8 = S<S<S<N5>>>; // Chatman Constant
    pub type N10 = S<S<N8>>;
    pub type N100 = S<S<S<S<S<S<S<S<S<S<N10>>>>>>>>>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_calculation() {
        // Simple sequence: 3 states, 2 transitions
        let complexity = calculate_complexity(2, 3, 1);
        assert_eq!(complexity, 1);

        // With branching: 5 states, 6 transitions
        let complexity = calculate_complexity(6, 5, 1);
        assert_eq!(complexity, 3);
    }

    #[test]
    fn test_workflow_metrics() {
        let metrics = WorkflowMetrics::new(5, 6, 3, 2, false);
        assert!(metrics.is_simple());
        assert!(!metrics.is_complex());
    }

    #[test]
    fn test_performance_estimation() {
        // Each transition takes ~2 ticks on average
        let ticks = estimate_execution_ticks(4, 2);
        assert_eq!(ticks, 8); // Chatman Constant

        assert!(meets_performance_requirements(8, 8));
        assert!(!meets_performance_requirements(9, 8));
    }

    #[test]
    fn test_compile_time_workflow() {
        // Workflow with 5 states and 6 transitions
        type MyWorkflow = CompileTimeWorkflow<5, 6>;

        const COMPLEXITY: usize = MyWorkflow::complexity();
        const IS_SIMPLE: bool = MyWorkflow::is_simple();
        const MEMORY: usize = MyWorkflow::memory_usage();

        assert_eq!(COMPLEXITY, 3);
        assert!(IS_SIMPLE);
        assert!(MEMORY > 0);
    }

    #[test]
    fn test_type_level_numbers() {
        use nat::*;

        assert_eq!(N0::VALUE, 0);
        assert_eq!(N1::VALUE, 1);
        assert_eq!(N8::VALUE, 8); // Chatman Constant
        assert_eq!(N10::VALUE, 10);
    }

    #[test]
    fn test_const_assertions() {
        const WORKFLOW_COMPLEXITY: usize = 5;
        const_assert!(WORKFLOW_COMPLEXITY <= 100);

        const ESTIMATED_TICKS: usize = 8;
        const_assert!(ESTIMATED_TICKS <= 8, "Exceeds Chatman Constant");
    }
}
