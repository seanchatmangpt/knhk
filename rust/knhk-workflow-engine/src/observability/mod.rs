//! Observability module
//!
//! Provides observability features including metrics, health checks,
//! performance monitoring, distributed tracing, dark matter detection,
//! and MAPE-K autonomic control.

mod dark_matter;
mod health;
mod mape_k;
mod metrics;
mod performance;
mod tracing;

pub use dark_matter::*;
pub use health::*;
pub use mape_k::*;
pub use metrics::*;
pub use performance::*;
pub use tracing::*;
