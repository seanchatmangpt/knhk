//! CPU/GPU Dispatch Routing
//!
//! Intelligent router that decides whether to execute on CPU or GPU based on:
//! - Operation size and characteristics
//! - Data transfer cost vs. computation cost
//! - Device availability and current utilization
//! - Memory constraints

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Dispatch routing error
#[derive(Error, Debug)]
pub enum DispatchError {
    #[error("No suitable device available")]
    NoDeviceAvailable,

    #[error("Device unavailable: {0}")]
    DeviceUnavailable(String),

    #[error("Memory transfer failed: {0}")]
    MemoryTransferFailed(String),

    #[error("Cost estimation failed: {0}")]
    CostEstimationFailed(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

/// Execution device
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, Hash)]
pub enum ExecutionDevice {
    /// Host CPU
    CPU,
    /// NVIDIA GPU
    CUDA,
    /// AMD GPU
    HIP,
    /// OpenCL device
    OpenCL,
    /// FPGA
    FPGA,
}

impl ExecutionDevice {
    /// Get device name
    pub fn name(&self) -> &'static str {
        match self {
            ExecutionDevice::CPU => "CPU",
            ExecutionDevice::CUDA => "CUDA",
            ExecutionDevice::HIP => "HIP",
            ExecutionDevice::OpenCL => "OpenCL",
            ExecutionDevice::FPGA => "FPGA",
        }
    }

    /// Get estimated memory bandwidth (GB/s)
    pub fn bandwidth_gb_s(&self) -> f32 {
        match self {
            ExecutionDevice::CPU => 40.0,     // DDR4/DDR5
            ExecutionDevice::CUDA => 900.0,   // RTX 4090
            ExecutionDevice::HIP => 576.0,    // MI250X
            ExecutionDevice::OpenCL => 256.0, // Varies
            ExecutionDevice::FPGA => 64.0,    // PCIe Gen4 x16
        }
    }

    /// Get estimated compute throughput (TFLOPS)
    pub fn compute_tflops(&self) -> f32 {
        match self {
            ExecutionDevice::CPU => 0.5,     // Core i9 scalar
            ExecutionDevice::CUDA => 83.0,   // RTX 4090 FP32
            ExecutionDevice::HIP => 47.6,    // MI250X FP32
            ExecutionDevice::OpenCL => 10.0, // Varies
            ExecutionDevice::FPGA => 5.0,    // Estimated
        }
    }
}

/// Operation profile for cost analysis
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperationProfile {
    /// Operation name
    pub name: String,
    /// Input data size (bytes)
    pub input_size: usize,
    /// Output data size (bytes)
    pub output_size: usize,
    /// Arithmetic intensity (FLOPs per byte)
    pub arithmetic_intensity: f32,
    /// Can be parallelized
    pub parallelizable: bool,
    /// Memory access pattern (sequential, random, etc)
    pub access_pattern: AccessPattern,
}

/// Memory access pattern
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AccessPattern {
    /// Sequential access (cache friendly)
    Sequential,
    /// Random access (cache unfriendly)
    Random,
    /// Strided access
    Strided,
    /// Mixed access pattern
    Mixed,
}

/// Device capability information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeviceCapability {
    /// Device type
    pub device: ExecutionDevice,
    /// Available memory (bytes)
    pub available_memory: u64,
    /// Current utilization (0.0 - 1.0)
    pub utilization: f32,
    /// Is device available
    pub available: bool,
    /// Device temperature (Celsius)
    pub temperature: f32,
    /// Power consumption (Watts)
    pub power_usage: f32,
}

/// Dispatch decision with reasoning
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DispatchDecision {
    /// Selected device
    pub device: ExecutionDevice,
    /// Estimated execution time (microseconds)
    pub estimated_time_us: u64,
    /// Host->Device transfer time (microseconds)
    pub h2d_time_us: u64,
    /// Device->Host transfer time (microseconds)
    pub d2h_time_us: u64,
    /// Total time including transfers
    pub total_time_us: u64,
    /// CPU would take (for reference)
    pub cpu_reference_time_us: u64,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
}

/// Device capability tracker
pub struct DeviceTracker {
    capabilities: Arc<RwLock<HashMap<ExecutionDevice, DeviceCapability>>>,
}

impl DeviceTracker {
    /// Create new device tracker
    pub fn new() -> Self {
        let mut capabilities = HashMap::new();

        // Initialize default capabilities
        for device in &[
            ExecutionDevice::CPU,
            ExecutionDevice::CUDA,
            ExecutionDevice::HIP,
            ExecutionDevice::OpenCL,
            ExecutionDevice::FPGA,
        ] {
            capabilities.insert(
                *device,
                DeviceCapability {
                    device: *device,
                    available_memory: 0,
                    utilization: 0.0,
                    available: false, // Will be detected
                    temperature: 0.0,
                    power_usage: 0.0,
                },
            );
        }

        // CPU always available
        if let Some(cpu) = capabilities.get_mut(&ExecutionDevice::CPU) {
            cpu.available = true;
        }

        tracing::info!("Device tracker: initialized");

        Self {
            capabilities: Arc::new(RwLock::new(capabilities)),
        }
    }

