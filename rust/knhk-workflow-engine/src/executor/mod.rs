//! Workflow execution engine module
//!
//! This module contains the core workflow engine implementation, split into logical components:
//! - `engine.rs`: Core engine structure
//! - `construction.rs`: Engine construction and initialization
//! - `events.rs`: Event loop handlers (timer, external events)
//! - `workflow_registration.rs`: Workflow registration with validation
//! - `workflow_query.rs`: Workflow query operations (get, list)
//! - `case.rs`: Case management (create, start, execute, cancel, get, list)
//! - `task.rs`: Task execution with resource allocation
//! - `pattern.rs`: Pattern execution with reflex bridge integration
//! - `accessors.rs`: Getter methods for engine components
//! - `fortune5.rs`: Fortune 5 integration methods
//! - `rdf_query.rs`: Runtime RDF query API
//!
//! # New Self-Executing Workflow Components (Covenant 1)
//!
//! - `loader.rs`: Load workflows from Turtle/RDF definitions
//! - `runtime.rs`: Execute workflows with state machine (Turtle as definition)
//! - `telemetry.rs`: OpenTelemetry integration for full observability

mod accessors;
mod case;
mod construction;
mod engine;
mod events;
mod fortune5;
mod loader;
mod pattern;
mod provenance;
mod rdf_query;
mod runtime;
mod task;
mod telemetry;
mod workflow_execution;
mod workflow_query;
mod workflow_registration;
mod xes_export;

pub use engine::WorkflowEngine;
pub use loader::{
    WorkflowLoader, WorkflowDefinition, TaskDefinition, FlowDefinition,
    VariableDefinition, SplitType, JoinType, ExecutionMode,
    TimeoutPolicy, RetryPolicy,
};
pub use runtime::{WorkflowRuntime, ExecutionState, WorkflowState, TaskResult, TaskExecutor};
pub use telemetry::{WorkflowTelemetry, WorkflowEvent, TaskEvent};
