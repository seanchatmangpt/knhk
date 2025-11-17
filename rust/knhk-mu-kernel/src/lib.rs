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
#![forbid(unsafe_code)] // Except in verified modules
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
pub mod guards;
pub mod guards_simd;
pub mod isa;
pub mod patterns;
pub mod receipts;
pub mod sigma;
pub mod timing;
pub mod timing_const;

// Warm modules (μ_warm - ≤1ms, can allocate)
pub mod compiler;
pub mod compiler_proof;
pub mod concurrency;
#[cfg(any(feature = "concurrent-structures", test))]
pub mod concurrent;
pub mod mape;
pub mod overlay;
pub mod overlay_compiler;
pub mod overlay_proof;
pub mod overlay_safety;
pub mod overlay_types;
pub mod proofs;
pub mod protocols;
pub mod sigma_ir;
pub mod sigma_types;

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
pub use compiler_proof::{CertifiedSigma, CompilationCertificate, ProofBuilder};
pub use concurrency::{
    BestEffort, CoreLocal, Deterministic as ReplayDeterministic, DeterministicScheduler,
    GlobalOrdered, GuardSet, HappensBefore, LogicalClock, Priority, PriorityHigh, PriorityLow,
    PriorityNormal, ReplayLog, SchedulableTask, Shared, Timestamp, WorkQueue,
};
pub use core::{MuError, MuKernel, MuResult, MuState};
pub use guards::{GuardContext, GuardResult};
pub use guards_simd::{evaluate_guards_batch, GuardBitmap, SimdGuardBatch, SimdGuardEvaluator};
pub use isa::{MuInstruction, MuOps};
pub use mape::{AnalyzeOp, ExecuteOp, MapeKColon, MonitorOp, PlanOp};
pub use overlay::{
    DeltaSigma, KernelPromotion, OverlayAlgebra, PromoteError, ProofAlgebra, ProofCarryingOverlay,
    RolloutStrategy,
};
pub use overlay_proof::{ComposedProof, OverlayProof, ProofMethod, ProofStrength};
pub use overlay_safety::{ColdUnsafe, HotSafe, SafeProof, SafetyPromotion, WarmSafe};
pub use overlay_types::{
    OverlayChange, OverlayChanges, OverlayError, OverlayMetadata, OverlayValue, PerfImpact,
    SnapshotId,
};
pub use patterns::PatternId;
pub use receipts::{Receipt, ReceiptChain};
pub use sigma::{SigmaCompiled, SigmaHash};
pub use sigma_ir::{validation, SigmaIR};
pub use sigma_types::{CompiledGuard, CompiledPattern, CompiledTask, WithinChatmanConstant};
pub use timing::{TickBudget, TickCounter};
pub use timing_const::{
    compute_task_wcet, total_tick_cost, within_chatman, ConstTickCost, ParallelSplitPattern,
    SequencePattern, SynchronizationPattern,
};

// Lock-free concurrent data structures (Phase 10)
#[cfg(feature = "concurrent-structures")]
pub use concurrent::{
    Atomic, AtomicArc, AtomicArcCell, ConcurrentHAMT, Guard, HazardGuard, LockFreeSkipList,
    MichaelScottQueue, TreiberStack, WeakArc,
};

// AHI re-exports
pub use ahi::{
    AhiContext, AhiOverlayProof, AhiProvenOverlay, Cold, Decision, Hot, InvariantId,
    ObservationSlice, RiskClass, SubmitToken, TimescaleClass, Warm,
};

// Constitutional re-exports
pub use constitutional::{
    ChatmanBounded, ClosedWorld, Constitutional, ConstitutionalReceipt, Deterministic, Doctrine,
    DoctrineAligned,
};

// Proof re-exports
pub use proofs::combinators::ProofBuilder as ProofsBuilder;
pub use proofs::{
    Aligned, Bounded, ChatmanCompliant, ChatmanProof, ConstNonZero, ConstRange, IsPowerOfTwo,
    IsSorted, IsWithinChatman, NonZero, PowerOfTwo, PowerOfTwoProof, Predicate, Proof, ProofChain,
    ProofError, ProofExt, ProofResult, ProofValidator, Proven, ProvenChatmanBounded,
    ProvenNonEmpty, ProvenSorted, ProvenUnique, Sorted, Witness,
};

// Protocol re-exports
pub use protocols::{
    AnalyzePhase, ExecutePhase, KnowledgePhase, MapeKCycle, MonitorPhase, OverlayPipeline,
    PlanPhase, PromotePhase, Session, SessionProtocol, ShadowPhase, StateMachine, TestPhase,
    ValidatePhase,
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
