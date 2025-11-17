//! KNHK Phase 10: Market Deployment and Licensing System
//!
//! This module provides comprehensive infrastructure for:
//! - Commercial licensing (Community, Professional, Enterprise tiers)
//! - Workflow marketplace and templates
//! - Usage-based billing and invoicing
//! - Telemetry and analytics collection
//! - Cloud deployment integration (AWS, GCP, Azure)

pub mod billing;
pub mod deployment;
pub mod licensing;
pub mod marketplace;
pub mod telemetry;

/// Phase 10 version identifier
pub const PHASE_10_VERSION: &str = "0.10.0";

/// Maximum concurrent marketplace downloads per license tier
pub const MAX_CONCURRENT_DOWNLOADS: usize = 5;

/// Invoice generation interval (24 hours)
pub const INVOICE_INTERVAL_HOURS: u64 = 24;

/// License validation cache TTL (1 hour)
pub const LICENSE_CACHE_TTL_SECS: u64 = 3600;

/// Telemetry batch size before sending upstream
pub const TELEMETRY_BATCH_SIZE: usize = 100;

#[derive(Debug, thiserror::Error)]
pub enum MarketplaceError {
    #[error("License validation failed: {0}")]
    LicenseValidation(String),

    #[error("Marketplace operation failed: {0}")]
    Marketplace(String),

    #[error("Billing calculation error: {0}")]
    Billing(String),

    #[error("Telemetry collection failed: {0}")]
    Telemetry(String),

    #[error("Deployment configuration error: {0}")]
    Deployment(String),

    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Authorization denied: {0}")]
    AuthorizationError(String),

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("Feature not available in current license tier")]
    FeatureUnavailable,
}

pub type Result<T> = std::result::Result<T, MarketplaceError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_string() {
        assert_eq!(PHASE_10_VERSION, "0.10.0");
    }

    #[test]
    fn test_constants_defined() {
        assert_eq!(MAX_CONCURRENT_DOWNLOADS, 5);
        assert_eq!(INVOICE_INTERVAL_HOURS, 24);
        assert_eq!(LICENSE_CACHE_TTL_SECS, 3600);
        assert_eq!(TELEMETRY_BATCH_SIZE, 100);
    }
}
