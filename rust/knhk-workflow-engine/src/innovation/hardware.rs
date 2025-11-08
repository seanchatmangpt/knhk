//! Hardware acceleration
//!
//! Provides hardware-accelerated operations using SIMD, GPU, and specialized hardware.

use crate::error::{WorkflowError, WorkflowResult};
use crate::performance::simd;

/// Hardware acceleration capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareAcceleration {
    /// No acceleration available
    None,
    /// SIMD (SSE/AVX/NEON) available
    Simd,
    /// GPU acceleration available
    Gpu,
    /// Both SIMD and GPU available
    Both,
}

/// Hardware accelerator
pub struct HardwareAccelerator {
    /// Acceleration type
    acceleration: HardwareAcceleration,
    /// SIMD available
    simd_available: bool,
    /// GPU available
    gpu_available: bool,
}

impl HardwareAccelerator {
    /// Create new hardware accelerator
    pub fn new() -> Self {
        let simd_available = is_simd_available();
        let gpu_available = Self::detect_gpu();

        let acceleration = match (simd_available, gpu_available) {
            (true, true) => HardwareAcceleration::Both,
            (true, false) => HardwareAcceleration::Simd,
            (false, true) => HardwareAcceleration::Gpu,
            (false, false) => HardwareAcceleration::None,
        };

        Self {
            acceleration,
            simd_available,
            gpu_available,
        }
    }

    /// Detect GPU availability
    fn detect_gpu() -> bool {
        // FUTURE: Implement GPU detection
        // For now, return false
        false
    }

    /// Get acceleration type
    pub fn acceleration(&self) -> HardwareAcceleration {
        self.acceleration
    }

    /// Check if SIMD is available
    pub fn is_simd_available(&self) -> bool {
        self.simd_available
    }

    /// Check if GPU is available
    pub fn is_gpu_available(&self) -> bool {
        self.gpu_available
    }

    /// Accelerated hash computation
    pub fn accelerated_hash(&self, data: &[u8]) -> u64 {
        match self.acceleration {
            HardwareAcceleration::Simd | HardwareAcceleration::Both => simd_hash(data),
            _ => {
                // Fallback to standard hash
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                data.hash(&mut hasher);
                hasher.finish()
            }
        }
    }

    /// Accelerated pattern matching
    pub fn accelerated_pattern_match(&self, pattern: &[u8], data: &[u8]) -> Option<usize> {
        match self.acceleration {
            HardwareAcceleration::Simd | HardwareAcceleration::Both => {
                simd_pattern_match(pattern, data)
            }
            _ => {
                // Fallback to standard pattern matching
                data.windows(pattern.len())
                    .position(|window| window == pattern)
            }
        }
    }

    /// Accelerated batch processing
    pub fn accelerated_batch_process(&self, items: &[&[u8]]) -> Vec<u64> {
        match self.acceleration {
            HardwareAcceleration::Simd | HardwareAcceleration::Both => {
                items.iter().map(|item| simd_hash(item)).collect()
            }
            _ => {
                // Fallback to standard processing
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                items
                    .iter()
                    .map(|item| {
                        let mut hasher = DefaultHasher::new();
                        item.hash(&mut hasher);
                        hasher.finish()
                    })
                    .collect()
            }
        }
    }
}

impl Default for HardwareAccelerator {
    fn default() -> Self {
        Self::new()
    }
}
