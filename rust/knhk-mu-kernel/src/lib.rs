//! μ-Kernel for Knowledge Operations
//!
//! This is not an application. This is an ISA (Instruction Set Architecture)
//! for knowledge operations, implemented in Rust as a metalanguage.
//!
//! # Core Principles
//!
//! 1. **A = μ(O)** - Actions are deterministic projections of observations
//! 2. **τ ≤ 8** - Hot path completes in ≤8 CPU cycles (Chatman Constant)
//! 3. **Σ ⊨ Q** - Ontology respects invariants (enforced at compile and runtime)
//! 4. **μ ∘ μ = μ** - Idempotent execution via immutable Σ*
//! 5. **hash(A) = hash(μ(O))** - Cryptographic provenance via receipts
//!
//! # Architecture
//!
//! ```text
//! μ-kernel:
//!   μ_hot   : ≤8 cycles, no allocation, no branches, no I/O
//!   μ_warm  : ≤1ms, can allocate, async allowed
//!   μ_cold  : unbounded, LLM calls, analytics
//!
//! State:
//!   Σ* : Compiled ontology snapshot (immutable)
//!   O  : Observations (input stream)
//!   R  : Receipts (proof chain)
//!   τ  : Tick counter (cycle-accurate)
//!   ρ  : Resource budget
//!
//! Operations:
//!   μ_eval_task  : Execute workflow task
//!   μ_dispatch   : Pattern dispatch
//!   μ_guard      : Invariant check
//!   μ_receipt    : Generate proof
//! ```

#![no_std]
#![cfg_attr(not(test), no_main)]
#![forbid(unsafe_code)]  // Except in verified modules
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![warn(missing_docs)]
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![feature(asm_const)]
#![feature(core_intrinsics)]
#![feature(negative_impls)]

extern crate alloc;

// Core modules (μ_hot - no allocation, ≤8 cycles)
pub mod core;
pub mod isa;
pub mod sigma;
pub mod guards;
pub mod guards_simd;
pub mod patterns;
pub mod receipts;
pub mod timing;
pub mod timing_const;

// Warm modules (μ_warm - ≤1ms, can allocate)
pub mod overlay;
pub mod overlay_types;
pub mod overlay_proof;
pub mod overlay_safety;
pub mod overlay_compiler;
pub mod compiler;
pub mod compiler_proof;
pub mod sigma_ir;
pub mod sigma_types;
pub mod mape;
pub mod concurrency;
#[cfg(any(feature = "concurrent-structures", test))]
pub mod concurrent;
pub mod proofs;
pub mod protocols;

// Cold modules (μ_cold - unbounded)
pub mod analytics;
pub mod llm_interface;

// AHI layer (Anticipatory Hybrid Intelligence - user space)
pub mod ahi;

// Constitutional traits (cross-layer guarantees)
pub mod constitutional;

// Verification
#[cfg(feature = "verification")]
pub mod verification;

// Re-exports
pub use core::{MuKernel, MuState, MuResult, MuError};
pub use isa::{MuOps, MuInstruction};
pub use sigma::{SigmaCompiled, SigmaHash};
pub use guards::{GuardContext, GuardResult};
pub use guards_simd::{SimdGuardBatch, SimdGuardEvaluator, evaluate_guards_batch, GuardBitmap};
pub use patterns::{PatternId};
pub use receipts::{Receipt, ReceiptChain};
pub use timing::{TickCounter, TickBudget};
pub use timing_const::{
    ConstTickCost, SequencePattern, ParallelSplitPattern, SynchronizationPattern,
    total_tick_cost, within_chatman, compute_task_wcet,
};
pub use overlay::{
    DeltaSigma, OverlayAlgebra, ProofCarryingOverlay,
    ProofAlgebra, KernelPromotion, RolloutStrategy, PromoteError,
};
pub use overlay_types::{
    OverlayValue, OverlayChanges, OverlayChange, OverlayMetadata,
    OverlayError, SnapshotId, PerfImpact,
};
pub use overlay_proof::{
    OverlayProof, ComposedProof, ProofStrength, ProofMethod,
};
pub use overlay_safety::{
    SafeProof, HotSafe, WarmSafe, ColdUnsafe, SafetyPromotion,
};
pub use mape::{MapeKColon, MonitorOp, AnalyzeOp, PlanOp, ExecuteOp};
pub use compiler_proof::{CompilationCertificate, CertifiedSigma, ProofBuilder};
pub use sigma_ir::{SigmaIR, validation};
pub use sigma_types::{CompiledTask, CompiledPattern, CompiledGuard, WithinChatmanConstant};
pub use concurrency::{
    CoreLocal, Shared, GuardSet,
    WorkQueue, GlobalOrdered, BestEffort,
    DeterministicScheduler, SchedulableTask, Priority, PriorityHigh, PriorityNormal, PriorityLow,
    LogicalClock, Timestamp, HappensBefore,
    ReplayLog, Deterministic as ReplayDeterministic,
};

