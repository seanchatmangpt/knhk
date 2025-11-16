// knhk-sidecar: Phase 5 Capacity Planning with SLO Models for Fortune 500
// Cache heat tracking, SLO-based capacity models, and admission control
// Implements R1/W1/C1 SLO classes with Pareto-based heat analysis

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant, SystemTime};
use tracing::{debug, info, warn, trace};

/// SLO Classes for Fortune 500 systems
///
/// R1 (Red Line): Hot path operations
/// - Target latency: ≤8 ticks (≤2ns)
/// - Required hit rate: 99%+
/// - Cache strategy: L1 pinning, HSM/ultra-fast storage
///
/// W1 (Warm Line): Standard operations
/// - Target latency: ≤500ms
/// - Required hit rate: 95%+
/// - Cache strategy: In-memory cache, fast SSD
///
/// C1 (Cold Line): Background/batch operations
/// - Target latency: ≤24h
/// - Required hit rate: 80%+
/// - Cache strategy: Persistent storage, disk acceptable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SloClass {
    R1,  // Hot path: ≤8 ticks (≤2ns), requires 99%+ cache hit rate
    W1,  // Warm path: ≤500ms, requires 95%+ cache hit rate
    C1,  // Cold path: ≤24h, allows cold data from persistent storage
}

impl SloClass {
    /// Get the target latency for this SLO class
    pub fn target_latency(&self) -> Duration {
        match self {
            SloClass::R1 => Duration::from_nanos(2),
            SloClass::W1 => Duration::from_millis(500),
            SloClass::C1 => Duration::from_secs(86400), // 24 hours
        }
    }

    /// Get required cache hit rate for this SLO class
    pub fn required_hit_rate(&self) -> f64 {
        match self {
            SloClass::R1 => 0.99,  // 99%+
            SloClass::W1 => 0.95,  // 95%+
            SloClass::C1 => 0.80,  // 80%+
        }
    }

    /// Get name for logging/diagnostics
    pub fn name(&self) -> &'static str {
        match self {
            SloClass::R1 => "R1 (Hot)",
            SloClass::W1 => "W1 (Warm)",
            SloClass::C1 => "C1 (Cold)",
        }
    }
}

impl Default for SloClass {
    fn default() -> Self {
        SloClass::W1
    }
}

/// SLO-based capacity prediction with detailed resource requirements
///
/// Predicts L1/L2 cache sizes, expected hit rates, cost estimates, and confidence levels
#[derive(Debug, Clone)]
pub struct SloPrediction {
    pub slo_class: SloClass,
    pub required_l1_size: usize,      // Bytes needed in L1 (hot cache)
    pub required_l2_size: usize,      // Bytes needed in L2 (warm cache)
    pub expected_hit_rate: f64,       // Percentage 0.0-1.0
    pub cost_estimate_daily: f64,     // $$$ estimate
    pub confidence: f64,              // Confidence 0.0-1.0
    pub working_set_items: usize,     // Number of items in working set
    pub l1_items: usize,              // Number of items in L1
    pub l2_items: usize,              // Number of items in L2
}

impl SloPrediction {
    /// Create new SLO prediction
    pub fn new(slo_class: SloClass) -> Self {
        Self {
            slo_class,
            required_l1_size: 0,
            required_l2_size: 0,
            expected_hit_rate: 0.0,
            cost_estimate_daily: 0.0,
            confidence: 0.0,
            working_set_items: 0,
            l1_items: 0,
            l2_items: 0,
        }
    }

    /// Check if this prediction meets SLO requirements
    pub fn meets_slo(&self) -> bool {
        self.expected_hit_rate >= self.slo_class.required_hit_rate()
    }

    /// Get storage tier recommendation
    pub fn storage_tier(&self) -> &'static str {
        match self.slo_class {
            SloClass::R1 => "HSM (Hardware Security Module) / Ultra-fast NVMe",
            SloClass::W1 => "In-memory cache / Fast SSD",
            SloClass::C1 => "Persistent storage / Disk acceptable",
        }
    }
}

/// Heat item for Pareto analysis
///
/// Represents a data item's access patterns for working set identification
#[derive(Debug, Clone)]
pub struct HeatItem {
    pub key: String,
    pub access_count: u64,
    pub last_access: SystemTime,
    pub size_bytes: usize,
    pub hit_count: u64,
    pub miss_count: u64,
}

impl HeatItem {
    /// Get hit rate for this item
    pub fn hit_rate(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            0.0
        } else {
            self.hit_count as f64 / total as f64
        }
    }
}

/// Growth projection for capacity planning
#[derive(Debug, Clone)]
pub struct GrowthProjection {
    pub current_size: usize,
    pub projected_size_30d: usize,
    pub projected_size_90d: usize,
    pub growth_rate: f64,           // Items per day
    pub urgency: CapacityUrgency,
}

