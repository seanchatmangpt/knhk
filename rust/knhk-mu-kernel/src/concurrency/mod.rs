//! Deterministic Multi-Core Concurrency for μ-Kernel
//!
//! This module implements a deterministic, multi-core scheduler where concurrency
//! is a property expressed and constrained by Rust types.
//!
//! # Core Principles
//!
//! 1. **Type-Safe Concurrency**: CoreLocal<T> vs Shared<T> enforced at compile time
//! 2. **Deterministic Scheduling**: Same inputs → same outputs → same receipts
//! 3. **Lock-Free Design**: Per-core queues, no blocking operations
//! 4. **Logical Time**: Lamport clocks for total ordering
//! 5. **Replay Capability**: Cross-machine reproducibility
//!
//! # Architecture
//!
//! ```text
//! Scheduler:
//!   CoreLocal<WorkQueue>[CORES]  - Per-core lock-free queues
//!   GlobalOrdered<Decision>      - Totally ordered global queue
//!   LogicalClock                 - Lamport timestamps
//!   ReplayLog                    - Deterministic event log
//!
//! Task Types:
//!   SchedulableTask<B, P, G>     - Resource contracts in types
//!     B: TickBudget trait        - Compile-time tick limits
//!     P: Priority trait          - Scheduling priority
//!     G: GuardSet trait          - Required guards
//!
//! Execution:
//!   1. Enqueue task with typed constraints
//!   2. Schedule via logical timestamps
//!   3. Execute on core-local queue
//!   4. Record to replay log
//!   5. Emit deterministic receipt
//! ```
//!
//! # Usage
//!
//! ```rust,no_run
//! use knhk_mu_kernel::concurrency::*;
//!
//! // Create deterministic scheduler (4 cores)
//! let scheduler = DeterministicScheduler::<4>::new();
//!
//! // Create task with typed constraints
//! let task = SchedulableTask::new(
//!     task_id,
//!     TickBudget::chatman(),    // ≤8 ticks
//!     Priority::high(),          // High priority
//!     guards,                    // Required guards
//!     work,
//! );
//!
//! // Schedule deterministically
//! scheduler.enqueue(task)?;
//!
//! // Execute (deterministic)
//! let result = scheduler.run_cycle()?;
//!
//! // Replay from seed
//! let replay = scheduler.replay(seed);
//! for event in replay {
//!     assert_eq!(event, original_event);
//! }
//! ```

#![allow(unsafe_code)] // Verified unsafe for lock-free structures

pub mod logical_time;
pub mod queues;
pub mod replay;
pub mod scheduler;
pub mod types;

// Re-exports
pub use logical_time::{HappensBefore, LogicalClock, Timestamp};
pub use queues::{BestEffort, GlobalOrdered, QueueError, WorkQueue};
pub use replay::{Deterministic, ReplayIterator, ReplayLog, ReplaySeed};
pub use scheduler::{
    DeterministicScheduler, ExecutionResult, Priority, PriorityHigh, PriorityLow, PriorityNormal,
    SchedulableTask, SchedulerError,
};
pub use types::{CoreLocal, GuardSet, NotSend, NotSync, Shared};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_structure() {
        // Ensure module compiles and types are accessible
        let _clock = LogicalClock::new();
    }
}
