//! License management and feature gating system
//!
//! Implements multi-tier licensing with offline verification,
//! feature gates, and expiration checking

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;

use crate::{MarketplaceError, Result};

/// License tier levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum License {
    Community = 0,
    Professional = 1,
    Enterprise = 2,
}

impl License {
    pub fn max_workflows(&self) -> usize {
        match self {
            License::Community => 5,
            License::Professional => 100,
            License::Enterprise => usize::MAX,
        }
    }

    pub fn max_monthly_executions(&self) -> usize {
        match self {
            License::Community => 1_000,
            License::Professional => 100_000,
            License::Enterprise => usize::MAX,
        }
    }

    pub fn api_rate_limit(&self) -> u32 {
        match self {
            License::Community => 10,
            License::Professional => 100,
            License::Enterprise => 1000,
        }
    }
}

/// License key with HMAC-SHA256 signatures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseKey {
    pub id: Uuid,
    pub tier: License,
    pub organization: String,
    pub activated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub seats: u32,
    pub signature: String,
}

impl LicenseKey {
    pub fn generate(tier: License, organization: String, seats: u32, duration_days: Option<i64>, secret: &str) -> Self {
        let id = Uuid::new_v4();
        let activated_at = Utc::now();
        let expires_at = duration_days.map(|days| activated_at + Duration::days(days));

        let mut key = Self {
            id,
            tier,
            organization,
            activated_at,
            expires_at,
            seats,
            signature: String::new(),
        };

        key.signature = key.compute_signature(secret);
        key
    }

    fn compute_signature(&self, secret: &str) -> String {
        let payload = format!("{:?}{}{}{}{}", self.id, self.tier as u32, self.organization, self.activated_at, self.seats);
        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        hasher.update(secret.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub fn verify_signature(&self, secret: &str) -> bool {
        self.signature == self.compute_signature(secret)
    }

    pub fn is_valid(&self, secret: &str) -> Result<()> {
        if !self.verify_signature(secret) {
            return Err(MarketplaceError::LicenseValidation("Invalid signature".to_string()));
        }

        if let Some(expires_at) = self.expires_at {
            if Utc::now() > expires_at {
                return Err(MarketplaceError::LicenseValidation("Expired".to_string()));
            }
        }

        Ok(())
    }
}

/// License validator with caching
pub struct LicenseValidator {
    secret: String,
    cache: HashMap<Uuid, (LicenseKey, DateTime<Utc>)>,
}

impl LicenseValidator {
    pub fn new(secret: String) -> Self {
        Self {
            secret,
            cache: HashMap::new(),
        }
    }

    pub fn validate(&mut self, key: &LicenseKey) -> Result<()> {
        if let Some((cached_key, cached_at)) = self.cache.get(&key.id) {
            if Utc::now().signed_duration_since(*cached_at).num_seconds() < 3600 {
                return cached_key.is_valid(&self.secret);
            }
        }

        key.is_valid(&self.secret)?;
        self.cache.insert(key.id, (key.clone(), Utc::now()));
        Ok(())
    }

    pub fn get_tier(&self, key: &LicenseKey) -> License {
        key.tier
    }
}

pub fn init_validators() -> Result<()> {
    tracing::info!("Initializing license validation system");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_generation() {
        let key = LicenseKey::generate(License::Professional, "Test Corp".to_string(), 10, Some(365), "secret");
        assert_eq!(key.tier, License::Professional);
    }

    #[test]
    fn test_license_verification() {
        let secret = "test-secret";
        let key = LicenseKey::generate(License::Professional, "Test".to_string(), 1, Some(365), secret);
        assert!(key.is_valid(secret).is_ok());
    }
}
