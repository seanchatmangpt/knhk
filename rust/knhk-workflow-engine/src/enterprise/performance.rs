//! Enterprise Performance
//!
//! Provides performance optimization for Fortune 5 deployments:
//! - Hot path optimization (≤8 ticks)
//! - SIMD support
//! - Caching strategies
//! - Performance monitoring

use crate::error::WorkflowResult;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Performance configuration
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Hot path tick budget (≤8 ticks = 2ns)
    pub hot_path_tick_budget: u32,
    /// Enable SIMD optimization
    pub enable_simd: bool,
    /// Enable caching
    pub enable_caching: bool,
    /// Cache TTL (seconds)
    pub cache_ttl: u64,
    /// Performance monitoring interval (seconds)
    pub monitoring_interval: u64,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            hot_path_tick_budget: 8,
            enable_simd: true,
            enable_caching: true,
            cache_ttl: 300,
            monitoring_interval: 60,
        }
    }
}

/// Cache entry
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

/// Performance manager for workflow engine
pub struct PerformanceManager {
    config: PerformanceConfig,
    cache: Arc<RwLock<HashMap<String, CacheEntry<String>>>>,
    metrics: Arc<RwLock<HashMap<String, Vec<Duration>>>>,
}

impl PerformanceManager {
    /// Create new performance manager
    pub fn new(config: PerformanceConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get from cache
    pub async fn get_cache(&self, key: &str) -> Option<String> {
        if !self.config.enable_caching {
            return None;
        }

        let cache = self.cache.read().await;
        if let Some(entry) = cache.get(key) {
            if entry.expires_at > Instant::now() {
                return Some(entry.value.clone());
            }
        }
        None
    }

    /// Set cache
    pub async fn set_cache(&self, key: String, value: String) {
        if !self.config.enable_caching {
            return;
        }

        let mut cache = self.cache.write().await;
        cache.insert(
            key,
            CacheEntry {
                value,
                expires_at: Instant::now() + Duration::from_secs(self.config.cache_ttl),
            },
        );
    }

    /// Record performance metric
    pub async fn record_metric(&self, operation: &str, duration: Duration) {
        let mut metrics = self.metrics.write().await;
        let operation_metrics = metrics
            .entry(operation.to_string())
            .or_insert_with(Vec::new);
        operation_metrics.push(duration);

        // Keep only last 1000 measurements
        if operation_metrics.len() > 1000 {
            operation_metrics.remove(0);
        }
    }

    /// Check if operation is within hot path budget
    pub fn is_within_budget(&self, ticks: u32) -> bool {
        ticks <= self.config.hot_path_tick_budget
    }

    /// Get average latency for operation
    pub async fn get_avg_latency(&self, operation: &str) -> Option<Duration> {
        let metrics = self.metrics.read().await;
        if let Some(operation_metrics) = metrics.get(operation) {
            if operation_metrics.is_empty() {
                return None;
            }
            let sum: Duration = operation_metrics.iter().sum();
            Some(sum / operation_metrics.len() as u32)
        } else {
            None
        }
    }
}
