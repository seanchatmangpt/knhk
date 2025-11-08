//! Workflow execution engine module
//!
//! This module contains the core workflow engine implementation, split into logical components:
//! - `engine.rs`: Core engine structure
//! - `construction.rs`: Engine construction and initialization
//! - `events.rs`: Event loop handlers (timer, external events)
//! - `workflow.rs`: Workflow management (register, get, list)
//! - `case.rs`: Case management (create, start, execute, cancel, get, list)
//! - `task.rs`: Task execution with resource allocation
//! - `pattern.rs`: Pattern execution with reflex bridge integration
//! - `accessors.rs`: Getter methods for engine components
//! - `fortune5.rs`: Fortune 5 integration methods

mod accessors;
mod case;
mod construction;
mod engine;
mod events;
mod fortune5;
mod pattern;
mod task;
mod workflow;

pub use engine::WorkflowEngine;
