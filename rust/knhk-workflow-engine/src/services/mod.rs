//! Service modules for workflow engine
//!
//! Provides:
//! - Timer service for time-based patterns (30/31)
//! - Event sidecar for external event handling (pattern 16)
//! - Admission gate for case validation
//! - Work item service for human task management

pub mod timer;
pub mod event_sidecar;
pub mod admission;
pub mod work_items;

pub use timer::{TimerService, TimerFired};
pub use event_sidecar::EventSidecar;
pub use admission::AdmissionGate;
pub use work_items::WorkItemService;

