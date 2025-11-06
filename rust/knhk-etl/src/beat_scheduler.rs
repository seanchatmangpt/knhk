// rust/knhk-etl/src/beat_scheduler.rs
// 8-beat epoch scheduler with branchless cadence
// Implements: cycle counter, tick calculation, pulse detection, fiber rotation

use core::sync::atomic::{AtomicU64, Ordering};
use alloc::vec::Vec;
use alloc::string::String;
use crate::fiber::Fiber;
use crate::park::{ParkManager, ExecutionResult};
use crate::ingest::RawTriple;
use crate::ring_conversion::{raw_triples_to_soa, soa_to_raw_triples};
use knhk_hot::BeatScheduler as CBeatScheduler;
use knhk_hot::{DeltaRing, AssertionRing, Receipt as HotReceipt};

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
/// Uses C branchless beat scheduler for cycle/tick/pulse generation
pub struct BeatScheduler {
    /// C beat scheduler initialized flag
    c_beat_initialized: bool,
    /// Delta rings (one per domain) - C SoA rings
    delta_rings: Vec<DeltaRing>,
    /// Assertion rings (one per domain) - C SoA rings
    assertion_rings: Vec<AssertionRing>,
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

        // Create delta rings (one per domain) - C SoA rings
        let mut delta_rings = Vec::with_capacity(domain_count);
        for _ in 0..domain_count {
            delta_rings.push(
                DeltaRing::new(ring_capacity as u64)
                    .map_err(|e| BeatSchedulerError::FiberError(e))?,
            );
        }

        // Create assertion rings (one per domain) - C SoA rings
        let mut assertion_rings = Vec::with_capacity(domain_count);
        for _ in 0..domain_count {
            assertion_rings.push(
                AssertionRing::new(ring_capacity as u64)
                    .map_err(|e| BeatSchedulerError::FiberError(e))?,
            );
        }

        // Create fibers (one per shard)
        let mut fibers = Vec::with_capacity(shard_count);
        for shard_id in 0..shard_count {
            fibers.push(Fiber::new(shard_id as u32, 8)); // Tick budget = 8
        }

        // Initialize C beat scheduler (call once at startup)
        CBeatScheduler::init();

