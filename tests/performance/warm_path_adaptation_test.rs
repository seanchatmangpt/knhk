///! Warm Path Adaptation Cycle Tests
//! Validates warm path MAPE-K cycle performance (sub-second for small Δ)

mod tick_measurement;

use std::time::Instant;
use tick_measurement::measure_ticks;

const ADAPTATION_SLO_MS: u128 = 1000; // 1 second for small changes

/// Simulated warm path MAPE-K cycle
struct MapekCycle {
    current_state: Vec<(u64, u64, u64)>,
    patterns: Vec<String>,
}

impl MapekCycle {
    fn new() -> Self {
        Self {
            current_state: Vec::new(),
            patterns: vec!["pattern_a".to_string(), "pattern_b".to_string()],
        }
    }

    /// Monitor: Detect changes in workload
    fn monitor(&self, new_data: &[(u64, u64, u64)]) -> WorkloadChange {
        let delta_size = new_data.len().abs_diff(self.current_state.len());
        let change_rate = delta_size as f64 / self.current_state.len().max(1) as f64;

        WorkloadChange {
            delta_size,
            change_rate,
            requires_adaptation: change_rate > 0.1,
        }
    }

    /// Analyze: Determine if adaptation is needed
    fn analyze(&self, change: &WorkloadChange) -> AdaptationPlan {
        if !change.requires_adaptation {
            return AdaptationPlan::NoChange;
        }

        if change.delta_size < 100 {
            AdaptationPlan::MinorAdjustment
        } else {
            AdaptationPlan::MajorRestructure
        }
    }

    /// Plan: Generate adaptation strategy
    fn plan(&self, adaptation: &AdaptationPlan) -> Vec<AdaptationAction> {
        match adaptation {
            AdaptationPlan::NoChange => vec![],
            AdaptationPlan::MinorAdjustment => {
                vec![AdaptationAction::UpdateIndex, AdaptationAction::RefreshCache]
            }
            AdaptationPlan::MajorRestructure => {
                vec![
                    AdaptationAction::UpdateIndex,
                    AdaptationAction::RefreshCache,
                    AdaptationAction::ReorganizeData,
                ]
            }
        }
    }

    /// Execute: Apply adaptation actions
    fn execute(&mut self, actions: &[AdaptationAction], new_data: Vec<(u64, u64, u64)>) {
        for action in actions {
            match action {
                AdaptationAction::UpdateIndex => {
                    // Simulated index update
                    let _ = new_data.iter().map(|(s, p, _)| (s, p)).collect::<Vec<_>>();
                }
                AdaptationAction::RefreshCache => {
                    // Simulated cache refresh
                    let _ = new_data.first();
                }
                AdaptationAction::ReorganizeData => {
                    // Simulated data reorganization
                    let mut sorted = new_data.clone();
                    sorted.sort_unstable();
                }
            }
        }
        self.current_state = new_data;
    }

    /// Complete MAPE-K cycle
    fn adapt(&mut self, new_data: Vec<(u64, u64, u64)>) -> AdaptationMetrics {
        let start = Instant::now();

        let change = self.monitor(&new_data);
        let adaptation = self.analyze(&change);
        let actions = self.plan(&adaptation);
        self.execute(&actions, new_data);

        let duration = start.elapsed();

        AdaptationMetrics {
            duration_ms: duration.as_millis(),
            delta_size: change.delta_size,
            actions_executed: actions.len(),
        }
    }
}

#[derive(Debug)]
struct WorkloadChange {
    delta_size: usize,
    change_rate: f64,
    requires_adaptation: bool,
}

#[derive(Debug, PartialEq)]
enum AdaptationPlan {
    NoChange,
    MinorAdjustment,
    MajorRestructure,
}

#[derive(Debug)]
enum AdaptationAction {
    UpdateIndex,
    RefreshCache,
    ReorganizeData,
}

#[derive(Debug)]
struct AdaptationMetrics {
    duration_ms: u128,
    delta_size: usize,
    actions_executed: usize,
}

#[test]
fn test_small_delta_adaptation_latency() {
    println!("\n=== Warm Path Small Delta Adaptation Test ===");

    let mut mapek = MapekCycle::new();

    // Initial state: 100 triples
    let initial_data: Vec<(u64, u64, u64)> = (0..100).map(|i| (i, i * 2, i * 3)).collect();
    mapek.current_state = initial_data;

    // Small change: +10 triples
    let new_data: Vec<(u64, u64, u64)> = (0..110).map(|i| (i, i * 2, i * 3)).collect();

    let metrics = mapek.adapt(new_data);

    println!(
        "Small delta (Δ=10): {}ms, {} actions",
        metrics.duration_ms, metrics.actions_executed
    );

    assert!(
        metrics.duration_ms <= ADAPTATION_SLO_MS,
        "Small delta adaptation violated SLO: {}ms > {}ms",
        metrics.duration_ms,
        ADAPTATION_SLO_MS
    );
}

