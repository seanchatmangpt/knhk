// rust/knhk-sidecar/src/lib.rs
// KGC Sidecar - Local proxy service for enterprise apps

pub mod error;
pub mod batching;
pub mod retry;
pub mod circuit_breaker;
pub mod warm_client;
pub mod server;

pub use error::{SidecarError, Result};
pub use batching::{BatchConfig, BatchManager};
pub use retry::{RetryConfig, RetryExecutor};
pub use circuit_breaker::{CircuitBreaker, CircuitBreakerConfig, CircuitBreakerState};
pub use warm_client::{WarmClient, WarmClientConfig};
pub use server::{ServerConfig, KgcSidecarService, start_server};

