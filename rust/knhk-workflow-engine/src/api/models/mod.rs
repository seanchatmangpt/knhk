//! Unified API models
//!
//! Request, response, and error models used by all transport layers.

use serde::{Deserialize, Serialize};

pub mod errors;
pub mod requests;
pub mod responses;

// Re-export for convenience
pub use errors::{ApiError, ApiResult};
pub use requests::*;
pub use responses::*;

/// Case history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseHistoryEntry {
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Event type
    pub event_type: String,
    /// Event data
    pub data: serde_json::Value,
}
