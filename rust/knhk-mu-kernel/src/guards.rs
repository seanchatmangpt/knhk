//! Q - Invariant Enforcement as Hardware
//!
//! Guards are not "checks"; they are compiled, branchless predicates
//! that enforce Q (invariants) at both compile-time and runtime.

use crate::isa::{GuardContext, ObsValue};
use crate::timing::TickBudget;

pub use crate::sigma::GuardType;

/// Guard ID
pub type GuardId = u16;

/// Guard result (branchless)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GuardResult {
    /// Guard passed
    Pass = 0,
    /// Guard failed
    Fail = 1,
}

impl GuardResult {
    /// Convert to bool (branchless)
    #[inline(always)]
    pub const fn as_bool(&self) -> bool {
        matches!(self, GuardResult::Pass)
    }

    /// Combine results with AND (branchless)
    #[inline(always)]
    pub const fn and(&self, other: &GuardResult) -> GuardResult {
        const RESULTS: [[GuardResult; 2]; 2] = [
            [GuardResult::Fail, GuardResult::Fail], // Fail AND _
            [GuardResult::Fail, GuardResult::Pass], // Pass AND _
        ];
        RESULTS[*self as usize][*other as usize]
    }
}

/// Compiled guard (branchless evaluation)
#[repr(C, align(64))]
pub struct CompiledGuard {
    /// Guard ID
    pub id: GuardId,
    /// Guard type
    pub guard_type: GuardType,
    /// Evaluator function pointer
    evaluator: fn(&GuardContext) -> GuardResult,
}

impl CompiledGuard {
    /// Create a new compiled guard
    pub const fn new(
        id: GuardId,
        guard_type: GuardType,
        evaluator: fn(&GuardContext) -> GuardResult,
    ) -> Self {
        Self {
            id,
            guard_type,
            evaluator,
        }
    }

    /// Evaluate guard (branchless)
    #[inline(always)]
    pub fn eval(&self, ctx: &GuardContext) -> GuardResult {
        (self.evaluator)(ctx)
    }
}

/// Tick budget guard (enforces Chatman Constant)
#[inline(always)]
pub fn guard_tick_budget(ctx: &GuardContext) -> GuardResult {
    // Extract tick budget from context
    let budget = ctx.params[0];
    let used = ctx.params[1];

    // Branchless comparison
    let passed = (used <= budget) as u8;
    const RESULTS: [GuardResult; 2] = [GuardResult::Fail, GuardResult::Pass];
    RESULTS[passed as usize]
}

/// Carry invariant guard (KGC preservation)
#[inline(always)]
pub fn guard_carry_invariant(ctx: &GuardContext) -> GuardResult {
    // Check that knowledge graph is preserved
    let input_hash = ctx.params[0];
    let output_hash = ctx.params[1];

    // For KGC, output must extend input (simplified check)
    let preserved = (output_hash >= input_hash) as u8;
    const RESULTS: [GuardResult; 2] = [GuardResult::Fail, GuardResult::Pass];
    RESULTS[preserved as usize]
}

/// Authorization guard
#[inline(always)]
pub fn guard_authorization(ctx: &GuardContext) -> GuardResult {
    // Check authorization bitmap
    let required_role = ctx.params[0];
    let user_roles = ctx.params[1];

    // Branchless bitmap check
    let authorized = ((user_roles & required_role) != 0) as u8;
    const RESULTS: [GuardResult; 2] = [GuardResult::Fail, GuardResult::Pass];
    RESULTS[authorized as usize]
}

/// Schema validation guard
#[inline(always)]
pub fn guard_schema_validation(ctx: &GuardContext) -> GuardResult {
    // Validate against schema hash
    let expected_schema = ctx.params[0];
    let actual_schema = ctx.params[1];

    let valid = (expected_schema == actual_schema) as u8;
    const RESULTS: [GuardResult; 2] = [GuardResult::Fail, GuardResult::Pass];
    RESULTS[valid as usize]
}

/// Guard evaluator registry
#[repr(C, align(4096))]
pub struct GuardRegistry {
    /// Compiled guards (fixed array)
    guards: [Option<CompiledGuard>; 1024],
}

impl GuardRegistry {
    /// Create a new guard registry
    pub const fn new() -> Self {
        Self {
            guards: [None; 1024],
        }
    }

    /// Register a guard
    pub fn register(&mut self, guard: CompiledGuard) -> Result<(), GuardError> {
        let idx = guard.id as usize;
        if idx >= 1024 {
            return Err(GuardError::InvalidId);
        }

        if self.guards[idx].is_some() {
            return Err(GuardError::AlreadyRegistered);
        }

        self.guards[idx] = Some(guard);
        Ok(())
    }

    /// Get guard by ID (O(1))
    #[inline(always)]
    pub fn get(&self, id: GuardId) -> Option<&CompiledGuard> {
        let idx = id as usize;
        if idx < 1024 {
            self.guards[idx].as_ref()
        } else {
            None
        }
    }

