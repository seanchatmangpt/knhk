// knhk-sidecar: Metrics collection and OTEL integration

// ACCEPTABLE: Mutex poisoning .expect() is allowed in this module (unrecoverable error)
#![allow(clippy::expect_used)]

use crate::error::SidecarError;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Request metrics
#[derive(Debug, Clone, Default)]
pub struct RequestMetrics {
    pub total: u64,
    pub success: u64,
    pub failure: u64,
}

/// Latency metrics
#[derive(Debug, Clone)]
pub struct LatencyMetrics {
    pub p50_ms: u64,
    pub p95_ms: u64,
    pub p99_ms: u64,
}

impl Default for LatencyMetrics {
    fn default() -> Self {
        Self {
            p50_ms: 0,
            p95_ms: 0,
            p99_ms: 0,
        }
    }
}

/// Batch metrics
#[derive(Debug, Clone, Default)]
pub struct BatchMetrics {
    pub total_batches: u64,
    pub avg_batch_size: f64,
    pub max_batch_size: usize,
}

/// Circuit breaker metrics
#[derive(Debug, Clone)]
pub struct CircuitBreakerMetrics {
    pub state: String,
    pub failure_count: u32,
    pub success_count: u32,
}

impl Default for CircuitBreakerMetrics {
    fn default() -> Self {
        Self {
            state: "closed".to_string(),
            failure_count: 0,
            success_count: 0,
        }
    }
}

/// Retry metrics
#[derive(Debug, Clone, Default)]
pub struct RetryMetrics {
    pub total_retries: u64,
    pub successful_retries: u64,
    pub failed_retries: u64,
}

/// Complete metrics snapshot
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub requests: RequestMetrics,
    pub latency: LatencyMetrics,
    pub batch: BatchMetrics,
    pub circuit_breaker: CircuitBreakerMetrics,
    pub retry: RetryMetrics,
}

impl Default for MetricsSnapshot {
    fn default() -> Self {
        Self {
            requests: RequestMetrics::default(),
            latency: LatencyMetrics::default(),
            batch: BatchMetrics::default(),
            circuit_breaker: CircuitBreakerMetrics::default(),
            retry: RetryMetrics::default(),
        }
    }
}

/// Metrics collector
pub struct MetricsCollector {
    requests: Arc<Mutex<RequestMetrics>>,
    latencies: Arc<Mutex<VecDeque<u64>>>,
    batch_sizes: Arc<Mutex<VecDeque<usize>>>,
    circuit_breaker: Arc<Mutex<CircuitBreakerMetrics>>,
    retry: Arc<Mutex<RetryMetrics>>,
    max_latency_samples: usize,
}

impl MetricsCollector {
    /// Create new metrics collector
    pub fn new(max_samples: usize) -> Self {
        Self {
            requests: Arc::new(Mutex::new(RequestMetrics::default())),
            latencies: Arc::new(Mutex::new(VecDeque::with_capacity(max_samples))),
            batch_sizes: Arc::new(Mutex::new(VecDeque::with_capacity(max_samples))),
            circuit_breaker: Arc::new(Mutex::new(CircuitBreakerMetrics::default())),
            retry: Arc::new(Mutex::new(RetryMetrics::default())),
            max_latency_samples: max_samples,
        }
    }

    /// Record request
    pub fn record_request(&self, success: bool) {
        // ACCEPTABLE: Mutex poisoning is an unrecoverable error. Panicking is appropriate.
        // See Rust std docs: https://doc.rust-lang.org/std/sync/struct.Mutex.html#poisoning
        let mut metrics = self
            .requests
            .lock()
            .expect("Metrics mutex poisoned - unrecoverable state");
        metrics.total += 1;
        if success {
            metrics.success += 1;
        } else {
            metrics.failure += 1;
        }
    }

    /// Record latency
    pub fn record_latency(&self, latency_ms: u64) {
        let mut latencies = self
            .latencies
            .lock()
            .expect("Latency metrics mutex poisoned - unrecoverable state");
        latencies.push_back(latency_ms);
        if latencies.len() > self.max_latency_samples {
            latencies.pop_front();
        }
    }

    /// Record batch size
    pub fn record_batch_size(&self, size: usize) {
        let mut batch_sizes = self
            .batch_sizes
            .lock()
            .expect("Batch metrics mutex poisoned - unrecoverable state");
        batch_sizes.push_back(size);
        if batch_sizes.len() > self.max_latency_samples {
            batch_sizes.pop_front();
        }
    }

