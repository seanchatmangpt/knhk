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
//!
//! ## 2027 Advanced Phases (Cutting-Edge Development Lifecycle)
//! - **Verification Phase**: Compile-time correctness proofs using type-level programming
//! - **Optimization Phase**: Auto-tuning with profiling, adaptive algorithms, and PGO
//! - **Security Phase**: Capability-based access control with zero-cost abstractions
//! - **Observability Phase**: Zero-cost distributed tracing with compile-time filtering
//! - **Deployment Phase**: Zero-downtime deployment with health checks and canary releases

// Production features
pub mod ab_testing;
pub mod hyper_advanced;
pub mod adaptive_optimizer;
pub mod analytics;
pub mod deterministic;
pub mod experiment;
pub mod formal;
pub mod hardware;
pub mod workflow_templates;
pub mod zero_copy;

// Hyper-advanced features
pub mod arena;
pub mod const_eval;
pub mod gat_query;
pub mod lockfree;
pub mod simd_hash;
pub mod type_state;

// Microkernel features (AHI Constitution)
pub mod auto_specialize;
pub mod cluster_types;
pub mod linear_resources;
pub mod refinement_guards;
pub mod verified_kernel;

// 2027 Innovations (Hyper-Advanced Rust)
pub mod async_kernel;
pub mod custom_allocators;
pub mod effect_system;
pub mod simd_kernels;

// 2027 Advanced Phases
pub mod deployment_phase;
pub mod observability_phase;
pub mod optimization_phase;
pub mod security_phase;
pub mod verification_phase;

// Production feature exports
pub use ab_testing::{
    ABTestConfig, ABTestOrchestrator, ABTestResults, ShadowTestResult, TestRecommendation,
    TestVariant,
};
pub use adaptive_optimizer::{
    AdaptiveOptimizer, OptimizationAction, OptimizationRecommendation, OptimizationStrategy,
    PerformanceMetrics,
};
pub use analytics::{
    AnalyticsEngine, DataPoint, ExecutiveSummary, GuardFailureAnalysis, QueryBuilder,
    SnapshotComparison,
};
pub use deterministic::{
    DeltaLogEntry, DeterministicContext, DeterministicExecutor, ExecutionStep,
};
pub use experiment::*;
pub use formal::{FormalVerifier, Property, VerificationResult, Violation};
pub use hardware::{HardwareAcceleration, HardwareAccelerator};
pub use workflow_templates::{TemplateLibrary, TemplateMetadata, WorkflowTemplate};
pub use zero_copy::{ZeroCopyBytes, ZeroCopyStr, ZeroCopyTriple, ZeroCopyTripleBatch};

// Hyper-advanced feature exports
pub use arena::{Arena, ArenaStats, TypedArena, WorkflowContext};
pub use const_eval::{
    AssertChatmanCompliant, Choice, ConstOptimizer, ConstWorkflow, Loop, Parallel, ParallelCompose,
    Sequence, Sequential, WorkflowBuilder, WorkflowMetrics, WorkflowPattern as ConstPattern,
    CHATMAN_CONSTANT, MAX_NESTING_DEPTH, MAX_WORKFLOW_STEPS,
};
pub use gat_query::{
    Filter, Fusible, GetReceiptById, GetReceiptsByWorkflow, Map, Query,
    QueryBuilder as GatQueryBuilder, Reduce, ScanReceipts, ValidatedQuery,
};
pub use lockfree::{LockFreeReceiptIndex, LockFreeReceiptQueue};
pub use simd_hash::{simd_constant_time_eq, SimdAlignedBuffer, SimdHashVerifier, SimdMerkleTree};
pub use type_state::{
    Completed, Configured, ConstrainedWorkflow, Executing, Failed, Identity, Proof,
    TypeStateProperty, TypedWorkflow, Uninitialized, Validated, WorkflowPattern, WorkflowTransform,
};

