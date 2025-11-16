// knhk-sidecar: Capacity planning for Fortune 5
// Cache heat tracking, capacity models, and SLO-based admission control

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Cache heat metrics with time-series tracking
#[derive(Debug, Clone)]
pub struct CacheHeatMetrics {
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub l1_hits: u64,
    pub l1_misses: u64,
    pub last_updated: Instant,
    /// Historical hit rates for trend analysis
    hit_rate_history: VecDeque<(Instant, f64)>,
    /// Historical L1 hit rates
    l1_rate_history: VecDeque<(Instant, f64)>,
}

impl Default for CacheHeatMetrics {
    fn default() -> Self {
        Self {
            cache_hits: 0,
            cache_misses: 0,
            l1_hits: 0,
            l1_misses: 0,
            last_updated: Instant::now(),
            hit_rate_history: VecDeque::with_capacity(100),
            l1_rate_history: VecDeque::with_capacity(100),
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
        } else {
            self.l1_misses += 1;
        }
        self.update_history();
    }

    /// Record cache miss
    pub fn record_miss(&mut self, l1: bool) {
        self.cache_misses += 1;
        if l1 {
            self.l1_misses += 1;
        }
        self.update_history();
    }

    /// Update historical metrics
    fn update_history(&mut self) {
        let now = Instant::now();
        self.last_updated = now;

        // Update history every 100 operations
        if (self.cache_hits + self.cache_misses) % 100 == 0 {
            self.hit_rate_history.push_back((now, self.hit_rate()));
            self.l1_rate_history.push_back((now, self.l1_hit_rate()));

            // Keep only last 100 samples
            if self.hit_rate_history.len() > 100 {
                self.hit_rate_history.pop_front();
            }
            if self.l1_rate_history.len() > 100 {
                self.l1_rate_history.pop_front();
            }
        }
    }

    /// Get trend direction (positive = improving, negative = degrading)
    pub fn get_trend(&self) -> f64 {
        if self.hit_rate_history.len() < 2 {
            return 0.0;
        }

        let recent = self.hit_rate_history.back().map(|(_, r)| *r).unwrap_or(0.0);
        let old = self.hit_rate_history.front().map(|(_, r)| *r).unwrap_or(0.0);

        recent - old
    }
}

/// Capacity prediction model
#[derive(Debug, Clone)]
pub struct CapacityModel {
    /// Working set size estimate (in entries)
    pub working_set_size: usize,
    /// Predicted cache size requirement
    pub recommended_cache_size: usize,
    /// Predicted L1 cache size
    pub recommended_l1_size: usize,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
}

/// SLO-based admission decision
#[derive(Debug, Clone)]
pub struct AdmissionDecision {
    /// Whether request should be admitted
    pub admit: bool,
    /// Reason for decision
    pub reason: String,
    /// Predicted latency if admitted
    pub predicted_latency: Option<Duration>,
    /// Current capacity utilization (0.0 - 1.0)
    pub capacity_utilization: f64,
}

/// Capacity planning manager with predictive models
///
/// Tracks cache heat, provides capacity planning models, and implements
/// SLO-based admission control for Fortune 5.
pub struct CapacityManager {
    heat_metrics: HashMap<String, CacheHeatMetrics>,
    capacity_threshold: f64, // Cache hit rate threshold (default: 0.95)
    /// Current system load metrics
    current_load: SystemLoad,
    /// SLO targets for admission control
    slo_targets: SloTargets,
    /// Working set analysis
    working_set_tracker: WorkingSetTracker,
}

impl CapacityManager {
    /// Create new capacity manager
    pub fn new(capacity_threshold: f64) -> Self {
        Self {
            heat_metrics: HashMap::new(),
            capacity_threshold,
            current_load: SystemLoad::default(),
            slo_targets: SloTargets::default(),
            working_set_tracker: WorkingSetTracker::new(),
        }
    }

