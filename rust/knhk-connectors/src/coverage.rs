// rust/knhk-connectors/src/coverage.rs
// Dark Matter 80/20 Coverage Tracker
//
// Implements real-time coverage tracking for predicate/hook access patterns.
// Identifies the critical 20% of operations that handle 80% of queries.
//
// **Architecture**:
// - Lock-free atomic counters for hot path tracking
// - Efficient predicate frequency analysis
// - 80/20 Pareto distribution calculation
// - Hot core identification (S ⊂ O)

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

/// Maximum number of unique predicates to track (power of 2 for efficiency)
const MAX_PREDICATES: usize = 256;

/// Predicate access counter (lock-free for hot path performance)
pub struct PredicateCounter {
    /// Predicate IRI hash -> access count
    counts: [AtomicU64; MAX_PREDICATES],
    /// Total accesses
    total: AtomicU64,
}

impl PredicateCounter {
    /// Create new predicate counter
    pub const fn new() -> Self {
        const ZERO: AtomicU64 = AtomicU64::new(0);
        Self {
            counts: [ZERO; MAX_PREDICATES],
            total: AtomicU64::new(0),
        }
    }

    /// Record predicate access (hot path - zero-allocation)
    ///
    /// # Performance
    /// - Lock-free atomic increment
    /// - No allocations
    /// - ≤8 ticks overhead
    #[inline(always)]
    pub fn record(&self, predicate_hash: u64) {
        let index = (predicate_hash % MAX_PREDICATES as u64) as usize;
        self.counts[index].fetch_add(1, Ordering::Relaxed);
        self.total.fetch_add(1, Ordering::Relaxed);
    }

    /// Get total accesses
    pub fn total(&self) -> u64 {
        self.total.load(Ordering::Relaxed)
    }

    /// Get access count for predicate
    pub fn get(&self, predicate_hash: u64) -> u64 {
        let index = (predicate_hash % MAX_PREDICATES as u64) as usize;
        self.counts[index].load(Ordering::Relaxed)
    }

    /// Calculate 80/20 distribution
    ///
    /// Returns (hot_predicates, hot_percentage, coverage_percentage)
    /// where hot_predicates handle coverage_percentage of total accesses
    pub fn pareto_distribution(&self) -> (Vec<(usize, u64)>, f64, f64) {
        let total = self.total();
        if total == 0 {
            return (Vec::new(), 0.0, 0.0);
        }

        // Collect non-zero counts
        let mut predicate_counts: Vec<(usize, u64)> = self
            .counts
            .iter()
            .enumerate()
            .filter_map(|(idx, count)| {
                let c = count.load(Ordering::Relaxed);
                if c > 0 {
                    Some((idx, c))
                } else {
                    None
                }
            })
            .collect();

        // Sort by count descending
        predicate_counts.sort_by(|a, b| b.1.cmp(&a.1));

        if predicate_counts.is_empty() {
            return (Vec::new(), 0.0, 0.0);
        }

        // Calculate 80% threshold
        let threshold = (total as f64 * 0.8) as u64;
        let mut cumulative = 0u64;
        let mut hot_count = 0usize;

        for (_idx, count) in &predicate_counts {
            cumulative += count;
            hot_count += 1;
            if cumulative >= threshold {
                break;
            }
        }

        let hot_percentage = (hot_count as f64 / predicate_counts.len() as f64) * 100.0;
        let coverage_percentage = (cumulative as f64 / total as f64) * 100.0;

        (predicate_counts, hot_percentage, coverage_percentage)
    }

    /// Reset all counters
    pub fn reset(&self) {
        for count in &self.counts {
            count.store(0, Ordering::Relaxed);
        }
        self.total.store(0, Ordering::Relaxed);
    }
}

