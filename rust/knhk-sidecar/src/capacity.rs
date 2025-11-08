// knhk-sidecar: Capacity planning for Fortune 5
// Cache heat tracking and capacity models

use std::collections::HashMap;
use std::time::Instant;

/// Cache heat metrics
#[derive(Debug, Clone)]
pub struct CacheHeatMetrics {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub last_updated: Instant,
}

impl Default for CacheHeatMetrics {
    fn default() -> Self {
        Self {
            cache_hits: 0,
            cache_misses: 0,
            l1_hits: 0,
            l1_misses: 0,
            last_updated: Instant::now(),
        }
    }
}

impl CacheHeatMetrics {
    /// Get cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            return 0.0;
        }
        self.cache_hits as f64 / total as f64
    }

    /// Get L1 hit rate
    pub fn l1_hit_rate(&self) -> f64 {
        let total = self.l1_hits + self.l1_misses;
        if total == 0 {
            return 0.0;
        }
        self.l1_hits as f64 / total as f64
    }

    /// Record cache hit
    pub fn record_hit(&mut self, l1: bool) {
        self.cache_hits += 1;
        if l1 {
            self.l1_hits += 1;
        }
        self.last_updated = Instant::now();
    }

    /// Record cache miss
    pub fn record_miss(&mut self, l1: bool) {
        self.cache_misses += 1;
        if l1 {
            self.l1_misses += 1;
        }
        self.last_updated = Instant::now();
    }
}

/// Capacity planning manager
///
/// Tracks cache heat and provides capacity planning models for Fortune 5.
pub struct CapacityManager {
    heat_metrics: HashMap<String, CacheHeatMetrics>,
    capacity_threshold: f64, // Cache hit rate threshold (default: 0.95)
}

impl CapacityManager {
    /// Create new capacity manager
    pub fn new(capacity_threshold: f64) -> Self {
        Self {
            heat_metrics: HashMap::new(),
            capacity_threshold,
        }
    }

    /// Record cache access for predicate
    pub fn record_access(&mut self, predicate: &str, hit: bool, l1: bool) {
        let metrics = self
            .heat_metrics
            .entry(predicate.to_string())
            .or_insert_with(CacheHeatMetrics::default);

        if hit {
            metrics.record_hit(l1);
        } else {
            metrics.record_miss(l1);
        }
    }

    /// Get cache heat for predicate
    pub fn get_heat(&self, predicate: &str) -> Option<&CacheHeatMetrics> {
        self.heat_metrics.get(predicate)
    }

    /// Check if predicate meets capacity threshold
    pub fn meets_capacity(&self, predicate: &str) -> bool {
        self.heat_metrics
            .get(predicate)
            .map(|m| m.hit_rate() >= self.capacity_threshold)
            .unwrap_or(false)
    }

    /// Get top-N hottest predicates
    pub fn get_hottest_predicates(&self, n: usize) -> Vec<(String, f64)> {
        let mut predicates: Vec<(String, f64)> = self
            .heat_metrics
            .iter()
            .map(|(pred, metrics)| (pred.clone(), metrics.hit_rate()))
            .collect();

        predicates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        predicates.into_iter().take(n).collect()
    }

    /// Predict if data will be in L1 cache
    ///
    /// Uses heat map to predict cache locality for admission decisions.
    pub fn predict_l1_locality(&self, predicate: &str) -> f64 {
        self.heat_metrics
            .get(predicate)
            .map(|m| m.l1_hit_rate())
            .unwrap_or(0.0)
    }

    /// Get capacity report
    pub fn get_capacity_report(&self) -> CapacityReport {
        let total_predicates = self.heat_metrics.len();
        let predicates_meeting_threshold = self
            .heat_metrics
            .values()
            .filter(|m| m.hit_rate() >= self.capacity_threshold)
            .count();

        CapacityReport {
            total_predicates,
            predicates_meeting_threshold,
            capacity_threshold: self.capacity_threshold,
            hottest_predicates: self.get_hottest_predicates(10),
        }
    }
}

/// Capacity report
#[derive(Debug, Clone)]
pub struct CapacityReport {
    pub total_predicates: usize,
    pub predicates_meeting_threshold: usize,
    pub capacity_threshold: f64,
    pub hottest_predicates: Vec<(String, f64)>,
}
