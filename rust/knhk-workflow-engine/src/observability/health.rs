#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Health checks for workflow engine

use crate::error::WorkflowResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Healthy
    Healthy,
    /// Degraded (operational but with issues)
    Degraded,
    /// Unhealthy
    Unhealthy,
}

/// Component health
#[derive(Debug, Clone)]
pub struct ComponentHealth {
    /// Component name
    pub name: String,
    /// Health status
    pub status: HealthStatus,
    /// Last check time (timestamp in seconds since epoch)
    pub last_check_secs: Option<u64>,
    /// Error message (if unhealthy)
    pub error: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Health checker for workflow engine
pub struct HealthChecker {
    components: Arc<Mutex<HashMap<String, ComponentHealth>>>,
    check_interval: Duration,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(check_interval_secs: u64) -> Self {
        Self {
            components: Arc::new(Mutex::new(HashMap::new())),
            check_interval: Duration::from_secs(check_interval_secs),
        }
    }

    /// Register a component
    pub fn register_component(&self, name: String, initial_status: HealthStatus) {
        let mut components = self.components.lock().unwrap();
        components.insert(
            name.clone(),
            ComponentHealth {
                name,
                status: initial_status,
                last_check_secs: Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                ),
                error: None,
                metadata: HashMap::new(),
            },
        );
    }

    /// Update component health
    pub fn update_component(
        &self,
        name: &str,
        status: HealthStatus,
        error: Option<String>,
    ) -> WorkflowResult<()> {
        let mut components = self.components.lock().map_err(|e| {
            crate::error::WorkflowError::Internal(format!("Failed to acquire health lock: {}", e))
        })?;

        if let Some(component) = components.get_mut(name) {
            component.status = status;
            component.last_check_secs = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
            component.error = error;
        } else {
            components.insert(
                name.to_string(),
                ComponentHealth {
                    name: name.to_string(),
                    status,
                    last_check_secs: Some(
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    ),
                    error,
                    metadata: HashMap::new(),
                },
            );
        }

        Ok(())
    }

    /// Get overall health status
    pub fn get_health(&self) -> HealthStatus {
        let components = self.components.lock().unwrap();

        let mut has_unhealthy = false;
        let mut has_degraded = false;

        for component in components.values() {
            match component.status {
                HealthStatus::Unhealthy => has_unhealthy = true,
                HealthStatus::Degraded => has_degraded = true,
                HealthStatus::Healthy => {}
            }
        }

        if has_unhealthy {
            HealthStatus::Unhealthy
        } else if has_degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        }
    }

    /// Get component health
    pub fn get_component_health(&self, name: &str) -> Option<ComponentHealth> {
        let components = self.components.lock().unwrap();
        components.get(name).cloned()
    }

    /// Get all component healths
    pub fn get_all_components(&self) -> Vec<ComponentHealth> {
        let components = self.components.lock().unwrap();
        components.values().cloned().collect()
    }

    /// Check if component needs health check
    pub fn needs_check(&self, name: &str) -> bool {
        let components = self.components.lock().unwrap();
        if let Some(component) = components.get(name) {
            let now_secs = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            component
                .last_check_secs
                .map(|check| now_secs.saturating_sub(check) >= self.check_interval.as_secs())
                .unwrap_or(true)
        } else {
            true
        }
    }

    /// Kubernetes-style readiness probe
    /// Returns true if service is ready to accept traffic
    pub fn readiness_probe(&self) -> WorkflowResult<bool> {
        let health = self.get_health();
        // Ready if healthy or degraded (can still serve requests)
        Ok(matches!(health, HealthStatus::Healthy | HealthStatus::Degraded))
    }

    /// Kubernetes-style liveness probe
    /// Returns true if service is alive (not deadlocked/crashed)
    pub fn liveness_probe(&self) -> WorkflowResult<bool> {
        // Service is live if we can acquire the lock (not deadlocked)
        let _guard = self.components.lock().map_err(|e| {
            crate::error::WorkflowError::Internal(format!("Liveness check failed: {}", e))
        })?;
        Ok(true)
    }

    /// Kubernetes-style startup probe
    /// Returns true if service has finished initialization
    pub fn startup_probe(&self) -> WorkflowResult<bool> {
        let components = self.components.lock().unwrap();

        // Check if critical components are registered and not unhealthy
        let required_components = vec!["state_store", "pattern_registry"];

        for comp_name in required_components {
            if let Some(comp) = components.get(comp_name) {
                if comp.status == HealthStatus::Unhealthy {
                    return Ok(false);
                }
            } else {
                // Critical component not registered yet
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get health details as JSON-serializable map
    pub fn get_health_details(&self) -> HashMap<String, String> {
        let components = self.components.lock().unwrap();
        let mut details = HashMap::new();

        for (name, comp) in components.iter() {
            let status_str = match comp.status {
                HealthStatus::Healthy => "healthy",
                HealthStatus::Degraded => "degraded",
                HealthStatus::Unhealthy => "unhealthy",
            };
            details.insert(name.clone(), status_str.to_string());

            if let Some(ref error) = comp.error {
                details.insert(format!("{}_error", name), error.clone());
            }
        }

        details
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new(5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_checker() {
        let checker = HealthChecker::default();
        checker.register_component("state_store".to_string(), HealthStatus::Healthy);
        checker.register_component("pattern_registry".to_string(), HealthStatus::Healthy);

        assert_eq!(checker.get_health(), HealthStatus::Healthy);

        checker
            .update_component("state_store", HealthStatus::Degraded, None)
            .unwrap();
        assert_eq!(checker.get_health(), HealthStatus::Degraded);
    }
}
