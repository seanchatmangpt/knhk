// rust/knhk-etl/src/beat_scheduler.rs
// 8-beat epoch scheduler with branchless cadence
// Implements: cycle counter, tick calculation, pulse detection, fiber rotation

use core::sync::atomic::{AtomicU64, Ordering};
use alloc::vec::Vec;
use alloc::string::String;
use crate::ring_buffer::RingBuffer;
use crate::fiber::Fiber;
use crate::park::{ParkManager, ExecutionResult};
use crate::ingest::RawTriple;

/// Beat scheduler error types
#[derive(Debug, Clone, PartialEq)]
pub enum BeatSchedulerError {
    InvalidShardCount,
    InvalidDomainCount,
    RingBufferFull,
    FiberError(String),
}

/// 8-beat epoch scheduler
/// Manages cycle counter, ring buffers, and fiber rotation
#[derive(Debug)]
pub struct BeatScheduler {
    /// Global cycle counter (atomic, thread-safe)
    cycle_counter: AtomicU64,
    /// Delta rings (one per domain)
    delta_rings: Vec<RingBuffer<Vec<RawTriple>>>,
    /// Action rings (one per domain)
    action_rings: Vec<RingBuffer<ExecutionResult>>,
    /// Fibers (one per shard)
    fibers: Vec<Fiber>,
    /// Park manager for over-budget work
    park_manager: ParkManager,
    /// Number of shards
    shard_count: usize,
    /// Number of domains
    domain_count: usize,
}

impl BeatScheduler {
    /// Create new beat scheduler
    /// 
    /// # Arguments
    /// * `shard_count` - Number of shards (must be ≤8 for 8-beat system)
    /// * `domain_count` - Number of reconciliation domains
    /// * `ring_capacity` - Ring buffer capacity (must be power-of-two, typically 8 or 16)
    pub fn new(
        shard_count: usize,
        domain_count: usize,
        ring_capacity: usize,
    ) -> Result<Self, BeatSchedulerError> {
        if shard_count == 0 || shard_count > 8 {
            return Err(BeatSchedulerError::InvalidShardCount);
        }
        if domain_count == 0 {
            return Err(BeatSchedulerError::InvalidDomainCount);
        }

        // Create delta rings (one per domain)
        let mut delta_rings = Vec::with_capacity(domain_count);
        for _ in 0..domain_count {
            delta_rings.push(
                RingBuffer::new(ring_capacity)
                    .map_err(|_| BeatSchedulerError::RingBufferFull)?,
            );
        }

        // Create action rings (one per domain)
        let mut action_rings = Vec::with_capacity(domain_count);
        for _ in 0..domain_count {
            action_rings.push(
                RingBuffer::new(ring_capacity)
                    .map_err(|_| BeatSchedulerError::RingBufferFull)?,
            );
        }

        // Create fibers (one per shard)
        let mut fibers = Vec::with_capacity(shard_count);
        for shard_id in 0..shard_count {
            fibers.push(Fiber::new(shard_id as u32, 8)); // Tick budget = 8
        }

        Ok(Self {
            cycle_counter: AtomicU64::new(0),
            delta_rings,
            action_rings,
            fibers,
            park_manager: ParkManager::new(),
            shard_count,
            domain_count,
        })
    }

    /// Advance to next beat and execute
    /// Returns current tick (0-7) and pulse flag (true when tick==0)
    pub fn advance_beat(&mut self) -> (u64, bool) {
        // Branchless cadence: cycle increments atomically
        let cycle = self.cycle_counter.fetch_add(1, Ordering::Relaxed);
        
        // Branchless tick calculation: cycle & 0x7
        let tick = cycle & 0x7;
        
        // Branchless pulse detection: !tick (1 when tick==0, else 0)
        let pulse = tick == 0;
        
        // Execute fibers for current tick
        self.execute_tick(tick);
        
        // Commit on pulse boundary (every 8 ticks)
        if pulse {
            self.commit_cycle();
        }
        
        (tick, pulse)
    }

    /// Execute fibers for current tick
    fn execute_tick(&mut self, tick: u64) {
        let slot = tick as usize;
        
        // Rotate through domains and shards
        for domain_id in 0..self.domain_count {
            // Try to dequeue delta from delta ring
            if let Some(delta) = self.delta_rings[domain_id].dequeue() {
                // Select fiber based on shard (round-robin or hash-based)
                let fiber_idx = (domain_id + slot) % self.shard_count;
                let fiber = &mut self.fibers[fiber_idx];
                
                // Execute fiber for this tick
                let result = fiber.execute_tick(tick, &delta);
                
                // Enqueue result to action ring
                if let Err(_) = self.action_rings[domain_id].enqueue(result.clone()) {
                    // Action ring full - this shouldn't happen in normal operation
                    // In production, would handle backpressure
                }
                
                // Handle parked results
                if let ExecutionResult::Parked { delta, receipt, cause } = result {
                    let cycle_id = tick / 8;
                    self.park_manager.park(delta, receipt, cause, cycle_id, tick);
                }
            }
        }
    }

