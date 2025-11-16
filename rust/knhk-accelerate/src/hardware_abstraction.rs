// Hardware Abstraction Layer
// Unified interface for GPU, FPGA, and SIMD acceleration
// Automatic backend selection and fallback

use serde::{Deserialize, Serialize};
use thiserror::Error;
use crate::{GPUAccelerator, GPUConfig, FPGAOffload, SIMDKernel, AccelerationDevice};

#[derive(Error, Debug)]
pub enum AbstractionError {
    #[error("No suitable acceleration backend available")]
    NoBackendAvailable,

    #[error("Backend operation failed: {0}")]
    BackendFailed(String),

    #[error("Unsupported operation for this backend")]
    UnsupportedOperation,

    #[error("Backend initialization failed")]
    InitializationFailed,
}

/// Acceleration capability
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AccelerationCapability {
    /// Matrix multiplication
    MatMul,
    /// Neural network training
    NeuralTraining,
    /// Pattern matching
    PatternMatching,
    /// Vector operations
    VectorOps,
    /// Reduction operations
    Reduction,
    /// General computation
    Compute,
}

/// Acceleration backend type
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AccelerationBackend {
    /// CPU with SIMD
    CPU,
    /// GPU (CUDA/ROCm/OpenCL)
    GPU,
    /// FPGA
    FPGA,
    /// CPU fallback
    CPUFallback,
}

impl AccelerationBackend {
    /// Get backend capabilities
    pub fn capabilities(&self) -> Vec<AccelerationCapability> {
        match self {
            AccelerationBackend::CPU => vec![
                AccelerationCapability::MatMul,
                AccelerationCapability::VectorOps,
                AccelerationCapability::Reduction,
            ],
            AccelerationBackend::GPU => vec![
                AccelerationCapability::MatMul,
                AccelerationCapability::NeuralTraining,
                AccelerationCapability::VectorOps,
                AccelerationCapability::Reduction,
                AccelerationCapability::Compute,
            ],
            AccelerationBackend::FPGA => vec![
                AccelerationCapability::PatternMatching,
                AccelerationCapability::Compute,
            ],
            AccelerationBackend::CPUFallback => vec![
                AccelerationCapability::VectorOps,
                AccelerationCapability::Reduction,
            ],
        }
    }

    /// Estimated throughput (GB/s)
    pub fn throughput_gb_s(&self) -> f32 {
        match self {
            AccelerationBackend::CPU => 32.0,           // AVX-512 with DDR5
            AccelerationBackend::GPU => 896.0,          // A100 with HBM2e
            AccelerationBackend::FPGA => 256.0,         // PCIe Gen4 x16
            AccelerationBackend::CPUFallback => 16.0,   // Scalar operations
        }
    }

    /// Estimated latency (microseconds)
    pub fn latency_us(&self) -> u64 {
        match self {
            AccelerationBackend::CPU => 1,           // Cache hit
            AccelerationBackend::GPU => 10,          // Kernel launch + execution
            AccelerationBackend::FPGA => 2,          // Direct hardware execution
            AccelerationBackend::CPUFallback => 100, // Scalar path
        }
    }
}

/// Hardware abstraction layer
pub struct HardwareAbstraction {
    /// Preferred backend
    pub preferred_backend: AccelerationBackend,
    /// Available backends
    pub available_backends: Vec<AccelerationBackend>,
    /// Current active backend
    pub active_backend: Option<AccelerationBackend>,
    /// GPU accelerator (if available)
    pub gpu: Option<GPUAccelerator>,
    /// FPGA offloader (if available)
    pub fpga: Option<FPGAOffload>,
    /// SIMD kernel (CPU fallback)
    pub simd: Option<SIMDKernel>,
}

impl HardwareAbstraction {
    /// Create hardware abstraction layer with auto-detection
    pub fn new(preferred_device: AccelerationDevice) -> Result<Self, AbstractionError> {
        // Phase 9 implementation stub
        // TODO: Implement hardware abstraction
        // Step 1: Detect available backends
        // Step 2: Initialize preferred backend
        // Step 3: Setup fallback chain

        let mut available = vec![AccelerationBackend::CPU];
        let mut gpu = None;
        let mut fpga = None;
        let simd = Some(SIMDKernel::new(crate::kernels::KernelType::Custom));

        // Try to initialize GPU
        if preferred_device == AccelerationDevice::CUDA || preferred_device == AccelerationDevice::Auto {
            if let Ok(gpu_accel) = GPUAccelerator::new(GPUConfig::default()) {
                available.push(AccelerationBackend::GPU);
                gpu = Some(gpu_accel);
            }
        }

        // Try to initialize FPGA
        if preferred_device == AccelerationDevice::FPGA || preferred_device == AccelerationDevice::Auto {
            available.push(AccelerationBackend::FPGA);
        }

        let active_backend = match preferred_device {
            AccelerationDevice::CUDA if gpu.is_some() => Some(AccelerationBackend::GPU),
            AccelerationDevice::FPGA => Some(AccelerationBackend::FPGA),
            _ => Some(AccelerationBackend::CPU),
        };

        tracing::info!(
            "Hardware abstraction: available backends: {:?}",
            available
        );

        Ok(Self {
            preferred_backend: match preferred_device {
                AccelerationDevice::CUDA => AccelerationBackend::GPU,
                AccelerationDevice::FPGA => AccelerationBackend::FPGA,
                _ => AccelerationBackend::CPU,
            },
            available_backends: available,
            active_backend,
            gpu,
            fpga,
            simd,
        })
    }