impl Default for PredicateCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Dark Matter coverage metrics
#[derive(Debug, Clone)]
pub struct CoverageMetrics {
    /// Total unique predicates tracked
    pub total_predicates: usize,
    /// Hot core size (predicates handling 80% of traffic)
    pub hot_core_size: usize,
    /// Percentage of predicates in hot core
    pub hot_percentage: f64,
    /// Coverage percentage (what % of traffic hot core handles)
    pub coverage_percentage: f64,
    /// Total accesses recorded
    pub total_accesses: u64,
    /// Top predicates by access count
    pub top_predicates: Vec<(usize, u64, f64)>,
}

impl CoverageMetrics {
    /// Calculate coverage metrics from counter
    pub fn from_counter(counter: &PredicateCounter) -> Self {
        let (predicate_counts, hot_percentage, coverage_percentage) = counter.pareto_distribution();

        let total = counter.total();
        let total_predicates = predicate_counts.len();

        // Calculate hot core size (predicates handling 80% of traffic)
        let threshold = (total as f64 * 0.8) as u64;
        let mut cumulative = 0u64;
        let mut hot_core_size = 0usize;

        for (_idx, count) in &predicate_counts {
            cumulative += count;
            hot_core_size += 1;
            if cumulative >= threshold {
                break;
            }
        }

        // Get top 10 predicates with percentages
        let top_predicates: Vec<(usize, u64, f64)> = predicate_counts
            .iter()
            .take(10)
            .map(|(idx, count)| {
                let percentage = (*count as f64 / total as f64) * 100.0;
                (*idx, *count, percentage)
            })
            .collect();

        Self {
            total_predicates,
            hot_core_size,
            hot_percentage,
            coverage_percentage,
            total_accesses: total,
            top_predicates,
        }
    }

    /// Check if metrics meet 80/20 threshold
    ///
    /// Returns true if hot_percentage ≤ 20% and coverage_percentage ≥ 80%
    pub fn meets_pareto_threshold(&self) -> bool {
        self.hot_percentage <= 20.0 && self.coverage_percentage >= 80.0
    }

    /// Calculate sparsity ratio (μ → S)
    ///
    /// Returns ratio of hot core to total predicates
    pub fn sparsity_ratio(&self) -> f64 {
        if self.total_predicates == 0 {
            return 0.0;
        }
        self.hot_core_size as f64 / self.total_predicates as f64
    }
}

/// Dark Matter coverage tracker (global singleton for efficiency)
pub struct DarkMatterTracker {
    counter: PredicateCounter,
    /// Predicate hash -> IRI mapping (for debugging)
    predicate_map: BTreeMap<u64, String>,
}

impl DarkMatterTracker {
    /// Create new tracker
    pub fn new() -> Self {
        Self {
            counter: PredicateCounter::new(),
            predicate_map: BTreeMap::new(),
        }
    }

    /// Record predicate access (hot path)
    #[inline(always)]
    pub fn record(&self, predicate_hash: u64) {
        self.counter.record(predicate_hash);
    }

    /// Register predicate IRI for debugging
    pub fn register_predicate(&mut self, predicate_hash: u64, iri: String) {
        self.predicate_map.insert(predicate_hash, iri);
    }

    /// Get coverage metrics
    pub fn metrics(&self) -> CoverageMetrics {
        CoverageMetrics::from_counter(&self.counter)
    }

    /// Reset tracker
    pub fn reset(&mut self) {
        self.counter.reset();
        self.predicate_map.clear();
    }

    /// Get predicate IRI by hash
    pub fn get_predicate_iri(&self, hash: u64) -> Option<&String> {
        self.predicate_map.get(&hash)
    }
}