/// Capacity urgency levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapacityUrgency {
    Ok,        // Plenty of capacity
    Warning,   // Should plan expansion
    Critical,  // Immediate expansion needed
}

/// Cost model for storage tiers
#[derive(Debug, Clone)]
pub struct CostModel {
    pub l1_cost_per_gb_daily: f64,   // Premium (HSM): ~$2-5/GB/day
    pub l2_cost_per_gb_daily: f64,   // Standard (SSD): ~$0.05-0.20/GB/day
    pub l3_cost_per_gb_daily: f64,   // Economy (Disk): ~$0.01-0.05/GB/day
}

impl Default for CostModel {
    fn default() -> Self {
        Self {
            l1_cost_per_gb_daily: 3.0,    // Premium
            l2_cost_per_gb_daily: 0.10,   // Standard
            l3_cost_per_gb_daily: 0.02,   // Economy
        }
    }
}

impl CostModel {
    /// Estimate daily cost for cache allocation
    pub fn estimate_daily_cost(&self, l1_gb: f64, l2_gb: f64) -> f64 {
        (l1_gb * self.l1_cost_per_gb_daily) + (l2_gb * self.l2_cost_per_gb_daily)
    }
}

/// Scale recommendation with urgency level
#[derive(Debug, Clone)]
pub struct ScaleRecommendation {
    pub current_capacity: usize,
    pub recommended_capacity: usize,
    pub urgency: CapacityUrgency,
    pub growth_projection: GrowthProjection,
    pub reason: String,
}

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
        let old = self
            .hit_rate_history
            .front()
            .map(|(_, r)| *r)
            .unwrap_or(0.0);

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

/// Capacity planning manager with SLO-based predictive models
///
/// Tracks cache heat, provides SLO-based capacity predictions, and implements
/// admission control for Fortune 500 systems. Supports R1/W1/C1 SLO classes
/// with Pareto principle working set analysis.
pub struct CapacityManager {
    heat_metrics: HashMap<String, CacheHeatMetrics>,
    heat_items: HashMap<String, HeatItem>,
    capacity_threshold: f64,
    current_load: SystemLoad,
    slo_targets: SloTargets,
    working_set_tracker: WorkingSetTracker,
    /// Historical growth measurements (timestamp, size)
    growth_history: VecDeque<(SystemTime, usize)>,
    /// Cost model for capacity planning
    cost_model: CostModel,
    /// Track SLO class distributions
    slo_class_distribution: HashMap<SloClass, u64>,
}

impl CapacityManager {
    /// Create new capacity manager
    pub fn new(capacity_threshold: f64) -> Self {
        Self {
            heat_metrics: HashMap::new(),
            heat_items: HashMap::new(),
            capacity_threshold,
            current_load: SystemLoad::default(),
            slo_targets: SloTargets::default(),
            working_set_tracker: WorkingSetTracker::new(),
            growth_history: VecDeque::with_capacity(100),
            cost_model: CostModel::default(),
            slo_class_distribution: HashMap::new(),
        }
    }

