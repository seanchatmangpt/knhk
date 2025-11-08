#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Performance monitoring for workflow engine

use crate::resilience::PathType;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Performance metrics
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Hot path samples (nanoseconds)
    pub hot_path_samples: VecDeque<u64>,
    /// Warm path samples (microseconds)
    pub warm_path_samples: VecDeque<u64>,
    /// Cold path samples (milliseconds)
    pub cold_path_samples: VecDeque<u64>,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Cache miss rate
    pub cache_miss_rate: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            hot_path_samples: VecDeque::with_capacity(1000),
            warm_path_samples: VecDeque::with_capacity(1000),
            cold_path_samples: VecDeque::with_capacity(1000),
            cache_hit_rate: 0.0,
            cache_miss_rate: 0.0,
        }
    }
}

/// Performance monitor
pub struct PerformanceMonitor {
    metrics: Arc<Mutex<PerformanceMetrics>>,
    max_samples: usize,
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(max_samples: usize) -> Self {
        Self {
            metrics: Arc::new(Mutex::new(PerformanceMetrics::default())),
            max_samples,
        }
    }

    /// Record a latency sample
    pub fn record_latency(&self, path_type: PathType, latency: Duration) {
        let mut metrics = self.metrics.lock().unwrap();

        let _sample = match path_type {
            PathType::Hot => {
                let ns = latency.as_nanos() as u64;
                metrics.hot_path_samples.push_back(ns);
                while metrics.hot_path_samples.len() > self.max_samples {
                    metrics.hot_path_samples.pop_front();
                }
                ns
            }
            PathType::Warm => {
                let us = latency.as_micros() as u64;
                metrics.warm_path_samples.push_back(us);
                while metrics.warm_path_samples.len() > self.max_samples {
                    metrics.warm_path_samples.pop_front();
                }
                us
            }
            PathType::Cold => {
                let ms = latency.as_millis() as u64;
                metrics.cold_path_samples.push_back(ms);
                while metrics.cold_path_samples.len() > self.max_samples {
                    metrics.cold_path_samples.pop_front();
                }
                ms
            }
        };
    }

    /// Record cache hit
    pub fn record_cache_hit(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        // Simple moving average for cache hit rate
        let total = metrics.cache_hit_rate + metrics.cache_miss_rate;
        if total > 0.0 {
            metrics.cache_hit_rate = (metrics.cache_hit_rate * 0.9) + (1.0 * 0.1);
            metrics.cache_miss_rate = metrics.cache_miss_rate * 0.9;
        } else {
            metrics.cache_hit_rate = 1.0;
        }
    }

    /// Record cache miss
    pub fn record_cache_miss(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        let total = metrics.cache_hit_rate + metrics.cache_miss_rate;
        if total > 0.0 {
            metrics.cache_hit_rate = metrics.cache_hit_rate * 0.9;
            metrics.cache_miss_rate = (metrics.cache_miss_rate * 0.9) + (1.0 * 0.1);
        } else {
            metrics.cache_miss_rate = 1.0;
        }
    }

    /// Get P99 latency for a path type
    pub fn get_p99_latency(&self, path_type: PathType) -> Option<Duration> {
        let metrics = self.metrics.lock().unwrap();

        let samples: Vec<u64> = match path_type {
            PathType::Hot => metrics.hot_path_samples.iter().copied().collect(),
            PathType::Warm => metrics.warm_path_samples.iter().copied().collect(),
            PathType::Cold => metrics.cold_path_samples.iter().copied().collect(),
        };

        if samples.is_empty() {
            return None;
        }

        let mut sorted = samples;
        sorted.sort();
        let index = (sorted.len() as f64 * 0.99).ceil() as usize - 1;
        let p99_value = sorted.get(index.min(sorted.len() - 1)).copied()?;

        Some(match path_type {
            PathType::Hot => Duration::from_nanos(p99_value),
            PathType::Warm => Duration::from_micros(p99_value),
            PathType::Cold => Duration::from_millis(p99_value),
        })
    }

    /// Get cache hit rate
    pub fn get_cache_hit_rate(&self) -> f64 {
        let metrics = self.metrics.lock().unwrap();
        metrics.cache_hit_rate
    }

    /// Get metrics snapshot
    pub fn get_metrics(&self) -> PerformanceMetrics {
        let metrics = self.metrics.lock().unwrap();
        metrics.clone()
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor() {
        let monitor = PerformanceMonitor::default();

        monitor.record_latency(PathType::Hot, Duration::from_nanos(1));
        monitor.record_latency(PathType::Hot, Duration::from_nanos(2));

        let p99 = monitor.get_p99_latency(PathType::Hot);
        assert!(p99.is_some());
    }
}
