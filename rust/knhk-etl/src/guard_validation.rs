// rust/knhk-etl/src/guard_validation.rs
// Branchless guard validation helpers for hot path
// Based on simdjson: eliminate branches for better branch prediction

use crate::load::PredRun;

/// Branchless guard validation helpers
///
/// Pattern from simdjson: use arithmetic instead of branches to avoid branch misprediction.
/// These functions return boolean results that can be used in conditional moves or arithmetic,
/// allowing the compiler to generate branchless code.
///
/// # Performance Benefits
/// - Eliminates branch misprediction penalties
/// - Better instruction-level parallelism
/// - More predictable performance (constant-time operations)
///
/// Branchless validation: run length ≤ max_run_len
///
/// Returns 1 if valid (run.len ≤ max_len), 0 otherwise.
/// Uses arithmetic comparison instead of branch.
///
/// # Example
/// ```rust
/// if validate_run_len_branchless(run, 8) != 0 {
///     // Valid
/// }
/// ```
#[inline(always)]
pub fn validate_run_len_branchless(run: &PredRun, max_len: u64) -> u64 {
    // Arithmetic comparison: (run.len <= max_len) ? 1 : 0
    // Compiler generates conditional move, not branch
    u64::from(run.len <= max_len)
}

/// Branchless validation: run length ≤ tick budget
///
/// Returns 1 if valid (run.len ≤ tick_budget), 0 otherwise.
#[inline(always)]
pub fn validate_tick_budget_branchless(run: &PredRun, tick_budget: u32) -> u64 {
    u64::from(run.len <= tick_budget as u64)
}

/// Branchless validation: total triples ≤ max_run_len
///
/// Returns 1 if valid (count ≤ max_len), 0 otherwise.
#[inline(always)]
pub fn validate_triple_count_branchless(count: usize, max_len: usize) -> u64 {
    u64::from(count <= max_len)
}

/// Branchless validation: run offset + length ≤ capacity
///
/// Returns 1 if valid (run.off + run.len ≤ capacity), 0 otherwise.
#[inline(always)]
pub fn validate_run_capacity_branchless(run: &PredRun, capacity: u64) -> u64 {
    u64::from(run.off.saturating_add(run.len) <= capacity)
}

/// Combined branchless validation: all guard constraints
///
/// Returns 1 if all validations pass, 0 otherwise.
/// Uses bitwise AND to combine results without branches.
///
/// # Validations
/// - run.len ≤ 8 (Chatman Constant)
/// - run.len ≤ tick_budget
/// - run.off + run.len ≤ capacity
#[inline(always)]
pub fn validate_all_guards_branchless(run: &PredRun, tick_budget: u32, capacity: u64) -> u64 {
    let len_valid = validate_run_len_branchless(run, 8);
    let budget_valid = validate_tick_budget_branchless(run, tick_budget);
    let capacity_valid = validate_run_capacity_branchless(run, capacity);

    // Bitwise AND: all must be 1 for result to be 1
    len_valid & budget_valid & capacity_valid
}

/// Branchless predicate matching
///
/// Returns 1 if predicate matches, 0 otherwise.
/// Uses arithmetic comparison instead of branch.
///
/// # Example
/// ```rust
/// let mask = (0..run.len)
///     .map(|i| match_predicate_branchless(soa.p[run.off + i], target_pred))
///     .sum::<u64>();
/// ```
#[inline(always)]
pub fn match_predicate_branchless(predicate: u64, target: u64) -> u64 {
    u64::from(predicate == target)
}

/// Branchless subject matching
///
/// Returns 1 if subject matches, 0 otherwise.
#[inline(always)]
pub fn match_subject_branchless(subject: u64, target: u64) -> u64 {
    u64::from(subject == target)
}

/// Branchless object matching
///
/// Returns 1 if object matches, 0 otherwise.
#[inline(always)]
pub fn match_object_branchless(object: u64, target: u64) -> u64 {
    u64::from(object == target)
}

/// Branchless ASK_SP matching (subject and predicate)
///
/// Returns 1 if both subject and predicate match, 0 otherwise.
/// Uses bitwise AND to combine results.
#[inline(always)]
pub fn match_ask_sp_branchless(subject: u64, predicate: u64, target_s: u64, target_p: u64) -> u64 {
    match_subject_branchless(subject, target_s) & match_predicate_branchless(predicate, target_p)
}

/// Branchless ASK_SPO matching (subject, predicate, object)
///
/// Returns 1 if all three match, 0 otherwise.
#[inline(always)]
pub fn match_ask_spo_branchless(
    subject: u64,
    predicate: u64,
    object: u64,
    target_s: u64,
    target_p: u64,
    target_o: u64,
) -> u64 {
    match_subject_branchless(subject, target_s)
        & match_predicate_branchless(predicate, target_p)
        & match_object_branchless(object, target_o)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::load::PredRun;

    #[test]
    fn test_validate_run_len_branchless() {
        let run = PredRun {
            pred: 100,
            off: 0,
            len: 5,
        };
        assert_eq!(validate_run_len_branchless(&run, 8), 1);
        assert_eq!(validate_run_len_branchless(&run, 4), 0);
    }

    #[test]
    fn test_validate_tick_budget_branchless() {
        let run = PredRun {
            pred: 100,
            off: 0,
            len: 5,
        };
        assert_eq!(validate_tick_budget_branchless(&run, 8), 1);
        assert_eq!(validate_tick_budget_branchless(&run, 4), 0);
    }

    #[test]
    fn test_validate_all_guards_branchless() {
        let run = PredRun {
            pred: 100,
            off: 0,
            len: 5,
        };
        assert_eq!(validate_all_guards_branchless(&run, 8, 8), 1);

        let invalid_run = PredRun {
            pred: 100,
            off: 0,
            len: 9,
        };
        assert_eq!(validate_all_guards_branchless(&invalid_run, 8, 8), 0);
    }

    #[test]
    fn test_match_predicate_branchless() {
        assert_eq!(match_predicate_branchless(100, 100), 1);
        assert_eq!(match_predicate_branchless(100, 200), 0);
    }

    #[test]
    fn test_match_ask_sp_branchless() {
        assert_eq!(match_ask_sp_branchless(1, 100, 1, 100), 1);
        assert_eq!(match_ask_sp_branchless(1, 100, 1, 200), 0);
        assert_eq!(match_ask_sp_branchless(1, 100, 2, 100), 0);
    }

    #[test]
    fn test_match_ask_spo_branchless() {
        assert_eq!(match_ask_spo_branchless(1, 100, 1000, 1, 100, 1000), 1);
        assert_eq!(match_ask_spo_branchless(1, 100, 1000, 1, 100, 2000), 0);
    }
}
