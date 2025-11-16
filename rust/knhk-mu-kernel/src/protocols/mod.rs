//! Type-Level Protocol State Machines
//!
//! This module provides compile-time protocol enforcement through the type system.
//! It implements the typestate pattern to ensure protocol correctness at compile time.
//!
//! # Overview
//!
//! The protocols module contains:
//! - **Session Types** - Linear session types for protocol validation
//! - **State Machines** - Generic type-level state machines
//! - **MAPE-K Protocol** - Autonomic control loop enforcement
//! - **Overlay Protocol** - Overlay promotion pipeline enforcement
//!
//! All protocol violations are caught at compile time with zero runtime overhead.
//!
//! # Key Principles
//!
//! 1. **Type-Level State Tracking** - Current state encoded in type parameter
//! 2. **Linear Types** - States consumed by transitions (use once)
//! 3. **Zero Runtime Cost** - All types are zero-sized markers
//! 4. **Compile-Time Validation** - Invalid transitions are type errors
//!
//! # Examples
//!
//! ## MAPE-K Protocol
//!
//! ```no_run
//! use knhk_mu_kernel::protocols::mape_protocol::*;
//!
//! // Create MAPE-K cycle - must follow exact order
//! let cycle = MapeKCycle::new();
//! let cycle = cycle.monitor(receipt);
//! let cycle = cycle.analyze();
//! let cycle = cycle.plan();
//! let cycle = cycle.execute();
//! let cycle = cycle.update_knowledge();
//!
//! // This would not compile:
//! // cycle.plan(); // ERROR: no method `plan` on MonitorPhase
//! ```
//!
//! ## Overlay Promotion Protocol
//!
//! ```no_run
//! use knhk_mu_kernel::protocols::overlay_protocol::*;
//!
//! // Create promotion pipeline - enforces Shadow → Test → Validate → Promote
//! let pipeline = OverlayPipeline::new(overlay);
//! let pipeline = pipeline.deploy_shadow()?;
//! let pipeline = pipeline.run_tests()?;
//! let pipeline = pipeline.validate()?;
//! let result = pipeline.promote()?;
//!
//! // Cannot skip validation:
//! // pipeline.promote(); // ERROR: no method `promote` on ShadowPhase
//! ```
//!
//! ## Generic State Machine
//!
//! ```no_run
//! use knhk_mu_kernel::protocols::state_machine::*;
//!
//! let machine = StateMachine::<Initial>::new();
//! let machine = machine.start();
//! let machine = machine.pause();
//! let machine = machine.resume();
//! let machine = machine.stop();
//! ```
//!
//! # Protocol Invariants
//!
//! All protocols enforce their invariants at compile time:
//!
//! ## MAPE-K Invariants
//! - Must start at Monitor phase
//! - Cannot skip phases (M → A → P → E → K)
//! - Cannot repeat phase without completing cycle
//! - Must cycle through all phases to return to Monitor
//!
//! ## Overlay Promotion Invariants
//! - Must deploy to shadow before testing
//! - Cannot promote without validation
//! - Cannot skip testing phase
//! - Rollback is always available
//! - Canary deployments follow strict percentage rules
//!
//! ## State Machine Invariants
//! - States are zero-sized (no runtime cost)
//! - Transitions consume old state (linear types)
//! - Invalid transitions are type errors
//! - Terminal states have no transitions
//!
//! # Zero-Cost Abstractions
//!
//! All protocol types are zero-sized:
//! ```
//! assert_eq!(size_of::<StateMachine<Initial>>(), 0);
//! assert_eq!(size_of::<MapeKCycle<MonitorPhase>>(), 0);
//! assert_eq!(size_of::<Session<Uninitialized>>(), 0);
//! ```
//!
//! The type system provides all the safety with zero runtime overhead.

pub mod session_types;
pub mod state_machine;
pub mod mape_protocol;
pub mod overlay_protocol;

// Re-exports for convenience

/// Session type exports
pub use session_types::{
    Session, SessionProtocol, Linear,
    Uninitialized, Initialized, Active, Completed, Failed,
    Choice, Sequence, Recursive,
    ProtocolValidation, Capability,
    Read, Write, Execute,
    ReadOnly, ReadWrite,
    Composed, Dual,
    Send, Recv, Channel,
    Indexed, Z, S,
};

/// State machine exports
pub use state_machine::{
    StateMachine, StatefulMachine, Builder,
    Initial, Running, Paused, Stopped, Error,
    ConditionalTransition, Parallel,
    Guarded, TimedMachine,
};

/// MAPE-K protocol exports
pub use mape_protocol::{
    MapeKCycle, MapeKCycleWithData, MapeKData,
    MonitorPhase, AnalyzePhase, PlanPhase, ExecutePhase, KnowledgePhase,
    MapeKBranch, TimedMapeK, CycleCounter,
};

/// Overlay protocol exports
pub use overlay_protocol::{
    OverlayPipeline, OverlayPipelineWithData,
    ShadowPhase, TestPhase, ValidatePhase, PromotePhase,
    PromotedPhase, RolledBackPhase,
    TestResults, PerfMetrics,
    RollbackProtocol, RollbackReason,
    CanaryDeployment, CanaryInitial, CanaryRollingOut, CanaryComplete,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_types_zero_sized() {
        // Session types
        assert_eq!(core::mem::size_of::<Session<Uninitialized>>(), 0);
        assert_eq!(core::mem::size_of::<Session<Initialized>>(), 0);
        assert_eq!(core::mem::size_of::<Session<Active>>(), 0);

        // State machines
        assert_eq!(core::mem::size_of::<StateMachine<Initial>>(), 0);
        assert_eq!(core::mem::size_of::<StateMachine<Running>>(), 0);
        assert_eq!(core::mem::size_of::<StateMachine<Stopped>>(), 0);

        // MAPE-K
        assert_eq!(core::mem::size_of::<MapeKCycle<MonitorPhase>>(), 0);
        assert_eq!(core::mem::size_of::<MapeKCycle<AnalyzePhase>>(), 0);
        assert_eq!(core::mem::size_of::<MapeKCycle<ExecutePhase>>(), 0);
    }

    #[test]
    fn test_protocol_composition() {
        // Can compose different protocols
        let _composed: Composed<Session<Uninitialized>, StateMachine<Initial>> =
            Composed::new();
    }
}
