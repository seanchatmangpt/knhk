//! Workflow Innovation Features
//!
//! Advanced features that compose existing packages to create new capabilities.
//! These features demonstrate the power of the self-executing workflow system.
//!
//! # Features
//!
//! ## Production Features
//! - **Workflow Templates**: Pre-built patterns (ETL, Saga, Fan-out/Fan-in, etc.)
//! - **Adaptive Optimizer**: ML-driven workflow optimization using MAPE-K + receipts
//! - **A/B Testing**: Shadow execution for safe testing
//! - **Analytics Engine**: Query and analyze Γ(O) history
//! - **Deterministic Execution**: Delta-based deterministic replay
//! - **Formal Verification**: Mathematical proofs of workflow properties
//! - **Hardware Acceleration**: GPU/SIMD acceleration for workflows
//! - **Zero-Copy Execution**: Memory-efficient workflow execution
//! - **Experimental Features**: Cutting-edge workflow capabilities
//!
//! ## Hyper-Advanced Features (Type System Mastery)
//! - **Type-State Machine**: Compile-time workflow state validation with phantom types
//! - **Lock-Free Queue**: High-performance concurrent receipt queue with epoch-based GC
//! - **SIMD Hash Verification**: Hardware-accelerated cryptographic operations
//! - **GAT Query Engine**: Zero-cost query abstraction with Generic Associated Types
//! - **Arena Allocator**: Custom bump allocator for workflow contexts
//! - **Const Evaluation**: Compile-time workflow validation and optimization
//!
//! ## Microkernel Features (AHI Constitution)
//! - **Verified Kernel**: Restricted Rust subset with total functions, no panics, Chatman enforcement
//! - **Refinement Guards**: Guards as types, doctrine constraints via phantom parameters
//! - **Cluster Types**: Distributed determinism with role-based access, quorum at compile time
//! - **Auto-Specialization**: Hardware-adaptive kernels selected under doctrine control
//! - **Linear Resources**: Resource quotas and priorities as linear/indexed types
//!
//! ## 2027 Innovations (Hyper-Advanced Rust)
//! - **Effect System**: Track side effects at type level with algebraic effect handlers
//! - **SIMD Kernels**: Real AVX2/AVX-512/NEON implementations for 4-16x speedup
//! - **Custom Allocators**: Arena, bump, pool, and stack allocators for zero-allocation execution
//! - **Async Kernel**: Non-blocking type-state machines with futures-based execution

// Production features
pub mod workflow_templates;
pub mod adaptive_optimizer;
pub mod ab_testing;
pub mod analytics;
pub mod deterministic;
pub mod experiment;
pub mod formal;
pub mod hardware;
pub mod zero_copy;

// Hyper-advanced features
pub mod type_state;
pub mod lockfree;
pub mod simd_hash;
pub mod gat_query;
pub mod arena;
pub mod const_eval;

// Microkernel features (AHI Constitution)
pub mod verified_kernel;
pub mod refinement_guards;
pub mod cluster_types;
pub mod auto_specialize;
pub mod linear_resources;

// 2027 Innovations (Hyper-Advanced Rust)
pub mod effect_system;
pub mod simd_kernels;
pub mod custom_allocators;
pub mod async_kernel;

// Production feature exports
pub use workflow_templates::{WorkflowTemplate, TemplateLibrary, TemplateMetadata};
pub use adaptive_optimizer::{
    AdaptiveOptimizer, OptimizationStrategy, OptimizationRecommendation,
    OptimizationAction, PerformanceMetrics,
};
pub use ab_testing::{
    ABTestOrchestrator, ABTestConfig, ABTestResults, TestVariant,
    TestRecommendation, ShadowTestResult,
};
pub use analytics::{
    AnalyticsEngine, QueryBuilder, DataPoint, GuardFailureAnalysis,
    SnapshotComparison, ExecutiveSummary,
};
pub use deterministic::{DeterministicExecutor, DeterministicContext, DeltaLogEntry, ExecutionStep};
pub use experiment::*;
pub use formal::{FormalVerifier, Property, VerificationResult, Violation};
pub use hardware::{HardwareAccelerator, HardwareAcceleration};
pub use zero_copy::{ZeroCopyTriple, ZeroCopyTripleBatch, ZeroCopyStr, ZeroCopyBytes};

