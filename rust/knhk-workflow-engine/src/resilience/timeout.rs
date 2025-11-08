#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Timeout management for workflow operations

use crate::error::{WorkflowError, WorkflowResult};
use std::time::Duration;
use tokio::time::{timeout, Timeout};

/// Timeout configuration for different path types
#[derive(Debug, Clone)]
pub struct TimeoutConfig {
    /// Hot path timeout (≤8 ticks = 2ns)
    pub hot_path_ns: u64,
    /// Warm path timeout (≤500 µs = 1ms p99)
    pub warm_path_us: u64,
    /// Cold path timeout (≤200 ms = 500ms p99)
    pub cold_path_ms: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            hot_path_ns: 2,
            warm_path_us: 500,
            cold_path_ms: 200,
        }
    }
}

/// Path type for timeout routing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PathType {
    /// Hot path: ≤8 ticks (2ns)
    Hot,
    /// Warm path: ≤500 µs (1ms p99)
    Warm,
    /// Cold path: ≤200 ms (500ms p99)
    Cold,
}

/// Timeout manager for workflow operations
pub struct TimeoutManager {
    config: TimeoutConfig,
}

impl TimeoutManager {
    /// Create a new timeout manager
    pub fn new(config: TimeoutConfig) -> Self {
        Self { config }
    }

    /// Execute a future with timeout based on path type
    pub async fn execute_with_timeout<F, T>(
        &self,
        path_type: PathType,
        future: F,
    ) -> WorkflowResult<T>
    where
        F: std::future::Future<Output = WorkflowResult<T>>,
    {
        let duration = match path_type {
            PathType::Hot => Duration::from_nanos(self.config.hot_path_ns),
            PathType::Warm => Duration::from_micros(self.config.warm_path_us),
            PathType::Cold => Duration::from_millis(self.config.cold_path_ms),
        };

        match timeout(duration, future).await {
            Ok(result) => result,
            Err(_) => Err(WorkflowError::Timeout),
        }
    }

    /// Get timeout duration for a path type
    pub fn timeout_for(&self, path_type: PathType) -> Duration {
        match path_type {
            PathType::Hot => Duration::from_nanos(self.config.hot_path_ns),
            PathType::Warm => Duration::from_micros(self.config.warm_path_us),
            PathType::Cold => Duration::from_millis(self.config.cold_path_ms),
        }
    }
}

impl Default for TimeoutManager {
    fn default() -> Self {
        Self::new(TimeoutConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_timeout_hot_path() {
        let manager = TimeoutManager::default();

        let result = manager
            .execute_with_timeout(PathType::Hot, async {
                sleep(Duration::from_nanos(10)).await;
                Ok(42)
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WorkflowError::Timeout));
    }

    #[tokio::test]
    async fn test_timeout_warm_path_success() {
        let manager = TimeoutManager::default();

        let result = manager
            .execute_with_timeout(PathType::Warm, async { Ok(42) })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }
}
