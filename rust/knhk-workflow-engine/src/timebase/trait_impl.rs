//! Timebase trait for abstracting time operations

use async_trait::async_trait;
use std::time::{Duration, Instant, SystemTime};

/// Timebase trait for abstracting time operations
#[async_trait]
pub trait Timebase: Send + Sync {
    /// Get wall clock time (civil time for calendars, SLAs)
    fn now_wall(&self) -> SystemTime;

    /// Get monotonic time (for timeouts, intervals)
    fn now_mono(&self) -> Instant;

    /// Get time scale factor (1.0 = real time)
    fn scale(&self) -> f64;

    /// Sleep for duration (completes when virtual time reaches now + d)
    async fn sleep(&self, d: Duration);

    /// Sleep until wall clock time
    async fn sleep_until_wall(&self, t: SystemTime);

    /// Sleep until monotonic time
    async fn sleep_until_mono(&self, t: Instant);
}
