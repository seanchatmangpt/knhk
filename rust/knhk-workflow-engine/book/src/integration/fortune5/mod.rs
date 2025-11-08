//! Fortune 5 Enterprise Integration
//!
//! Provides SPIFFE/SPIRE, KMS, multi-region, SLO, and promotion gate integration
//! for Fortune 5 enterprise deployments.
//!
//! # Features
//!
//! - **SPIFFE/SPIRE**: Service identity and authentication
//! - **KMS Integration**: Key management for secrets
//! - **Multi-Region**: Cross-region replication and failover
//! - **SLO Tracking**: Service level objective monitoring
//! - **Promotion Gates**: Safe deployment with rollback

mod config;
mod integration;
mod slo;

pub use config::*;
pub use integration::Fortune5Integration;
pub use slo::{RuntimeClass, SloManager, SloMetrics};
