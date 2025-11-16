//! Dark Matter/Energy Detection and 80/20 Observability
//!
//! Detects and measures the "dark matter" of workflow execution:
//! - Unobserved code paths (dark matter)
//! - Unmeasured resource consumption (dark energy)
//! - Focus on 80/20: 20% of paths that account for 80% of execution
//!
//! **Philosophy**: Like dark matter in physics, ~95% of system behavior is invisible.
//! We must instrument the critical 20% that gives us 80% of insights.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Dark matter detector for workflow execution
///
/// Tracks:
/// - Code paths that execute but aren't instrumented
/// - Resources consumed without telemetry
/// - Critical path execution (80/20 analysis)
pub struct DarkMatterDetector {
    /// Observed code paths (light matter)
    observed_paths: Arc<RwLock<HashSet<CodePath>>>,
    /// Unobserved paths detected during execution (dark matter)
    dark_paths: Arc<RwLock<Vec<DarkPath>>>,
    /// Resource consumption tracking
    resource_consumption: Arc<RwLock<HashMap<String, ResourceMetrics>>>,
    /// 80/20 analysis: critical paths
    critical_paths: Arc<RwLock<Vec<CriticalPath>>>,
}

/// Code path identifier
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct CodePath {
    /// Module name
    pub module: String,
    /// Function name
    pub function: String,
    /// Path type (hot/warm/cold)
    pub path_type: PathType,
}

/// Path type classification
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub enum PathType {
    /// Hot path: ≤8 ticks, executed frequently
    Hot,
    /// Warm path: >8 ticks, executed occasionally
    Warm,
    /// Cold path: Error handling, rare cases
    Cold,
}

/// Dark path (unobserved execution)
#[derive(Debug, Clone)]
pub struct DarkPath {
    /// Approximate location
    pub location: String,
    /// Estimated execution count
    pub execution_count: u64,
    /// Estimated resource impact (0.0-1.0)
    pub impact_score: f64,
    /// First detected
    pub first_seen: Instant,
    /// Last detected
    pub last_seen: Instant,
}

/// Resource metrics
#[derive(Debug, Clone)]
pub struct ResourceMetrics {
    /// Total allocations (bytes)
    pub total_allocated: u64,
    /// Peak allocation (bytes)
    pub peak_allocated: u64,
    /// CPU time consumed (microseconds)
    pub cpu_time_us: u64,
    /// Number of observations
    pub observation_count: u64,
}

/// Critical path (80/20 analysis)
#[derive(Debug, Clone)]
pub struct CriticalPath {
    /// Path identifier
    pub path: CodePath,
    /// Percentage of total execution time
    pub time_percentage: f64,
    /// Percentage of total resource consumption
    pub resource_percentage: f64,
    /// Execution frequency
    pub execution_count: u64,
    /// Average duration
    pub avg_duration: Duration,
    /// 80/20 rank (1 = most critical)
    pub rank: usize,
}

/// Dark energy metrics (unmeasured overhead)
#[derive(Debug, Clone)]
pub struct DarkEnergyMetrics {
    /// Total observed execution time
    pub observed_time: Duration,
    /// Total wall-clock time
    pub wall_clock_time: Duration,
    /// Dark energy (unexplained time)
    pub dark_energy: Duration,
    /// Dark energy percentage (should be <5%)
    pub dark_energy_percentage: f64,
    /// Sources of dark energy
    pub sources: Vec<DarkEnergySource>,
}

/// Source of dark energy (unmeasured overhead)
#[derive(Debug, Clone)]
pub struct DarkEnergySource {
    /// Source name
    pub name: String,
    /// Estimated contribution
    pub estimated_time: Duration,
    /// Confidence (0.0-1.0)
    pub confidence: f64,
}

