//! Enterprise workflow engine with full 43-pattern YAWL support
//!
//! This crate provides a complete workflow engine that:
//! - Parses Turtle/YAWL workflow definitions
//! - Executes all 43 Van der Aalst workflow patterns
//! - Provides enterprise APIs (REST + gRPC)
//! - Manages workflow cases with state persistence
//! - Integrates with KNHK infrastructure (OTEL, lockchain, connectors)
//!
//! # Features
//!
//! - **Full Pattern Support**: All 43 Van der Aalst workflow patterns
//! - **YAWL Compatibility**: Parses and executes YAWL workflow definitions
//! - **Enterprise APIs**: REST and gRPC interfaces
//! - **State Persistence**: Sled-based state store
//! - **Observability**: OTEL integration for tracing
//! - **Provenance**: Lockchain integration for audit trails
//! - **Fortune 5 Ready**: Enterprise-grade features for Fortune 5 deployments
//!
//! # Usage
//!
//! ```rust,no_run
//! use knhk_workflow_engine::{WorkflowEngine, WorkflowParser, StateStore};
//!
//! // Create state store
//! let state_store = StateStore::new("./workflow_db")?;
//!
//! // Create engine
//! let engine = WorkflowEngine::new(state_store);
//!
//! // Parse workflow from Turtle
//! let mut parser = WorkflowParser::new()?;
//! let spec = parser.parse_file("workflow.ttl")?;
//!
//! // Register workflow
//! engine.register_workflow(spec).await?;
//!
//! // Create and execute case
//! let case_id = engine.create_case(spec_id, serde_json::json!({})).await?;
//! engine.start_case(case_id).await?;
//! engine.execute_case(case_id).await?;
//! ```
//!
//! # Pattern Categories
//!
//! - **Basic Control Flow** (1-5): Sequence, Parallel Split, Synchronization, Exclusive Choice, Simple Merge
//! - **Advanced Branching** (6-11): Multi-Choice, Structured Synchronizing Merge, Multi-Merge, Discriminator, Arbitrary Cycles, Implicit Termination
//! - **Multiple Instance** (12-15): MI Without Sync, MI With Design-Time Knowledge, MI With Runtime Knowledge, MI Without Runtime Knowledge
//! - **State-Based** (16-18): Deferred Choice, Interleaved Parallel Routing, Milestone
//! - **Cancellation** (19-25): Cancel Activity, Cancel Case, Cancel Region, Cancel MI Activity, Complete MI Activity, Blocking Discriminator, Cancelling Discriminator
//! - **Advanced Patterns** (26-39): Advanced workflow patterns
//! - **Trigger Patterns** (40-43): Event-driven workflow patterns

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(missing_docs)]

pub mod api;
pub mod cache;
pub mod capabilities;
pub mod case;
pub mod cluster;
pub mod compiler;
pub mod compliance;
pub mod config;
pub mod constants;
pub mod data;
pub mod enterprise;
pub mod error;
pub mod events;
pub mod execution;
pub mod executor;
pub mod ggen;
pub mod hooks;
pub mod innovation;
pub mod integration;

// Macros are exported via #[macro_export] in otel_macros.rs

#[macro_use]
pub mod observability;
pub mod parser;
pub mod patterns;
pub mod performance;
pub mod process_mining;
pub mod reflex;
pub mod resilience;
pub mod resource;
pub mod security;
pub mod self_validation;
pub mod services;
pub mod state;
pub mod templates;
pub mod testing;
pub mod timebase;
pub mod utils;
pub mod validation;
pub mod visualization;
pub mod worklets;

pub use capabilities::{
    validate_capabilities, CapabilityMetadata, CapabilityRegistry, CapabilityStatus,
    CapabilityValidationReport, CapabilityValidator,
};
pub use case::{Case, CaseId, CaseState};
pub use enterprise::{
    EnterpriseConfig, ObservabilityConfig, PerformanceConfig, ReliabilityConfig, ScalabilityConfig,
    SecurityConfig,
};
pub use error::{WorkflowError, WorkflowResult};
pub use execution::{
    ExecutionEngine, ExecutionHandle, ExecutionPipeline, ExecutionStatus, WorkQueue,
};
pub use executor::WorkflowEngine;
pub use ggen::{
    generate_documentation_from_spec, generate_tests_from_spec, generate_workflow_from_rdf,
    GgenGenerator,
};
pub use innovation::{
    DeltaLogEntry, DeterministicContext, DeterministicExecutor, ExecutionStep, FormalVerifier,
    HardwareAcceleration, HardwareAccelerator, Property, VerificationResult, Violation,
    ZeroCopyBytes, ZeroCopyStr, ZeroCopyTriple, ZeroCopyTripleBatch,
};
pub use parser::{WorkflowParser, WorkflowSpec, WorkflowSpecId};
pub use patterns::{PatternId, PatternRegistry, RegisterAllExt};
pub use performance::{HotPathResult, HotPathValidator, PerformanceMetrics};
pub use process_mining::{WorkflowEvent, XesExporter};
pub use reflex::{PromotableSegment, PromotionAnalysis, ReflexBridge};
pub use security::*;
pub use services::{AdmissionGate, EventSidecar, TimerFired, TimerService, WorkItemService};
pub use state::{StateEvent, StateManager, StateStore};
pub use templates::TemplateLibrary;
pub use testing::{
    assert_pattern_failure, assert_pattern_has_next_state, assert_pattern_has_variable,
    assert_pattern_success, assert_pattern_variable_equals, create_test_capability,
    create_test_context, create_test_context_for_workflow, create_test_context_with_vars,
    create_test_registry, create_test_resource, create_test_role, create_test_worklet,
    property_all_workflows_registrable, property_all_workflows_valid_structure,
    property_workflow_execution_terminates, CoverageAnalyzer, CoverageReport,
    IntegrationTestHelper, MutationOperator, MutationScore, MutationTester, PerformanceTestHelper,
    PropertyTestGenerator, TaskBuilder, WorkflowPropertyTester, WorkflowSpecBuilder,
    WorkflowTestFixture, WorkflowTestGenerator,
};
// TestDataBuilder is now in chicago-tdd-tools - import directly:
// use chicago_tdd_tools::builders::TestDataBuilder;
pub use visualization::WorkflowVisualizer;