    /// Record cache access for predicate
    pub fn record_access(&mut self, predicate: &str, hit: bool, l1: bool) {
        let metrics = self.heat_metrics.entry(predicate.to_string()).or_default();

        if hit {
            metrics.record_hit(l1);
            self.working_set_tracker.record_hit(predicate);
        } else {
            metrics.record_miss(l1);
            self.working_set_tracker.record_miss(predicate);
        }

        // Update system load
        self.current_load.record_access(hit);
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
    /// Uses heat map and access patterns to predict cache locality
    pub fn predict_l1_locality(&self, predicate: &str) -> f64 {
        if let Some(metrics) = self.heat_metrics.get(predicate) {
            // Consider both current L1 hit rate and trend
            let current_rate = metrics.l1_hit_rate();
            let trend = metrics.get_trend();

            // Weighted prediction
            (current_rate * 0.8) + (trend * 0.2)
        } else {
            // No history - assume cold start
            0.0
        }
    }

    /// Build capacity prediction model
    pub fn build_capacity_model(&self) -> CapacityModel {
        let working_set = self.working_set_tracker.estimate_working_set();
        let hot_predicates = self.get_hottest_predicates(100);

        // Estimate cache sizes based on working set and access patterns
        let total_predicates = self.heat_metrics.len();
        let hot_count = hot_predicates.iter().filter(|(_, rate)| *rate > 0.8).count();

        // L1 should hold the hottest data (10% of working set)
        let l1_size = (working_set as f64 * 0.1).ceil() as usize;

        // Main cache should hold full working set + 20% buffer
        let cache_size = (working_set as f64 * 1.2).ceil() as usize;

        // Calculate confidence based on data quality
        let confidence = if total_predicates > 100 {
            0.95 // High confidence with enough data
        } else if total_predicates > 10 {
            0.8 // Moderate confidence
        } else {
            0.5 // Low confidence with limited data
        };

        CapacityModel {
            working_set_size: working_set,
            recommended_cache_size: cache_size.max(1000), // Minimum 1000 entries
            recommended_l1_size: l1_size.max(100), // Minimum 100 entries
            confidence,
        }
    }

    /// SLO-based admission control
    ///
    /// Decides whether to admit a request based on current capacity and SLO targets
    pub fn admission_control(
        &mut self,
        predicate: &str,
        runtime_class: RuntimeClass,
    ) -> AdmissionDecision {
        // Get current system state
        let utilization = self.current_load.get_utilization();
        let predicted_latency = self.predict_latency(predicate, runtime_class);

        // Check against SLO targets
        let slo_target = self.slo_targets.get_target(runtime_class);

        // Decision logic
        let (admit, reason) = if utilization > 0.95 {
            // System overloaded
            (false, "System at capacity (>95% utilization)".to_string())
        } else if let Some(latency) = predicted_latency {
            if latency <= slo_target {
                // Within SLO
                (true, format!("Predicted latency {:?} within SLO {:?}", latency, slo_target))
            } else {
                // Would violate SLO
                (false, format!("Predicted latency {:?} exceeds SLO {:?}", latency, slo_target))
            }
        } else {
            // No prediction available, use conservative approach
            if utilization < 0.8 {
                (true, "System has capacity, no latency prediction available".to_string())
            } else {
                (false, "High utilization, cannot guarantee SLO".to_string())
            }
        };

        // Record admission decision
        self.current_load.record_admission(admit);

        AdmissionDecision {
            admit,
            reason,
            predicted_latency,
            capacity_utilization: utilization,
        }
    }

    /// Predict latency for a request
    fn predict_latency(&self, predicate: &str, runtime_class: RuntimeClass) -> Option<Duration> {
        // Get cache heat for prediction
        let heat = self.heat_metrics.get(predicate)?;

        // Base latency by runtime class
        let base_latency = match runtime_class {
            RuntimeClass::R1 => Duration::from_nanos(1), // 1ns for R1
            RuntimeClass::W1 => Duration::from_micros(100), // 100µs for W1
            RuntimeClass::C1 => Duration::from_millis(100), // 100ms for C1
        };

        // Adjust based on cache heat
        let cache_factor = if heat.hit_rate() > 0.95 {
            0.5 // Fast path - cache hit
        } else if heat.hit_rate() > 0.8 {
            1.0 // Normal path
        } else {
            2.0 // Slow path - likely cache miss
        };

        // Further adjust for L1 locality
        let l1_factor = if heat.l1_hit_rate() > 0.9 {
            0.8 // L1 hit - very fast
        } else {
            1.0
        };

        // Calculate final prediction
        let predicted = base_latency.mul_f64(cache_factor * l1_factor);

        Some(predicted)
    }

    /// Get capacity report
    pub fn get_capacity_report(&self) -> CapacityReport {
        let total_predicates = self.heat_metrics.len();
        let predicates_meeting_threshold = self
            .heat_metrics
            .values()
            .filter(|m| m.hit_rate() >= self.capacity_threshold)
            .count();

        let model = self.build_capacity_model();

        CapacityReport {
            total_predicates,
            predicates_meeting_threshold,
            capacity_threshold: self.capacity_threshold,
            hottest_predicates: self.get_hottest_predicates(10),
            capacity_model: Some(model),
            system_load: self.current_load.clone(),
        }
    }

    /// Identify working set (predicates accessed frequently)
    pub fn identify_working_set(&self) -> Vec<String> {
        self.working_set_tracker.get_working_set()
    }

    /// Get cache size recommendations
    pub fn get_cache_recommendations(&self) -> CacheRecommendations {
        let model = self.build_capacity_model();
        let working_set = self.identify_working_set();

        // Analyze access patterns
        let mut access_patterns = HashMap::new();
        for (predicate, metrics) in &self.heat_metrics {
            let pattern = if metrics.hit_rate() > 0.95 {
                "hot"
            } else if metrics.hit_rate() > 0.7 {
                "warm"
            } else {
                "cold"
            };
            access_patterns.insert(predicate.clone(), pattern.to_string());
        }

        CacheRecommendations {
            l1_size: model.recommended_l1_size,
            l2_size: model.recommended_cache_size,
            eviction_policy: self.recommend_eviction_policy(),
            working_set,
            access_patterns,
            optimization_tips: self.generate_optimization_tips(),
        }
    }

    /// Recommend eviction policy based on access patterns
    fn recommend_eviction_policy(&self) -> String {
        // Analyze access patterns to recommend policy
        let mut temporal_locality = 0.0;
        let mut frequency_score = 0.0;

        for metrics in self.heat_metrics.values() {
            temporal_locality += metrics.l1_hit_rate();
            frequency_score += metrics.hit_rate();
        }

        let count = self.heat_metrics.len().max(1) as f64;
        temporal_locality /= count;
        frequency_score /= count;

        if temporal_locality > 0.8 {
            "LRU - High temporal locality detected".to_string()
        } else if frequency_score > 0.8 {
            "LFU - High frequency patterns detected".to_string()
        } else {
            "ARC - Mixed access patterns, adaptive replacement recommended".to_string()
        }
    }

    /// Generate optimization tips based on analysis
    fn generate_optimization_tips(&self) -> Vec<String> {
        let mut tips = Vec::new();

        // Analyze cache performance
        let avg_hit_rate = self.heat_metrics.values()
            .map(|m| m.hit_rate())
            .sum::<f64>() / self.heat_metrics.len().max(1) as f64;

        if avg_hit_rate < 0.8 {
            tips.push("Cache hit rate below 80% - consider increasing cache size".to_string());
        }

        // Check for hot spots
        let hot_predicates = self.get_hottest_predicates(5);
        if !hot_predicates.is_empty() && hot_predicates[0].1 > 0.95 {
            tips.push(format!(
                "Hot predicate '{}' with {}% hit rate - consider L1 pinning",
                hot_predicates[0].0,
                (hot_predicates[0].1 * 100.0) as u32
            ));
        }

        // Working set analysis
        let model = self.build_capacity_model();
        if model.working_set_size > model.recommended_cache_size {
            tips.push(format!(
                "Working set ({}) exceeds cache size ({}) - increase cache allocation",
                model.working_set_size,
                model.recommended_cache_size
            ));
        }

        if tips.is_empty() {
            tips.push("Cache performing optimally".to_string());
        }

        tips
    }
}

/// Capacity report
#[derive(Debug, Clone)]
pub struct CapacityReport {
    pub total_predicates: usize,
    pub predicates_meeting_threshold: usize,
    pub capacity_threshold: f64,
    pub hottest_predicates: Vec<(String, f64)>,
    pub capacity_model: Option<CapacityModel>,
    pub system_load: SystemLoad,
}

/// Cache recommendations
#[derive(Debug, Clone)]
pub struct CacheRecommendations {
    pub l1_size: usize,
    pub l2_size: usize,
    pub eviction_policy: String,
    pub working_set: Vec<String>,
    pub access_patterns: HashMap<String, String>,
    pub optimization_tips: Vec<String>,
}

/// System load tracking
#[derive(Debug, Clone, Default)]
struct SystemLoad {
    total_requests: u64,
    cache_hits: u64,
    admissions_accepted: u64,
    admissions_rejected: u64,
    last_reset: Option<Instant>,
}

impl SystemLoad {
    fn record_access(&mut self, hit: bool) {
        self.total_requests += 1;
        if hit {
            self.cache_hits += 1;
        }
    }

