//! Observability module
//!
//! Provides observability features including metrics, health checks,
//! performance monitoring, distributed tracing, and dark matter detection.

mod dark_matter;
mod health;
mod metrics;
mod performance;
mod tracing;

pub use dark_matter::*;
pub use health::*;
pub use metrics::*;
pub use performance::*;
pub use tracing::*;
