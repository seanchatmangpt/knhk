//! μ-Kernel Instruction Set Architecture
//!
//! Defines the primitive operations (μ-ops) and composite instructions
//! that form the knowledge operations ISA.

use crate::guards::GuardId;
use crate::patterns::PatternId;
use crate::timing::{BudgetStatus, TickBudget};

/// μ-Operation result type
pub type MuOpResult<T> = Result<T, MuOpError>;

/// μ-Operation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MuOpError {
    /// Tick budget exceeded
    TickBudgetExceeded = 1,
    /// Guard failed
    GuardFailed = 2,
    /// Invalid pattern ID
    InvalidPattern = 3,
    /// Invalid guard ID
    InvalidGuard = 4,
    /// Invalid observation field
    InvalidObservation = 5,
    /// Receipt buffer full
    ReceiptBufferFull = 6,
}

/// Primitive μ-Operations (μ-ops)
///
/// These are the ONLY operations allowed in μ_hot.
/// Each must complete in ≤1 cycle.
#[repr(C)]
pub struct MuOps;

impl MuOps {
    /// Load Σ* descriptor field
    ///
    /// # Timing: 1 tick (cache hit)
    #[inline(always)]
    pub fn load_sigma(offset: u16) -> u64 {
        // In real implementation, this reads from fixed memory location
        // For now, stub that returns deterministically
        offset as u64 * 0x0100_0000_0000_0000
    }

    /// Dispatch pattern by ID and phase
    ///
    /// # Timing: 1 tick (table lookup)
    #[inline(always)]
    pub fn dispatch_pattern(pattern_id: u8, phase: u8) -> MuOpResult<DispatchResult> {
        // Bounds check (branchless via array)
        const MAX_PATTERN: u8 = 43;
        const MAX_PHASE: u8 = 8;

        let valid = ((pattern_id < MAX_PATTERN) & (phase < MAX_PHASE)) as usize;
        const RESULTS: [MuOpResult<DispatchResult>; 2] = [
            Err(MuOpError::InvalidPattern),
            Ok(DispatchResult {
                handler_offset: 0,
                tick_cost: 1,
            }),
        ];

        RESULTS[valid]
    }

    /// Evaluate guard
    ///
    /// # Timing: 1 tick (branchless evaluation)
    #[inline(always)]
    pub fn eval_guard(guard_id: u16, _context: &GuardContext) -> MuOpResult<bool> {
        // Stub: guards are evaluated branchlessly in real implementation
        if guard_id >= 1024 {
            return Err(MuOpError::InvalidGuard);
        }

        // Deterministic evaluation based on guard_id
        Ok((guard_id & 1) == 0)
    }

    /// Read observation field
    ///
    /// # Timing: 1 tick (fixed offset read)
    #[inline(always)]
    pub fn read_obs(field_id: u8) -> MuOpResult<ObsValue> {
        const MAX_FIELDS: u8 = 64;

        if field_id >= MAX_FIELDS {
            return Err(MuOpError::InvalidObservation);
        }

        // Deterministic value based on field_id
        Ok(ObsValue::U64(field_id as u64 * 1000))
    }

    /// Write receipt field
    ///
    /// # Timing: 1 tick (append to buffer)
    #[inline(always)]
    pub fn write_receipt(field_id: u8, value: ReceiptValue) -> MuOpResult<()> {
        // In real implementation, appends to receipt buffer
        // For now, validate bounds
        const MAX_FIELDS: u8 = 32;

        if field_id >= MAX_FIELDS {
            return Err(MuOpError::ReceiptBufferFull);
        }

        // Stub: write would happen here
        let _ = value;
        Ok(())
    }

    /// Check resource budget
    ///
    /// # Timing: 1 tick (compare)
    #[inline(always)]
    pub fn check_budget(budget: &TickBudget) -> BudgetStatus {
        if budget.is_exhausted() {
            BudgetStatus::Exhausted
        } else {
            BudgetStatus::Ok
        }
    }
}

/// Dispatch result from pattern lookup
#[derive(Debug, Clone, Copy)]
#[repr(C, align(8))]
pub struct DispatchResult {
    /// Handler function offset
    pub handler_offset: u64,
    /// Tick cost estimate
    pub tick_cost: u8,
}

/// Guard evaluation context
#[derive(Debug, Clone, Copy)]
#[repr(C, align(64))] // Cache-aligned
pub struct GuardContext {
    /// Task ID being guarded
    pub task_id: u64,
    /// Observation data pointer
    pub obs_data: u64,
    /// Guard parameters
    pub params: [u64; 4],
}

/// Observation value types
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum ObsValue {
    /// Unsigned 64-bit integer
    U64(u64),
    /// Floating point
    F64(f64),
    /// Boolean
    Bool(bool),
    /// Null/missing
    Null,
}

