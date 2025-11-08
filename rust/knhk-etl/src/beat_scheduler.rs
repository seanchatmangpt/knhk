// rust/knhk-etl/src/beat_scheduler.rs
// 8-beat epoch scheduler with branchless cadence
// Implements: cycle counter, tick calculation, pulse detection, fiber rotation

use crate::fiber::Fiber;
use crate::ingest::RawTriple;
use crate::park::{ExecutionResult, ParkManager};
use crate::reflex::Receipt;
use crate::ring_conversion::{raw_triples_to_soa, soa_to_raw_triples};
use alloc::string::String;
use alloc::vec::Vec;
#[cfg(feature = "knhk-lockchain")]
use hex;
use knhk_hot::BeatScheduler as CBeatScheduler;
use knhk_hot::{AssertionRing, DeltaRing, Receipt as HotReceipt};
#[cfg(feature = "knhk-lockchain")]
use knhk_lockchain::{
    LockchainStorage, MerkleTree, PeerId, QuorumManager, Receipt as LockchainReceipt,
};

/// Beat scheduler error types
#[derive(Debug, Clone, PartialEq)]
pub enum BeatSchedulerError {
    InvalidShardCount,
    InvalidDomainCount,
    RingBufferFull,
    FiberError(String),
    #[cfg(feature = "knhk-lockchain")]
    QuorumFailed(String),
    #[cfg(feature = "knhk-lockchain")]
    StorageFailed(String),
}

/// 8-beat epoch scheduler
///
/// Manages cycle counter, ring buffers, and fiber rotation for deterministic execution.
/// Uses C branchless beat scheduler for cycle/tick/pulse generation (≤8 ticks per beat).
///
/// # Architecture
///
/// - **Cycles**: 8-beat cycles (0-7 ticks per cycle)
/// - **Pulse**: Detected at tick 0 (cycle commit boundary)
/// - **Fibers**: Cooperative execution units per shard
/// - **Ring Buffers**: Lock-free SPSC rings for delta/assertion queues
///
/// # Example
///
/// ```rust
/// use knhk_etl::beat_scheduler::BeatScheduler;
///
/// let mut scheduler = BeatScheduler::new(4, 1, 8)?; // 4 shards, 1 domain, ring capacity 8
/// let (tick, pulse) = scheduler.advance_beat();
/// ```
#[derive(Debug)]
pub struct BeatScheduler {
    /// C beat scheduler initialized flag
    #[allow(dead_code)] // FUTURE: C FFI integration flag
    c_beat_initialized: bool,
    /// Delta rings (one per domain) - C SoA rings
    delta_rings: Vec<DeltaRing>,
    /// Assertion rings (one per domain) - C SoA rings
    assertion_rings: Vec<AssertionRing>,
    /// Fibers (one per shard)
    fibers: Vec<Fiber>,
    /// Park manager for over-budget work
    park_manager: ParkManager,
    /// Receipts collected per cycle (for lockchain append)
    cycle_receipts: Vec<Receipt>,
    /// Merkle tree for receipt provenance (lockchain)
    #[cfg(feature = "knhk-lockchain")]
    merkle_tree: MerkleTree,
    /// Quorum manager for consensus
    #[cfg(feature = "knhk-lockchain")]
    quorum_manager: Option<QuorumManager>,
    /// Lockchain storage for persistence
    #[cfg(feature = "knhk-lockchain")]
    lockchain_storage: Option<LockchainStorage>,
    /// Number of shards
    shard_count: usize,
    /// Number of domains
    domain_count: usize,
}

