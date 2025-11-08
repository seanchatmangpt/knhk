#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
#![allow(clippy::unwrap_used)] // Supporting infrastructure - unwrap() acceptable for now
//! Load balancing for distributed workflow engine

use crate::error::WorkflowResult;
use std::sync::{Arc, Mutex};

/// Load balance strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadBalanceStrategy {
    /// Round-robin
    RoundRobin,
    /// Least connections
    LeastConnections,
    /// Health-based
    HealthBased,
    /// Random
    Random,
}

/// Backend instance
#[derive(Debug, Clone)]
pub struct Backend {
    /// Backend ID
    pub id: String,
    /// Backend address
    pub address: String,
    /// Active connections
    pub connections: usize,
    /// Health status
    pub healthy: bool,
    /// Last health check timestamp (milliseconds since epoch)
    pub last_health_check_ms: Option<u64>,
}

/// Load balancer
pub struct LoadBalancer {
    backends: Arc<Mutex<Vec<Backend>>>,
    strategy: LoadBalanceStrategy,
    current_index: Arc<Mutex<usize>>,
}

impl LoadBalancer {
    /// Create a new load balancer
    pub fn new(strategy: LoadBalanceStrategy) -> Self {
        Self {
            backends: Arc::new(Mutex::new(Vec::new())),
            strategy,
            current_index: Arc::new(Mutex::new(0)),
        }
    }

    /// Add a backend
    pub fn add_backend(&self, backend: Backend) {
        let mut backends = self.backends.lock().unwrap();
        backends.push(backend);
    }

    /// Remove a backend
    pub fn remove_backend(&self, id: &str) {
        let mut backends = self.backends.lock().unwrap();
        backends.retain(|b| b.id != id);
    }

    /// Select a backend
    pub fn select_backend(&self) -> WorkflowResult<Option<Backend>> {
        let backends = self.backends.lock().map_err(|e| {
            crate::error::WorkflowError::Internal(format!("Failed to acquire backend lock: {}", e))
        })?;

        if backends.is_empty() {
            return Ok(None);
        }

        // Filter to healthy backends only
        let healthy_backends: Vec<Backend> =
            backends.iter().filter(|b| b.healthy).cloned().collect();

        if healthy_backends.is_empty() {
            return Ok(None);
        }

        let selected = match self.strategy {
            LoadBalanceStrategy::RoundRobin => {
                let mut index = self.current_index.lock().unwrap();
                let backend = healthy_backends[*index % healthy_backends.len()].clone();
                *index = (*index + 1) % healthy_backends.len();
                backend
            }
            LoadBalanceStrategy::LeastConnections => healthy_backends
                .iter()
                .min_by_key(|b| b.connections)
                .cloned()
                .unwrap(),
            LoadBalanceStrategy::HealthBased => {
                // Select healthiest backend (most recent health check)
                healthy_backends
                    .iter()
                    .max_by_key(|b| b.last_health_check_ms)
                    .cloned()
                    .unwrap()
            }
            LoadBalanceStrategy::Random => {
                let index = fastrand::usize(..healthy_backends.len());
                healthy_backends[index].clone()
            }
        };

        Ok(Some(selected))
    }

    /// Update backend health
    pub fn update_backend_health(&self, id: &str, healthy: bool) {
        let mut backends = self.backends.lock().unwrap();
        if let Some(backend) = backends.iter_mut().find(|b| b.id == id) {
            backend.healthy = healthy;
            backend.last_health_check_ms = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            );
        }
    }

    /// Increment backend connections
    pub fn increment_connections(&self, id: &str) {
        let mut backends = self.backends.lock().unwrap();
        if let Some(backend) = backends.iter_mut().find(|b| b.id == id) {
            backend.connections += 1;
        }
    }

    /// Decrement backend connections
    pub fn decrement_connections(&self, id: &str) {
        let mut backends = self.backends.lock().unwrap();
        if let Some(backend) = backends.iter_mut().find(|b| b.id == id) {
            backend.connections = backend.connections.saturating_sub(1);
        }
    }
}

impl Default for LoadBalancer {
    fn default() -> Self {
        Self::new(LoadBalanceStrategy::RoundRobin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_balancer() {
        let balancer = LoadBalancer::new(LoadBalanceStrategy::RoundRobin);

        balancer.add_backend(Backend {
            id: "backend-1".to_string(),
            address: "127.0.0.1:8080".to_string(),
            connections: 0,
            healthy: true,
            last_health_check_ms: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            ),
        });

        let backend = balancer.select_backend().unwrap();
        assert!(backend.is_some());
    }
}