        Ok(Self {
            c_beat_initialized: true,
            delta_rings,
            assertion_rings,
            fibers,
            park_manager: ParkManager::new(),
            shard_count,
            domain_count,
        })
    }

    /// Advance to next beat and execute
    /// Returns current tick (0-7) and pulse flag (true when tick==0)
    pub fn advance_beat(&mut self) -> (u64, bool) {
        // Use C branchless beat scheduler
        let cycle = CBeatScheduler::next();
        
        // Branchless tick calculation: cycle & 0x7
        let tick = CBeatScheduler::tick(cycle);
        
        // Branchless pulse detection: pulse == 1 when tick==0
        let pulse_val = CBeatScheduler::pulse(cycle);
        let pulse = pulse_val == 1;
        
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
        let cycle_id = CBeatScheduler::current();
        
        // Rotate through domains and shards
        for domain_id in 0..self.domain_count {
            // Try to dequeue delta from C delta ring at tick slot
            if let Some((S, P, O, cycle_ids)) = self.delta_rings[domain_id].dequeue(tick, 8) {
                // Convert SoA arrays back to RawTriple for fiber execution
                let delta = soa_to_raw_triples(&S, &P, &O);
                
                // Select fiber based on shard (round-robin or hash-based)
                let fiber_idx = (domain_id + slot) % self.shard_count;
                let fiber = &mut self.fibers[fiber_idx];

                // Execute fiber for this tick
                let result = fiber.execute_tick(tick, &delta);

                // Handle result (parked or completed)
                match result {
                    ExecutionResult::Completed { action, receipt } => {
                        // Convert receipt to C receipt and enqueue to assertion ring
                        let hot_receipt = HotReceipt {
                            cycle_id: receipt.cycle_id,
                            shard_id: receipt.shard_id,
                            hook_id: receipt.hook_id,
                            ticks: receipt.ticks,
                            lanes: receipt.lanes,
                            span_id: receipt.span_id,
                            a_hash: receipt.a_hash,
                        };
                        
                        // Convert action payload back to SoA for assertion ring
                        // Note: For now, we use the original delta SoA arrays
                        if let Err(_e) = self.assertion_rings[domain_id].enqueue(tick, &S, &P, &O, &hot_receipt) {
                            // Ring full - park the result (use TickBudgetExceeded as closest match)
                            self.park_manager.park(delta, receipt, crate::park::ParkCause::TickBudgetExceeded, cycle_id, tick);
                        }
                    }
                    ExecutionResult::Parked { delta, receipt, cause } => {
                        self.park_manager.park(delta, receipt, cause, cycle_id, tick);
                    }
                }
            }
        }
    }

    /// Commit cycle on pulse boundary
    /// This is where receipts are finalized and lockchain is updated
    fn commit_cycle(&mut self) {
        // Dequeue from assertion rings for all tick slots (0-7)
        for domain_id in 0..self.domain_count {
            for tick in 0..8 {
                if let Some((S, P, O, receipts)) = self.assertion_rings[domain_id].dequeue(tick, 8) {
                    // Process receipts and assertions
                    // Note: Full commit cycle implementation planned for v1.0:
                    // 1. Verify hash(A) = hash(μ(O))
                    // 2. Append receipt to lockchain
                    // 3. Emit action to output
                    // For now, receipts are collected but not processed further
                    for receipt in &receipts {
                        // Receipts are available for lockchain/emit stages
                        // This is where hash verification and lockchain append would happen
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
    /// * `cycle_id` - Current cycle ID (stamped by sidecar, used for validation)
    pub fn enqueue_delta(
        &self,
        domain_id: usize,
        delta: Vec<RawTriple>,
        cycle_id: u64,
    ) -> Result<(), BeatSchedulerError> {
        if domain_id >= self.domain_count {
            return Err(BeatSchedulerError::InvalidDomainCount);
        }

        // Convert RawTriple to SoA arrays
        let (S, P, O) = raw_triples_to_soa(&delta)
            .map_err(|e| BeatSchedulerError::FiberError(e))?;

        // Get current tick from cycle_id
        let tick = CBeatScheduler::tick(cycle_id);

        // Enqueue to C delta ring at tick slot
        self.delta_rings[domain_id]
            .enqueue(tick, &S, &P, &O, cycle_id)
            .map_err(|e| BeatSchedulerError::FiberError(e))
    }

    /// Get current cycle
    pub fn current_cycle(&self) -> u64 {
        CBeatScheduler::current()
    }

    /// Get current tick (0-7)
    pub fn current_tick(&self) -> u64 {
        let cycle = self.current_cycle();
        CBeatScheduler::tick(cycle)
    }

    /// Get pulse flag (true when tick==0)
    /// Note: This checks the current cycle, which may have advanced since last advance_beat()
    pub fn is_pulse(&self) -> bool {
        let cycle = self.current_cycle();
        CBeatScheduler::pulse(cycle) == 1
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
        let scheduler = match BeatScheduler::new(4, 2, 8) {
            Ok(s) => s,
            Err(e) => panic!("Failed to create beat scheduler: {:?}", e),
        };
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
        let mut scheduler = match BeatScheduler::new(4, 1, 8) {
            Ok(s) => s,
            Err(e) => panic!("Failed to create beat scheduler: {:?}", e),
        };
        
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
        let scheduler = match BeatScheduler::new(4, 1, 8) {
            Ok(s) => s,
            Err(e) => panic!("Failed to create beat scheduler: {:?}", e),
        };
        
        let delta = vec![RawTriple {
            subject: "s1".to_string(),
            predicate: "p1".to_string(),
            object: "o1".to_string(),
            graph: None,
        }];
        
        // Initialize beat scheduler first
        CBeatScheduler::init();
        let cycle_id = CBeatScheduler::current();
        assert!(scheduler.enqueue_delta(0, delta, cycle_id).is_ok());
    }

    #[test]
    fn test_beat_scheduler_integration() {
        // Integration test: enqueue → execute → commit cycle
        CBeatScheduler::init();
        let mut scheduler = match BeatScheduler::new(2, 1, 8) {
            Ok(s) => s,
            Err(e) => panic!("Failed to create beat scheduler: {:?}", e),
        };
        
        // Enqueue delta
        let delta = vec![RawTriple {
            subject: "http://example.org/s1".to_string(),
            predicate: "http://example.org/p1".to_string(),
            object: "http://example.org/o1".to_string(),
            graph: None,
        }];
        
        let cycle_id = CBeatScheduler::current();
        assert!(scheduler.enqueue_delta(0, delta, cycle_id).is_ok());
        
        // Advance beat and execute tick
        let (tick, pulse) = scheduler.advance_beat();
        assert!(tick < 8);
        
        // Commit cycle on pulse boundary
        if pulse {
            scheduler.commit_cycle();
        }
    }

    #[test]
    fn test_beat_scheduler_tick_calculation() {
        CBeatScheduler::init();
        let _scheduler = match BeatScheduler::new(4, 1, 8) {
            Ok(s) => s,
            Err(e) => panic!("Failed to create beat scheduler: {:?}", e),
        };
        
        // Test branchless modulo-8 calculation using C beat scheduler
        for cycle in 0..16 {
            let tick = CBeatScheduler::tick(cycle);
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

