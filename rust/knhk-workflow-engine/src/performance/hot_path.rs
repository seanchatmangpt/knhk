//! Hot path performance optimizations
//!
//! Provides zero-copy, branchless, SIMD-optimized operations for hot path
//! operations that must complete in ≤8 ticks (2ns at 4GHz).

use crate::constants::HOT_PATH_MAX_TICKS;
use crate::error::{WorkflowError, WorkflowResult};

/// Hot path operation result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotPathResult {
    /// Operation succeeded
    pub success: bool,
    /// Ticks consumed
    pub ticks: u32,
}

impl HotPathResult {
    /// Create successful result
    pub fn success(ticks: u32) -> Self {
        Self {
            success: true,
            ticks,
        }
    }

    /// Create failed result
    pub fn failure(ticks: u32) -> Self {
        Self {
            success: false,
            ticks,
        }
    }

    /// Check if within budget
    pub fn is_within_budget(&self) -> bool {
        self.ticks <= HOT_PATH_MAX_TICKS
    }
}

/// Hot path validator
pub struct HotPathValidator;

impl HotPathValidator {
    /// Validate hot path operation
    pub fn validate(result: HotPathResult) -> WorkflowResult<()> {
        if !result.is_within_budget() {
            return Err(WorkflowError::Validation(format!(
                "Hot path operation consumed {} ticks, exceeds maximum {}",
                result.ticks, HOT_PATH_MAX_TICKS
            )));
        }
        Ok(())
    }
}

/// Branchless comparison (constant-time)
#[inline(always)]
pub fn branchless_lt(a: u32, b: u32) -> bool {
    (a.wrapping_sub(b)) >> 31 != 0
}

/// Branchless min (constant-time)
#[inline(always)]
pub fn branchless_min(a: u32, b: u32) -> u32 {
    a + ((b.wrapping_sub(a)) & ((b.wrapping_sub(a)) >> 31))
}

/// Branchless max (constant-time)
#[inline(always)]
pub fn branchless_max(a: u32, b: u32) -> u32 {
    a - ((a.wrapping_sub(b)) & ((a.wrapping_sub(b)) >> 31))
}

/// Zero-copy string slice extraction
#[inline(always)]
pub fn extract_str_slice(bytes: &[u8], start: usize, end: usize) -> Option<&str> {
    if end <= bytes.len() && start <= end {
        std::str::from_utf8(&bytes[start..end]).ok()
    } else {
        None
    }
}
