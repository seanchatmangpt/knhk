//! Thermal Scaling for Resource Scheduling
//!
//! Implements TRIZ Principle 37: Thermal Expansion
//! - Scale execution resources based on "temperature" (load)
//! - Resources expand/contract with workload temperature
//!
//! Based on: org.yawlfoundation.yawl.scheduling.SchedulingService

use crate::error::{WorkflowError, WorkflowResult};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Load temperature (0.0 = cold, 1.0 = hot)
#[derive(Debug, Clone, Copy)]
pub struct LoadTemperature(f64);

impl LoadTemperature {
    /// Create from workload metrics
    pub fn from_workload(active_tasks: u32, max_capacity: u32, queue_length: u32) -> Self {
        let task_ratio = active_tasks as f64 / max_capacity.max(1) as f64;
        let queue_ratio = (queue_length as f64 / 100.0).min(1.0);
        let temperature = (task_ratio * 0.7 + queue_ratio * 0.3).min(1.0);
        Self(temperature)
    }

    /// Get temperature value
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Check if hot (temperature > 0.8)
    pub fn is_hot(&self) -> bool {
        self.0 > 0.8
    }

    /// Check if cold (temperature < 0.2)
    pub fn is_cold(&self) -> bool {
        self.0 < 0.2
    }
}

/// Resource pool with thermal scaling (TRIZ Principle 37: Thermal Expansion)
pub struct ThermalResourcePool {
    /// Current load temperature
    temperature: Arc<Mutex<f64>>, // Using Mutex as AtomicF64 not available in stable Rust
    /// Base pool size
    base_size: u32,
    /// Current pool size (expands/contracts with temperature)
    current_size: Arc<AtomicU32>,
    /// Maximum pool size
    max_size: u32,
    /// Minimum pool size
    min_size: u32,
    /// Active resources
    active_resources: Arc<RwLock<u32>>,
    /// Queue length
    queue_length: Arc<AtomicU32>,
}

impl ThermalResourcePool {
    /// Create a new thermal resource pool
    pub fn new(base_size: u32, min_size: u32, max_size: u32) -> Self {
        Self {
            temperature: Arc::new(Mutex::new(0.0)),
            base_size,
            current_size: Arc::new(AtomicU32::new(base_size)),
            max_size,
            min_size,
            active_resources: Arc::new(RwLock::new(0)),
            queue_length: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Update temperature and scale resources (TRIZ Principle 37: Thermal Expansion)
    pub async fn update_temperature(&self) -> WorkflowResult<()> {
        let active = *self.active_resources.read().await;
        let queue = self.queue_length.load(Ordering::Relaxed);
        let current_size = self.current_size.load(Ordering::Relaxed);

        let temperature = LoadTemperature::from_workload(active, current_size, queue);
        *self.temperature.lock().unwrap() = temperature.value();

        // Scale resources based on temperature (TRIZ Principle 37)
        let new_size = if temperature.is_hot() {
            // Expand: increase pool size
            (current_size as f64 * 1.5).min(self.max_size as f64) as u32
        } else if temperature.is_cold() {
            // Contract: decrease pool size
            (current_size as f64 * 0.8).max(self.min_size as f64) as u32
        } else {
            // Maintain current size
            current_size
        };

        self.current_size.store(new_size, Ordering::Relaxed);

        tracing::debug!(
            "Thermal scaling: temperature={:.2}, size={}->{}",
            temperature.value(),
            current_size,
            new_size
        );

        Ok(())
    }

    /// Get current temperature
    pub fn get_temperature(&self) -> LoadTemperature {
        LoadTemperature(*self.temperature.lock().unwrap())
    }

    /// Get current pool size
    pub fn get_pool_size(&self) -> u32 {
        self.current_size.load(Ordering::Relaxed)
    }

    /// Increment active resources
    pub async fn acquire_resource(&self) -> WorkflowResult<()> {
        let mut active = self.active_resources.write().await;
        *active += 1;
        self.update_temperature().await?;
        Ok(())
    }

    /// Decrement active resources
    pub async fn release_resource(&self) -> WorkflowResult<()> {
        let mut active = self.active_resources.write().await;
        if *active > 0 {
            *active -= 1;
        }
        self.update_temperature().await?;
        Ok(())
    }

    /// Update queue length
    pub async fn update_queue(&self, length: u32) -> WorkflowResult<()> {
        self.queue_length.store(length, Ordering::Relaxed);
        self.update_temperature().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_thermal_scaling() {
        let pool = ThermalResourcePool::new(10, 5, 20);

        // Initially cold
        let temp = pool.get_temperature();
        assert!(temp.is_cold() || temp.value() < 0.5);

        // Simulate high load
        for _ in 0..15 {
            pool.acquire_resource().await.unwrap();
        }
        pool.update_queue(50).await.unwrap();

        // Should be hot and expanded
        let temp = pool.get_temperature();
        assert!(temp.is_hot() || temp.value() > 0.5);
        assert!(pool.get_pool_size() >= 10);
    }
}

