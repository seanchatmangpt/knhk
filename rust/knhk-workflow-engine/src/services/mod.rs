//! Service modules for workflow engine
//!
//! Provides:
//! - Timer service for time-based patterns (30/31)
//! - Event sidecar for external event handling (pattern 16)
//! - Admission gate for case validation
//! - Work item service for human task management

pub mod admission;
pub mod event_sidecar;
pub mod timer;
pub mod work_items;

pub use admission::AdmissionGate;
pub use event_sidecar::EventSidecar;
pub use timer::TimerFired;
pub use work_items::WorkItemService;

// TimerService is generic over Timebase, so we export a type alias for common use
pub use timer::TimerService;