// Lock-free concurrent data structures (Phase 10)
#[cfg(feature = "concurrent-structures")]
pub use concurrent::{
    LockFreeSkipList, HazardGuard,
    ConcurrentHAMT,
    TreiberStack, MichaelScottQueue,
    Guard, Atomic,
    AtomicArc, WeakArc, AtomicArcCell,
};

// AHI re-exports
pub use ahi::{
    Decision, ObservationSlice, InvariantId, RiskClass,
    AhiContext, AhiProvenOverlay, AhiOverlayProof, SubmitToken,
    Hot, Warm, Cold, TimescaleClass,
};

// Constitutional re-exports
pub use constitutional::{
    DoctrineAligned, ChatmanBounded, ClosedWorld, Deterministic, Constitutional,
    Doctrine, ConstitutionalReceipt,
};

// Proof re-exports
pub use proofs::{
    Proven, Witness, Proof, Predicate,
    NonZero, ChatmanCompliant, Sorted, PowerOfTwo,
    Bounded, ConstNonZero, Aligned,
    ConstRange, ChatmanProof, PowerOfTwoProof,
    IsWithinChatman, IsPowerOfTwo, IsSorted,
    ProvenSorted, ProvenUnique, ProvenNonEmpty, ProvenChatmanBounded,
    ProofExt, ProofResult, ProofError,
    ProofChain, ProofValidator,
};
pub use proofs::combinators::ProofBuilder as ProofsBuilder;

// Protocol re-exports
pub use protocols::{
    Session, SessionProtocol, StateMachine, MapeKCycle, OverlayPipeline,
    MonitorPhase, AnalyzePhase, PlanPhase, ExecutePhase, KnowledgePhase,
    ShadowPhase, TestPhase, ValidatePhase, PromotePhase,
};

/// The Chatman Constant - maximum ticks for hot path
pub const CHATMAN_CONSTANT: u64 = 8;

/// μ-kernel version (semantic versioning for ISA)
pub const MU_KERNEL_VERSION: (u8, u8, u8) = (2027, 0, 0);

/// Memory layout constants
pub mod memory {
    /// Σ* descriptor base address
    pub const SIGMA_BASE: usize = 0x0000_0000_0000;
    /// Σ* descriptor size (256MB)
    pub const SIGMA_SIZE: usize = 0x0000_1000_0000;

    /// Pattern dispatch table base
    pub const PATTERN_BASE: usize = 0x0000_1000_0000;
    /// Pattern dispatch table size (256MB)
    pub const PATTERN_SIZE: usize = 0x0000_1000_0000;

    /// Guard evaluators base
    pub const GUARD_BASE: usize = 0x0000_2000_0000;
    /// Guard evaluators size (256MB)
    pub const GUARD_SIZE: usize = 0x0000_1000_0000;

    /// O_in buffer base
    pub const OBS_BASE: usize = 0x0000_3000_0000;
    /// O_in buffer size (64KB)
    pub const OBS_SIZE: usize = 0x0000_0001_0000;

    /// Receipt accumulator base
    pub const RECEIPT_BASE: usize = 0x0000_3001_0000;
    /// Receipt accumulator size (64KB)
    pub const RECEIPT_SIZE: usize = 0x0000_0001_0000;

    /// μ_warm space base
    pub const WARM_BASE: usize = 0x0000_4000_0000;
    /// μ_cold space base
    pub const COLD_BASE: usize = 0x8000_0000_0000;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chatman_constant() {
        assert_eq!(CHATMAN_CONSTANT, 8);
    }

    #[test]
    fn test_memory_layout_non_overlapping() {
        use memory::*;

        assert!(SIGMA_BASE + SIGMA_SIZE <= PATTERN_BASE);
        assert!(PATTERN_BASE + PATTERN_SIZE <= GUARD_BASE);
        assert!(GUARD_BASE + GUARD_SIZE <= OBS_BASE);
        assert!(OBS_BASE + OBS_SIZE <= RECEIPT_BASE);
        assert!(RECEIPT_BASE + RECEIPT_SIZE <= WARM_BASE);
    }
}
