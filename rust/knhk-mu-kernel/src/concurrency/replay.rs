//! Deterministic Replay Infrastructure
//!
//! Enables cross-machine reproducibility of scheduler execution.

use alloc::vec::Vec;
use core::slice::Iter;
use crate::concurrency::logical_time::Timestamp;

/// Replay seed (for deterministic initialization)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReplaySeed {
    /// Random seed
    pub seed: u64,
    /// Initial timestamp
    pub initial_timestamp: u64,
    /// Number of cores
    pub cores: u8,
}

impl ReplaySeed {
    /// Create new replay seed
    pub const fn new(seed: u64, initial_timestamp: u64, cores: u8) -> Self {
        Self {
            seed,
            initial_timestamp,
            cores,
        }
    }

    /// Deterministic seed from timestamp
    pub const fn from_timestamp(timestamp: u64) -> Self {
        Self {
            seed: timestamp,
            initial_timestamp: 0,
            cores: 1,
        }
    }
}

/// Replay event (logged during execution)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayEvent {
    /// Task enqueued
    TaskEnqueued {
        task_id: u64,
        core_id: u8,
        timestamp: Timestamp,
    },
    /// Task executed
    TaskExecuted {
        task_id: u64,
        core_id: u8,
        timestamp: Timestamp,
        ticks: u64,
        output_hash: [u64; 4],
    },
    /// Scheduler state change
    StateChange {
        timestamp: Timestamp,
        state_hash: u64,
    },
    /// External input (breaks determinism if not replayed)
    ExternalInput {
        timestamp: Timestamp,
        input_hash: u64,
    },
}

impl ReplayEvent {
    /// Get event timestamp
    pub fn timestamp(&self) -> Timestamp {
        match self {
            Self::TaskEnqueued { timestamp, .. } => *timestamp,
            Self::TaskExecuted { timestamp, .. } => *timestamp,
            Self::StateChange { timestamp, .. } => *timestamp,
            Self::ExternalInput { timestamp, .. } => *timestamp,
        }
    }

    /// Check if event is deterministic
    ///
    /// ExternalInput events are non-deterministic and must be replayed.
    pub const fn is_deterministic(&self) -> bool {
        !matches!(self, Self::ExternalInput { .. })
    }
}

/// Replay log (event log for deterministic replay)
///
/// Records all non-deterministic events during execution.
/// Can be used to replay execution with same results.
///
/// # Properties
///
/// 1. **Complete**: All non-deterministic events logged
/// 2. **Ordered**: Events in timestamp order
/// 3. **Compact**: Only non-deterministic data logged
/// 4. **Verifiable**: Checksums for integrity
pub struct ReplayLog {
    /// Recorded events
    events: Vec<ReplayEvent>,
    /// Checksum (for integrity verification)
    checksum: u64,
}

impl ReplayLog {
    /// Create new replay log
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            checksum: 0,
        }
    }

    /// Record event
    pub fn record(&mut self, event: ReplayEvent) {
        // Update checksum (simple XOR for now)
        self.checksum ^= event.timestamp().as_raw();

        self.events.push(event);
    }

    /// Get events
    pub fn events(&self) -> &[ReplayEvent] {
        &self.events
    }

    /// Get event count
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get checksum
    pub fn checksum(&self) -> u64 {
        self.checksum
    }

    /// Verify checksum
    pub fn verify(&self) -> bool {
        let mut computed = 0u64;
        for event in &self.events {
            computed ^= event.timestamp().as_raw();
        }
        computed == self.checksum
    }

    /// Clear log
    pub fn clear(&mut self) {
        self.events.clear();
        self.checksum = 0;
    }

    /// Create replay iterator
    pub fn replay(&self) -> ReplayIterator {
        ReplayIterator {
            events: self.events.iter(),
        }
    }
}

impl Default for ReplayLog {
    fn default() -> Self {
        Self::new()
    }
}

/// Replay iterator (for deterministic replay)
pub struct ReplayIterator<'a> {
    events: Iter<'a, ReplayEvent>,
}

impl<'a> Iterator for ReplayIterator<'a> {
    type Item = &'a ReplayEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.events.next()
    }
}

/// Deterministic trait (types that support deterministic replay)
pub trait Deterministic {
    /// Replay seed type
    type Seed;

    /// Replay from seed
    fn replay(&self, seed: Self::Seed) -> ReplayIterator;

    /// Extract current seed
    fn seed(&self) -> Self::Seed;
}

/// Replay comparison result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayResult {
    /// Replay matches original exactly
    ExactMatch,
    /// Replay matches with minor differences
    MinorDifference { mismatch_count: usize },
    /// Replay diverged significantly
    Diverged { first_mismatch: usize },
}

/// Compare two replay logs for determinism verification
pub fn compare_replays(original: &ReplayLog, replay: &ReplayLog) -> ReplayResult {
    if original.len() != replay.len() {
        return ReplayResult::Diverged {
            first_mismatch: original.len().min(replay.len()),
        };
    }

    let mut mismatches = 0;
    let mut first_mismatch = None;

    for (i, (orig, repl)) in original.events.iter().zip(replay.events.iter()).enumerate() {
        if orig != repl {
            mismatches += 1;
            if first_mismatch.is_none() {
                first_mismatch = Some(i);
            }
        }
    }

    match mismatches {
        0 => ReplayResult::ExactMatch,
        1..=10 => ReplayResult::MinorDifference {
            mismatch_count: mismatches,
        },
        _ => ReplayResult::Diverged {
            first_mismatch: first_mismatch.unwrap(),
        },
    }
}

