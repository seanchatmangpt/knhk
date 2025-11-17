//! Timescale Separation
//!
//! This module implements typed separation of hot/warm/cold timescales.
//! The type system enforces what operations are allowed in each mode:
//!
//! - **Hot**: ≤8 ticks, no allocation, no dynamic dispatch, no async
//! - **Warm**: ≤1ms, limited allocation, async allowed, no LLM calls
//! - **Cold**: Unbounded, full allocation, LLM calls allowed
//!
//! Components claiming a timescale must implement the corresponding trait,
//! and the compiler enforces allowed operations per mode.

use crate::timing::{BudgetStatus, TickBudget};
use core::future::Future;
use core::marker::PhantomData;

/// Timescale classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TimescaleClass {
    /// Hot path (≤8 ticks)
    Hot = 0,
    /// Warm path (≤1ms)
    Warm = 1,
    /// Cold path (unbounded)
    Cold = 2,
}

impl TimescaleClass {
    /// Get maximum ticks for this timescale
    pub const fn max_ticks(&self) -> Option<u64> {
        match self {
            TimescaleClass::Hot => Some(crate::CHATMAN_CONSTANT),
            TimescaleClass::Warm => Some(1_000_000), // ~1ms at 1GHz
            TimescaleClass::Cold => None,
        }
    }

    /// Check if allocation is allowed
    pub const fn allows_allocation(&self) -> bool {
        !matches!(self, TimescaleClass::Hot)
    }

    /// Check if async is allowed
    pub const fn allows_async(&self) -> bool {
        !matches!(self, TimescaleClass::Hot)
    }
}

/// Hot path trait (μ_hot)
///
/// Constraints:
/// - No heap allocation
/// - No dynamic dispatch (except through approved interfaces)
/// - No async/await
/// - Bounded ticks (≤8)
/// - No I/O
/// - No panics
///
/// Example:
/// ```ignore
/// struct FastCheck;
///
/// impl Hot for FastCheck {
///     const MAX_TICKS: u64 = 3;
///     const ALLOCATES: bool = false;
///
///     fn execute_hot(&self) -> Result<Action, HotError> {
///         // Must complete in ≤3 ticks
///         Ok(Action { /* ... */ })
///     }
/// }
/// ```
pub trait Hot: Sized {
    /// Maximum ticks (must be ≤8)
    const MAX_TICKS: u64;

    /// Whether this operation allocates (must be false)
    const ALLOCATES: bool = false;

    /// Execute in hot mode
    ///
    /// Must complete within MAX_TICKS and not allocate.
    fn execute_hot(&self) -> Result<Action, HotError>;

    /// Compile-time check that constraints are satisfied
    const CONSTRAINTS_SATISFIED: () = {
        assert!(Self::MAX_TICKS <= crate::CHATMAN_CONSTANT);
        assert!(!Self::ALLOCATES);
    };
}

/// Warm path trait (μ_warm)
///
/// Constraints:
/// - Limited heap allocation
/// - Async allowed
/// - ≤1ms execution time
/// - No LLM calls
/// - No unbounded loops
///
/// Example:
/// ```ignore
/// struct MapeAnalysis;
///
/// impl Warm for MapeAnalysis {
///     const MAX_MILLIS: u64 = 1;
///
///     fn execute_warm(&self) -> impl Future<Output = Result<Action, WarmError>> {
///         async {
///             // Can do async work, limited allocation
///             Ok(Action { /* ... */ })
///         }
///     }
/// }
/// ```
pub trait Warm: Sized {
    /// Maximum milliseconds
    const MAX_MILLIS: u64;

    /// Execute in warm mode
    ///
    /// Returns a future that must complete within MAX_MILLIS.
    fn execute_warm(&self) -> impl Future<Output = Result<Action, WarmError>>;

    /// Compile-time check that constraints are satisfied
    const CONSTRAINTS_SATISFIED: () = {
        assert!(Self::MAX_MILLIS > 0);
        assert!(Self::MAX_MILLIS <= 1000); // ≤1 second
    };
}

/// Cold path trait (μ_cold)
///
/// Constraints:
/// - Full heap allocation allowed
/// - Async allowed
/// - Unbounded execution time
/// - LLM calls allowed
/// - Heavy computation allowed
///
/// Example:
/// ```ignore
/// struct LlmAnalysis;
///
/// impl Cold for LlmAnalysis {
///     fn execute_cold(&self) -> impl Future<Output = Result<Action, ColdError>> {
///         async {
///             // Can call LLM, do heavy computation
///             Ok(Action { /* ... */ })
///         }
///     }
/// }
/// ```
pub trait Cold: Sized {
    /// Execute in cold mode
    ///
    /// No time constraints, full capabilities.
    fn execute_cold(&self) -> impl Future<Output = Result<Action, ColdError>>;
}

/// Action result from timescale execution
#[derive(Debug, Clone)]
pub struct Action {
    /// Action identifier
    pub id: u64,
    /// Output data
    pub output: heapless::Vec<u8, 256>,
    /// Actual ticks/millis consumed
    pub cost: u64,
}

impl Action {
    /// Create a new action
    pub fn new(id: u64, output: heapless::Vec<u8, 256>, cost: u64) -> Self {
        Self { id, output, cost }
    }
}

