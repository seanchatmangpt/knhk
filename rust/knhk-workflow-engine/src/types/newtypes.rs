//! Newtype Pattern for Type Safety
//!
//! Wraps primitive types in distinct newtypes to prevent mixing incompatible values
//! at compile time, providing zero-cost abstraction with strong type safety.

use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};

/// Priority level for workflow tasks (0-255, higher is more urgent)
///
/// This newtype prevents accidentally using raw integers where priority is expected,
/// catching errors at compile time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PriorityLevel(u8);

impl PriorityLevel {
    /// Lowest priority level
    pub const MIN: Self = Self(0);

    /// Highest priority level
    pub const MAX: Self = Self(255);

    /// Default normal priority
    pub const NORMAL: Self = Self(128);

    /// High priority
    pub const HIGH: Self = Self(192);

    /// Low priority
    pub const LOW: Self = Self(64);

    /// Critical priority
    pub const CRITICAL: Self = Self(255);

    /// Create a new priority level
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use knhk_workflow_engine::types::newtypes::PriorityLevel;
    ///
    /// let priority = PriorityLevel::new(128);
    /// assert_eq!(priority, PriorityLevel::NORMAL);
    /// ```
    #[inline(always)]
    pub const fn new(value: u8) -> Self {
        Self(value)
    }

    /// Get the raw priority value
    #[inline(always)]
    pub const fn value(self) -> u8 {
        self.0
    }

    /// Check if this is a high priority task (>= 192)
    #[inline(always)]
    pub const fn is_high(self) -> bool {
        self.0 >= 192
    }

    /// Check if this is a critical priority task (255)
    #[inline(always)]
    pub const fn is_critical(self) -> bool {
        self.0 == 255
    }
}

impl Default for PriorityLevel {
    #[inline(always)]
    fn default() -> Self {
        Self::NORMAL
    }
}

impl fmt::Display for PriorityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "priority:{}", self.0)
    }
}

/// Timeout duration in milliseconds
///
/// Newtype wrapper for timeout values with compile-time validation of maximum timeout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TimeoutMs(u64);

impl TimeoutMs {
    /// Maximum allowed timeout (5 minutes)
    pub const MAX_TIMEOUT: Self = Self(300_000);

    /// Default timeout (30 seconds)
    pub const DEFAULT: Self = Self(30_000);

    /// Short timeout (5 seconds)
    pub const SHORT: Self = Self(5_000);

    /// Long timeout (2 minutes)
    pub const LONG: Self = Self(120_000);

    /// No timeout (effectively infinite, but still bounded)
    pub const INFINITE: Self = Self::MAX_TIMEOUT;

    /// Create a new timeout value, clamping to MAX_TIMEOUT
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use knhk_workflow_engine::types::newtypes::TimeoutMs;
    ///
    /// let timeout = TimeoutMs::new(60_000); // 60 seconds
    /// ```
    #[inline]
    pub const fn new(ms: u64) -> Self {
        if ms > Self::MAX_TIMEOUT.0 {
            Self::MAX_TIMEOUT
        } else {
            Self(ms)
        }
    }

    /// Create a timeout from seconds
    #[inline]
    pub const fn from_secs(secs: u64) -> Self {
        Self::new(secs * 1000)
    }

    /// Get the timeout in milliseconds
    #[inline(always)]
    pub const fn as_millis(self) -> u64 {
        self.0
    }

    /// Get the timeout as std::time::Duration
    #[inline]
    pub const fn as_duration(self) -> std::time::Duration {
        std::time::Duration::from_millis(self.0)
    }

    /// Check if timeout has expired
    #[inline]
    pub fn is_expired(self, elapsed: std::time::Duration) -> bool {
        elapsed.as_millis() as u64 >= self.0
    }
}

impl Default for TimeoutMs {
    #[inline(always)]
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl fmt::Display for TimeoutMs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 >= 1000 {
            write!(f, "{}s", self.0 / 1000)
        } else {
            write!(f, "{}ms", self.0)
        }
    }
}

/// Retry count for resilience patterns
///
/// Type-safe wrapper for retry attempts with maximum bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RetryCount(u32);

impl RetryCount {
    /// Maximum retry attempts
    pub const MAX_RETRIES: Self = Self(10);

    /// Default retry count
    pub const DEFAULT: Self = Self(3);

    /// No retries
    pub const NONE: Self = Self(0);

    /// Create a new retry count, clamping to MAX_RETRIES
    #[inline]
    pub const fn new(count: u32) -> Self {
        if count > Self::MAX_RETRIES.0 {
            Self::MAX_RETRIES
        } else {
            Self(count)
        }
    }

    /// Get the retry count value
    #[inline(always)]
    pub const fn value(self) -> u32 {
        self.0
    }