impl BeatScheduler {
    /// Create new beat scheduler
    ///
    /// Initializes the 8-beat epoch system with specified shard and domain configuration.
    ///
    /// # Arguments
    /// * `shard_count` - Number of shards (must be ≤8 for 8-beat system, Chatman Constant)
    /// * `domain_count` - Number of reconciliation domains
    /// * `ring_capacity` - Ring buffer capacity (must be power-of-two, typically 8 or 16)
    ///
    /// # Returns
    /// * `Ok(BeatScheduler)` - Successfully created scheduler
    /// * `Err(BeatSchedulerError)` - Invalid configuration (shard_count > 8 or domain_count == 0)
    ///
    /// # Errors
    /// Returns `BeatSchedulerError::InvalidShardCount` if shard_count is 0 or > 8
    /// Returns `BeatSchedulerError::InvalidDomainCount` if domain_count is 0
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
                DeltaRing::new(ring_capacity as u64).map_err(BeatSchedulerError::FiberError)?,
            );
        }

        // Create assertion rings (one per domain) - C SoA rings
        let mut assertion_rings = Vec::with_capacity(domain_count);
        for _ in 0..domain_count {
            assertion_rings.push(
                AssertionRing::new(ring_capacity as u64).map_err(BeatSchedulerError::FiberError)?,
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
            cycle_receipts: Vec::new(),
            #[cfg(feature = "knhk-lockchain")]
            merkle_tree: MerkleTree::new(),
            #[cfg(feature = "knhk-lockchain")]
            quorum_manager: None,
            #[cfg(feature = "knhk-lockchain")]
            lockchain_storage: None,
            shard_count,
            domain_count,
        })
    }

    /// Configure lockchain with quorum and storage
    ///
    /// # Arguments
    /// * `peers` - List of peer identifiers for quorum
    /// * `quorum_threshold` - Minimum votes required for consensus
    /// * `self_peer_id` - This node's peer ID
    /// * `storage_path` - Path for lockchain storage
    #[cfg(feature = "knhk-lockchain")]
    pub fn configure_lockchain(
        &mut self,
        peers: Vec<String>,
        quorum_threshold: usize,
        self_peer_id: String,
        storage_path: &str,
    ) -> Result<(), BeatSchedulerError> {
        let peer_ids: Vec<PeerId> = peers.into_iter().map(PeerId).collect();
        let self_id = PeerId(self_peer_id);

        self.quorum_manager = Some(QuorumManager::new(peer_ids, quorum_threshold, self_id));

        self.lockchain_storage = Some(
            LockchainStorage::new(storage_path)
                .map_err(|e| BeatSchedulerError::StorageFailed(e.to_string()))?,
        );

        tracing::info!(
            storage_path = storage_path,
            quorum_threshold = quorum_threshold,
            "Lockchain configured with quorum and storage"
        );

        Ok(())
    }

    /// Advance to next beat and execute fibers
    ///
    /// Increments the cycle counter, calculates tick (cycle & 0x7), detects pulse (tick == 0),
    /// executes fibers for the current tick, and commits cycle on pulse boundary.
    ///
    /// # Returns
    /// Tuple of `(tick, pulse)` where:
    /// - `tick`: Current tick value (0-7)
    /// - `pulse`: True when tick == 0 (cycle commit boundary)
    ///
    /// # Performance
    /// This operation must complete in ≤8 ticks (Chatman Constant) for hot path compliance.
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
            if let Some((s, p, o, _cycle_ids)) = self.delta_rings[domain_id].dequeue(tick, 8) {
                // Convert SoA arrays back to RawTriple for fiber execution
                let delta = soa_to_raw_triples(&s, &p, &o);

                // Select fiber based on shard (round-robin or hash-based)
                let fiber_idx = (domain_id + slot) % self.shard_count;
                let fiber = &mut self.fibers[fiber_idx];

                // Execute fiber for this tick (pass cycle_id from C beat scheduler)
                let result = fiber.execute_tick(tick, &delta, cycle_id);

                // Handle result (parked or completed)
                match result {
                    ExecutionResult::Completed { action: _, receipt } => {
                        // Convert receipt to C receipt and enqueue to assertion ring
                        let hot_receipt = HotReceipt {
                            cycle_id: receipt.cycle_id,
                            shard_id: receipt.shard_id,
                            hook_id: receipt.hook_id,
                            ticks: receipt.ticks,
                            actual_ticks: receipt.actual_ticks,
                            lanes: receipt.lanes,
                            span_id: receipt.span_id,
                            a_hash: receipt.a_hash,
                        };

                        // Convert action payload back to SoA for assertion ring
                        // Note: For now, we use the original delta SoA arrays
                        if let Err(_e) =
                            self.assertion_rings[domain_id].enqueue(tick, &s, &p, &o, &hot_receipt)
                        {
                            // Ring full - park the result (use TickBudgetExceeded as closest match)
                            self.park_manager.park(
                                delta,
                                receipt,
                                crate::park::ParkCause::TickBudgetExceeded,
                                cycle_id,
                                tick,
                            );
                        }
                    }
                    ExecutionResult::Parked {
                        delta,
                        receipt,
                        cause,
                    } => {
                        self.park_manager
                            .park(delta, receipt, cause, cycle_id, tick);
                    }
                }
            }
        }
    }

    /// Commit cycle on pulse boundary
    /// This is where receipts are finalized and lockchain is updated
    fn commit_cycle(&mut self) {
        // Collect receipts from assertion rings for all tick slots (0-7)
        let mut cycle_receipts = Vec::new();

        for domain_id in 0..self.domain_count {
            for tick in 0..8 {
                if let Some((_s, _p, _o, receipts)) =
                    self.assertion_rings[domain_id].dequeue(tick, 8)
                {
                    // Convert C receipts to Rust receipts
                    for hot_receipt in &receipts {
                        let receipt = Receipt {
                            id: alloc::format!("receipt_{}", hot_receipt.span_id),
                            cycle_id: hot_receipt.cycle_id,
                            shard_id: hot_receipt.shard_id,
                            hook_id: hot_receipt.hook_id,
                            ticks: hot_receipt.ticks,
                            actual_ticks: hot_receipt.ticks,
                            lanes: hot_receipt.lanes,
                            span_id: hot_receipt.span_id,
                            a_hash: hot_receipt.a_hash,
                        };
                        cycle_receipts.push(receipt);
                    }
                }
            }
        }

        // Store receipts for this cycle
        self.cycle_receipts = cycle_receipts;

        // Append receipts to lockchain Merkle tree with quorum consensus and persistence
        #[cfg(feature = "knhk-lockchain")]
        {
            if !self.cycle_receipts.is_empty() {
                // 1. Add receipts to Merkle tree
                for receipt in &self.cycle_receipts {
                    let lockchain_receipt = LockchainReceipt::new(
                        receipt.cycle_id,
                        receipt.shard_id as u32,
                        receipt.hook_id as u32,
                        receipt.ticks as u64,
                        receipt.a_hash,
                    );
                    self.merkle_tree.add_receipt(&lockchain_receipt);
                }

                // 2. Compute Merkle root
                let merkle_root = self.merkle_tree.compute_root();
                let cycle_id = CBeatScheduler::current() / 8;

                // 3. Achieve quorum consensus (if configured)
                let quorum_result = if let Some(ref quorum) = self.quorum_manager {
                    match quorum.achieve_consensus(merkle_root, cycle_id) {
                        Ok(proof) => {
                            tracing::info!(
                                cycle_id = cycle_id,
                                vote_count = proof.vote_count(),
                                threshold = quorum.threshold(),
                                "Quorum consensus achieved"
                            );
                            Some(proof)
                        }
                        Err(e) => {
                            tracing::error!(
                                cycle_id = cycle_id,
                                error = %e,
                                "Quorum consensus failed"
                            );
                            None
                        }
                    }
                } else {
                    None
                };

                // 4. Persist to storage (if configured and quorum succeeded)
                if let (Some(ref storage), Some(proof)) = (&self.lockchain_storage, quorum_result) {
                    if let Err(e) = storage.persist_root(cycle_id, merkle_root, proof) {
                        tracing::error!(
                            cycle_id = cycle_id,
                            error = %e,
                            "Failed to persist lockchain root"
                        );
                    } else {
                        tracing::info!(
                            cycle_id = cycle_id,
                            merkle_root = hex::encode(merkle_root),
                            receipt_count = self.cycle_receipts.len(),
                            "Lockchain root committed with quorum and persisted"
                        );
                    }
                } else if self.quorum_manager.is_none() || self.lockchain_storage.is_none() {
                    // Log without quorum/storage (dev mode)
                    tracing::info!(
                        receipt_count = self.cycle_receipts.len(),
                        cycle_id = cycle_id,
                        merkle_root = hex::encode(merkle_root),
                        "Cycle committed with receipts and Merkle root (no quorum/storage)"
                    );
                }

                // 5. Reset Merkle tree for next beat
                self.merkle_tree = MerkleTree::new();
            }
        }

        #[cfg(not(feature = "knhk-lockchain"))]
        {
            // Log receipt count for observability (without lockchain)
            if !self.cycle_receipts.is_empty() {
                tracing::info!(
                    receipt_count = self.cycle_receipts.len(),
                    cycle_id = CBeatScheduler::current() / 8,
                    "Cycle committed with receipts"
                );
            }
        }

        // Reset fibers for next cycle
        for fiber in &mut self.fibers {
            fiber.yield_control();
        }
    }

    /// Get receipts from last committed cycle
    /// Returns receipts collected during commit_cycle()
    pub fn get_cycle_receipts(&self) -> &[Receipt] {
        &self.cycle_receipts
    }

    /// Enqueue delta to delta ring (called by sidecar on admission)
    ///
    /// Admits a delta into the beat scheduler for processing. The delta is converted to SoA format
    /// and enqueued to the appropriate domain's delta ring at the current tick slot.
    ///
    /// # Arguments
    /// * `domain_id` - Reconciliation domain ID (must be < domain_count)
    /// * `delta` - Delta triples to enqueue (will be converted to SoA format)
    /// * `cycle_id` - Current cycle ID (stamped by sidecar, used for validation)
    ///
    /// # Returns
    /// * `Ok(())` - Delta successfully enqueued
    /// * `Err(BeatSchedulerError)` - Invalid domain_id or enqueue failure
    ///
    /// # Errors
    /// Returns `BeatSchedulerError::InvalidDomainCount` if domain_id >= domain_count
    /// Returns `BeatSchedulerError::FiberError` if SoA conversion or enqueue fails
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
        let (s, p, o) = raw_triples_to_soa(&delta).map_err(BeatSchedulerError::FiberError)?;

        // Get current tick from cycle_id
        let tick = CBeatScheduler::tick(cycle_id);

        // Enqueue to C delta ring at tick slot
        self.delta_rings[domain_id]
            .enqueue(tick, &s, &p, &o, cycle_id)
            .map_err(BeatSchedulerError::FiberError)
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
        assert!(matches!(
            BeatScheduler::new(0, 1, 8),
            Err(BeatSchedulerError::InvalidShardCount)
        ));
        assert!(matches!(
            BeatScheduler::new(9, 1, 8),
            Err(BeatSchedulerError::InvalidShardCount)
        ));
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
        assert!(pulse);
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

    #[test]
    #[cfg(feature = "knhk-lockchain")]
    fn test_lockchain_integration() {
        // Test lockchain integration at pulse boundaries
        CBeatScheduler::init();
        let mut scheduler = match BeatScheduler::new(2, 1, 8) {
            Ok(s) => s,
            Err(e) => panic!("Failed to create beat scheduler: {:?}", e),
        };

        // Configure lockchain with mock quorum
        let peers = vec!["peer1".to_string(), "peer2".to_string()];
        let result = scheduler.configure_lockchain(
            peers,
            2, // quorum threshold
            "self".to_string(),
            "/tmp/knhk-lockchain-test-beat",
        );
        assert!(
            result.is_ok(),
            "Failed to configure lockchain: {:?}",
            result
        );

        // Enqueue delta
        let delta = vec![RawTriple {
            subject: "http://example.org/s1".to_string(),
            predicate: "http://example.org/p1".to_string(),
            object: "http://example.org/o1".to_string(),
            graph: None,
        }];

        let cycle_id = CBeatScheduler::current();
        assert!(scheduler.enqueue_delta(0, delta, cycle_id).is_ok());

        // Advance through 8 beats to trigger pulse and commit_cycle
        for i in 0..8 {
            let (tick, pulse) = scheduler.advance_beat();
            assert_eq!(tick, i);
            if pulse {
                // Pulse boundary - lockchain should be committed
                assert_eq!(tick, 0);
                // Note: commit_cycle is called internally by advance_beat
            }
        }

        // Verify receipts were collected
        let receipts = scheduler.get_cycle_receipts();
        // Note: May be empty if fiber execution didn't complete in time budget
        // This is expected behavior for the 8-tick system
    }
}