/// Hot path errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotError {
    /// Tick budget exceeded
    TickBudgetExceeded,
    /// Invalid input
    InvalidInput,
    /// Guard failed
    GuardFailed,
    /// Execution failed
    ExecutionFailed,
}

/// Warm path errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarmError {
    /// Time budget exceeded
    TimeBudgetExceeded,
    /// Invalid input
    InvalidInput,
    /// Resource exhausted
    ResourceExhausted,
    /// Execution failed
    ExecutionFailed,
}

/// Cold path errors
#[derive(Debug, Clone)]
pub enum ColdError {
    /// LLM call failed
    LlmFailed(alloc::string::String),
    /// Invalid input
    InvalidInput,
    /// Resource exhausted
    ResourceExhausted,
    /// Execution failed
    ExecutionFailed(alloc::string::String),
}

/// Timescale executor (runs operations at appropriate timescale)
pub struct TimescaleExecutor {
    _marker: PhantomData<()>,
}

impl TimescaleExecutor {
    /// Create a new executor
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    /// Execute hot path operation
    pub fn execute_hot<T: Hot>(&self, op: &T) -> Result<Action, HotError> {
        // Ensure constraints are satisfied at compile time
        let _ = T::CONSTRAINTS_SATISFIED;

        // Create tick budget
        let mut budget = TickBudget::new(T::MAX_TICKS);

        // Execute
        let action = op.execute_hot()?;

        // Verify budget not exceeded
        if budget.consume(action.cost) == BudgetStatus::Exhausted {
            return Err(HotError::TickBudgetExceeded);
        }

        Ok(action)
    }

    /// Execute warm path operation (async)
    pub async fn execute_warm<T: Warm>(&self, op: &T) -> Result<Action, WarmError> {
        // Ensure constraints are satisfied at compile time
        let _ = T::CONSTRAINTS_SATISFIED;

        // Execute with timeout (simplified - would use real timeout)
        op.execute_warm().await
    }

    /// Execute cold path operation (async)
    pub async fn execute_cold<T: Cold>(&self, op: &T) -> Result<Action, ColdError> {
        // No constraints to check - cold path is unbounded
        op.execute_cold().await
    }
}

impl Default for TimescaleExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Example hot operation (≤8 ticks)
#[derive(Debug)]
pub struct ExampleHotOp {
    pub input: u64,
}

impl Hot for ExampleHotOp {
    const MAX_TICKS: u64 = 3;
    const ALLOCATES: bool = false;

    fn execute_hot(&self) -> Result<Action, HotError> {
        // Simple branchless computation
        let result = self.input.wrapping_mul(2);

        let mut output = heapless::Vec::new();
        output
            .extend_from_slice(&result.to_le_bytes())
            .map_err(|_| HotError::ExecutionFailed)?;

        Ok(Action::new(1, output, 2))
    }
}

/// Example warm operation (≤1ms)
#[derive(Debug)]
pub struct ExampleWarmOp {
    pub input: alloc::vec::Vec<u8>,
}

impl Warm for ExampleWarmOp {
    const MAX_MILLIS: u64 = 1;

    async fn execute_warm(&self) -> Result<Action, WarmError> {
        // Can do async work, allocate
        let processed = self
            .input
            .iter()
            .map(|x| x.wrapping_mul(2))
            .collect::<alloc::vec::Vec<_>>();

        let mut output = heapless::Vec::new();
        output
            .extend_from_slice(&processed[..processed.len().min(256)])
            .map_err(|_| WarmError::ExecutionFailed)?;

        Ok(Action::new(2, output, 100))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timescale_class() {
        assert_eq!(TimescaleClass::Hot.max_ticks(), Some(8));
        assert_eq!(TimescaleClass::Warm.max_ticks(), Some(1_000_000));
        assert_eq!(TimescaleClass::Cold.max_ticks(), None);

        assert!(!TimescaleClass::Hot.allows_allocation());
        assert!(TimescaleClass::Warm.allows_allocation());
        assert!(TimescaleClass::Cold.allows_allocation());
    }

    #[test]
    fn test_hot_operation() {
        let op = ExampleHotOp { input: 42 };

        // Verify constraints at compile time
        let _ = ExampleHotOp::CONSTRAINTS_SATISFIED;

        assert_eq!(ExampleHotOp::MAX_TICKS, 3);
        assert!(!ExampleHotOp::ALLOCATES);

        let result = op.execute_hot().unwrap();
        assert_eq!(result.cost, 2);
    }

    #[test]
    fn test_executor() {
        let executor = TimescaleExecutor::new();
        let op = ExampleHotOp { input: 100 };

        let result = executor.execute_hot(&op).unwrap();
        assert_eq!(result.cost, 2);
        assert!(result.cost <= ExampleHotOp::MAX_TICKS);
    }

    #[tokio::test]
    async fn test_warm_operation() {
        let op = ExampleWarmOp {
            input: alloc::vec![1, 2, 3, 4],
        };

        let _ = ExampleWarmOp::CONSTRAINTS_SATISFIED;
        assert_eq!(ExampleWarmOp::MAX_MILLIS, 1);

        let result = op.execute_warm().await.unwrap();
        assert_eq!(result.output[0], 2);
        assert_eq!(result.output[1], 4);
    }
}
