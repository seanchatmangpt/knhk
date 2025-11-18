//! Hardware acceleration module for KNHK workflow engine
//!
//! Provides GPU, FPGA, and SIMD acceleration for pattern dispatch.
//!
//! # DOCTRINE ALIGNMENT
//! - Principle: Chatman Constant (Covenant 5), Q (Invariants)
//! - Covenant 5: All hot paths ≤8 ticks (hardware acceleration achieves this at scale)
//! - Covenant 2: Invariants Are Law (performance guarantees enforced)
//!
//! # Architecture
//! ```
//! CPU (baseline):  1-8μs per pattern   (100% availability)
//! SIMD (AVX-512):  0.1-1μs per pattern (10x speedup, 90% availability)
//! GPU (WGPU):      0.01-1μs per pattern (100x speedup, requires license)
//! FPGA (Xilinx):   0.01-0.1μs per pattern (1000x speedup, Enterprise only)
//! ```

use std::marker::PhantomData;

pub mod cpu;
pub mod simd;

#[cfg(feature = "gpu")]
pub mod gpu;

#[cfg(feature = "fpga")]
pub mod fpga;

pub mod selector;
pub mod adaptive;

/// Hardware acceleration backend
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccelerationBackend {
    /// CPU-only baseline (1-8μs per pattern)
    CPU,
    /// SIMD (AVX-512) acceleration (0.1-1μs per pattern)
    SIMD,
    /// GPU (WGPU) acceleration (0.01-1μs per pattern, Pro+ license required)
    #[cfg(feature = "gpu")]
    GPU,
    /// FPGA (Xilinx) acceleration (0.01-0.1μs per pattern, Enterprise license required)
    #[cfg(feature = "fpga")]
    FPGA,
}

/// Pattern accelerator trait (implemented by all backends)
pub trait PatternAccelerator: Send + Sync {
    /// Dispatch batch of patterns
    fn dispatch(&self, patterns: &[crate::executor::pattern::PatternId])
        -> Vec<crate::executor::workflow_execution::Receipt>;

    /// Get backend type
    fn backend(&self) -> AccelerationBackend;

    /// Check if backend is available
    fn is_available(&self) -> bool;

    /// Get expected latency for given batch size (in microseconds)
    fn expected_latency_us(&self, batch_size: usize) -> f64;

    /// Get max throughput (patterns/sec)
    fn max_throughput(&self) -> usize;
}

/// Workload characteristics for backend selection
#[derive(Debug, Clone)]
pub struct WorkloadCharacteristics {
    /// Number of patterns in batch
    pub batch_size: usize,

    /// Latency SLA requirement
    pub latency_sla: LatencySLA,

    /// Throughput requirement (patterns/sec)
    pub throughput_requirement: Option<usize>,

    /// Cost sensitivity (optimize for cost vs performance)
    pub cost_sensitive: bool,
}

/// Latency SLA tiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LatencySLA {
    /// Interactive: <10μs
    Interactive,
    /// Realtime: <100μs
    Realtime,
    /// Batch: <1ms
    Batch,
    /// BestEffort: no SLA
    BestEffort,
}

/// Hardware acceleration capabilities (zero-sized, compile-time check)
pub struct HardwareCapabilities<L> {
    _license: PhantomData<fn() -> L>,
}

impl<L> HardwareCapabilities<L> {
    /// Create new hardware capabilities
    pub const fn new() -> Self {
        Self {
            _license: PhantomData,
        }
    }
}

// Re-exports
pub use cpu::CPUAccelerator;
pub use simd::SIMDAccelerator;

#[cfg(feature = "gpu")]
pub use gpu::GPUAccelerator;

#[cfg(feature = "fpga")]
pub use fpga::FPGAAccelerator;

pub use selector::BackendSelector;
pub use adaptive::AdaptiveAccelerator;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acceleration_backend_ordering() {
        // Backends should be ordered by performance
        assert!(AccelerationBackend::SIMD > AccelerationBackend::CPU);

        #[cfg(feature = "gpu")]
        assert!(AccelerationBackend::GPU > AccelerationBackend::SIMD);

        #[cfg(feature = "fpga")]
        assert!(AccelerationBackend::FPGA > AccelerationBackend::GPU);
    }

    #[test]
    fn test_latency_sla_ordering() {
        // Stricter SLAs should be "less than" (numerically lower latency)
        assert!(LatencySLA::Interactive < LatencySLA::Realtime);
        assert!(LatencySLA::Realtime < LatencySLA::Batch);
        assert!(LatencySLA::Batch < LatencySLA::BestEffort);
    }
}
