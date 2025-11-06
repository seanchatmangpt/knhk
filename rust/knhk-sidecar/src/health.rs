// rust/knhk-sidecar/src/health.rs
// Health check implementation

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

pub struct HealthChecker {
    status: Arc<Mutex<HealthStatus>>,
    last_check: Arc<Mutex<Option<Instant>>>,
    check_interval: Duration,
}

impl HealthChecker {
    pub fn new(check_interval_ms: u64) -> Self {
        Self {
            status: Arc::new(Mutex::new(HealthStatus::Healthy)),
            last_check: Arc::new(Mutex::new(None)),
            check_interval: Duration::from_millis(check_interval_ms),
        }
    }

    pub fn status(&self) -> HealthStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn set_healthy(&self) {
        *self.status.lock().unwrap() = HealthStatus::Healthy;
    }

    pub fn set_degraded(&self, reason: String) {
        *self.status.lock().unwrap() = HealthStatus::Degraded(reason);
    }

    pub fn set_unhealthy(&self, reason: String) {
        *self.status.lock().unwrap() = HealthStatus::Unhealthy(reason);
    }

    pub async fn check(&self) -> HealthStatus {
        let now = Instant::now();
        let mut last_check = self.last_check.lock().unwrap();

        if let Some(last) = *last_check {
            if now.duration_since(last) < self.check_interval {
                return self.status();
            }
        }

        *last_check = Some(now);

        // Perform health checks
        // For now, just return current status
        // In production, this would check:
        // - ETL pipeline health
        // - Warm path availability
        // - Hook registry availability
        // - Circuit breaker states

        self.status()
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new(5000) // 5 second check interval
    }
}

