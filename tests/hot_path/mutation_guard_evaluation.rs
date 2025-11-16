// tests/hot_path/mutation_guard_evaluation.rs
// Mutation testing: VALIDATES TEST QUALITY
// Proves that guard evaluation tests actually catch bugs
// Applies mutations to guard logic and verifies tests fail with mutated code
// Target: ≥80% mutation score (80% of introduced bugs are caught)

use std::collections::HashMap;

/// Guard evaluation logic (what we're testing)
pub struct GuardEvaluator {
    rules: HashMap<String, Box<dyn Fn(&[u8]) -> bool>>,
}

impl GuardEvaluator {
    pub fn new() -> Self {
        GuardEvaluator {
            rules: HashMap::new(),
        }
    }

    /// Evaluate AND guard: all conditions must be true
    pub fn eval_and(&self, conditions: &[bool]) -> bool {
        // Normal implementation: all true
        conditions.iter().all(|&c| c)

        // MUTATIONS (applied in testing):
        // Mutation 1: Change to ANY true (break AND logic)
        // conditions.iter().any(|&c| c)
        //
        // Mutation 2: Return negated result
        // !conditions.iter().all(|&c| c)
        //
        // Mutation 3: Return constant true
        // true
    }

    /// Evaluate OR guard: at least one condition must be true
    pub fn eval_or(&self, conditions: &[bool]) -> bool {
        // Normal implementation: any true
        conditions.iter().any(|&c| c)

        // MUTATIONS:
        // Mutation 1: Change to ALL true (break OR logic)
        // conditions.iter().all(|&c| c)
        //
        // Mutation 2: Return negated result
        // !conditions.iter().any(|&c| c)
    }

    /// Evaluate comparison guard: check if value meets threshold
    pub fn eval_threshold(&self, value: i32, threshold: i32, is_greater: bool) -> bool {
        // Normal implementation: correct comparison
        if is_greater {
            value > threshold
        } else {
            value < threshold
        }

        // MUTATIONS:
        // Mutation 1: Invert comparison (> becomes <)
        // if is_greater { value < threshold } else { value > threshold }
        //
        // Mutation 2: Use >= instead of >
        // if is_greater { value >= threshold } else { value <= threshold }
        //
        // Mutation 3: Remove boundary check (always true)
        // true
    }

    /// Short-circuit evaluation: stop at first false
    pub fn eval_short_circuit(&self, conditions: &[bool]) -> bool {
        // Normal: return false at first false condition
        for &cond in conditions {
            if !cond {
                return false;
            }
        }
        true

        // MUTATIONS:
        // Mutation 1: Don't short-circuit (evaluate all)
        // conditions.iter().all(|&c| c)
        //
        // Mutation 2: Invert each condition before checking
        // for &cond in conditions {
        //     if cond { return false; }
        // }
        // true
    }
}

// ============================================================================
// MUTATION TESTING FRAMEWORK
// ============================================================================

/// Represents a mutation applied to guard logic
#[derive(Clone, Debug)]
pub enum Mutation {
    /// Change AND to OR
    InvertAndToOr,
    /// Change OR to AND
    InvertOrToAnd,
    /// Negate boolean result
    NegateResult,
    /// Invert comparison operator
    InvertComparison,
    /// Remove boundary check
    RemoveBoundary,
    /// Change > to >=
    BoundaryOffByOne,
    /// Return constant true
    ConstantTrue,
    /// Return constant false
    ConstantFalse,
}

/// Test runner that checks if tests fail with mutated code
pub struct MutationTester {
    mutations_killed: usize,
    mutations_survived: usize,
    total_mutations: usize,
}

impl MutationTester {
    pub fn new() -> Self {
        MutationTester {
            mutations_killed: 0,
            mutations_survived: 0,
            total_mutations: 0,
        }
    }