impl Default for DarkMatterTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Hash predicate IRI for tracking
///
/// Uses simple FNV-1a hash for performance
#[inline(always)]
pub fn hash_predicate_iri(iri: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64; // FNV offset basis
    for byte in iri.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3); // FNV prime
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn test_predicate_counter_basic() {
        let counter = PredicateCounter::new();

        // Record some accesses
        counter.record(1);
        counter.record(1);
        counter.record(2);
        counter.record(1);

        assert_eq!(counter.total(), 4);
        assert_eq!(counter.get(1), 3);
        assert_eq!(counter.get(2), 1);
    }

    #[test]
    fn test_pareto_distribution() {
        let counter = PredicateCounter::new();

        // Simulate 80/20 distribution
        // 20% of predicates (2 out of 10) handle 80% of traffic
        for _ in 0..800 {
            counter.record(1); // Hot predicate 1
        }
        for _ in 0..800 {
            counter.record(2); // Hot predicate 2
        }
        // Remaining 20% of traffic spread across 8 predicates
        for i in 3..11 {
            for _ in 0..50 {
                counter.record(i);
            }
        }

        let (counts, hot_pct, coverage_pct) = counter.pareto_distribution();

        assert!(counts.len() > 0);
        assert!(coverage_pct >= 80.0);
        assert!(hot_pct <= 25.0); // Should be close to 20%
    }

    #[test]
    fn test_coverage_metrics() {
        let counter = PredicateCounter::new();

        // Create perfect 80/20 distribution
        for _ in 0..80 {
            counter.record(1);
        }
        for _ in 0..20 {
            counter.record(2);
        }

        let metrics = CoverageMetrics::from_counter(&counter);

        assert_eq!(metrics.total_accesses, 100);
        assert_eq!(metrics.total_predicates, 2);
        assert_eq!(metrics.hot_core_size, 1); // Only predicate 1 needed for 80%
        assert!(metrics.coverage_percentage >= 80.0);
    }

    #[test]
    fn test_hash_predicate_iri() {
        let iri1 = "http://example.org/predicate1";
        let iri2 = "http://example.org/predicate2";

        let hash1 = hash_predicate_iri(iri1);
        let hash2 = hash_predicate_iri(iri2);

        // Hashes should be different
        assert_ne!(hash1, hash2);

        // Same IRI should produce same hash
        assert_eq!(hash1, hash_predicate_iri(iri1));
    }

    #[test]
    fn test_dark_matter_tracker() {
        let mut tracker = DarkMatterTracker::new();

        let pred1 = hash_predicate_iri("http://example.org/name");
        let pred2 = hash_predicate_iri("http://example.org/email");

        tracker.register_predicate(pred1, "http://example.org/name".to_string());
        tracker.register_predicate(pred2, "http://example.org/email".to_string());

        // Simulate traffic
        for _ in 0..100 {
            tracker.record(pred1);
        }
        for _ in 0..10 {
            tracker.record(pred2);
        }

        let metrics = tracker.metrics();

        assert_eq!(metrics.total_accesses, 110);
        assert!(metrics.hot_core_size >= 1);
        assert!(tracker.get_predicate_iri(pred1).is_some());
    }

    #[test]
    fn test_meets_pareto_threshold() {
        let counter = PredicateCounter::new();

        // Perfect 80/20: 1 predicate handles 80% of 100 accesses
        for _ in 0..80 {
            counter.record(1);
        }
        for i in 2..6 {
            for _ in 0..5 {
                counter.record(i);
            }
        }

        let metrics = CoverageMetrics::from_counter(&counter);

        // Should meet threshold: 20% of predicates (1/5) handle 80% of traffic
        assert!(metrics.hot_percentage <= 20.0);
        assert!(metrics.coverage_percentage >= 80.0);
        assert!(metrics.meets_pareto_threshold());
    }

    #[test]
    fn test_sparsity_ratio() {
        let counter = PredicateCounter::new();

        // 2 predicates out of 10 handle 80% of traffic
        for _ in 0..400 {
            counter.record(1);
        }
        for _ in 0..400 {
            counter.record(2);
        }
        for i in 3..11 {
            for _ in 0..25 {
                counter.record(i);
            }
        }

        let metrics = CoverageMetrics::from_counter(&counter);

        // Sparsity ratio should be ~0.2 (2/10)
        assert!(metrics.sparsity_ratio() > 0.15);
        assert!(metrics.sparsity_ratio() < 0.30);
    }
}
