//! # KNHK Process Mining Integration
//!
//! This crate provides process mining and workflow analytics capabilities for KNHK.
//! It extracts event logs from OpenTelemetry spans, discovers process patterns,
//! analyzes performance, and identifies optimization opportunities.
//!
//! ## Features
//!
//! - **Event Log Extraction**: Convert OTEL spans to process mining event logs
//! - **Process Discovery**: Discover actual workflow structure from execution traces
//! - **Performance Analytics**: Analyze cycle times, throughput, bottlenecks
//! - **Pattern Validation**: Verify workflows match expected patterns
//! - **Optimization Recommendations**: Data-driven process improvement
//! - **Poka-Yoke Type Safety**: Make invalid states impossible through type system
//!
//! ## Architecture
//!
//! ```text
//! Workflow Execution
//!     ↓ (OTEL Spans)
//! Event Log Extraction
//!     ↓ (EventLogBuilder)
//! Process Discovery
//!     ↓ (DiscoveryEngine)
//! Analytics & Reports
//!     ↓ (ProcessAnalyzer)
//! Optimization Recommendations
//! ```
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use knhk_process_mining::{EventLogBuilder, ProcessAnalyzer, DiscoveryEngine};
//!
//! // Extract event log from telemetry spans
//! let event_log = EventLogBuilder::new()
//!     .add_span_context(span_data)
//!     .build()?;
//!
//! // Discover process structure
//! let discovered_process = DiscoveryEngine::new()
//!     .discover(&event_log)?;
//!
//! // Analyze performance
//! let analytics = ProcessAnalyzer::new(&event_log)
//!     .analyze()?;
//!
//! println!("Avg cycle time: {}ms", analytics.avg_cycle_time_ms);
//! println!("Throughput: {} cases/hour", analytics.throughput_per_hour);
//! println!("Bottlenecks: {:?}", analytics.bottlenecks);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Poka-Yoke Type-Safe API
//!
//! This crate uses Rust's type system to make invalid states impossible:
//!
//! ```rust,no_run
//! use knhk_process_mining::{
//!     builders::EventBuilder,
//!     types::{CaseID, ActivityName, Timestamp},
//!     resource_handles::EventLog,
//!     typed_pipeline::ProcessMiningPipeline,
//! };
//!
//! // Type-safe domain objects (cannot be invalid)
//! let case_id = CaseID::new(1)?;              // Cannot be zero
//! let activity = ActivityName::new("Task")?;  // Cannot be empty
//!
//! // Type-state builder (compile-time required fields)
//! let event = EventBuilder::new()
//!     .with_case_id(case_id)
//!     .with_activity(activity)
//!     .with_timestamp(Timestamp::now())
//!     .build();  // Only available when all fields set
//!
//! // Resource lifecycle (cannot use after close)
//! let mut log = EventLog::new();
//! log = log.add_event(event)?;
//! let closed = log.close();
//! let analytics = closed.analyze();
//!
//! // Type-safe pipeline (enforced ordering)
//! let results = ProcessMiningPipeline::new()
//!     .load_from_event_log(closed)?
//!     .discover_process()
//!     .validate_model()
//!     .complete()
//!     .into_results();
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ## Integration Points
//!
//! 1. **OTEL Spans**: Primary event source
//! 2. **Workflow Engine**: Execution traces
//! 3. **Validation**: Pattern conformance
//! 4. **Performance**: Optimization opportunities
//!
//! ## Key Principles
//!
//! - **Schema-First**: Event logs conform to process mining standards
//! - **Performance-Aware**: ≤8 tick constraint for hot path operations
//! - **Non-Intrusive**: Extract from existing telemetry, no code changes
//! - **Actionable**: Clear optimization recommendations
//! - **Type-Safe**: Invalid states impossible through Poka-Yoke patterns

pub mod analytics;
pub mod discovery;
pub mod event_log;

// Poka-Yoke type safety modules
pub mod builders;
pub mod resource_handles;
pub mod state_machine;
pub mod typed_pipeline;
pub mod types;

// Re-export main types (legacy API)
pub use analytics::{BottleneckDetector, PerformanceAnalytics, ProcessAnalyzer};
pub use discovery::{DiscoveryEngine, PatternValidator, ProcessGraph};
pub use event_log::{EventLog, EventLogBuilder, ProcessEvent};

// Re-export Poka-Yoke types (type-safe API)
pub use builders::{ConfigBuilder, ConfigError, Event, EventBuilder, ProcessMiningConfig};
pub use resource_handles::{
    AnalysisResults, AnalyzerCompleted, AnalyzerConfigured, AnalyzerRunning,
    EventLog as TypedEventLog, EventLogClosed, EventLogError, EventLogOpen, ProcessAnalytics,
    ProcessAnalyzer as TypedProcessAnalyzer,
};
pub use state_machine::{StateError, WorkflowState, WorkflowStateSnapshot};
pub use typed_pipeline::{
    OwnedPipelineResults, PipelineError, PipelineResults, ProcessGraph as TypedProcessGraph,
    ProcessMiningPipeline, ValidationReport,
};
pub use types::{
    ActivityName, CaseID, Count, Duration, EventID, InvalidIdError, InvalidProbabilityError,
    InvalidStringError, Probability, Timestamp,
};

use thiserror::Error;

/// Process mining errors
#[derive(Error, Debug)]
pub enum ProcessMiningError {
    #[error("Event log error: {0}")]
    EventLog(String),

    #[error("Discovery error: {0}")]
    Discovery(String),

    #[error("Analytics error: {0}")]
    Analytics(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for process mining operations
pub type Result<T> = std::result::Result<T, ProcessMiningError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_imports() {
        // Verify all modules are accessible
        let _ = EventLogBuilder::new();
        let _ = DiscoveryEngine::default();
        let _ = BottleneckDetector::default();
    }
}