    /// Update circuit breaker metrics
    pub fn update_circuit_breaker(&self, state: String, failure_count: u32, success_count: u32) {
        let mut cb = self
            .circuit_breaker
            .lock()
            .expect("Circuit breaker metrics mutex poisoned - unrecoverable state");
        cb.state = state;
        cb.failure_count = failure_count;
        cb.success_count = success_count;
    }

    /// Record retry
    pub fn record_retry(&self, success: bool) {
        let mut retry = self
            .retry
            .lock()
            .expect("Retry metrics mutex poisoned - unrecoverable state");
        retry.total_retries += 1;
        if success {
            retry.successful_retries += 1;
        } else {
            retry.failed_retries += 1;
        }
    }

    /// Get metrics snapshot
    pub fn snapshot(&self) -> MetricsSnapshot {
        let requests = self
            .requests
            .lock()
            .expect("Metrics mutex poisoned - unrecoverable state")
            .clone();

        // Calculate latency percentiles
        let mut latencies = self
            .latencies
            .lock()
            .expect("Latency metrics mutex poisoned - unrecoverable state")
            .clone();
        let mut latency_vec: Vec<u64> = latencies.iter().cloned().collect();
        latency_vec.sort();

        let latency = if latency_vec.is_empty() {
            LatencyMetrics::default()
        } else {
            let p50_idx = (latency_vec.len() as f64 * 0.5) as usize;
            let p95_idx = (latency_vec.len() as f64 * 0.95) as usize;
            let p99_idx = (latency_vec.len() as f64 * 0.99) as usize;

            LatencyMetrics {
                p50_ms: latency_vec.get(p50_idx).copied().unwrap_or(0),
                p95_ms: latency_vec.get(p95_idx).copied().unwrap_or(0),
                p99_ms: latency_vec.get(p99_idx).copied().unwrap_or(0),
            }
        };

        // Calculate batch metrics
        let batch_sizes = self
            .batch_sizes
            .lock()
            .expect("Batch metrics mutex poisoned - unrecoverable state")
            .clone();
        let batch_vec: Vec<usize> = batch_sizes.iter().cloned().collect();

        let batch = if batch_vec.is_empty() {
            BatchMetrics::default()
        } else {
            let sum: usize = batch_vec.iter().sum();
            let avg = sum as f64 / batch_vec.len() as f64;
            let max = batch_vec.iter().max().copied().unwrap_or(0);

            BatchMetrics {
                total_batches: batch_vec.len() as u64,
                avg_batch_size: avg,
                max_batch_size: max,
            }
        };

        let circuit_breaker = self
            .circuit_breaker
            .lock()
            .expect("Circuit breaker metrics mutex poisoned - unrecoverable state")
            .clone();
        let retry = self
            .retry
            .lock()
            .expect("Retry metrics mutex poisoned - unrecoverable state")
            .clone();

        MetricsSnapshot {
            requests,
            latency,
            batch,
            circuit_breaker,
            retry,
        }
    }

    /// Reset metrics
    pub fn reset(&self) {
        *self
            .requests
            .lock()
            .expect("Metrics mutex poisoned - unrecoverable state") = RequestMetrics::default();
        *self
            .latencies
            .lock()
            .expect("Latency metrics mutex poisoned - unrecoverable state") = VecDeque::new();
        *self
            .batch_sizes
            .lock()
            .expect("Batch metrics mutex poisoned - unrecoverable state") = VecDeque::new();
        *self
            .circuit_breaker
            .lock()
            .expect("Circuit breaker metrics mutex poisoned - unrecoverable state") =
            CircuitBreakerMetrics::default();
        *self
            .retry
            .lock()
            .expect("Retry metrics mutex poisoned - unrecoverable state") = RetryMetrics::default();
    }
}

/// Latency timer for measuring request duration
pub struct LatencyTimer {
    start: Instant,
    collector: Arc<MetricsCollector>,
}

impl LatencyTimer {
    /// Start new timer
    pub fn start(collector: Arc<MetricsCollector>) -> Self {
        Self {
            start: Instant::now(),
            collector,
        }
    }

    /// Record latency and return duration
    pub fn finish(self) -> Duration {
        let duration = self.start.elapsed();
        self.collector.record_latency(duration.as_millis() as u64);
        duration
    }
}
