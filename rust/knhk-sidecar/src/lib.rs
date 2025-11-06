// knhk-sidecar: KGC Sidecar Service
// gRPC proxy for enterprise apps with batching, retries, circuit-breaking, and TLS

pub mod error;
pub mod batch;
pub mod retry;
pub mod circuit_breaker;
pub mod tls;
pub mod metrics;
pub mod health;
pub mod client;
pub mod server;
pub mod config;

pub use error::{SidecarError, SidecarResult};
pub use server::SidecarServer;
pub use client::SidecarClient;
pub use config::SidecarConfig;