// Microkernel feature exports
pub use auto_specialize::{
    AdaptationTrigger, AdaptiveExecutor, ArmNeon, AutoSelector, CpuCapability, DataProfile,
    GenericCpu, KernelSelection, KernelVariant, LargeData, PerformanceMonitor, ScalarKernel,
    SimdAvx2Kernel, SkewedData, SmallData, SpecializedExecutor, X86Avx2, X86Avx512,
};
pub use cluster_types::{
    ClusterConfig, ClusterRole, Committed, ConsensusOp, ConsensusState, DistributedContext,
    FiveWayReplication, Follower, Leader, LogEntry, Observer, Proposal, ReplicatedLog,
    ReplicationFactor, StateMachine, TripleReplication,
};
pub use linear_resources::{
    Background, BackgroundScheduler, Batch, ConsumedToken, HotPathScheduler, Interactive,
    PriorityClass, ResourcePool, ResourceQuota, ResourceToken, ScheduledAction, SloBand, P0, P1,
    P2, P3, P4,
};
pub use refinement_guards::{
    BoundedResource, BudgetGuard, CausalityGuard, CertifiedOp, CriticalSector, DoctrineConstraint,
    GuardProperty, GuardVector, InvariantQ, LegalityGuard, NoRetrocausation, PrivateSector,
    ProofToken, PublicSector, RelaxedDoctrine, SectorLevel, StrictDoctrine,
    TypedWorkflow as RefinementWorkflow,
};
pub use verified_kernel::{
    ExecutionPhase, GuardCheckOp, GuardResult, KernelError, KernelOp, KernelProof, KernelResult,
    KernelSequence, KernelState, TickBudget, VerifiedContext,
};

// 2027 Innovation exports
pub use async_kernel::{
    AsyncCompleted, AsyncExecutor, AsyncFailed, AsyncKernelOp, AsyncKernelResult, AsyncPending,
    AsyncRunning, AsyncSuspended, AsyncWorkflow, AsyncWorkflowBuilder, AsyncWorkflowState,
    Cancellable, CancellationToken, Cancelled, TimeoutError, WithTimeout,
};
pub use custom_allocators::{
    AllocatorStats, Arena as CustomArena, BumpAllocator, ObjectPool, PooledObject, StackAllocator,
};
pub use effect_system::{
    AllEffects, Effect, EffectContext, EffectSet, Effectful, EffectfulWorkflow, Error, Io,
    IoHandler, NoEffects, Permission, Pure, Resource, State, StateHandler, Union,
};
pub use simd_kernels::{
    Avx2, Avx512, Neon, Scalar, SimdDispatcher, SimdReceiptValidator, SimdVectorSum, SimdWidth,
};

// 2027 Advanced Phase exports
pub use deployment_phase::{
    CanaryDeployment, Completed as DeploymentCompleted, Deployment, DeploymentMetrics,
    DeploymentState, ErrorRateRollback, Failed as DeploymentFailed, HealthCheck, HealthMonitor,
    HealthStatus, HttpHealthCheck, LatencyRollback, Pending as DeploymentPending, RequestGuard,
    RollbackStrategy, RolledBack, RollingOut, ShutdownCoordinator, TcpHealthCheck, Validating,
};
pub use observability_phase::{
    AdaptiveSample, AlwaysSample, Counter, Debug, Error as TraceError, Exemplar, Gauge, Histogram,
    Info, LogAggregator, LogEntry as TraceLogEntry, Metric, MetricWithExemplars, NeverSample,
    ProbabilisticSample, SamplingStrategy, Span, Trace, TraceContext, TraceLevel, Warn,
};
pub use optimization_phase::{
    AdaptiveSelector, AutoTune, BranchHint, CacheAligned, HotPathDetector, InlineControl,
    LoopUnroller, OptimizationLevel, Os, PerfCounter, PgoCollector, Prefetch, Vectorize, O0, O2,
    O3,
};
pub use security_phase::{
    Admin, Attenuated, AuditEntry, AuditTrail, Authority, Capability, Classified, Confidential,
    ConstantTime, CryptoKey, Execute, MemoryGuard, Permission as SecurityPermission, Public, Read,
    Secret, SecureChannel, Secured, SecurityLevel, TopSecret, Write,
};
pub use verification_phase::{
    AbstractValue, Certified, CorrectnessCondition, Deterministic, HoareTriple, Invariant,
    MemorySafe, ModelChecker, NoDataRaces, Owns, Postcondition, Precondition,
    Proof as VerificationProof, SymbolicPath, Terminates, Tested, Unverified, VerifiableProperty,
    VerificationLevel, Verified, VerifiedWorkflow,
};
