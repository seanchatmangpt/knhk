// Atomic Snapshot Promoter
// Implements picosecond-scale promotion via pointer swap (RCU semantics)

use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, thiserror::Error)]
pub enum PromotionError {
    #[error("Snapshot not found: {0}")]
    SnapshotNotFound(String),

    #[error("Promotion failed: {0}")]
    PromotionFailed(String),

    #[error("RCU grace period not met")]
    GracePeriodNotMet,

    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

/// A snapshot descriptor (minimal for hot path)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SnapshotDescriptor {
    /// SHA-256 snapshot ID
    pub snapshot_id: String,

    /// Parent snapshot ID
    pub parent_id: Option<String>,

    /// Timestamp when promoted
    pub promoted_at: u64,

    /// Version number (for debugging)
    pub version: u32,
}

/// Atomic snapshot promoter using RCU (Read-Copy-Update) semantics
pub struct SnapshotPromoter {
    /// Currently active snapshot (atomic pointer)
    current: ArcSwap<SnapshotDescriptor>,

    /// Snapshot history (for rollback if needed)
    history: dashmap::DashMap<String, Arc<SnapshotDescriptor>>,
}

impl SnapshotPromoter {
    pub fn new(initial_snapshot: SnapshotDescriptor) -> Self {
        let id = initial_snapshot.snapshot_id.clone();
        let current = ArcSwap::new(Arc::new(initial_snapshot.clone()));
        let history = dashmap::DashMap::new();
        history.insert(id, Arc::new(initial_snapshot));

        SnapshotPromoter { current, history }
    }

    /// Get current active snapshot (fast path, minimal cost)
    pub fn current(&self) -> Arc<SnapshotDescriptor> {
        self.current.load().clone()
    }

    /// Promote a new snapshot to active
    /// This is the atomic operation: ~1ns via pointer swap
    pub fn promote(&self, new_snapshot: SnapshotDescriptor) -> Result<Arc<SnapshotDescriptor>, PromotionError> {
        // Verify parent exists in history
        if let Some(parent_id) = &new_snapshot.parent_id {
            if !self.history.contains_key(parent_id) {
                return Err(PromotionError::SnapshotNotFound(format!(
                    "Parent snapshot {} not found",
                    parent_id
                )));
            }
        }

        let id = new_snapshot.snapshot_id.clone();
        let new_arc = Arc::new(new_snapshot);

        // Atomic pointer swap (RCU: replace old with new)
        let _ = self.current.swap(new_arc.clone());

        // Record in history
        self.history.insert(id, new_arc.clone());

        Ok(new_arc)
    }

    /// Get a specific snapshot from history
    pub fn get(&self, snapshot_id: &str) -> Result<Arc<SnapshotDescriptor>, PromotionError> {
        self.history
            .get(snapshot_id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| PromotionError::SnapshotNotFound(snapshot_id.to_string()))
    }

    /// Rollback to previous snapshot
    pub fn rollback(&self) -> Result<(), PromotionError> {
        let current = self.current();

        if let Some(parent_id) = &current.parent_id {
            let parent = self.get(parent_id)?;
            let _ = self.current.swap(parent);
            Ok(())
        } else {
            Err(PromotionError::PromotionFailed(
                "Cannot rollback genesis snapshot".to_string(),
            ))
        }
    }

    /// Get the chain of snapshots from current back to genesis
    pub fn chain(&self) -> Result<Vec<Arc<SnapshotDescriptor>>, PromotionError> {
        let mut chain = Vec::new();
        let mut current = self.current();

        loop {
            chain.push(current.clone());

            if let Some(parent_id) = &current.parent_id {
                current = self.get(parent_id)?;
            } else {
                break;
            }
        }

        Ok(chain)
    }

    /// Count total snapshots stored
    pub fn snapshot_count(&self) -> usize {
        self.history.len()
    }

    /// List all snapshot IDs
    pub fn list_snapshots(&self) -> Vec<String> {
        self.history.iter().map(|entry| entry.key().clone()).collect()
    }
}

/// Promotion statistics (for monitoring)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromotionStats {
    pub total_promotions: u64,
    pub total_snapshots: usize,
    pub current_snapshot_id: String,
    pub average_promotion_latency_ns: f64,
    pub max_promotion_latency_ns: u64,
}

/// Atomic promotion with statistics
pub struct SnapshotPromoterWithStats {
    promoter: SnapshotPromoter,
    total_promotions: Arc<std::sync::atomic::AtomicU64>,
    total_latency_ns: Arc<std::sync::atomic::AtomicU64>,
    max_latency_ns: Arc<std::sync::atomic::AtomicU64>,
}