    /// Select backend by capability
    pub fn select_by_capability(&mut self, capability: AccelerationCapability) -> Result<AccelerationBackend, AbstractionError> {
        // Phase 9 implementation stub
        // TODO: Implement capability-based backend selection
        // Step 1: Check preferred backend has capability
        // Step 2: If not, find best alternative
        // Step 3: Fall back to CPU if needed

        for backend in &self.available_backends {
            if backend.capabilities().contains(&capability) {
                self.active_backend = Some(*backend);
                return Ok(*backend);
            }
        }

        Err(AbstractionError::NoBackendAvailable)
    }

    /// Get backend throughput
    pub fn get_backend_throughput(&self, backend: AccelerationBackend) -> f32 {
        backend.throughput_gb_s()
    }

    /// Check if backend supports operation
    pub fn supports_operation(&self, backend: AccelerationBackend, op: AccelerationCapability) -> bool {
        backend.capabilities().contains(&op)
    }

    /// Get status of all backends
    pub fn get_status(&self) -> HardwareStatus {
        HardwareStatus {
            available_backends: self.available_backends.clone(),
            active_backend: self.active_backend,
            gpu_available: self.gpu.is_some(),
            fpga_available: self.fpga.is_some(),
            simd_available: self.simd.is_some(),
        }
    }

    /// Benchmark all backends
    pub fn benchmark(&self) -> Result<Vec<BenchmarkResult>, AbstractionError> {
        // Phase 9 implementation stub
        // TODO: Implement backend benchmarking
        // Step 1: Run standard workload on each backend
        // Step 2: Measure throughput and latency
        // Step 3: Return results

        let mut results = vec![];

        for backend in &self.available_backends {
            results.push(BenchmarkResult {
                backend: *backend,
                throughput_gb_s: backend.throughput_gb_s(),
                latency_us: backend.latency_us(),
            });
        }

        Ok(results)
    }
}

/// Hardware status information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HardwareStatus {
    pub available_backends: Vec<AccelerationBackend>,
    pub active_backend: Option<AccelerationBackend>,
    pub gpu_available: bool,
    pub fpga_available: bool,
    pub simd_available: bool,
}

/// Benchmark result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub backend: AccelerationBackend,
    pub throughput_gb_s: f32,
    pub latency_us: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acceleration_capability() {
        let cap = AccelerationCapability::MatMul;
        assert_eq!(cap, AccelerationCapability::MatMul);
    }

    #[test]
    fn test_backend_capabilities() {
        let gpu_caps = AccelerationBackend::GPU.capabilities();
        assert!(gpu_caps.contains(&AccelerationCapability::MatMul));
        assert!(gpu_caps.contains(&AccelerationCapability::NeuralTraining));

        let fpga_caps = AccelerationBackend::FPGA.capabilities();
        assert!(fpga_caps.contains(&AccelerationCapability::PatternMatching));
    }

    #[test]
    fn test_backend_throughput() {
        assert!(AccelerationBackend::GPU.throughput_gb_s() > AccelerationBackend::CPU.throughput_gb_s());
        assert!(AccelerationBackend::CPU.throughput_gb_s() > AccelerationBackend::CPUFallback.throughput_gb_s());
    }

    #[test]
    fn test_hardware_abstraction_creation() {
        let hw = HardwareAbstraction::new(AccelerationDevice::Auto);
        assert!(hw.is_ok());
    }

    #[test]
    fn test_hardware_abstraction_status() {
        let hw = HardwareAbstraction::new(AccelerationDevice::Auto).unwrap();
        let status = hw.get_status();
        assert!(!status.available_backends.is_empty());
    }

    #[test]
    fn test_hardware_abstraction_supports_operation() {
        let hw = HardwareAbstraction::new(AccelerationDevice::Auto).unwrap();
        let supports = hw.supports_operation(
            AccelerationBackend::GPU,
            AccelerationCapability::MatMul
        );
        assert!(supports);
    }

    #[test]
    fn test_hardware_abstraction_benchmark() {
        let hw = HardwareAbstraction::new(AccelerationDevice::Auto).unwrap();
        let results = hw.benchmark();
        assert!(results.is_ok());
    }
}