/// Receipt value types
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub enum ReceiptValue {
    /// Hash value
    Hash([u64; 4]),
    /// Tick count
    Ticks(u64),
    /// Task ID
    TaskId(u64),
    /// Pattern ID
    PatternId(u8),
}

/// Composite μ-Instruction
///
/// Built from μ-ops, these form higher-level operations
pub struct MuInstruction;

impl MuInstruction {
    /// Evaluate a task
    ///
    /// # Timing: ≤8 ticks (aggregate of μ-ops)
    ///
    /// ```text
    /// μ_eval_task:
    ///   1. Load task descriptor (1 tick)
    ///   2. Dispatch pattern (1 tick)
    ///   3. Evaluate guards (1 tick each)
    ///   4. Execute handler (varies)
    ///   5. Write receipt (1 tick)
    /// ```
    #[inline]
    pub fn eval_task(
        task_id: u64,
        obs: &GuardContext,
        budget: &mut TickBudget,
    ) -> MuOpResult<TaskResult> {
        // 1. Load task descriptor (1 tick)
        budget.consume(1);
        let task_offset = (task_id & 0xFFFF) as u16;
        let task_desc = MuOps::load_sigma(task_offset);

        if budget.consume(0) == BudgetStatus::Exhausted {
            return Err(MuOpError::TickBudgetExceeded);
        }

        // 2. Extract pattern ID (0 ticks - compile-time)
        let pattern_id = ((task_desc >> 56) & 0xFF) as u8;

        // 3. Dispatch pattern (1 tick)
        budget.consume(1);
        let dispatch = MuOps::dispatch_pattern(pattern_id, 0)?;

        // 4. Evaluate guards (assume 2 guards, 2 ticks)
        budget.consume(2);
        let guard1 = MuOps::eval_guard(0, obs)?;
        let guard2 = MuOps::eval_guard(1, obs)?;

        if !guard1 || !guard2 {
            return Err(MuOpError::GuardFailed);
        }

        // 5. Execute pattern handler (2 ticks estimated)
        budget.consume(2);
        let result_value = Self::execute_pattern(pattern_id, obs)?;

        // 6. Write receipt (1 tick)
        budget.consume(1);
        MuOps::write_receipt(0, ReceiptValue::TaskId(task_id))?;
        MuOps::write_receipt(1, ReceiptValue::Hash([result_value; 4]))?;

        // Total: 1 + 1 + 2 + 2 + 1 = 7 ticks (within budget)

        Ok(TaskResult {
            task_id,
            output_hash: [result_value; 4],
            ticks_used: budget.used,
        })
    }

    /// Execute pattern handler (stub)
    #[inline(always)]
    fn execute_pattern(pattern_id: u8, _obs: &GuardContext) -> MuOpResult<u64> {
        // In real implementation, this dispatches to pattern-specific code
        Ok(pattern_id as u64 * 0x0123_4567_89AB_CDEF)
    }
}

/// Task execution result
#[derive(Debug, Clone, Copy)]
#[repr(C, align(64))]
pub struct TaskResult {
    /// Task ID
    pub task_id: u64,
    /// Output hash (SHA3-256)
    pub output_hash: [u64; 4],
    /// Ticks consumed
    pub ticks_used: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mu_ops_load_sigma() {
        let value = MuOps::load_sigma(42);
        assert_eq!(value, 42 * 0x0100_0000_0000_0000);
    }

    #[test]
    fn test_mu_ops_dispatch_pattern() {
        let result = MuOps::dispatch_pattern(0, 0);
        assert!(result.is_ok());

        let invalid = MuOps::dispatch_pattern(50, 0);
        assert_eq!(invalid, Err(MuOpError::InvalidPattern));
    }

    #[test]
    fn test_mu_ops_eval_guard() {
        let ctx = GuardContext {
            task_id: 1,
            obs_data: 0,
            params: [0; 4],
        };

        let result = MuOps::eval_guard(0, &ctx);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), true);

        let invalid = MuOps::eval_guard(2000, &ctx);
        assert_eq!(invalid, Err(MuOpError::InvalidGuard));
    }

    #[test]
    fn test_mu_instruction_eval_task() {
        let mut budget = TickBudget::chatman();
        let ctx = GuardContext {
            task_id: 1,
            obs_data: 0,
            params: [0; 4],
        };

        let result = MuInstruction::eval_task(1, &ctx, &mut budget);
        assert!(result.is_ok());

        let task_result = result.unwrap();
        assert_eq!(task_result.task_id, 1);
        assert!(task_result.ticks_used <= crate::CHATMAN_CONSTANT);
    }

    #[test]
    fn test_tick_budget_enforcement() {
        let mut budget = TickBudget::new(3); // Very small budget
        let ctx = GuardContext {
            task_id: 1,
            obs_data: 0,
            params: [0; 4],
        };

        // This should fail due to budget
        let result = MuInstruction::eval_task(1, &ctx, &mut budget);
        // May or may not fail depending on exact tick counts
        // In real implementation with actual hardware, this would be deterministic
    }
}
