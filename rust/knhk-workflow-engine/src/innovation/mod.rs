//! Workflow Innovation Features
//!
//! Advanced features that compose existing packages to create new capabilities.
//! These features demonstrate the power of the self-executing workflow system.
//!
//! # Features
//!
//! - **Workflow Templates**: Pre-built patterns (ETL, Saga, Fan-out/Fan-in, etc.)
//! - **Adaptive Optimizer**: ML-driven workflow optimization using MAPE-K + receipts
//! - **A/B Testing**: Shadow execution for safe testing
//! - **Analytics Engine**: Query and analyze Γ(O) history
//! - **Deterministic Execution**: Delta-based deterministic replay
//! - **Formal Verification**: Mathematical proofs of workflow properties
//! - **Hardware Acceleration**: GPU/SIMD acceleration for workflows
//! - **Zero-Copy Execution**: Memory-efficient workflow execution
//! - **Experimental Features**: Cutting-edge workflow capabilities

pub mod workflow_templates;
pub mod adaptive_optimizer;
pub mod ab_testing;
pub mod analytics;
pub mod deterministic;
pub mod experiment;
pub mod formal;
pub mod hardware;
pub mod zero_copy;

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