// Hyper-advanced feature exports
pub use type_state::{
    TypedWorkflow, Uninitialized, Configured, Validated, Executing, Completed, Failed,
    WorkflowPattern, WorkflowTransform, Identity, ConstrainedWorkflow, Proof, TypeStateProperty,
};
pub use lockfree::{LockFreeReceiptQueue, LockFreeReceiptIndex};
pub use simd_hash::{SimdHashVerifier, SimdMerkleTree, SimdAlignedBuffer, simd_constant_time_eq};
pub use gat_query::{
    Query, Filter, Map, Reduce, ScanReceipts, GetReceiptById, GetReceiptsByWorkflow,
    QueryBuilder as GatQueryBuilder, ValidatedQuery, Fusible,
};
pub use arena::{Arena, TypedArena, WorkflowContext, ArenaStats};
pub use const_eval::{
    ConstWorkflow, WorkflowPattern as ConstPattern, Sequential, Parallel, Choice, Loop,
    Sequence, ParallelCompose, WorkflowMetrics, WorkflowBuilder, ConstOptimizer,
    AssertChatmanCompliant, CHATMAN_CONSTANT, MAX_WORKFLOW_STEPS, MAX_NESTING_DEPTH,
};

// Microkernel feature exports
pub use verified_kernel::{
    KernelResult, KernelError, TickBudget, KernelState, ExecutionPhase, GuardResult,
    KernelOp, GuardCheckOp, KernelSequence, KernelProof, VerifiedContext,
};
pub use refinement_guards::{
    SectorLevel, PublicSector, PrivateSector, CriticalSector, GuardProperty,
    BudgetGuard, CausalityGuard, LegalityGuard, ProofToken, CertifiedOp,
    GuardVector, TypedWorkflow as RefinementWorkflow, DoctrineConstraint,
    StrictDoctrine, RelaxedDoctrine, InvariantQ, NoRetrocausation, BoundedResource,
};
pub use cluster_types::{
    ClusterRole, Leader, Follower, Observer, ClusterConfig, ReplicationFactor,
    TripleReplication, FiveWayReplication, ConsensusOp, Proposal, Committed,
    LogEntry, ReplicatedLog, DistributedContext, ConsensusState, StateMachine,
};
pub use auto_specialize::{
    CpuCapability, GenericCpu, X86Avx2, X86Avx512, ArmNeon, DataProfile,
    SmallData, LargeData, SkewedData, KernelVariant, ScalarKernel, SimdAvx2Kernel,
    AutoSelector, KernelSelection, SpecializedExecutor, AdaptationTrigger,
    PerformanceMonitor, AdaptiveExecutor,
};
pub use linear_resources::{
    ResourceToken, ConsumedToken, ResourceQuota, PriorityClass, P0, P1, P2, P3, P4,
    SloBand, Interactive, Batch, Background, ScheduledAction, HotPathScheduler,
    BackgroundScheduler, ResourcePool,
};

// 2027 Innovation exports
pub use effect_system::{
    Effect, Pure, Io, State, Error, Resource, EffectSet, NoEffects, AllEffects,
    Effectful, IoHandler, StateHandler, Union, EffectfulWorkflow, Permission, EffectContext,
};
pub use simd_kernels::{
    SimdWidth, Scalar, Avx2, Avx512, Neon, SimdVectorSum, SimdDispatcher, SimdReceiptValidator,
};
pub use custom_allocators::{
    Arena as CustomArena, BumpAllocator, ObjectPool, PooledObject, StackAllocator, AllocatorStats,
};
pub use async_kernel::{
    AsyncWorkflowState, AsyncPending, AsyncRunning, AsyncSuspended, AsyncCompleted, AsyncFailed,
    AsyncWorkflow, AsyncKernelResult, AsyncKernelOp, WithTimeout, TimeoutError,
    CancellationToken, Cancellable, Cancelled, AsyncExecutor, AsyncWorkflowBuilder,
};
