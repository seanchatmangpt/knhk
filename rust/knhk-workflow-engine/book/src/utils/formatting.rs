//! Formatting utilities
//!
//! Provides formatting functions for display and serialization.

use std::fmt;

/// Format duration for display
pub fn format_duration(duration: std::time::Duration) -> String {
    let nanos = duration.as_nanos();
    if nanos < 1_000 {
        format!("{}ns", nanos)
    } else if nanos < 1_000_000 {
        format!("{:.2}μs", nanos as f64 / 1_000.0)
    } else if nanos < 1_000_000_000 {
        format!("{:.2}ms", nanos as f64 / 1_000_000.0)
    } else {
        format!("{:.2}s", nanos as f64 / 1_000_000_000.0)
    }
}

/// Format ticks for display
pub fn format_ticks(ticks: u32) -> String {
    format!("{} ticks", ticks)
}

/// Format pattern ID for display
pub fn format_pattern_id(id: u32) -> String {
    format!("pattern:{}", id)
}

/// Format workflow spec ID for display
pub fn format_workflow_spec_id(id: &impl fmt::Display) -> String {
    format!("workflow:{}", id)
}

/// Format case ID for display
pub fn format_case_id(id: &impl fmt::Display) -> String {
    format!("case:{}", id)
}
