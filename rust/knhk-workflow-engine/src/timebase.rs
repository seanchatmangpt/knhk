//! Timebase abstraction for deterministic time management
//!
//! Provides trait-based clock system with real (`SysClock`) and simulated (`SimClock`) implementations
//! for production and testing scenarios.

use async_trait::async_trait;
use std::cmp::Reverse;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::oneshot;

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

/// Real clock implementation for production
#[derive(Clone, Default)]
pub struct SysClock;

#[async_trait]
impl Timebase for SysClock {
    fn now_wall(&self) -> SystemTime {
        SystemTime::now()
    }

    fn now_mono(&self) -> Instant {
        Instant::now()
    }

    fn scale(&self) -> f64 {
        1.0
    }

    async fn sleep(&self, d: Duration) {
        tokio::time::sleep(d).await;
    }

    async fn sleep_until_wall(&self, t: SystemTime) {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let tgt = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
        let duration = tgt.saturating_sub(now);
        tokio::time::sleep(duration).await;
    }

    async fn sleep_until_mono(&self, t: Instant) {
        tokio::time::sleep_until(t.into()).await;
    }
}

/// Simulated clock for testing and model runs
#[derive(Clone)]
pub struct SimClock {
    /// Monotonic epoch
    mono: Arc<parking_lot::Mutex<Instant>>,
    /// Wall epoch
    wall: Arc<parking_lot::Mutex<SystemTime>>,
    /// Heap of (due_mono, id, waker)
    q: Arc<
        parking_lot::Mutex<
            std::collections::BinaryHeap<(
                std::cmp::Reverse<Instant>,
                u64,
                futures::future::BoxFuture<'static, ()>,
            )>,
        >,
    >,
    /// Time scale factor
    scale: Arc<parking_lot::Mutex<f64>>,
    /// Next ID for scheduled tasks
    next_id: Arc<parking_lot::Mutex<u64>>,
}

impl SimClock {
    /// Create a new simulated clock
    pub fn new(start_wall: SystemTime, start_mono: Instant, scale: f64) -> Self {
        Self {
            mono: Arc::new(parking_lot::Mutex::new(start_mono)),
            wall: Arc::new(parking_lot::Mutex::new(start_wall)),
            q: Arc::new(parking_lot::Mutex::new(std::collections::BinaryHeap::new())),
            scale: Arc::new(parking_lot::Mutex::new(scale)),
            next_id: Arc::new(parking_lot::Mutex::new(0)),
        }
    }

    /// Freeze time (set scale to 0.0)
    pub fn freeze(&self) {
        self.set_scale(0.0);
    }

    /// Set time scale factor
    pub fn set_scale(&self, s: f64) {
        *self.scale.lock() = s;
    }

    /// Warp monotonic time forward by delta and run due tasks
    pub fn warp_mono(&self, delta: Duration) {
        let mut mono = self.mono.lock();
        let mut wall = self.wall.lock();
        *mono += delta;
        // Scale wall time by scale factor
        let scale = *self.scale.lock();
        if scale > 0.0 {
            let scaled_delta = Duration::from_secs_f64(delta.as_secs_f64() * scale);
            *wall += scaled_delta;
        }
        drop(mono);
        drop(wall);
        self.run_due();
    }

    /// Set wall clock time and realign
    pub fn set_wall(&self, t: SystemTime) {
        *self.wall.lock() = t;
    }

    /// Jump to a specific business day (helper for tests)
    pub fn jump_to_business_day(&self, day_yyyymmdd: &str) -> Result<(), String> {
        // Parse date string (YYYY-MM-DD)
        let parts: Vec<&str> = day_yyyymmdd.split('-').collect();
        if parts.len() != 3 {
            return Err("Invalid date format, expected YYYY-MM-DD".to_string());
        }

        let year: i32 = parts[0].parse().map_err(|_| "Invalid year")?;
        let month: u32 = parts[1].parse().map_err(|_| "Invalid month")?;
        let day: u32 = parts[2].parse().map_err(|_| "Invalid day")?;

        use chrono::{NaiveDate, NaiveDateTime};
        let date =
            NaiveDate::from_ymd_opt(year, month, day).ok_or_else(|| "Invalid date".to_string())?;
        let datetime = date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| "Invalid datetime".to_string())?;

