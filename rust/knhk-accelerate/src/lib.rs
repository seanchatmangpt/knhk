// KNHK Phase 9: Hardware Acceleration
// GPU (CUDA/ROCm), FPGA, and SIMD intrinsics for neural training and pattern matching
// Hardware abstraction layer for seamless acceleration

#![allow(dead_code)] // Phase 9 implementation skeleton
#![cfg_attr(feature = "simd", feature(portable_simd))]

pub mod gpu;
pub mod fpga;
pub mod kernels;
pub mod dispatch;
pub mod memory;
pub mod simd;
pub mod hardware_abstraction;

pub use gpu::{GPUAccelerator, GPUConfig, GPUError, DeviceType, MemoryInfo};
pub use fpga::{FPGAOffload, FPGAConfig, PatternMatcher};
pub use kernels::{KernelExecutor, KernelType, OptimizationLevel, GridDimensions, ActivationType, SIMDKernel};
pub use dispatch::{DispatchRouter, ExecutionDevice, OperationProfile, DispatchDecision};
pub use memory::{MemoryManager, MemoryError, AllocationType, MemoryStats};
pub use simd::{SIMDKernel as SIMDOps, VectorOperation, SIMDOptimization};
pub use hardware_abstraction::{HardwareAbstraction, AccelerationBackend, AccelerationCapability};

/// Prelude for Phase 9 hardware acceleration
pub mod prelude {
    pub use crate::gpu::{GPUAccelerator, GPUConfig};
    pub use crate::fpga::FPGAOffload;
    pub use crate::simd::SIMDKernel;
    pub use crate::hardware_abstraction::HardwareAbstraction;
}

/// Hardware acceleration configuration
#[derive(Clone, Debug)]
pub struct AccelerationConfig {
    /// Enable GPU acceleration
    pub gpu_enabled: bool,
    /// Enable FPGA offloading
    pub fpga_enabled: bool,
    /// Enable SIMD optimization
    pub simd_enabled: bool,
    /// Preferred device for acceleration
    pub preferred_device: AccelerationDevice,
    /// Auto-select best available device
    pub auto_select: bool,
}

/// Acceleration device preference
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccelerationDevice {
    /// CPU with SIMD
    CPU,
    /// NVIDIA CUDA
    CUDA,
    /// AMD ROCm
    ROCm,
    /// OpenCL (vendor-agnostic)
    OpenCL,
    /// FPGA via PCIE
    FPGA,
    /// Auto-select (use available)
    Auto,
}

impl Default for AccelerationConfig {
    fn default() -> Self {
        Self {
            gpu_enabled: true,
            fpga_enabled: true,
            simd_enabled: true,
            preferred_device: AccelerationDevice::Auto,
            auto_select: true,
        }
    }
}

impl AccelerationConfig {
    /// Get accelerators available on this system
    pub fn detect_available() -> Vec<AccelerationDevice> {
        // Phase 9 implementation stub
        // TODO: Implement hardware detection
        // Step 1: Check for CUDA runtime
        // Step 2: Check for ROCm runtime
        // Step 3: Check for OpenCL support
        // Step 4: Check for FPGA devices via PCIE
        // Step 5: Check SIMD support (AVX-512, AVX2, etc)

        let mut available = vec![AccelerationDevice::CPU];

        #[cfg(feature = "gpu")]
        available.push(AccelerationDevice::CUDA);

        #[cfg(feature = "opencl")]
        available.push(AccelerationDevice::OpenCL);

        #[cfg(feature = "fpga")]
        available.push(AccelerationDevice::FPGA);

        tracing::info!(
            "Hardware detection: found {} accelerators",
            available.len()
        );

        available
    }

    /// Validate configuration consistency
    pub fn validate(&self) -> Result<(), String> {
        if !self.gpu_enabled && !self.fpga_enabled && !self.simd_enabled {
            return Err("At least one acceleration backend must be enabled".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acceleration_config_defaults() {
        let config = AccelerationConfig::default();
        assert!(config.gpu_enabled);
        assert!(config.fpga_enabled);
        assert!(config.simd_enabled);
        assert_eq!(config.preferred_device, AccelerationDevice::Auto);
    }

    #[test]
    fn test_acceleration_config_validation() {
        let mut config = AccelerationConfig::default();
        assert!(config.validate().is_ok());

        config.gpu_enabled = false;
        config.fpga_enabled = false;
        config.simd_enabled = false;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_detect_available_hardware() {
        let available = AccelerationConfig::detect_available();
        assert!(!available.is_empty());
        assert!(available.contains(&AccelerationDevice::CPU));
    }

    #[test]
    fn test_phase_9_prelude_imports() {
        // Verify all public types are accessible
        let _cfg = AccelerationConfig::default();
        let _dev = AccelerationDevice::Auto;
    }
}
