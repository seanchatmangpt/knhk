#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Rate limiting for workflow operations

use crate::error::{WorkflowError, WorkflowResult};
use governor::{
    clock::DefaultClock,
    state::keyed::DefaultKeyedStateStore,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter,
};
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum number of requests per time window
    pub max_requests: u32,
    /// Time window duration in seconds
    pub window_seconds: u64,
    /// Burst size (allows short bursts above sustained rate)
    pub burst_size: Option<u32>,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_seconds: 60,
            burst_size: None,
        }
    }
}

/// Rate limiter for workflow operations
pub struct RateLimiter {
    limiter: Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    name: String,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(name: String, config: RateLimitConfig) -> WorkflowResult<Self> {
        let max_requests = NonZeroU32::new(config.max_requests)
            .ok_or_else(|| WorkflowError::Validation("max_requests must be > 0".to_string()))?;

        let quota = if let Some(burst) = config.burst_size {
            let burst = NonZeroU32::new(burst)
                .ok_or_else(|| WorkflowError::Validation("burst_size must be > 0".to_string()))?;
            Quota::with_period(Duration::from_secs(config.window_seconds))
                .ok_or_else(|| WorkflowError::Validation("window_seconds must be > 0".to_string()))?
                .allow_burst(burst)
        } else {
            Quota::with_period(Duration::from_secs(config.window_seconds))
                .ok_or_else(|| WorkflowError::Validation("window_seconds must be > 0".to_string()))?
                .allow_burst(max_requests)
        };

        let limiter = Arc::new(GovernorRateLimiter::direct(quota));

        Ok(Self { limiter, name })
    }

    /// Check if a request is allowed (non-blocking)
    pub fn check(&self) -> WorkflowResult<()> {
        self.limiter.check().map_err(|_| {
            WorkflowError::ResourceUnavailable(format!("Rate limit exceeded for {}", self.name))
        })
    }

    /// Wait until a request is allowed (blocking)
    pub async fn wait(&self) -> WorkflowResult<()> {
        loop {
            match self.limiter.check() {
                Ok(_) => return Ok(()),
                Err(_negative) => {
                    // FUTURE: Use wait_time_from with QuantaInstant when available
                    // For now, use a fixed delay based on rate limit window
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Get rate limiter name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Keyed rate limiter for per-workflow/per-pattern rate limiting
pub struct KeyedRateLimiter<K>
where
    K: std::hash::Hash + Eq + Clone + Send + Sync + 'static + std::fmt::Debug,
{
    limiter: Arc<GovernorRateLimiter<K, DefaultKeyedStateStore<K>, DefaultClock>>,
    name: String,
}

impl<K: std::hash::Hash + Eq + Clone + Send + Sync + std::fmt::Debug + 'static>
    KeyedRateLimiter<K>
{
    /// Create a new keyed rate limiter
    pub fn new(name: String, config: RateLimitConfig) -> WorkflowResult<Self> {
        let max_requests = NonZeroU32::new(config.max_requests)
            .ok_or_else(|| WorkflowError::Validation("max_requests must be > 0".to_string()))?;

        let quota = if let Some(burst) = config.burst_size {
            let burst = NonZeroU32::new(burst)
                .ok_or_else(|| WorkflowError::Validation("burst_size must be > 0".to_string()))?;
            Quota::with_period(Duration::from_secs(config.window_seconds))
                .ok_or_else(|| WorkflowError::Validation("window_seconds must be > 0".to_string()))?
                .allow_burst(burst)
        } else {
            Quota::with_period(Duration::from_secs(config.window_seconds))
                .ok_or_else(|| WorkflowError::Validation("window_seconds must be > 0".to_string()))?
                .allow_burst(max_requests)
        };

        let limiter = Arc::new(GovernorRateLimiter::keyed(quota));

        Ok(Self { limiter, name })
    }

    /// Check if a request is allowed for a key
    pub fn check_key(&self, key: &K) -> WorkflowResult<()> {
        self.limiter.check_key(key).map_err(|_| {
            WorkflowError::ResourceUnavailable(format!(
                "Rate limit exceeded for {} (key: {:?})",
                self.name, key
            ))
        })
    }

    /// Wait until a request is allowed for a key
    pub async fn wait_key(&self, key: &K) -> WorkflowResult<()> {
        loop {
            match self.limiter.check_key(key) {
                Ok(_) => return Ok(()),
                Err(_negative) => {
                    // FUTURE: Use negative.wait_time_from() when governor API is available
                    // For now, use a fixed delay
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let config = RateLimitConfig::default();
        let limiter = RateLimiter::new("test".to_string(), config);
        assert!(limiter.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_allows_requests() {
        let config = RateLimitConfig {
            max_requests: 10,
            window_seconds: 1,
            burst_size: None,
        };
        let limiter = RateLimiter::new("test".to_string(), config).unwrap();

        // Should allow initial requests
        for _ in 0..10 {
            assert!(limiter.check().is_ok());
        }

        // Should reject after limit
        assert!(limiter.check().is_err());
    }
}
