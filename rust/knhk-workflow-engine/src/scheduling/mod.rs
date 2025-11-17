//! YAWL Scheduling Service
//!
//! Implements YAWL scheduling with TRIZ Principle 37: Thermal Expansion
//! - Scale scheduling resources based on load temperature

pub mod calendar;

pub use calendar::{CalendarEntry, CalendarEntryType, CalendarService, ResourceCalendar, WorkingHours};

