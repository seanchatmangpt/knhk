//! Hook registry for workflow engine
//!
//! Provides hooks for workflow lifecycle events, task execution, and case management.
//! Hooks allow external systems to intercept and extend workflow behavior.

mod registry;
mod schema;

pub use registry::{HookError, HookRegistry, HookType, WorkflowHook};
pub use schema::{SchemaRegistry, SchemaValidationError};