#[test]
fn test_medium_delta_adaptation_latency() {
    println!("\n=== Warm Path Medium Delta Adaptation Test ===");

    let mut mapek = MapekCycle::new();

    // Initial state: 100 triples
    let initial_data: Vec<(u64, u64, u64)> = (0..100).map(|i| (i, i * 2, i * 3)).collect();
    mapek.current_state = initial_data;

    // Medium change: +50 triples
    let new_data: Vec<(u64, u64, u64)> = (0..150).map(|i| (i, i * 2, i * 3)).collect();

    let metrics = mapek.adapt(new_data);

    println!(
        "Medium delta (Δ=50): {}ms, {} actions",
        metrics.duration_ms, metrics.actions_executed
    );

    assert!(
        metrics.duration_ms <= ADAPTATION_SLO_MS,
        "Medium delta adaptation violated SLO: {}ms > {}ms",
        metrics.duration_ms,
        ADAPTATION_SLO_MS
    );
}

#[test]
fn test_no_change_adaptation_latency() {
    println!("\n=== Warm Path No Change Adaptation Test ===");

    let mut mapek = MapekCycle::new();

    // Initial state: 100 triples
    let initial_data: Vec<(u64, u64, u64)> = (0..100).map(|i| (i, i * 2, i * 3)).collect();
    mapek.current_state = initial_data.clone();

    // No change
    let new_data = initial_data;

    let metrics = mapek.adapt(new_data);

    println!(
        "No change (Δ=0): {}ms, {} actions",
        metrics.duration_ms, metrics.actions_executed
    );

    assert!(
        metrics.duration_ms <= 100,
        "No-change adaptation should be < 100ms: {}ms",
        metrics.duration_ms
    );
    assert_eq!(
        metrics.actions_executed, 0,
        "No actions should be executed for no change"
    );
}

#[test]
fn test_mape_k_cycle_components() {
    println!("\n=== MAPE-K Cycle Component Performance Test ===");

    let mapek = MapekCycle::new();
    let initial_data: Vec<(u64, u64, u64)> = (0..100).map(|i| (i, i * 2, i * 3)).collect();
    let new_data: Vec<(u64, u64, u64)> = (0..110).map(|i| (i, i * 2, i * 3)).collect();

    // Measure Monitor phase
    let (change, monitor_measure) = measure_ticks(|| mapek.monitor(&new_data));
    println!("Monitor phase: {} ticks", monitor_measure.elapsed_ticks);

    // Measure Analyze phase
    let (adaptation, analyze_measure) = measure_ticks(|| mapek.analyze(&change));
    println!("Analyze phase: {} ticks", analyze_measure.elapsed_ticks);

    // Measure Plan phase
    let (actions, plan_measure) = measure_ticks(|| mapek.plan(&adaptation));
    println!("Plan phase: {} ticks", plan_measure.elapsed_ticks);

    println!(
        "Total MAPE overhead: {} ticks",
        monitor_measure.elapsed_ticks + analyze_measure.elapsed_ticks + plan_measure.elapsed_ticks
    );

    // Each phase should be very fast
    assert!(
        monitor_measure.elapsed_ticks < 1000,
        "Monitor phase too slow"
    );
    assert!(
        analyze_measure.elapsed_ticks < 1000,
        "Analyze phase too slow"
    );
    assert!(plan_measure.elapsed_ticks < 1000, "Plan phase too slow");
}

#[test]
fn test_repeated_adaptations() {
    println!("\n=== Repeated Adaptations Test ===");

    let mut mapek = MapekCycle::new();
    mapek.current_state = (0..100).map(|i| (i, i * 2, i * 3)).collect();

    let mut total_duration = 0u128;
    let iterations = 10;

    for i in 0..iterations {
        let size = 100 + (i * 5);
        let new_data: Vec<(u64, u64, u64)> = (0..size).map(|j| (j, j * 2, j * 3)).collect();

        let metrics = mapek.adapt(new_data);
        total_duration += metrics.duration_ms;
    }

    let avg_duration = total_duration / iterations as u128;
    println!("Average adaptation time: {}ms", avg_duration);

    assert!(
        avg_duration <= ADAPTATION_SLO_MS,
        "Average adaptation time violated SLO: {}ms > {}ms",
        avg_duration,
        ADAPTATION_SLO_MS
    );
}
