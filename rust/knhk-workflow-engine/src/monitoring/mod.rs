//! Monitoring and Observability - Andon & Gemba Systems
//!
//! Implements Toyota Production System concepts for workflow monitoring:
//!
//! - **Andon**: Visual control system with Green/Yellow/Red states
//!   - Real-time alerting when issues are detected
//!   - Ability to "stop the line" (pause workflows) on critical issues
//!   - Integration with OTEL for observability
//!
//! - **Gemba**: Real-time workflow observation
//!   - "Go to the real place" - observe execution where it happens
//!   - Capture execution context and performance metrics
//!   - Feed insights to Andon system for alerting
//!
//! # Usage
//!
//! ```rust
//! use knhk_workflow_engine::monitoring::{AndonSystem, AndonConfig, GembaWalker};
//!
//! // Create Andon system
//! let andon = Arc::new(AndonSystem::new(AndonConfig::default()));
//!
//! // Create Gemba walker
//! let gemba = GembaWalker::new(andon.clone(), 1000);
//!
//! // Observe workflow execution
//! gemba.observe(
//!     ObservationPoint::TaskExecution,
//!     workflow_id,
//!     context,
//!     metrics,
//! ).await?;
//!
//! // Check Andon state
//! let state = andon.state().await;
//! if state.should_stop() {
//!     // Stop workflow execution
//! }
//! ```

pub mod andon;
pub mod gemba;

pub use andon::{
    AndonAlert, AndonAlertType, AndonConfig, AndonState, AndonStatsSnapshot, AndonSystem,
};
pub use gemba::{
    GembaObservation, GembaWalker, ObservationContext, ObservationMetrics, ObservationPoint,
    PatternAnalysis, PerformanceSummary,
};

#[cfg(feature = "monitoring-integration")]
pub mod integration;