impl DarkMatterDetector {
    /// Create a new dark matter detector
    pub fn new() -> Self {
        Self {
            observed_paths: Arc::new(RwLock::new(HashSet::new())),
            dark_paths: Arc::new(RwLock::new(Vec::new())),
            resource_consumption: Arc::new(RwLock::new(HashMap::new())),
            critical_paths: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register an observed code path (light matter)
    pub fn observe_path(&self, module: &str, function: &str, path_type: PathType) {
        let path = CodePath {
            module: module.to_string(),
            function: function.to_string(),
            path_type,
        };

        let mut paths = self.observed_paths.write().unwrap();
        paths.insert(path);
    }

    /// Detect dark path (execution without instrumentation)
    ///
    /// Called when execution enters a code region without telemetry.
    /// Uses heuristics to estimate impact.
    pub fn detect_dark_path(&self, location: &str, estimated_impact: f64) {
        let mut dark = self.dark_paths.write().unwrap();

        // Check if we've seen this dark path before
        if let Some(existing) = dark.iter_mut().find(|p| p.location == location) {
            existing.execution_count += 1;
            existing.last_seen = Instant::now();
            // Update impact score with exponential moving average
            existing.impact_score = existing.impact_score * 0.9 + estimated_impact * 0.1;
        } else {
            // New dark path discovered
            dark.push(DarkPath {
                location: location.to_string(),
                execution_count: 1,
                impact_score: estimated_impact,
                first_seen: Instant::now(),
                last_seen: Instant::now(),
            });
        }
    }

    /// Track resource consumption for a code path
    pub fn track_resource_consumption(
        &self,
        path_id: &str,
        allocated_bytes: u64,
        cpu_time_us: u64,
    ) {
        let mut resources = self.resource_consumption.write().unwrap();

        resources
            .entry(path_id.to_string())
            .and_modify(|metrics| {
                metrics.total_allocated += allocated_bytes;
                metrics.peak_allocated = metrics.peak_allocated.max(allocated_bytes);
                metrics.cpu_time_us += cpu_time_us;
                metrics.observation_count += 1;
            })
            .or_insert(ResourceMetrics {
                total_allocated: allocated_bytes,
                peak_allocated: allocated_bytes,
                cpu_time_us,
                observation_count: 1,
            });
    }

    /// Perform 80/20 analysis
    ///
    /// Identifies the critical 20% of code paths that account for 80% of:
    /// - Execution time
    /// - Resource consumption
    /// - Execution frequency
    pub fn analyze_80_20(&self) -> Vec<CriticalPath> {
        let resources = self.resource_consumption.read().unwrap();

        // Calculate total resource consumption
        let total_cpu: u64 = resources.values().map(|m| m.cpu_time_us).sum();
        let total_alloc: u64 = resources.values().map(|m| m.total_allocated).sum();
        let total_executions: u64 = resources.values().map(|m| m.observation_count).sum();

        let mut critical: Vec<CriticalPath> = Vec::new();

        for (path_id, metrics) in resources.iter() {
            let time_pct = if total_cpu > 0 {
                (metrics.cpu_time_us as f64 / total_cpu as f64) * 100.0
            } else {
                0.0
            };

            let resource_pct = if total_alloc > 0 {
                (metrics.total_allocated as f64 / total_alloc as f64) * 100.0
            } else {
                0.0
            };

            let avg_duration = if metrics.observation_count > 0 {
                Duration::from_micros(metrics.cpu_time_us / metrics.observation_count)
            } else {
                Duration::ZERO
            };

            critical.push(CriticalPath {
                path: CodePath {
                    module: "workflow".to_string(),
                    function: path_id.clone(),
                    path_type: if time_pct > 10.0 {
                        PathType::Hot
                    } else if time_pct > 1.0 {
                        PathType::Warm
                    } else {
                        PathType::Cold
                    },
                },
                time_percentage: time_pct,
                resource_percentage: resource_pct,
                execution_count: metrics.observation_count,
                avg_duration,
                rank: 0, // Will be set after sorting
            });
        }

        // Sort by combined impact score
        critical.sort_by(|a, b| {
            let score_a = a.time_percentage + a.resource_percentage;
            let score_b = b.time_percentage + b.resource_percentage;
            score_b.partial_cmp(&score_a).unwrap()
        });

        // Assign ranks
        for (rank, path) in critical.iter_mut().enumerate() {
            path.rank = rank + 1;
        }

        // Store critical paths
        let mut critical_paths = self.critical_paths.write().unwrap();
        *critical_paths = critical.clone();

        critical
    }

    /// Calculate dark energy metrics
    ///
    /// Dark energy = unexplained time difference between wall clock and observed telemetry.
    /// Should be <5% for well-instrumented systems.
    pub fn calculate_dark_energy(
        &self,
        observed_time: Duration,
        wall_clock_time: Duration,
    ) -> DarkEnergyMetrics {
        let dark_energy = wall_clock_time.saturating_sub(observed_time);
        let dark_energy_percentage = if wall_clock_time.as_micros() > 0 {
            (dark_energy.as_micros() as f64 / wall_clock_time.as_micros() as f64) * 100.0
        } else {
            0.0
        };

        // Estimate sources of dark energy
        let mut sources = Vec::new();

        // Common sources
        if dark_energy_percentage > 5.0 {
            // Context switching overhead
            sources.push(DarkEnergySource {
                name: "OS context switching".to_string(),
                estimated_time: dark_energy / 3,
                confidence: 0.7,
            });

            // Uninstrumented library calls
            sources.push(DarkEnergySource {
                name: "Uninstrumented dependencies".to_string(),
                estimated_time: dark_energy / 3,
                confidence: 0.6,
            });

            // GC/allocator overhead
            sources.push(DarkEnergySource {
                name: "Allocator overhead".to_string(),
                estimated_time: dark_energy / 3,
                confidence: 0.5,
            });
        }

        DarkEnergyMetrics {
            observed_time,
            wall_clock_time,
            dark_energy,
            dark_energy_percentage,
            sources,
        }
    }

    /// Get dark matter report
    ///
    /// Returns all detected dark paths, sorted by impact.
    pub fn get_dark_matter_report(&self) -> Vec<DarkPath> {
        let mut dark = self.dark_paths.read().unwrap().clone();

        // Sort by impact score
        dark.sort_by(|a, b| b.impact_score.partial_cmp(&a.impact_score).unwrap());

        dark
    }

    /// Get instrumentation coverage percentage
    ///
    /// Returns percentage of code paths that are instrumented.
    /// Target: >95% coverage (dark matter <5%)
    pub fn get_coverage_percentage(&self) -> f64 {
        let observed = self.observed_paths.read().unwrap().len();
        let dark = self.dark_paths.read().unwrap().len();

        let total = observed + dark;
        if total > 0 {
            (observed as f64 / total as f64) * 100.0
        } else {
            100.0
        }
    }
}

impl Default for DarkMatterDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dark_matter_detector_creation() {
        let detector = DarkMatterDetector::new();
        assert_eq!(detector.get_coverage_percentage(), 100.0);
    }

    #[test]
    fn test_observe_path() {
        let detector = DarkMatterDetector::new();
        detector.observe_path("workflow", "execute", PathType::Hot);
        detector.observe_path("workflow", "validate", PathType::Warm);

        let observed = detector.observed_paths.read().unwrap();
        assert_eq!(observed.len(), 2);
    }

    #[test]
    fn test_detect_dark_path() {
        let detector = DarkMatterDetector::new();
        detector.observe_path("workflow", "execute", PathType::Hot);
        detector.detect_dark_path("unobserved::function", 0.8);

        let coverage = detector.get_coverage_percentage();
        assert!(coverage < 100.0);
        assert!(coverage > 0.0);
    }

    #[test]
    fn test_resource_tracking() {
        let detector = DarkMatterDetector::new();
        detector.track_resource_consumption("test_path", 1024, 100);
        detector.track_resource_consumption("test_path", 2048, 200);

        let resources = detector.resource_consumption.read().unwrap();
        let metrics = resources.get("test_path").unwrap();

        assert_eq!(metrics.total_allocated, 3072);
        assert_eq!(metrics.peak_allocated, 2048);
        assert_eq!(metrics.cpu_time_us, 300);
        assert_eq!(metrics.observation_count, 2);
    }

    #[test]
    fn test_80_20_analysis() {
        let detector = DarkMatterDetector::new();

        // Simulate hot path (80% of time)
        detector.track_resource_consumption("hot_path", 1000, 8000);

        // Simulate warm paths (15% of time)
        detector.track_resource_consumption("warm_path_1", 500, 1000);
        detector.track_resource_consumption("warm_path_2", 500, 500);

        // Simulate cold path (5% of time)
        detector.track_resource_consumption("cold_path", 100, 500);

        let critical = detector.analyze_80_20();

        // Hot path should be rank 1
        assert_eq!(critical[0].rank, 1);
        assert!(critical[0].time_percentage > 70.0);
    }

    #[test]
    fn test_dark_energy_calculation() {
        let detector = DarkMatterDetector::new();

        let observed = Duration::from_millis(95);
        let wall_clock = Duration::from_millis(100);

        let metrics = detector.calculate_dark_energy(observed, wall_clock);

        assert_eq!(metrics.dark_energy, Duration::from_millis(5));
        assert_eq!(metrics.dark_energy_percentage, 5.0);
    }

    #[test]
    fn test_dark_energy_low_overhead() {
        let detector = DarkMatterDetector::new();

        let observed = Duration::from_millis(98);
        let wall_clock = Duration::from_millis(100);

        let metrics = detector.calculate_dark_energy(observed, wall_clock);

        assert!(metrics.dark_energy_percentage < 5.0);
        assert!(metrics.sources.is_empty()); // Low overhead = no source attribution
    }

    #[test]
    fn test_dark_path_impact_update() {
        let detector = DarkMatterDetector::new();

        // First detection
        detector.detect_dark_path("mysterious_function", 0.5);

        // Second detection with higher impact
        detector.detect_dark_path("mysterious_function", 0.9);

        let report = detector.get_dark_matter_report();
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].execution_count, 2);
        // Impact score should be updated (exponential moving average)
        assert!(report[0].impact_score > 0.5);
        assert!(report[0].impact_score < 0.9);
    }
}
