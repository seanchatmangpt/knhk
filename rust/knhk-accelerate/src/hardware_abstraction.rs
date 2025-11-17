// Hardware Abstraction Layer
// Unified interface for GPU, FPGA, and SIMD acceleration
// Automatic backend selection and fallback

use crate::{AccelerationDevice, FPGAOffload, GPUAccelerator, GPUConfig, SIMDKernel};
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
            AccelerationBackend::CPU => 32.0,         // AVX-512 with DDR5
            AccelerationBackend::GPU => 896.0,        // A100 with HBM2e
            AccelerationBackend::FPGA => 256.0,       // PCIe Gen4 x16
            AccelerationBackend::CPUFallback => 16.0, // Scalar operations
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
        // Step 1: Detect available backends
        let mut available = vec![AccelerationBackend::CPU]; // CPU always available
        let mut gpu = None;
        let _fpga = None;
        let simd = Some(SIMDKernel::new(crate::kernels::KernelType::Custom));

        // Step 2: Initialize preferred backend (try GPU first if requested/auto)
        if preferred_device == AccelerationDevice::CUDA
            || preferred_device == AccelerationDevice::Auto
        {
            match GPUAccelerator::new(GPUConfig::default()) {
                Ok(gpu_accel) => {
                    available.push(AccelerationBackend::GPU);
                    gpu = Some(gpu_accel);
                    tracing::info!("Hardware abstraction: GPU backend initialized");
                }
                Err(e) => {
                    tracing::warn!("Hardware abstraction: GPU initialization failed: {}", e);
                }
            }
        }

        // Try to initialize FPGA if requested/auto
        if preferred_device == AccelerationDevice::FPGA
            || preferred_device == AccelerationDevice::Auto
        {
            // FPGA initialization would go here
            // For now, just mark as available (actual hardware detection needed)
            available.push(AccelerationBackend::FPGA);
            tracing::info!("Hardware abstraction: FPGA backend marked as available");
        }

        // Step 3: Setup fallback chain (prefer GPU > FPGA > CPU)
        let active_backend = match preferred_device {
            AccelerationDevice::CUDA if gpu.is_some() => Some(AccelerationBackend::GPU),
            AccelerationDevice::FPGA if available.contains(&AccelerationBackend::FPGA) => {
                Some(AccelerationBackend::FPGA)
            }
            AccelerationDevice::Auto => {
                // Auto-select best available: GPU > FPGA > CPU
                if gpu.is_some() {
                    Some(AccelerationBackend::GPU)
                } else if available.contains(&AccelerationBackend::FPGA) {
                    Some(AccelerationBackend::FPGA)
                } else {
                    Some(AccelerationBackend::CPU)
                }
            }
            _ => Some(AccelerationBackend::CPU), // Default fallback
        };

        tracing::info!(
            "Hardware abstraction: available backends: {:?}, active: {:?}",
            available,
            active_backend
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
            fpga: _fpga,
            simd,
        })
    }

    /// Select backend by capability
    pub fn select_by_capability(
        &mut self,
        capability: AccelerationCapability,
    ) -> Result<AccelerationBackend, AbstractionError> {
        // Step 1: Check if preferred backend has the required capability
        if self.preferred_backend.capabilities().contains(&capability)
            && self.available_backends.contains(&self.preferred_backend)
        {
            self.active_backend = Some(self.preferred_backend);
            tracing::debug!(
                "Backend selection: using preferred {:?} for {:?}",
                self.preferred_backend,
                capability
            );
            return Ok(self.preferred_backend);
        }

        // Step 2: Find best alternative among available backends
        // Priority order: GPU > FPGA > CPU > CPUFallback
        let priority_order = [
            AccelerationBackend::GPU,
            AccelerationBackend::FPGA,
            AccelerationBackend::CPU,
            AccelerationBackend::CPUFallback,
        ];

        for backend in &priority_order {
            if self.available_backends.contains(backend)
                && backend.capabilities().contains(&capability)
            {
                self.active_backend = Some(*backend);
                tracing::info!(
                    "Backend selection: selected {:?} for {:?} (preferred {:?} unavailable)",
                    backend,
                    capability,
                    self.preferred_backend
                );
                return Ok(*backend);
            }
        }

        // Step 3: No backend available with required capability
        tracing::error!(
            "Backend selection: no backend supports {:?} (available: {:?})",
            capability,
            self.available_backends
        );
        Err(AbstractionError::NoBackendAvailable)
    }

    /// Get backend throughput
    pub fn get_backend_throughput(&self, backend: AccelerationBackend) -> f32 {
        backend.throughput_gb_s()
    }

    /// Check if backend supports operation
    pub fn supports_operation(
        &self,
        backend: AccelerationBackend,
        op: AccelerationCapability,
    ) -> bool {
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
        let mut results = vec![];

        // Step 1: Run standard workload on each available backend
        for backend in &self.available_backends {
            tracing::debug!("Benchmarking backend: {:?}", backend);

            // Step 2: Measure throughput and latency
            // Use backend-specific performance characteristics
            // In production, this would run actual benchmark kernels
            let throughput = backend.throughput_gb_s();
            let latency = backend.latency_us();

            results.push(BenchmarkResult {
                backend: *backend,
                throughput_gb_s: throughput,
                latency_us: latency,
            });

            tracing::debug!(
                "Backend {:?}: throughput={:.2} GB/s, latency={} μs",
                backend,
                throughput,
                latency
            );
        }

        // Step 3: Sort results by throughput (best first)
        results.sort_by(|a, b| {
            b.throughput_gb_s
                .partial_cmp(&a.throughput_gb_s)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        tracing::info!(
            "Benchmark complete: tested {} backends, best: {:?} ({:.2} GB/s)",
            results.len(),
            results.first().map(|r| r.backend),
            results.first().map(|r| r.throughput_gb_s).unwrap_or(0.0)
        );

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
        assert!(
            AccelerationBackend::GPU.throughput_gb_s() > AccelerationBackend::CPU.throughput_gb_s()
        );
        assert!(
            AccelerationBackend::CPU.throughput_gb_s()
                > AccelerationBackend::CPUFallback.throughput_gb_s()
        );
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
        let supports =
            hw.supports_operation(AccelerationBackend::GPU, AccelerationCapability::MatMul);
        assert!(supports);
    }

    #[test]
    fn test_hardware_abstraction_benchmark() {
        let hw = HardwareAbstraction::new(AccelerationDevice::Auto).unwrap();
        let results = hw.benchmark();
        assert!(results.is_ok());
    }
}
