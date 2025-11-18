//! YAWL execution engine
//!
//! Covenant 5: All operations respect the Chatman constant (≤ 8 ticks for hot path)
//! Covenant 6: Full observability through OpenTelemetry

pub mod executor;
pub mod actor;
pub mod token;

// Re-export key types
pub use executor::WorkflowExecutor;
pub use actor::TaskActor;
pub use token::TokenManager;