    /// Set custom cost model
    pub fn set_cost_model(&mut self, cost_model: CostModel) {
        self.cost_model = cost_model;
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
        let _hot_count = hot_predicates
            .iter()
            .filter(|(_, rate)| *rate > 0.8)
            .count();

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
            recommended_l1_size: l1_size.max(100),        // Minimum 100 entries
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
                (
                    true,
                    format!(
                        "Predicted latency {:?} within SLO {:?}",
                        latency, slo_target
                    ),
                )
            } else {
                // Would violate SLO
                (
                    false,
                    format!(
                        "Predicted latency {:?} exceeds SLO {:?}",
                        latency, slo_target
                    ),
                )
            }
        } else {
            // No prediction available, use conservative approach
            if utilization < 0.8 {
                (
                    true,
                    "System has capacity, no latency prediction available".to_string(),
                )
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
        let avg_hit_rate = self
            .heat_metrics
            .values()
            .map(|m| m.hit_rate())
            .sum::<f64>()
            / self.heat_metrics.len().max(1) as f64;

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
                model.working_set_size, model.recommended_cache_size
            ));
        }

        if tips.is_empty() {
            tips.push("Cache performing optimally".to_string());
        }

        tips
    }

    // ========== PHASE 5: SLO-BASED CAPACITY PLANNING ==========

    /// Analyze heat map using Pareto principle (80/20)
    ///
    /// Identifies which data items generate majority of traffic.
    /// Returns sorted list of hot items with access frequencies.
    pub fn analyze_heat_map(&mut self) -> Vec<HeatItem> {
        // Build heat items from metrics
        for (key, metrics) in &self.heat_metrics {
            let heat_item = HeatItem {
                key: key.clone(),
                access_count: metrics.cache_hits + metrics.cache_misses,
                last_access: SystemTime::now(),
                size_bytes: 1024, // Default size estimate (should be tracked separately)
                hit_count: metrics.cache_hits,
                miss_count: metrics.cache_misses,
            };
            self.heat_items.insert(key.clone(), heat_item);
        }

        // Sort by access frequency (Pareto principle)
        let mut heat_items: Vec<_> = self.heat_items.values().cloned().collect();
        heat_items.sort_by(|a, b| b.access_count.cmp(&a.access_count));

        debug!(
            count = heat_items.len(),
            "Heat map analysis completed with Pareto sorting"
        );

        heat_items
    }

    /// Predict capacity needed for a specific SLO class
    ///
    /// Analyzes heat map and projects cache sizes required to achieve
    /// the target hit rate for the specified SLO class.
    pub fn predict_capacity_needed(&self, slo_class: SloClass) -> SloPrediction {
        let heat_items = {
            let mut items: Vec<_> = self.heat_items.values().cloned().collect();
            items.sort_by(|a, b| b.access_count.cmp(&a.access_count));
            items
        };

        if heat_items.is_empty() {
            debug!("No heat items available for capacity prediction");
            return SloPrediction::new(slo_class);
        }

        let total_accesses: u64 = heat_items.iter().map(|h| h.access_count).sum();
        if total_accesses == 0 {
            return SloPrediction::new(slo_class);
        }

        // Pareto analysis: find items that represent target % of traffic
        let mut prediction = SloPrediction::new(slo_class);

        // Calculate working set (top 20% of items generating 80% of traffic)
        let target_percentage = match slo_class {
            SloClass::R1 => 0.99,  // R1 needs 99% hit rate
            SloClass::W1 => 0.95,  // W1 needs 95% hit rate
            SloClass::C1 => 0.80,  // C1 needs 80% hit rate
        };

        let mut cumulative_accesses = 0u64;
        let mut l1_items = 0;
        let mut l2_items = 0;
        let mut l1_size = 0usize;
        let mut l2_size = 0usize;

        for (idx, item) in heat_items.iter().enumerate() {
            cumulative_accesses += item.access_count;
            let hit_rate = cumulative_accesses as f64 / total_accesses as f64;

            if hit_rate < target_percentage {
                // Still building L1/L2
                if idx < (heat_items.len() / 5).max(1) {
                    // Top 20% in L1
                    l1_items += 1;
                    l1_size += item.size_bytes;
                } else if idx < (heat_items.len() * 2 / 5).max(1) {
                    // Next 20% in L2
                    l2_items += 1;
                    l2_size += item.size_bytes;
                }
            } else {
                break;
            }
        }

        // Ensure minimum sizes
        l1_size = l1_size.max(64 * 1024); // Min 64KB
        l2_size = l2_size.max(1024 * 1024); // Min 1MB

        // Calculate expected hit rate with this allocation
        let expected_hit_rate = {
            let mut accum = 0u64;
            for item in &heat_items[..heat_items.len().min(l1_items + l2_items)] {
                accum += item.access_count;
            }
            if total_accesses > 0 {
                accum as f64 / total_accesses as f64
            } else {
                0.0
            }
        };

        // Estimate cost
        let l1_gb = l1_size as f64 / (1024.0 * 1024.0 * 1024.0);
        let l2_gb = l2_size as f64 / (1024.0 * 1024.0 * 1024.0);
        let cost_estimate = self.cost_model.estimate_daily_cost(l1_gb, l2_gb);

        // Calculate confidence based on data quality
        let total_predicates = self.heat_metrics.len();
        let confidence = match total_predicates {
            n if n > 1000 => 0.95,
            n if n > 100 => 0.85,
            n if n > 10 => 0.7,
            _ => 0.5,
        };

        prediction.required_l1_size = l1_size;
        prediction.required_l2_size = l2_size;
        prediction.expected_hit_rate = expected_hit_rate.min(1.0);
        prediction.cost_estimate_daily = cost_estimate;
        prediction.confidence = confidence;
        prediction.working_set_items = heat_items.len();
        prediction.l1_items = l1_items;
        prediction.l2_items = l2_items;

        info!(
            slo_class = ?slo_class,
            l1_size = l1_size,
            l2_size = l2_size,
            hit_rate = prediction.expected_hit_rate,
            cost = cost_estimate,
            "SLO capacity prediction completed"
        );

        prediction
    }

    /// Determine if request should be admitted based on SLO class
    ///
    /// Checks current hit rate against SLO requirements:
    /// - R1: Requires 99%+ hit rate
    /// - W1: Requires 95%+ hit rate
    /// - C1: Requires 80%+ hit rate
    pub fn should_admit_request(&mut self, slo_class: SloClass) -> bool {
        let current_hit_rate = self.current_load.get_hit_rate();
        let required_rate = slo_class.required_hit_rate();

        let should_admit = current_hit_rate >= required_rate;

        // Track SLO class admission
        *self.slo_class_distribution.entry(slo_class).or_insert(0) += 1;

        if should_admit {
            trace!(
                slo = ?slo_class,
                current = current_hit_rate,
                required = required_rate,
                "Request admitted"
            );
        } else {
            warn!(
                slo = ?slo_class,
                current = current_hit_rate,
                required = required_rate,
                "Request rejected - SLO hit rate not met"
            );
        }

        should_admit
    }

    /// Generate scale recommendation with growth projections
    ///
    /// Analyzes historical growth and projects future capacity needs
    /// for 30 and 90 day horizons.
    pub fn scale_recommendation(&self) -> ScaleRecommendation {
        // Record current working set size
        let current_size = self.working_set_tracker.estimate_working_set();

        // Calculate growth rate from history
        let growth_rate = if self.growth_history.len() >= 2 {
            let most_recent = self.growth_history.back().map(|(_, s)| *s).unwrap_or(0);
            let oldest = self.growth_history.front().map(|(_, s)| *s).unwrap_or(0);

            if let (Some((recent_time, _)), Some((old_time, _))) =
                (self.growth_history.back(), self.growth_history.front())
            {
                if let (Ok(recent_duration), Ok(old_duration)) =
                    (recent_time.elapsed(), old_time.elapsed())
                {
                    let time_diff = recent_duration.as_secs() - old_duration.as_secs();
                    if time_diff > 0 {
                        ((most_recent as i64 - oldest as i64) as f64) / (time_diff as f64 / 86400.0)
                    } else {
                        0.0
                    }
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        // Project future sizes
        let days_30 = (growth_rate * 30.0).max(0.0);
        let days_90 = (growth_rate * 90.0).max(0.0);
        let projected_30d = ((current_size as f64) + days_30) as usize;
        let projected_90d = ((current_size as f64) + days_90) as usize;

        // Determine urgency
        let urgency = if growth_rate > (current_size as f64 * 0.5 / 30.0) {
            CapacityUrgency::Critical
        } else if growth_rate > (current_size as f64 * 0.2 / 30.0) {
            CapacityUrgency::Warning
        } else {
            CapacityUrgency::Ok
        };

        let reason = format!(
            "Growth rate: {:.2} items/day. 30d projection: {}, 90d projection: {}",
            growth_rate, projected_30d, projected_90d
        );

        let growth_projection = GrowthProjection {
            current_size,
            projected_size_30d: projected_30d,
            projected_size_90d: projected_90d,
            growth_rate,
            urgency,
        };

        let recommended = match urgency {
            CapacityUrgency::Critical => (projected_90d as f64 * 1.5) as usize,
            CapacityUrgency::Warning => (projected_90d as f64 * 1.3) as usize,
            CapacityUrgency::Ok => (projected_90d as f64 * 1.1) as usize,
        };

        ScaleRecommendation {
            current_capacity: current_size,
            recommended_capacity: recommended,
            urgency,
            growth_projection,
            reason,
        }
    }

    /// Record growth data point for trend analysis
    pub fn record_growth_point(&mut self) {
        let current_size = self.working_set_tracker.estimate_working_set();
        self.growth_history.push_back((SystemTime::now(), current_size));

        // Keep only last 100 data points (roughly 100 days if recorded daily)
        if self.growth_history.len() > 100 {
            self.growth_history.pop_front();
        }
    }

    /// Get SLO class distribution summary
    pub fn get_slo_distribution(&self) -> HashMap<SloClass, u64> {
        self.slo_class_distribution.clone()
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

    fn get_hit_rate(&self) -> f64 {
        if self.total_requests > 0 {
            self.cache_hits as f64 / self.total_requests as f64
        } else {
            0.0
        }
    }

    fn get_utilization(&self) -> f64 {
        // Simple utilization model based on hit rate and admission rate
        let hit_rate = self.get_hit_rate();

        let admission_rate = if self.admissions_accepted + self.admissions_rejected > 0 {
            self.admissions_accepted as f64
                / (self.admissions_accepted + self.admissions_rejected) as f64
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
        let mut predicates: Vec<_> = self
            .access_counts
            .iter()
            .filter(|(_, count)| **count > 5) // At least 5 accesses
            .map(|(pred, count)| (pred.clone(), *count))
            .collect();

        predicates.sort_by(|a, b| b.1.cmp(&a.1));
        predicates
            .into_iter()
            .take(100) // Top 100 working set
            .map(|(pred, _)| pred)
            .collect()
    }
}