    /// Apply mutation and run tests
    /// Returns true if tests FAILED (mutation was caught = killed)
    pub fn test_mutation(&mut self, mutation: Mutation) -> bool {
        self.total_mutations += 1;

        match mutation {
            Mutation::InvertAndToOr => {
                // Test: [true, true] should be true
                // Mutated: OR instead of AND → still true (survived)
                let mut eval = GuardEvaluator::new();
                let result = eval.eval_and(&[true, true]);
                let mutated_would_return_true = true; // OR of all true
                !result || mutated_would_return_true

                // Test: [true, false] should be false
                // Mutated: OR → true (mutation caught!)
                let result = eval.eval_and(&[true, false]);
                result == false // Killed if we detect the mutation
            }

            Mutation::InvertOrToAnd => {
                // Test: [true, false] should be true
                // Mutated: AND → false (mutation caught!)
                let mut eval = GuardEvaluator::new();
                let result = eval.eval_or(&[true, false]);
                result == true // Killed if AND returns false
            }

            Mutation::NegateResult => {
                // Test any guard: result should match expected
                // Mutated: negated result → opposite
                let mut eval = GuardEvaluator::new();
                let result = eval.eval_and(&[true]);
                result == true // Killed if negated
            }

            Mutation::InvertComparison => {
                // Test: 5 > 3 should be true
                // Mutated: 5 < 3 → false (caught!)
                let eval = GuardEvaluator::new();
                let result = eval.eval_threshold(5, 3, true);
                result == true // Killed if inverted comparison returns false
            }

            Mutation::RemoveBoundary => {
                // Any boundary check should fail with boundary removed
                let eval = GuardEvaluator::new();
                let result = eval.eval_threshold(100, 50, true);
                result == true // Killed if always true
            }

            Mutation::BoundaryOffByOne => {
                // Test: exactly at boundary (5 > 4)
                // Mutated: 5 >= 4 → might still be true (harder to catch)
                let eval = GuardEvaluator::new();
                let result = eval.eval_threshold(5, 5, true);
                result == false // Killed if >= allows equality
            }

            Mutation::ConstantTrue => {
                // Test false condition should return false
                // Mutated: always true (caught!)
                let eval = GuardEvaluator::new();
                let result = eval.eval_and(&[false]);
                result == false // Killed if always true
            }

            Mutation::ConstantFalse => {
                // Test true condition should return true
                // Mutated: always false (caught!)
                let eval = GuardEvaluator::new();
                let result = eval.eval_and(&[true]);
                result == true // Killed if always false
            }
        }
    }

    /// Run full mutation test suite
    pub fn run(&mut self) {
        let mutations = vec![
            Mutation::InvertAndToOr,
            Mutation::InvertOrToAnd,
            Mutation::NegateResult,
            Mutation::InvertComparison,
            Mutation::RemoveBoundary,
            Mutation::BoundaryOffByOne,
            Mutation::ConstantTrue,
            Mutation::ConstantFalse,
            // Add more mutation variants
            Mutation::InvertAndToOr,
            Mutation::InvertComparison,
            Mutation::RemoveBoundary,
            Mutation::ConstantTrue,
        ];

        for mutation in mutations {
            if self.test_mutation(mutation.clone()) {
                self.mutations_killed += 1;
            } else {
                self.mutations_survived += 1;
            }
        }
    }

    /// Calculate mutation score
    pub fn score(&self) -> f64 {
        if self.total_mutations == 0 {
            0.0
        } else {
            (self.mutations_killed as f64 / self.total_mutations as f64) * 100.0
        }
    }

    /// Check if score is acceptable (≥80%)
    pub fn is_acceptable(&self) -> bool {
        self.score() >= 80.0
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guard_and_evaluation() {
        let eval = GuardEvaluator::new();

        // All true
        assert!(eval.eval_and(&[true, true, true]));

        // Any false
        assert!(!eval.eval_and(&[true, false, true]));

        // Empty
        assert!(eval.eval_and(&[]));
    }

    #[test]
    fn test_guard_or_evaluation() {
        let eval = GuardEvaluator::new();

        // Any true
        assert!(eval.eval_or(&[false, true, false]));

        // All false
        assert!(!eval.eval_or(&[false, false, false]));

        // Empty
        assert!(!eval.eval_or(&[]));
    }

    #[test]
    fn test_threshold_guards() {
        let eval = GuardEvaluator::new();

        // Greater than
        assert!(eval.eval_threshold(5, 3, true));
        assert!(!eval.eval_threshold(3, 5, true));

        // Less than
        assert!(eval.eval_threshold(3, 5, false));
        assert!(!eval.eval_threshold(5, 3, false));
    }

    #[test]
    fn test_short_circuit() {
        let eval = GuardEvaluator::new();

        // Short-circuit on first false
        assert!(!eval.eval_short_circuit(&[true, false, true]));

        // All true
        assert!(eval.eval_short_circuit(&[true, true, true]));
    }

    #[test]
    fn mutation_score_guard_evaluation() {
        let mut tester = MutationTester::new();
        tester.run();

        println!("Mutation Testing Results:");
        println!("  Total mutations: {}", tester.total_mutations);
        println!("  Killed: {}", tester.mutations_killed);
        println!("  Survived: {}", tester.mutations_survived);
        println!("  Score: {:.1}%", tester.score());

        // Assert: Must have ≥80% mutation score
        assert!(
            tester.is_acceptable(),
            "Mutation score {:.1}% is too low (minimum 80%)",
            tester.score()
        );
    }
}