        let timestamp = datetime.and_utc().timestamp();
        let system_time = SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp as u64);
        self.set_wall(system_time);

        Ok(())
    }

    /// Run all due tasks from the heap
    fn run_due(&self) {
        let mut q = self.q.lock();
        let mono = *self.mono.lock();
        let mut to_poll = Vec::new();

        // Pop all due tasks
        while let Some((Reverse(due), _id, fut)) = q.peek() {
            if *due <= mono {
                if let Some((_, _, fut)) = q.pop() {
                    to_poll.push(fut);
                }
            } else {
                break;
            }
        }

        drop(q);
        drop(mono);

        // Poll futures (they will wake via oneshot channels)
        for fut in to_poll {
            // Futures are already boxed and ready to poll
            // In a real implementation, we'd use a waker here
            // For now, we rely on the oneshot channel mechanism
            drop(fut);
        }
    }
}

#[async_trait]
impl Timebase for SimClock {
    fn now_wall(&self) -> SystemTime {
        *self.wall.lock()
    }

    fn now_mono(&self) -> Instant {
        *self.mono.lock()
    }

    fn scale(&self) -> f64 {
        *self.scale.lock()
    }

    async fn sleep(&self, d: Duration) {
        self.sleep_until_mono(self.now_mono() + d).await;
    }

    async fn sleep_until_wall(&self, t: SystemTime) {
        let now = self
            .now_wall()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default();
        let tgt = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
        let duration = tgt.saturating_sub(now);
        self.sleep(duration).await;
    }

    async fn sleep_until_mono(&self, t: Instant) {
        let (tx, rx) = oneshot::channel::<()>();
        let fut: futures::future::BoxFuture<'static, ()> = Box::pin(async move {
            let _ = tx.send(());
        });

        let id = {
            let mut n = self.next_id.lock();
            *n += 1;
            *n
        };

        self.q.lock().push((std::cmp::Reverse(t), id, fut));

        // Block until SimClock.warp_mono() runs_due and sends the signal
        let _ = rx.await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sys_clock() {
        let clock = SysClock;
        let now_wall = clock.now_wall();
        let now_mono = clock.now_mono();

        assert!(now_wall > SystemTime::UNIX_EPOCH);
        assert!(now_mono.elapsed() < Duration::from_secs(1));

        let start = Instant::now();
        clock.sleep(Duration::from_millis(10)).await;
        assert!(start.elapsed() >= Duration::from_millis(10));
    }

    #[tokio::test]
    async fn test_sim_clock_freeze() {
        let clock = Arc::new(SimClock::new(SystemTime::UNIX_EPOCH, Instant::now(), 0.0));
        clock.freeze();

        let mono1 = clock.now_mono();
        tokio::time::sleep(Duration::from_millis(10)).await;
        let mono2 = clock.now_mono();

        // Time should be frozen
        assert_eq!(mono1, mono2);
    }

    #[tokio::test]
    async fn test_sim_clock_warp() {
        let clock = Arc::new(SimClock::new(SystemTime::UNIX_EPOCH, Instant::now(), 1.0));

        let mono1 = clock.now_mono();
        clock.warp_mono(Duration::from_secs(10));
        let mono2 = clock.now_mono();

        assert!(mono2 >= mono1 + Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_sim_clock_jump_to_business_day() {
        let clock = Arc::new(SimClock::new(SystemTime::UNIX_EPOCH, Instant::now(), 1.0));

        clock.jump_to_business_day("2025-01-15").unwrap();
        let wall = clock.now_wall();
        let duration = wall.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        assert!(duration.as_secs() > 0);
    }
}