/// Replay statistics
#[derive(Debug, Clone, Copy)]
pub struct ReplayStats {
    /// Total events
    pub total_events: usize,
    /// Deterministic events
    pub deterministic_events: usize,
    /// Non-deterministic events
    pub non_deterministic_events: usize,
    /// Unique tasks
    pub unique_tasks: usize,
    /// Total ticks
    pub total_ticks: u64,
}

impl ReplayStats {
    /// Compute statistics from replay log
    pub fn from_log(log: &ReplayLog) -> Self {
        let mut unique_tasks = alloc::collections::BTreeSet::new();
        let mut total_ticks = 0u64;
        let mut deterministic = 0;
        let mut non_deterministic = 0;

        for event in log.events() {
            if event.is_deterministic() {
                deterministic += 1;
            } else {
                non_deterministic += 1;
            }

            match event {
                ReplayEvent::TaskEnqueued { task_id, .. } => {
                    unique_tasks.insert(*task_id);
                }
                ReplayEvent::TaskExecuted {
                    task_id, ticks, ..
                } => {
                    unique_tasks.insert(*task_id);
                    total_ticks += *ticks;
                }
                _ => {}
            }
        }

        Self {
            total_events: log.len(),
            deterministic_events: deterministic,
            non_deterministic_events: non_deterministic,
            unique_tasks: unique_tasks.len(),
            total_ticks,
        }
    }

    /// Calculate determinism ratio (0.0 to 1.0)
    pub fn determinism_ratio(&self) -> f64 {
        if self.total_events == 0 {
            1.0
        } else {
            self.deterministic_events as f64 / self.total_events as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_seed() {
        let seed = ReplaySeed::new(42, 0, 4);
        assert_eq!(seed.seed, 42);
        assert_eq!(seed.initial_timestamp, 0);
        assert_eq!(seed.cores, 4);
    }

    #[test]
    fn test_replay_log() {
        let mut log = ReplayLog::new();

        log.record(ReplayEvent::TaskEnqueued {
            task_id: 1,
            core_id: 0,
            timestamp: Timestamp::from_raw(1),
        });

        log.record(ReplayEvent::TaskExecuted {
            task_id: 1,
            core_id: 0,
            timestamp: Timestamp::from_raw(2),
            ticks: 5,
            output_hash: [0; 4],
        });

        assert_eq!(log.len(), 2);
        assert!(log.verify());
    }

    #[test]
    fn test_replay_log_checksum() {
        let mut log = ReplayLog::new();

        log.record(ReplayEvent::TaskEnqueued {
            task_id: 1,
            core_id: 0,
            timestamp: Timestamp::from_raw(10),
        });

        let checksum1 = log.checksum();
        assert!(log.verify());

        log.record(ReplayEvent::TaskExecuted {
            task_id: 1,
            core_id: 0,
            timestamp: Timestamp::from_raw(20),
            ticks: 5,
            output_hash: [0; 4],
        });

        let checksum2 = log.checksum();
        assert_ne!(checksum1, checksum2);
        assert!(log.verify());
    }

    #[test]
    fn test_compare_replays_exact() {
        let mut log1 = ReplayLog::new();
        let mut log2 = ReplayLog::new();

        let event = ReplayEvent::TaskEnqueued {
            task_id: 1,
            core_id: 0,
            timestamp: Timestamp::from_raw(1),
        };

        log1.record(event);
        log2.record(event);

        let result = compare_replays(&log1, &log2);
        assert_eq!(result, ReplayResult::ExactMatch);
    }

    #[test]
    fn test_compare_replays_diverged() {
        let mut log1 = ReplayLog::new();
        let mut log2 = ReplayLog::new();

        log1.record(ReplayEvent::TaskEnqueued {
            task_id: 1,
            core_id: 0,
            timestamp: Timestamp::from_raw(1),
        });

        log2.record(ReplayEvent::TaskEnqueued {
            task_id: 2,
            core_id: 0,
            timestamp: Timestamp::from_raw(1),
        });

        let result = compare_replays(&log1, &log2);
        match result {
            ReplayResult::MinorDifference { .. } => {}
            _ => panic!("Expected minor difference"),
        }
    }

    #[test]
    fn test_replay_stats() {
        let mut log = ReplayLog::new();

        log.record(ReplayEvent::TaskEnqueued {
            task_id: 1,
            core_id: 0,
            timestamp: Timestamp::from_raw(1),
        });

        log.record(ReplayEvent::TaskExecuted {
            task_id: 1,
            core_id: 0,
            timestamp: Timestamp::from_raw(2),
            ticks: 5,
            output_hash: [0; 4],
        });

        log.record(ReplayEvent::TaskExecuted {
            task_id: 2,
            core_id: 1,
            timestamp: Timestamp::from_raw(3),
            ticks: 7,
            output_hash: [0; 4],
        });

        let stats = ReplayStats::from_log(&log);

        assert_eq!(stats.total_events, 3);
        assert_eq!(stats.unique_tasks, 2);
        assert_eq!(stats.total_ticks, 12);
        assert_eq!(stats.determinism_ratio(), 1.0);
    }

    #[test]
    fn test_event_determinism() {
        let det_event = ReplayEvent::TaskExecuted {
            task_id: 1,
            core_id: 0,
            timestamp: Timestamp::from_raw(1),
            ticks: 5,
            output_hash: [0; 4],
        };

        let non_det_event = ReplayEvent::ExternalInput {
            timestamp: Timestamp::from_raw(1),
            input_hash: 42,
        };

        assert!(det_event.is_deterministic());
        assert!(!non_det_event.is_deterministic());
    }
}