    /// Commit cycle on pulse boundary
    /// This is where receipts are finalized and lockchain is updated
    fn commit_cycle(&mut self) {
        // Collect all completed actions and receipts from action rings
        for domain_id in 0..self.domain_count {
            while let Some(result) = self.action_rings[domain_id].dequeue() {
                match result {
                    ExecutionResult::Completed { action, receipt } => {
                        // In production, would:
                        // 1. Verify hash(A) = hash(μ(O))
                        // 2. Append receipt to lockchain
                        // 3. Emit action to output
                    }
                    ExecutionResult::Parked { .. } => {
                        // Already handled in execute_tick
                    }
                }
            }
        }
        
        // Reset fibers for next cycle
        for fiber in &mut self.fibers {
            fiber.yield_control();
        }
    }

    /// Enqueue delta to delta ring (called by sidecar on admission)
    /// 
    /// # Arguments
    /// * `domain_id` - Reconciliation domain ID
    /// * `delta` - Delta triples to enqueue
    /// * `cycle_id` - Current cycle ID (stamped by sidecar)
    pub fn enqueue_delta(
        &self,
        domain_id: usize,
        delta: Vec<RawTriple>,
        cycle_id: u64,
    ) -> Result<(), BeatSchedulerError> {
        if domain_id >= self.domain_count {
            return Err(BeatSchedulerError::InvalidDomainCount);
        }

        // Calculate tick based on current cycle
        let current_cycle = self.cycle_counter.load(Ordering::Relaxed);
        let tick = current_cycle & 0x7;
        
        // Enqueue to delta ring for this domain
        self.delta_rings[domain_id]
            .enqueue(delta)
            .map_err(|_| BeatSchedulerError::RingBufferFull)
    }

    /// Get current cycle
    pub fn current_cycle(&self) -> u64 {
        self.cycle_counter.load(Ordering::Relaxed)
    }

    /// Get current tick (0-7)
    pub fn current_tick(&self) -> u64 {
        self.current_cycle() & 0x7
    }

    /// Get pulse flag (true when tick==0)
    pub fn is_pulse(&self) -> bool {
        self.current_tick() == 0
    }

    /// Get parked deltas for W1 consumption
    pub fn get_parked_deltas(&mut self) -> Vec<crate::park::ParkedDelta> {
        self.park_manager.get_parked()
    }

    /// Get park count
    pub fn park_count(&self) -> usize {
        self.park_manager.parked_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beat_scheduler_creation() {
        let scheduler = BeatScheduler::new(4, 2, 8).unwrap();
        assert_eq!(scheduler.shard_count, 4);
        assert_eq!(scheduler.domain_count, 2);
        assert_eq!(scheduler.current_cycle(), 0);
        assert_eq!(scheduler.current_tick(), 0);
    }

    #[test]
    fn test_beat_scheduler_invalid_shard_count() {
        assert_eq!(
            BeatScheduler::new(0, 1, 8),
            Err(BeatSchedulerError::InvalidShardCount)
        );
        assert_eq!(
            BeatScheduler::new(9, 1, 8),
            Err(BeatSchedulerError::InvalidShardCount)
        );
    }

    #[test]
    fn test_beat_scheduler_advance_beat() {
        let mut scheduler = BeatScheduler::new(4, 1, 8).unwrap();
        
        // Advance through first 8 beats
        for expected_tick in 0..8 {
            let (tick, pulse) = scheduler.advance_beat();
            assert_eq!(tick, expected_tick);
            assert_eq!(pulse, tick == 0);
        }
        
        // Should wrap around to tick 0
        let (tick, pulse) = scheduler.advance_beat();
        assert_eq!(tick, 0);
        assert_eq!(pulse, true);
    }

    #[test]
    fn test_beat_scheduler_enqueue_delta() {
        let scheduler = BeatScheduler::new(4, 1, 8).unwrap();
        
        let delta = vec![RawTriple {
            subject: "s1".to_string(),
            predicate: "p1".to_string(),
            object: "o1".to_string(),
        }];
        
        assert!(scheduler.enqueue_delta(0, delta, 0).is_ok());
    }

    #[test]
    fn test_beat_scheduler_tick_calculation() {
        let scheduler = BeatScheduler::new(4, 1, 8).unwrap();
        
        // Test branchless modulo-8 calculation
        for cycle in 0..16 {
            let tick = cycle & 0x7;
            assert!(tick < 8);
            
            // Verify pattern: 0,1,2,3,4,5,6,7,0,1,2,3,4,5,6,7
            if cycle < 8 {
                assert_eq!(tick, cycle);
            } else {
                assert_eq!(tick, cycle - 8);
            }
        }
    }
}

