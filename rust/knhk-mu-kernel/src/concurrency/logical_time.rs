//! Logical Time and Causality Tracking
//!
//! Implements Lamport clocks for deterministic total ordering of events.

use core::cmp::Ordering as CmpOrdering;
use core::sync::atomic::{AtomicU64, Ordering};

/// Logical timestamp (Lamport clock)
///
/// Provides happened-before relationship for deterministic ordering.
///
/// # Properties
///
/// 1. **Monotonic**: Timestamps always increase
/// 2. **Causal**: a → b implies timestamp(a) < timestamp(b)
/// 3. **Total Order**: Any two events can be compared
///
/// # Usage
///
/// ```rust,no_run
/// use knhk_mu_kernel::concurrency::LogicalClock;
///
/// let clock = LogicalClock::new();
///
/// // Generate timestamp for local event
/// let t1 = clock.tick();
///
/// // Receive message with remote timestamp
/// let t2 = clock.recv(remote_timestamp);
///
/// // t2 > t1 (happened-after)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Timestamp(u64);

impl Timestamp {
    /// Create timestamp from raw value
    #[inline(always)]
    pub const fn from_raw(value: u64) -> Self {
        Self(value)
    }

    /// Get raw value
    #[inline(always)]
    pub const fn as_raw(self) -> u64 {
        self.0
    }

    /// Zero timestamp (initial)
    #[inline(always)]
    pub const fn zero() -> Self {
        Self(0)
    }

    /// Maximum timestamp
    #[inline(always)]
    pub const fn max() -> Self {
        Self(u64::MAX)
    }

    /// Increment timestamp
    #[inline(always)]
    pub const fn increment(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    /// Take maximum of two timestamps
    #[inline(always)]
    pub const fn max_with(self, other: Self) -> Self {
        if self.0 > other.0 {
            self
        } else {
            other
        }
    }
}

/// Logical clock (Lamport clock)
///
/// Thread-safe, atomic clock for generating logical timestamps.
#[repr(C, align(64))]
pub struct LogicalClock {
    /// Current timestamp (atomic)
    timestamp: AtomicU64,
}

impl LogicalClock {
    /// Create new logical clock
    #[inline]
    pub const fn new() -> Self {
        Self {
            timestamp: AtomicU64::new(0),
        }
    }

    /// Create clock with initial timestamp
    #[inline]
    pub const fn with_timestamp(timestamp: u64) -> Self {
        Self {
            timestamp: AtomicU64::new(timestamp),
        }
    }

    /// Get current timestamp (without incrementing)
    #[inline]
    pub fn now(&self) -> Timestamp {
        Timestamp(self.timestamp.load(Ordering::SeqCst))
    }

    /// Generate timestamp for local event (increment)
    ///
    /// # Lamport Rule 1
    ///
    /// Each process increments its counter before each event.
    #[inline]
    pub fn tick(&self) -> Timestamp {
        let ts = self.timestamp.fetch_add(1, Ordering::SeqCst);
        Timestamp(ts + 1)
    }

    /// Receive timestamp from remote event (sync)
    ///
    /// # Lamport Rule 2
    ///
    /// When receiving message with timestamp T:
    /// - Set clock to max(local_clock, T) + 1
    ///
    /// # Returns
    ///
    /// New local timestamp after synchronization.
    #[inline]
    pub fn recv(&self, remote: Timestamp) -> Timestamp {
        let mut current = self.timestamp.load(Ordering::SeqCst);

        loop {
            // New timestamp is max(current, remote) + 1
            let new = current.max(remote.0) + 1;

            match self.timestamp.compare_exchange_weak(
                current,
                new,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return Timestamp(new),
                Err(actual) => current = actual,
            }
        }
    }

    /// Fast forward clock to specific timestamp
    ///
    /// Used for deterministic replay.
    #[inline]
    pub fn fast_forward(&self, target: Timestamp) {
        let mut current = self.timestamp.load(Ordering::SeqCst);

        while current < target.0 {
            match self.timestamp.compare_exchange_weak(
                current,
                target.0,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return,
                Err(actual) => {
                    current = actual;
                    if current >= target.0 {
                        return;
                    }
                }
            }
        }
    }

    /// Reset clock (for testing)
    #[cfg(test)]
    pub fn reset(&self) {
        self.timestamp.store(0, Ordering::SeqCst);
    }
}

impl Default for LogicalClock {
    fn default() -> Self {
        Self::new()
    }
}

/// Happened-before relationship (→)
///
/// Defines causal ordering of events.
pub trait HappensBefore {
    /// Check if self happened before other (self → other)
    fn happens_before(&self, other: &Self) -> bool;