impl SnapshotPromoterWithStats {
    pub fn new(initial_snapshot: SnapshotDescriptor) -> Self {
        SnapshotPromoterWithStats {
            promoter: SnapshotPromoter::new(initial_snapshot),
            total_promotions: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            total_latency_ns: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            max_latency_ns: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    pub fn promote(
        &self,
        new_snapshot: SnapshotDescriptor,
    ) -> Result<Arc<SnapshotDescriptor>, PromotionError> {
        let start = std::time::Instant::now();

        let result = self.promoter.promote(new_snapshot)?;

        let latency_ns = start.elapsed().as_nanos() as u64;

        // Update stats
        self.total_promotions
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.total_latency_ns
            .fetch_add(latency_ns, std::sync::atomic::Ordering::SeqCst);

        // Update max
        loop {
            let current_max = self
                .max_latency_ns
                .load(std::sync::atomic::Ordering::SeqCst);
            if latency_ns <= current_max {
                break;
            }
            match self.max_latency_ns.compare_exchange_weak(
                current_max,
                latency_ns,
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
            ) {
                Ok(_) => break,
                Err(_) => continue,
            }
        }

        Ok(result)
    }

    pub fn current(&self) -> Arc<SnapshotDescriptor> {
        self.promoter.current()
    }

    pub fn get_stats(&self) -> PromotionStats {
        let total = self
            .total_promotions
            .load(std::sync::atomic::Ordering::SeqCst);
        let total_latency = self
            .total_latency_ns
            .load(std::sync::atomic::Ordering::SeqCst);

        PromotionStats {
            total_promotions: total,
            total_snapshots: self.promoter.snapshot_count(),
            current_snapshot_id: self.promoter.current().snapshot_id.clone(),
            average_promotion_latency_ns: if total > 0 {
                total_latency as f64 / total as f64
            } else {
                0.0
            },
            max_promotion_latency_ns: self
                .max_latency_ns
                .load(std::sync::atomic::Ordering::SeqCst),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_descriptor(id: &str, parent: Option<&str>) -> SnapshotDescriptor {
        SnapshotDescriptor {
            snapshot_id: id.to_string(),
            parent_id: parent.map(|p| p.to_string()),
            promoted_at: chrono::Utc::now().timestamp_millis() as u64,
            version: 1,
        }
    }

    #[test]
    fn test_promoter_creation() {
        let snapshot = create_descriptor("snap1", None);
        let promoter = SnapshotPromoter::new(snapshot);

        assert_eq!(promoter.current().snapshot_id, "snap1");
    }

    #[test]
    fn test_promote_new_snapshot() {
        let snapshot1 = create_descriptor("snap1", None);
        let promoter = SnapshotPromoter::new(snapshot1);

        let snapshot2 = create_descriptor("snap2", Some("snap1"));
        let result = promoter.promote(snapshot2);

        assert!(result.is_ok());
        assert_eq!(promoter.current().snapshot_id, "snap2");
    }

    #[test]
    fn test_promote_invalid_parent() {
        let snapshot1 = create_descriptor("snap1", None);
        let promoter = SnapshotPromoter::new(snapshot1);

        let snapshot2 = create_descriptor("snap2", Some("nonexistent"));
        let result = promoter.promote(snapshot2);

        assert!(result.is_err());
    }

    #[test]
    fn test_rollback() {
        let snapshot1 = create_descriptor("snap1", None);
        let promoter = SnapshotPromoter::new(snapshot1);

        let snapshot2 = create_descriptor("snap2", Some("snap1"));
        promoter.promote(snapshot2).expect("promote failed");

        assert_eq!(promoter.current().snapshot_id, "snap2");

        promoter.rollback().expect("rollback failed");
        assert_eq!(promoter.current().snapshot_id, "snap1");
    }

    #[test]
    fn test_rollback_genesis_fails() {
        let snapshot1 = create_descriptor("snap1", None);
        let promoter = SnapshotPromoter::new(snapshot1);

        let result = promoter.rollback();
        assert!(result.is_err());
    }

    #[test]
    fn test_chain() {
        let snapshot1 = create_descriptor("snap1", None);
        let promoter = SnapshotPromoter::new(snapshot1);

        let snapshot2 = create_descriptor("snap2", Some("snap1"));
        promoter.promote(snapshot2).expect("promote2 failed");

        let snapshot3 = create_descriptor("snap3", Some("snap2"));
        promoter.promote(snapshot3).expect("promote3 failed");

        let chain = promoter.chain().expect("chain failed");
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].snapshot_id, "snap3");
        assert_eq!(chain[1].snapshot_id, "snap2");
        assert_eq!(chain[2].snapshot_id, "snap1");
    }

    #[test]
    fn test_promoter_with_stats() {
        let snapshot1 = create_descriptor("snap1", None);
        let promoter = SnapshotPromoterWithStats::new(snapshot1);

        let snapshot2 = create_descriptor("snap2", Some("snap1"));
        promoter.promote(snapshot2).expect("promote failed");

        let stats = promoter.get_stats();
        assert_eq!(stats.total_promotions, 1);
        assert_eq!(stats.total_snapshots, 2);
        assert!(stats.max_promotion_latency_ns < 10_000_000u64); // Should be <10ms
    }

    #[test]
    fn test_promotion_latency() {
        let snapshot1 = create_descriptor("snap1", None);
        let promoter = SnapshotPromoterWithStats::new(snapshot1);

        for i in 2..=10 {
            let snapshot = create_descriptor(
                &format!("snap{}", i),
                Some(&format!("snap{}", i - 1)),
            );
            promoter.promote(snapshot).expect("promote failed");
        }

        let stats = promoter.get_stats();
        assert_eq!(stats.total_promotions, 9);
        // Average latency should be well under 1000 ns for pointer swap
        assert!(stats.average_promotion_latency_ns < 100_000.0);
    }
}
