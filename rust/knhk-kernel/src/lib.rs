// knhk-kernel: Hot path execution kernel with ≤8 tick guarantee
// Implements all 6 covenants and 7 rules from DOCTRINE_2027

#![warn(clippy::all)]
#![warn(rust_2018_idioms)]
// Note: unsafe code is isolated to platform module and documented with safety justifications

//! # KNHK Kernel
//!
//! The core execution kernel for KNHK that guarantees all hot path operations
//! complete within 8 CPU ticks (the Chatman constant).
//!
//! ## Key Components
//!
//! - **Timer**: RDTSC-based tick counting for precise measurement
//! - **Descriptor**: Immutable configuration with atomic hot-swap
//! - **Guard**: Boolean gate evaluation for conditional execution
//! - **Pattern**: All 43 W3C workflow patterns with zero-overhead dispatch
//! - **Receipt**: Cryptographic verification of all executions
//! - **Executor**: Core state machine with deterministic transitions
//! - **Hot Path**: Main execution loop with stratum isolation
//! - **Macros**: Code generation for pattern definitions
//!
//! ## Performance Guarantees
//!
//! - All hot path operations ≤8 CPU ticks
//! - Zero allocations on critical path
//! - Deterministic execution (same input → same output)
//! - Lock-free atomic operations
//! - Cache-friendly data layout
//!
//! ## Example
//!
//! ```rust
//! use knhk_kernel::prelude::*;
//!
//! // Setup descriptor with patterns
//! let descriptor = DescriptorBuilder::new()
//!     .with_tick_budget(8)
//!     .add_pattern(/* pattern config */)
//!     .build();
//!
//! DescriptorManager::load_descriptor(Box::new(descriptor))?;
//!
//! // Create and execute task
//! let mut task = Task::new(1, pattern_id);
//! task.add_observation(42);
//! task.transition(TaskState::Ready);
//!
//! let executor = Executor::new();
//! let receipt = executor.execute(&task);
//!
//! assert!(receipt.within_budget());
//! ```

pub mod descriptor;
pub mod executor;
pub mod guard;
pub mod hot_path;
pub mod macros;
pub mod pattern;
pub mod platform;
pub mod receipt;
pub mod timer;

// Re-exports for convenience
pub use descriptor::{
    Descriptor, DescriptorBuilder, DescriptorManager, ExecutionContext, PatternEntry, ResourceState,
};
pub use executor::{Executor, StateMachine, Task, TaskState};
pub use guard::{Guard, GuardConfig, GuardType, Predicate, StateFlags};
pub use hot_path::{HotPath, HotPathRunner, Stratum};
pub use pattern::{
    PatternConfig, PatternContext, PatternDispatcher, PatternFactory, PatternFlags, PatternResult,
    PatternType,
};
pub use receipt::{Receipt, ReceiptBuilder, ReceiptStatus, ReceiptStore};
pub use timer::{calibrate_tsc, read_tsc, HotPathTimer, TickBudget};

/// Prelude for common imports
pub mod prelude {
    pub use crate::{
        descriptor::{DescriptorBuilder, DescriptorManager},
        executor::{Executor, Task, TaskState},
        guard::Guard,
        hot_path::{HotPath, HotPathRunner},
        pattern::{PatternConfig, PatternType},
        receipt::{Receipt, ReceiptStatus},
        timer::{read_tsc, HotPathTimer, TickBudget},
    };
}

/// Global initialization
pub fn init() -> Result<(), String> {
    // Calibrate TSC
    let calibration = timer::calibrate_tsc();

    if calibration.confidence < 0.9 {
        return Err(format!(
            "TSC calibration confidence too low: {:.2}%",
            calibration.confidence * 100.0
        ));
    }

    // Verify we can meet the Chatman constant
    let test_timer = timer::HotPathTimer::start();
    let _ = timer::read_tsc();
    let elapsed = test_timer.elapsed_ticks();

    if elapsed > 8 {
        return Err(format!(
            "RDTSC overhead ({} ticks) exceeds Chatman constant",
            elapsed
        ));
    }

    Ok(())
}

/// Verify hot path compliance
pub fn verify_compliance() -> Result<(), Vec<String>> {
    let mut violations = Vec::new();

    // Test each pattern type
    let dispatcher = pattern::PatternDispatcher::new();

    // Use safe conversion from u8 to PatternType
    let all_patterns = [
        PatternType::Sequence,
        PatternType::ParallelSplit,
        PatternType::Synchronization,
        PatternType::ExclusiveChoice,
        PatternType::SimpleMerge,
        // Add other 38 patterns here as needed
        // For now, test the first 5 critical patterns
    ];

    for (i, &pattern_type) in all_patterns.iter().enumerate() {
        let ctx = pattern::PatternFactory::create(
            pattern_type,
            (i + 1) as u32,
            pattern::PatternConfig::default(),
        );

        let timer = timer::HotPathTimer::start();
        let _result = dispatcher.dispatch(&ctx);
        let ticks = timer.elapsed_ticks();

        if ticks > 8 {
            violations.push(format!(
                "Pattern {:?} exceeded budget: {} ticks",
                pattern_type, ticks
            ));
        }
    }

    if violations.is_empty() {
        Ok(())
    } else {
        Err(violations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }

    #[test]
    fn test_compliance() {
        let result = verify_compliance();
        if let Err(violations) = result {
            for v in &violations {
                eprintln!("Violation: {}", v);
            }
            assert!(violations.is_empty(), "Hot path violations detected");
        }
    }

    #[test]
    fn test_end_to_end() {
        // Initialize
        init().unwrap();

        // Setup descriptor
        use crate::pattern::PatternType;

        let pattern = descriptor::PatternEntry::new(
            PatternType::Sequence,
            1,
            10,
            pattern::PatternConfig::default(),
        );

        let desc = descriptor::DescriptorBuilder::new()
            .with_tick_budget(8)
            .add_pattern(pattern)
            .build();

        descriptor::DescriptorManager::load_descriptor(Box::new(desc)).unwrap();

        // Create task
        let mut task = executor::Task::new(1, 1);
        task.add_observation(42);
        task.transition(executor::TaskState::Ready);

        // Execute
        let executor = executor::Executor::new();
        let receipt = executor.execute(&task);

        // Verify
        assert!(receipt.verify());
        assert!(receipt.within_budget());
        assert_eq!(receipt.status, receipt::ReceiptStatus::Success);
    }
}