    /// Increment retry count
    #[inline]
    pub fn increment(self) -> Self {
        Self::new(self.0.saturating_add(1))
    }

    /// Check if retries are exhausted
    #[inline(always)]
    pub const fn is_exhausted(self) -> bool {
        self.0 == 0
    }

    /// Decrement retry count
    #[inline]
    pub fn decrement(self) -> Self {
        Self(self.0.saturating_sub(1))
    }
}

impl Default for RetryCount {
    #[inline(always)]
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl fmt::Display for RetryCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} retries", self.0)
    }
}

/// Batch size for parallel processing
///
/// Type-safe wrapper for batch sizes with sensible bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BatchSize(u32);

impl BatchSize {
    /// Minimum batch size
    pub const MIN: Self = Self(1);

    /// Maximum batch size
    pub const MAX: Self = Self(10_000);

    /// Default batch size
    pub const DEFAULT: Self = Self(100);

    /// Small batch
    pub const SMALL: Self = Self(10);

    /// Large batch
    pub const LARGE: Self = Self(1_000);

    /// Create a new batch size, clamping to valid range
    #[inline]
    pub const fn new(size: u32) -> WorkflowResult<Self> {
        if size == 0 {
            return Err(WorkflowError::Validation(
                "Batch size must be at least 1".to_string(),
            ));
        }
        if size > Self::MAX.0 {
            Ok(Self::MAX)
        } else {
            Ok(Self(size))
        }
    }

    /// Create without validation (const version)
    #[inline(always)]
    pub const fn new_unchecked(size: u32) -> Self {
        Self(size)
    }

    /// Get the batch size value
    #[inline(always)]
    pub const fn value(self) -> u32 {
        self.0
    }

    /// Get as usize for indexing
    #[inline(always)]
    pub const fn as_usize(self) -> usize {
        self.0 as usize
    }
}

impl Default for BatchSize {
    #[inline(always)]
    fn default() -> Self {
        Self::DEFAULT
    }
}

impl fmt::Display for BatchSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "batch:{}", self.0)
    }
}

/// Tick count for performance measurement (Chatman Constant: ≤8 ticks)
///
/// Type-safe wrapper for tick counts with performance budget enforcement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TickCount(u64);

impl TickCount {
    /// Chatman Constant: Maximum ticks for hot path (8 ticks)
    pub const CHATMAN_CONSTANT: Self = Self(8);

    /// Zero ticks
    pub const ZERO: Self = Self(0);

    /// Create a new tick count
    #[inline(always)]
    pub const fn new(ticks: u64) -> Self {
        Self(ticks)
    }

    /// Get the tick count value
    #[inline(always)]
    pub const fn value(self) -> u64 {
        self.0
    }

    /// Check if within Chatman Constant budget (≤8 ticks)
    #[inline(always)]
    pub const fn is_within_budget(self) -> bool {
        self.0 <= Self::CHATMAN_CONSTANT.0
    }

    /// Check if exceeds performance budget
    #[inline(always)]
    pub const fn exceeds_budget(self) -> bool {
        self.0 > Self::CHATMAN_CONSTANT.0
    }

    /// Add ticks
    #[inline(always)]
    pub const fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Default for TickCount {
    #[inline(always)]
    fn default() -> Self {
        Self::ZERO
    }
}

impl Add for TickCount {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub for TickCount {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl fmt::Display for TickCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ticks", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_level_constants() {
        assert!(PriorityLevel::CRITICAL.is_critical());
        assert!(PriorityLevel::HIGH.is_high());
        assert!(!PriorityLevel::NORMAL.is_high());
    }

    #[test]
    fn test_timeout_clamping() {
        let timeout = TimeoutMs::new(500_000); // Exceeds max
        assert_eq!(timeout, TimeoutMs::MAX_TIMEOUT);
    }

    #[test]
    fn test_retry_count_operations() {
        let mut retries = RetryCount::new(3);
        retries = retries.decrement();
        assert_eq!(retries.value(), 2);
        assert!(!retries.is_exhausted());
    }

    #[test]
    fn test_tick_count_budget() {
        let within = TickCount::new(5);
        assert!(within.is_within_budget());

        let exceeds = TickCount::new(10);
        assert!(exceeds.exceeds_budget());
    }

    #[test]
    fn test_zero_cost_newtypes() {
        // All newtypes should be zero-cost (same size as wrapped type)
        assert_eq!(
            std::mem::size_of::<PriorityLevel>(),
            std::mem::size_of::<u8>()
        );
        assert_eq!(
            std::mem::size_of::<TimeoutMs>(),
            std::mem::size_of::<u64>()
        );
        assert_eq!(
            std::mem::size_of::<TickCount>(),
            std::mem::size_of::<u64>()
        );
    }
}
