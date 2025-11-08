#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Security and zero-trust features for Fortune 500-level workflow engine

pub mod audit;
pub mod auth;
pub mod guards;
pub mod secrets;

pub use audit::{AuditEvent, AuditLevel, AuditLogger};
pub use auth::{AuthManager, AuthPolicy, Principal};
pub use guards::{GuardFunction, GuardResult, GuardValidator};
pub use secrets::{SecretManager, SecretProvider};