    /// Evaluate guard
    #[inline(always)]
    pub fn eval(&self, id: GuardId, ctx: &GuardContext) -> Result<GuardResult, GuardError> {
        match self.get(id) {
            Some(guard) => Ok(guard.eval(ctx)),
            None => Err(GuardError::NotFound),
        }
    }

    /// Evaluate multiple guards (AND combination)
    #[inline]
    pub fn eval_all(&self, ids: &[GuardId], ctx: &GuardContext) -> Result<GuardResult, GuardError> {
        let mut result = GuardResult::Pass;

        for &id in ids {
            let guard_result = self.eval(id, ctx)?;
            result = result.and(&guard_result);

            // Early exit if any guard fails (branchless optimization)
            if !result.as_bool() {
                return Ok(GuardResult::Fail);
            }
        }

        Ok(result)
    }
}

impl Default for GuardRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Guard errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardError {
    /// Invalid guard ID
    InvalidId,
    /// Guard already registered
    AlreadyRegistered,
    /// Guard not found
    NotFound,
    /// Evaluation failed
    EvaluationFailed,
}

/// Q enforcement - compile-time checks
pub mod compile_time {
    use super::*;

    /// Check if tick budget is satisfied
    pub const fn check_tick_budget(pattern_cost: u8, guard_count: u8) -> bool {
        // Each guard costs ~1 tick
        let total_cost = pattern_cost as u64 + guard_count as u64;
        total_cost <= crate::CHATMAN_CONSTANT
    }

    /// Check if pattern is valid
    pub const fn check_pattern_valid(pattern_id: u8) -> bool {
        pattern_id < 43
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guard_result_and() {
        assert_eq!(GuardResult::Pass.and(&GuardResult::Pass), GuardResult::Pass);
        assert_eq!(GuardResult::Pass.and(&GuardResult::Fail), GuardResult::Fail);
        assert_eq!(GuardResult::Fail.and(&GuardResult::Pass), GuardResult::Fail);
        assert_eq!(GuardResult::Fail.and(&GuardResult::Fail), GuardResult::Fail);
    }

    #[test]
    fn test_tick_budget_guard() {
        let ctx = GuardContext {
            task_id: 1,
            obs_data: 0,
            params: [8, 5, 0, 0], // budget=8, used=5
        };

        let result = guard_tick_budget(&ctx);
        assert_eq!(result, GuardResult::Pass);

        let ctx_exceeded = GuardContext {
            task_id: 1,
            obs_data: 0,
            params: [8, 10, 0, 0], // budget=8, used=10
        };

        let result = guard_tick_budget(&ctx_exceeded);
        assert_eq!(result, GuardResult::Fail);
    }

    #[test]
    fn test_authorization_guard() {
        let ctx = GuardContext {
            task_id: 1,
            obs_data: 0,
            params: [0b0100, 0b0110, 0, 0], // required=4, user=6 (has 4)
        };

        let result = guard_authorization(&ctx);
        assert_eq!(result, GuardResult::Pass);

        let ctx_unauthorized = GuardContext {
            task_id: 1,
            obs_data: 0,
            params: [0b1000, 0b0110, 0, 0], // required=8, user=6 (no 8)
        };

        let result = guard_authorization(&ctx_unauthorized);
        assert_eq!(result, GuardResult::Fail);
    }

    #[test]
    fn test_guard_registry() {
        let mut registry = GuardRegistry::new();

        let guard = CompiledGuard::new(0, GuardType::TickBudget, guard_tick_budget);
        registry.register(guard).unwrap();

        let ctx = GuardContext {
            task_id: 1,
            obs_data: 0,
            params: [8, 5, 0, 0],
        };

        let result = registry.eval(0, &ctx).unwrap();
        assert_eq!(result, GuardResult::Pass);
    }

    #[test]
    fn test_eval_all_guards() {
        let mut registry = GuardRegistry::new();

        registry
            .register(CompiledGuard::new(
                0,
                GuardType::TickBudget,
                guard_tick_budget,
            ))
            .unwrap();
        registry
            .register(CompiledGuard::new(
                1,
                GuardType::Authorization,
                guard_authorization,
            ))
            .unwrap();

        let ctx = GuardContext {
            task_id: 1,
            obs_data: 0,
            params: [8, 5, 0, 0],
        };

        let result = registry.eval_all(&[0], &ctx).unwrap();
        assert_eq!(result, GuardResult::Pass);
    }

    #[test]
    fn test_compile_time_checks() {
        use compile_time::*;

        assert!(check_tick_budget(3, 2)); // 3 + 2 = 5 ≤ 8
        assert!(!check_tick_budget(6, 5)); // 6 + 5 = 11 > 8

        assert!(check_pattern_valid(0));
        assert!(check_pattern_valid(42));
        assert!(!check_pattern_valid(50));
    }
}
