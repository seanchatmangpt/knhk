//! REST API server for Fortune 5 deployments
//!
//! Provides enterprise-grade REST API with:
//! - OpenAPI/Swagger documentation
//! - Authentication/Authorization
//! - Rate limiting
//! - Circuit breakers
//! - Request tracing
//! - Audit logging
//! - Health checks

pub mod handlers;
pub mod server;

pub use server::RestApiServer;
