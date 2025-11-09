//! Unified API service layer
//!
//! Service layer that encapsulates business logic for all transport layers.

pub mod case;
pub mod pattern;
pub mod workflow;

// Re-export for convenience
pub use case::CaseService;
pub use pattern::PatternService;
pub use workflow::WorkflowService;