    /// Update device capability
    pub fn update(&self, device: ExecutionDevice, capability: DeviceCapability) {
        let mut caps = self.capabilities.write();
        caps.insert(device, capability);
        tracing::debug!("Device tracker: updated {} capability", device.name());
    }

    /// Get device capability
    pub fn get(&self, device: ExecutionDevice) -> Option<DeviceCapability> {
        self.capabilities.read().get(&device).cloned()
    }

    /// Get all available devices
    pub fn available_devices(&self) -> Vec<ExecutionDevice> {
        self.capabilities
            .read()
            .iter()
            .filter(|(_, cap)| cap.available)
            .map(|(dev, _)| *dev)
            .collect()
    }

    /// Check if device is available
    pub fn is_available(&self, device: ExecutionDevice) -> bool {
        self.capabilities
            .read()
            .get(&device)
            .map(|cap| cap.available)
            .unwrap_or(false)
    }
}

impl Default for DeviceTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Dispatch router for intelligent CPU/GPU selection
pub struct DispatchRouter {
    device_tracker: Arc<DeviceTracker>,
    cost_history: Arc<RwLock<Vec<(String, DispatchDecision)>>>,
    enable_fallback: bool,
}

impl DispatchRouter {
    /// Create new dispatch router
    pub fn new() -> Self {
        let device_tracker = Arc::new(DeviceTracker::new());

        // Initialize device detection
        {
            let mut cpu_cap = device_tracker.get(ExecutionDevice::CPU).unwrap();
            cpu_cap.available_memory = 16_000_000_000; // 16GB typical
            device_tracker.update(ExecutionDevice::CPU, cpu_cap);
        }

        tracing::info!("Dispatch router: initialized");

        Self {
            device_tracker,
            cost_history: Arc::new(RwLock::new(Vec::new())),
            enable_fallback: true,
        }
    }

    /// Estimate data transfer time
    fn estimate_transfer_time(
        from_device: ExecutionDevice,
        to_device: ExecutionDevice,
        data_size: usize,
    ) -> u64 {
        // PCIe Gen4 x16: ~32 GB/s theoretical, ~16 GB/s practical
        let pcie_bandwidth = 16.0e9; // bytes/second

        let bandwidth = match (from_device, to_device) {
            // CPU <-> GPU transfers (PCIe)
            (ExecutionDevice::CPU, ExecutionDevice::CUDA)
            | (ExecutionDevice::CUDA, ExecutionDevice::CPU) => pcie_bandwidth,
            (ExecutionDevice::CPU, ExecutionDevice::HIP)
            | (ExecutionDevice::HIP, ExecutionDevice::CPU) => pcie_bandwidth,
            (ExecutionDevice::CPU, ExecutionDevice::FPGA)
            | (ExecutionDevice::FPGA, ExecutionDevice::CPU) => pcie_bandwidth,
            // On-device transfers (very fast)
            (ExecutionDevice::CUDA, ExecutionDevice::CUDA)
            | (ExecutionDevice::HIP, ExecutionDevice::HIP) => 900.0e9, // GPU memory bandwidth
            // GPU <-> GPU peer-to-peer (if available)
            (ExecutionDevice::CUDA, ExecutionDevice::HIP)
            | (ExecutionDevice::HIP, ExecutionDevice::CUDA) => 32.0e9, // PCIe
            _ => pcie_bandwidth,
        };

        ((data_size as f64) / bandwidth * 1e6) as u64
    }

    /// Make dispatch decision based on operation profile
    pub fn decide(&self, operation: &OperationProfile) -> Result<DispatchDecision, DispatchError> {
        let available_devices = self.device_tracker.available_devices();

        if available_devices.is_empty() {
            return Err(DispatchError::NoDeviceAvailable);
        }

        let mut best_decision = None;
        let mut best_time = u64::MAX;

        for device in available_devices {
            if let Ok(decision) = self.estimate_cost(operation, device) {
                if decision.total_time_us < best_time {
                    best_time = decision.total_time_us;
                    best_decision = Some(decision);
                }
            }
        }

        if let Some(decision) = best_decision {
            // Store in history
            {
                let mut history = self.cost_history.write();
                history.push((operation.name.clone(), decision.clone()));
                if history.len() > 1000 {
                    history.remove(0);
                }
            }

            tracing::debug!(
                "Dispatch router: selected {} for '{}' ({}us total)",
                decision.device.name(),
                operation.name,
                decision.total_time_us
            );

            Ok(decision)
        } else {
            Err(DispatchError::NoDeviceAvailable)
        }
    }

