// rust/knhk-sidecar/src/lib.rs
// KGC Sidecar gRPC service

pub mod error;
pub mod config;
pub mod circuit_breaker;
pub mod retry;
pub mod batching;
pub mod health;
pub mod service;
pub mod server;

pub use error::{SidecarError, SidecarResult};
pub use config::SidecarConfig;
pub use server::{start_server, run};

