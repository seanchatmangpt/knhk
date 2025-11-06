// knhk-sidecar: Health check endpoints

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::error::SidecarError;

/// Health status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Degraded,
}

/// Component health
#[derive(Debug, Clone)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    pub message: String,
    pub last_check: Instant,
}

impl ComponentHealth {
    pub fn new(name: String) -> Self {
        Self {
            name,
            status: HealthStatus::Healthy,
            message: "OK".to_string(),
            last_check: Instant::now(),
        }
    }

    pub fn update(&mut self, status: HealthStatus, message: String) {
        self.status = status;
        self.message = message;
        self.last_check = Instant::now();
    }
}

/// Health checker
pub struct HealthChecker {
    components: Arc<Mutex<std::collections::HashMap<String, ComponentHealth>>>,
    server_start_time: Instant,
}

impl HealthChecker {
    /// Create new health checker
    pub fn new() -> Self {
        Self {
            components: Arc::new(Mutex::new(std::collections::HashMap::new())),
            server_start_time: Instant::now(),
        }
    }

    /// Register component
    pub fn register_component(&self, name: String) {
        let mut components = self.components.lock().unwrap();
        components.insert(name.clone(), ComponentHealth::new(name));
    }

    /// Update component health
    pub fn update_component(&self, name: &str, status: HealthStatus, message: String) {
        let mut components = self.components.lock().unwrap();
        if let Some(component) = components.get_mut(name) {
            component.update(status, message);
        }
    }

    /// Check liveness (server is running)
    pub fn check_liveness(&self) -> (bool, String) {
        let uptime = self.server_start_time.elapsed();
        (
            true,
            format!("Server is running (uptime: {:?})", uptime)
        )
    }

    /// Check readiness (can connect to warm orchestrator)
    pub fn check_readiness(&self) -> (bool, String) {
        let components = self.components.lock().unwrap();
        
        // Check if warm orchestrator component exists and is healthy
        if let Some(warm_orch) = components.get("warm_orchestrator") {
            match warm_orch.status {
                HealthStatus::Healthy => (true, "Ready".to_string()),
                HealthStatus::Degraded => (true, "Degraded".to_string()),
                HealthStatus::Unhealthy => (false, format!("Warm orchestrator unhealthy: {}", warm_orch.message)),
            }
        } else {
            // If component not registered, assume ready
            (true, "Ready (warm orchestrator not checked)".to_string())
        }
    }

    /// Get overall health status
    pub fn get_health_status(&self) -> HealthStatus {
        let components = self.components.lock().unwrap();
        
        let mut has_unhealthy = false;
        let mut has_degraded = false;

        for component in components.values() {
            match component.status {
                HealthStatus::Unhealthy => {
                    has_unhealthy = true;
                }
                HealthStatus::Degraded => {
                    has_degraded = true;
                }
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

    /// Get component statuses
    pub fn get_component_statuses(&self) -> std::collections::HashMap<String, bool> {
        let components = self.components.lock().unwrap();
        components
            .iter()
            .map(|(name, component)| {
                (name.clone(), component.status == HealthStatus::Healthy)
            })
            .collect()
    }

    /// Get detailed health information
    pub fn get_health_info(&self) -> (bool, String, std::collections::HashMap<String, bool>) {
        let status = self.get_health_status();
        let healthy = status == HealthStatus::Healthy || status == HealthStatus::Degraded;
        let message = match status {
            HealthStatus::Healthy => "All components healthy".to_string(),
            HealthStatus::Degraded => "Some components degraded".to_string(),
            HealthStatus::Unhealthy => "Some components unhealthy".to_string(),
        };
        let components = self.get_component_statuses();
        (healthy, message, components)
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new()
    }
}