    fn record_admission(&mut self, admitted: bool) {
        if admitted {
            self.admissions_accepted += 1;
        } else {
            self.admissions_rejected += 1;
        }
    }

    fn get_utilization(&self) -> f64 {
        // Simple utilization model based on hit rate and admission rate
        let hit_rate = if self.total_requests > 0 {
            self.cache_hits as f64 / self.total_requests as f64
        } else {
            0.0
        };

        let admission_rate = if self.admissions_accepted + self.admissions_rejected > 0 {
            self.admissions_accepted as f64 /
                (self.admissions_accepted + self.admissions_rejected) as f64
        } else {
            1.0
        };

        // Lower hit rate or lower admission rate indicates higher utilization
        1.0 - (hit_rate * admission_rate * 0.5)
    }
}

/// Runtime class for SLO targets
#[derive(Debug, Clone, Copy)]
pub enum RuntimeClass {
    R1, // Read, 2ns SLO
    W1, // Write, 1ms SLO
    C1, // Compute, 500ms SLO
}

/// SLO targets for admission control
#[derive(Debug, Clone)]
struct SloTargets {
    r1_target: Duration,
    w1_target: Duration,
    c1_target: Duration,
}

impl Default for SloTargets {
    fn default() -> Self {
        Self {
            r1_target: Duration::from_nanos(2),
            w1_target: Duration::from_millis(1),
            c1_target: Duration::from_millis(500),
        }
    }
}

impl SloTargets {
    fn get_target(&self, runtime_class: RuntimeClass) -> Duration {
        match runtime_class {
            RuntimeClass::R1 => self.r1_target,
            RuntimeClass::W1 => self.w1_target,
            RuntimeClass::C1 => self.c1_target,
        }
    }
}

/// Working set tracker
#[derive(Debug)]
struct WorkingSetTracker {
    access_counts: HashMap<String, u64>,
    recent_accesses: VecDeque<String>,
    window_size: usize,
}

impl WorkingSetTracker {
    fn new() -> Self {
        Self {
            access_counts: HashMap::new(),
            recent_accesses: VecDeque::with_capacity(1000),
            window_size: 1000,
        }
    }

    fn record_hit(&mut self, predicate: &str) {
        *self.access_counts.entry(predicate.to_string()).or_insert(0) += 1;

        self.recent_accesses.push_back(predicate.to_string());
        if self.recent_accesses.len() > self.window_size {
            self.recent_accesses.pop_front();
        }
    }

    fn record_miss(&mut self, predicate: &str) {
        // Still track misses for working set analysis
        self.recent_accesses.push_back(predicate.to_string());
        if self.recent_accesses.len() > self.window_size {
            self.recent_accesses.pop_front();
        }
    }

    fn estimate_working_set(&self) -> usize {
        // Count unique predicates in recent window
        let unique: std::collections::HashSet<_> = self.recent_accesses.iter().collect();
        unique.len()
    }

    fn get_working_set(&self) -> Vec<String> {
        // Return predicates with significant access count
        let mut predicates: Vec<_> = self.access_counts
            .iter()
            .filter(|(_, count)| **count > 5) // At least 5 accesses
            .map(|(pred, count)| (pred.clone(), *count))
            .collect();

        predicates.sort_by(|a, b| b.1.cmp(&a.1));
        predicates.into_iter()
            .take(100) // Top 100 working set
            .map(|(pred, _)| pred)
            .collect()
    }
}