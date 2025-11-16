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