    /// Estimate execution cost on specific device
    fn estimate_cost(
        &self,
        operation: &OperationProfile,
        device: ExecutionDevice,
    ) -> Result<DispatchDecision, DispatchError> {
        let cap = self
            .device_tracker
            .get(device)
            .ok_or(DispatchError::DeviceUnavailable(device.name().to_string()))?;

        if !cap.available {
            return Err(DispatchError::DeviceUnavailable(device.name().to_string()));
        }

        // Calculate transfer times (only for non-CPU devices)
        let (h2d_time_us, d2h_time_us) = if device == ExecutionDevice::CPU {
            (0, 0)
        } else {
            let h2d =
                Self::estimate_transfer_time(ExecutionDevice::CPU, device, operation.input_size);
            let d2h =
                Self::estimate_transfer_time(device, ExecutionDevice::CPU, operation.output_size);
            (h2d, d2h)
        };

        // Estimate computation time
        let compute_tflops = device.compute_tflops();
        let flops = (operation.arithmetic_intensity * operation.input_size as f32) as u64;
        let compute_time_us = if compute_tflops > 0.0 {
            ((flops as f32) / (compute_tflops * 1e12) * 1e6) as u64
        } else {
            u64::MAX
        };

        let total_time_us = if device == ExecutionDevice::CPU {
            compute_time_us
        } else {
            h2d_time_us + compute_time_us + d2h_time_us
        };

        // CPU reference for comparison
        let cpu_compute = (flops as f32 / (0.5 * 1e12) * 1e6) as u64;

        Ok(DispatchDecision {
            device,
            estimated_time_us: compute_time_us,
            h2d_time_us,
            d2h_time_us,
            total_time_us,
            cpu_reference_time_us: cpu_compute,
            confidence: if operation.parallelizable { 0.9 } else { 0.5 },
        })
    }

    /// Get dispatch statistics
    pub fn get_stats(&self) -> DispatchStats {
        let history = self.cost_history.read();
        let mut device_counts: HashMap<ExecutionDevice, usize> = HashMap::new();

        for (_, decision) in history.iter() {
            *device_counts.entry(decision.device).or_insert(0) += 1;
        }

        DispatchStats {
            total_decisions: history.len(),
            device_distribution: device_counts,
            available_devices: self.device_tracker.available_devices(),
        }
    }
}

impl Default for DispatchRouter {
    fn default() -> Self {
        Self::new()
    }
}

/// Dispatch router statistics
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DispatchStats {
    /// Total dispatch decisions made
    pub total_decisions: usize,
    /// Distribution of decisions by device
    pub device_distribution: HashMap<ExecutionDevice, usize>,
    /// Available devices
    pub available_devices: Vec<ExecutionDevice>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_tracker_creation() {
        let tracker = DeviceTracker::new();
        assert!(tracker.is_available(ExecutionDevice::CPU));
    }

    #[test]
    fn test_device_capability_update() {
        let tracker = DeviceTracker::new();
        let mut cap = tracker.get(ExecutionDevice::CUDA).unwrap();
        cap.available = true;
        tracker.update(ExecutionDevice::CUDA, cap);
        assert!(tracker.is_available(ExecutionDevice::CUDA));
    }

    #[test]
    fn test_dispatch_router_creation() {
        let router = DispatchRouter::new();
        assert!(router.device_tracker.is_available(ExecutionDevice::CPU));
    }

    #[test]
    fn test_transfer_time_estimation() {
        let h2d = DispatchRouter::estimate_transfer_time(
            ExecutionDevice::CPU,
            ExecutionDevice::CUDA,
            1024 * 1024 * 100, // 100MB
        );
        assert!(h2d > 0);
    }

    #[test]
    fn test_dispatch_decision_cpu_only() {
        let router = DispatchRouter::new();
        let operation = OperationProfile {
            name: "test".to_string(),
            input_size: 1024,
            output_size: 1024,
            arithmetic_intensity: 1.0,
            parallelizable: false,
            access_pattern: AccessPattern::Sequential,
        };
        let decision = router.decide(&operation);
        assert!(decision.is_ok());
        assert_eq!(decision.unwrap().device, ExecutionDevice::CPU);
    }

    #[test]
    fn test_dispatch_stats() {
        let router = DispatchRouter::new();
        let operation = OperationProfile {
            name: "test".to_string(),
            input_size: 1024,
            output_size: 1024,
            arithmetic_intensity: 1.0,
            parallelizable: true,
            access_pattern: AccessPattern::Sequential,
        };
        let _ = router.decide(&operation);
        let stats = router.get_stats();
        assert_eq!(stats.total_decisions, 1);
    }
}
