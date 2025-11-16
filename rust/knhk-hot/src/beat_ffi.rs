// knhk-hot: Beat scheduler FFI bindings
// 8-beat epoch scheduler: branchless cycle/tick/pulse generation

#![allow(non_camel_case_types)]

use std::sync::atomic::{AtomicU64, Ordering};

// Global cycle counter (shared across all threads/pods)
static KNHK_GLOBAL_CYCLE: AtomicU64 = AtomicU64::new(0);

// Initialize beat scheduler (call once at startup)
// C function from libknhk.a
#[link(name = "knhk")]
extern "C" {
    pub fn knhk_beat_init();
}

// The following functions are static inline in C header knhk/beat.h
// We implement them in Rust since they're simple branchless operations

/// Advance cycle counter atomically, return old cycle value then increment
/// Branchless: single atomic operation
pub fn knhk_beat_next() -> u64 {
    KNHK_GLOBAL_CYCLE.fetch_add(1, Ordering::SeqCst)
}

/// Extract tick from cycle (0..7)
/// Branchless: bitwise mask operation
pub fn knhk_beat_tick(cycle: u64) -> u64 {
    cycle & 0x7
}

/// Compute pulse signal (1 when tick==0, else 0)
/// Branchless: mask-based, no conditional branches
pub fn knhk_beat_pulse(cycle: u64) -> u64 {
    let tick = cycle & 0x7;
    // Branchless: return 1 if tick==0, else 0
    // Use arithmetic underflow: when tick==0, (tick - 1) wraps to 0xFF...
    // Right-shift by 63 gives 1 when tick==0, else 0
    ((tick.wrapping_sub(1)) >> 63) & 1
}

/// Get current cycle without incrementing
pub fn knhk_beat_current() -> u64 {
    KNHK_GLOBAL_CYCLE.load(Ordering::SeqCst)
}

/// Safe wrapper for beat scheduler
pub struct BeatScheduler;

impl BeatScheduler {
    /// Initialize beat scheduler (call once at startup)
    pub fn init() {
        unsafe { knhk_beat_init() }
        // Reset the Rust-side counter to match C-side initialization
        KNHK_GLOBAL_CYCLE.store(0, Ordering::SeqCst);
    }

    /// Advance to next beat and return cycle
    pub fn next() -> u64 {
        knhk_beat_next()
    }

    /// Get tick from cycle (0..7)
    pub fn tick(cycle: u64) -> u64 {
        knhk_beat_tick(cycle)
    }

    /// Get pulse signal from cycle (1 when tick==0, else 0)
    pub fn pulse(cycle: u64) -> u64 {
        knhk_beat_pulse(cycle)
    }

    /// Get current cycle without incrementing
    pub fn current() -> u64 {
        knhk_beat_current()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beat_init() {
        BeatScheduler::init();
        let cycle = BeatScheduler::current();
        assert_eq!(cycle, 0);
    }

    #[test]
    fn test_beat_next() {
        BeatScheduler::init();
        let cycle1 = BeatScheduler::next();
        let cycle2 = BeatScheduler::next();
        assert_eq!(cycle2, cycle1 + 1);
    }

    #[test]
    fn test_beat_tick() {
        for i in 0..16 {
            let tick = BeatScheduler::tick(i);
            assert!(tick < 8);
            assert_eq!(tick, i & 0x7);
        }
    }

    #[test]
    fn test_beat_pulse() {
        // Pulse should be 1 when tick==0, else 0
        assert_eq!(BeatScheduler::pulse(0), 1);
        assert_eq!(BeatScheduler::pulse(8), 1);
        for i in 1..8 {
            assert_eq!(BeatScheduler::pulse(i), 0);
        }
    }
}