    /// Check if concurrent (neither happened before the other)
    fn is_concurrent(&self, other: &Self) -> bool {
        !self.happens_before(other) && !other.happens_before(self)
    }
}

impl HappensBefore for Timestamp {
    #[inline]
    fn happens_before(&self, other: &Self) -> bool {
        self.0 < other.0
    }
}

/// Event with timestamp (for deterministic ordering)
#[derive(Debug, Clone, Copy)]
pub struct TimestampedEvent<T> {
    /// Logical timestamp
    pub timestamp: Timestamp,
    /// Core ID (for tie-breaking)
    pub core_id: u8,
    /// Event data
    pub event: T,
}

impl<T> TimestampedEvent<T> {
    /// Create new timestamped event
    #[inline]
    pub const fn new(timestamp: Timestamp, core_id: u8, event: T) -> Self {
        Self {
            timestamp,
            core_id,
            event,
        }
    }
}

impl<T> PartialEq for TimestampedEvent<T> {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp && self.core_id == other.core_id
    }
}

impl<T> Eq for TimestampedEvent<T> {}

impl<T> PartialOrd for TimestampedEvent<T> {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for TimestampedEvent<T> {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        // Primary: timestamp
        match self.timestamp.cmp(&other.timestamp) {
            CmpOrdering::Equal => {
                // Tie-break: core_id (for determinism)
                self.core_id.cmp(&other.core_id)
            }
            other => other,
        }
    }
}

impl<T> HappensBefore for TimestampedEvent<T> {
    #[inline]
    fn happens_before(&self, other: &Self) -> bool {
        self.timestamp.happens_before(&other.timestamp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_creation() {
        let ts = Timestamp::from_raw(42);
        assert_eq!(ts.as_raw(), 42);

        assert_eq!(Timestamp::zero().as_raw(), 0);
        assert_eq!(Timestamp::max().as_raw(), u64::MAX);
    }

    #[test]
    fn test_timestamp_increment() {
        let ts = Timestamp::from_raw(10);
        let ts2 = ts.increment();
        assert_eq!(ts2.as_raw(), 11);
    }

    #[test]
    fn test_timestamp_max() {
        let ts1 = Timestamp::from_raw(10);
        let ts2 = Timestamp::from_raw(20);

        assert_eq!(ts1.max_with(ts2), ts2);
        assert_eq!(ts2.max_with(ts1), ts2);
    }

    #[test]
    fn test_logical_clock_tick() {
        let clock = LogicalClock::new();

        let t1 = clock.tick();
        assert_eq!(t1.as_raw(), 1);

        let t2 = clock.tick();
        assert_eq!(t2.as_raw(), 2);

        assert!(t1.happens_before(&t2));
    }

    #[test]
    fn test_logical_clock_recv() {
        let clock = LogicalClock::new();

        // Local events
        let t1 = clock.tick(); // 1
        let t2 = clock.tick(); // 2

        // Receive remote timestamp (ahead)
        let remote = Timestamp::from_raw(10);
        let t3 = clock.recv(remote); // max(2, 10) + 1 = 11

        assert!(t2.happens_before(&t3));
        assert_eq!(t3.as_raw(), 11);
    }

    #[test]
    fn test_logical_clock_recv_behind() {
        let clock = LogicalClock::new();

        // Local events
        clock.tick(); // 1
        clock.tick(); // 2
        clock.tick(); // 3

        // Receive remote timestamp (behind)
        let remote = Timestamp::from_raw(1);
        let t4 = clock.recv(remote); // max(3, 1) + 1 = 4

        assert_eq!(t4.as_raw(), 4);
    }

    #[test]
    fn test_happens_before() {
        let t1 = Timestamp::from_raw(10);
        let t2 = Timestamp::from_raw(20);

        assert!(t1.happens_before(&t2));
        assert!(!t2.happens_before(&t1));
        assert!(!t1.is_concurrent(&t2));
    }

    #[test]
    fn test_timestamped_event_ordering() {
        let e1 = TimestampedEvent::new(Timestamp::from_raw(10), 0, "event1");
        let e2 = TimestampedEvent::new(Timestamp::from_raw(20), 0, "event2");
        let e3 = TimestampedEvent::new(Timestamp::from_raw(10), 1, "event3");

        // e1 < e2 (timestamp)
        assert!(e1 < e2);

        // e1 < e3 (same timestamp, core_id tie-break)
        assert!(e1 < e3);

        // Total order: e1 < e3 < e2
        assert!(e1 < e3);
        assert!(e3 < e2);
    }

    #[test]
    fn test_fast_forward() {
        let clock = LogicalClock::new();

        clock.tick(); // 1
        clock.tick(); // 2

        clock.fast_forward(Timestamp::from_raw(100));

        let t = clock.tick();
        assert_eq!(t.as_raw(), 101);
    }

    #[test]
    fn test_concurrent_events() {
        // In different processes, concurrent events can have same timestamp
        // This is theoretical - in practice, happens-before prevents this
        let e1 = TimestampedEvent::new(Timestamp::from_raw(10), 0, "a");
        let e2 = TimestampedEvent::new(Timestamp::from_raw(10), 0, "b");

        // Equal timestamps + core_id = concurrent (but we use core_id to tie-break)
        assert_eq!(e1.cmp(&e2), CmpOrdering::Equal);
    }
}
