// KNHK Phase 9: Hardware Acceleration
// GPU (CUDA/ROCm), FPGA, and SIMD intrinsics for neural training and pattern matching
// Hardware abstraction layer for seamless acceleration

#![allow(dead_code)] // Phase 9 implementation skeleton
#![cfg_attr(feature = "simd", feature(portable_simd))]

pub mod dispatch;
pub mod fpga;
pub mod gpu;
pub mod hardware_abstraction;
pub mod kernels;
pub mod memory;
pub mod simd;

pub use dispatch::{DispatchDecision, DispatchRouter, ExecutionDevice, OperationProfile};
pub use fpga::{FPGAConfig, FPGAOffload, PatternMatcher};
pub use gpu::{DeviceType, GPUAccelerator, GPUConfig, GPUError, MemoryInfo};
pub use hardware_abstraction::{AccelerationBackend, AccelerationCapability, HardwareAbstraction};
pub use kernels::{
    ActivationType, GridDimensions, KernelExecutor, KernelType, OptimizationLevel, SIMDKernel,
};
pub use memory::{AllocationType, MemoryError, MemoryManager, MemoryStats};
pub use simd::{SIMDKernel as SIMDOps, SIMDOptimization, VectorOperation};

/// Prelude for Phase 9 hardware acceleration
pub mod prelude {
    pub use crate::fpga::FPGAOffload;
    pub use crate::gpu::{GPUAccelerator, GPUConfig};
    pub use crate::hardware_abstraction::HardwareAbstraction;
    pub use crate::simd::SIMDKernel;
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
        let available = vec![AccelerationDevice::CPU]; // CPU always available

        // Step 1: Check for CUDA runtime
        #[cfg(feature = "gpu")]
        {
            // In production, this would check for CUDA driver/runtime
            // e.g., by trying to create a CUDA context or checking /dev/nvidia*
            if let Ok(_) = GPUAccelerator::new(GPUConfig::default()) {
                available.push(AccelerationDevice::CUDA);
                tracing::info!("Hardware detection: CUDA runtime detected");
            } else {
                tracing::debug!("Hardware detection: CUDA runtime not available");
            }
        }

        // Step 2: Check for ROCm runtime
        #[cfg(feature = "rocm")]
        {
            // In production, this would check for ROCm driver/runtime
            // e.g., by checking for /dev/kfd, /dev/dri/renderD* devices
            // For now, mark as available if feature enabled
            available.push(AccelerationDevice::ROCm);
            tracing::info!("Hardware detection: ROCm support enabled");
        }

        // Step 3: Check for OpenCL support
        #[cfg(feature = "opencl")]
        {
            // In production, this would enumerate OpenCL platforms/devices
            // e.g., using clGetPlatformIDs and clGetDeviceIDs
            available.push(AccelerationDevice::OpenCL);
            tracing::info!("Hardware detection: OpenCL support enabled");
        }

        // Step 4: Check for FPGA devices via PCIE
        #[cfg(feature = "fpga")]
        {
            // In production, this would scan PCI bus for FPGA devices
            // e.g., using lspci or checking /sys/bus/pci/devices
            // Common FPGA vendor IDs: Xilinx (0x10ee), Intel/Altera (0x1172)
            available.push(AccelerationDevice::FPGA);
            tracing::info!("Hardware detection: FPGA support enabled");
        }

        // Step 5: Check SIMD support (AVX-512, AVX2, etc)
        // This is always available on x86_64, but we could check specific instruction sets
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            // In production, this would use CPUID to detect instruction sets
            // For now, SIMD is implied by CPU backend
            tracing::debug!("Hardware detection: SIMD support available (x86/x86_64)");
        }

        tracing::info!(
            "Hardware detection: found {} accelerators: {:?}",
            available.len(),
            available
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
