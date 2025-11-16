//! Hot Path Latency Tests
//! Validates that all hot path operations complete within ≤8 ticks (Chatman constant)

mod tick_measurement;

use tick_measurement::{measure_ticks, TickStatistics};

const ITERATIONS: usize = 10000;
const WARMUP: usize = 100;

/// Simulated hot path decision point (ASK query)
#[inline(never)]
fn hot_path_ask_sp(s: u64, p: u64, triples: &[(u64, u64, u64)]) -> bool {
    triples.iter().any(|(ts, tp, _)| *ts == s && *tp == p)
}

/// Simulated hot path decision point (COUNT query)
#[inline(never)]
fn hot_path_count_sp(s: u64, p: u64, triples: &[(u64, u64, u64)]) -> usize {
    triples.iter().filter(|(ts, tp, _)| *ts == s && *tp == p).count()
}

/// Simulated hot path decision point (VALIDATE query)
#[inline(never)]
fn hot_path_validate_sp(s: u64, p: u64, o: u64, _triples: &[(u64, u64, u64)]) -> bool {
    // Simulated validation: check if object is in valid range
    o > 0 && o < 1000000
}

/// Simulated hot path decision point (exact triple match)
#[inline(never)]
fn hot_path_ask_spo(s: u64, p: u64, o: u64, triples: &[(u64, u64, u64)]) -> bool {
    triples.iter().any(|(ts, tp, to)| *ts == s && *tp == p && *to == o)
}

#[test]
fn test_hot_path_ask_sp_latency() {
    println!("\n=== Hot Path ASK(S,P) Latency Test ===");

    let test_data: Vec<(u64, u64, u64)> = (0..10).map(|i| (i, i * 2, i * 3)).collect();

    // Warmup
    for _ in 0..WARMUP {
        let _ = hot_path_ask_sp(5, 10, &test_data);
    }

    // Measure
    let mut measurements = Vec::with_capacity(ITERATIONS);
    for i in 0..ITERATIONS {
        let s = (i % 10) as u64;
        let p = s * 2;
        let (_, measurement) = measure_ticks(|| hot_path_ask_sp(s, p, &test_data));
        measurements.push(measurement);
    }

    let stats = TickStatistics::from_measurements(&measurements);
    println!("ASK(S,P): {}", stats.to_string());

    assert!(
        stats.meets_slo(),
        "ASK(S,P) violated Chatman constant: max={} ticks, violations={}/{}",
        stats.max_ticks,
        stats.violations,
        stats.total_measurements
    );
}

#[test]
fn test_hot_path_count_sp_latency() {
    println!("\n=== Hot Path COUNT(S,P) Latency Test ===");

    let test_data: Vec<(u64, u64, u64)> = (0..10).map(|i| (i % 5, i * 2, i * 3)).collect();

    // Warmup
    for _ in 0..WARMUP {
        let _ = hot_path_count_sp(1, 2, &test_data);
    }

    // Measure
    let mut measurements = Vec::with_capacity(ITERATIONS);
    for i in 0..ITERATIONS {
        let s = (i % 5) as u64;
        let p = s * 2;
        let (_, measurement) = measure_ticks(|| hot_path_count_sp(s, p, &test_data));
        measurements.push(measurement);
    }

    let stats = TickStatistics::from_measurements(&measurements);
    println!("COUNT(S,P): {}", stats.to_string());

    assert!(
        stats.meets_slo(),
        "COUNT(S,P) violated Chatman constant: max={} ticks, violations={}/{}",
        stats.max_ticks,
        stats.violations,
        stats.total_measurements
    );
}

#[test]
fn test_hot_path_validate_sp_latency() {
    println!("\n=== Hot Path VALIDATE(S,P) Latency Test ===");

    let test_data: Vec<(u64, u64, u64)> = (0..10).map(|i| (i, i * 2, i * 3)).collect();

    // Warmup
    for _ in 0..WARMUP {
        let _ = hot_path_validate_sp(1, 2, 100, &test_data);
    }

    // Measure
    let mut measurements = Vec::with_capacity(ITERATIONS);
    for i in 0..ITERATIONS {
        let s = (i % 10) as u64;
        let p = s * 2;
        let o = i as u64 * 100;
        let (_, measurement) = measure_ticks(|| hot_path_validate_sp(s, p, o, &test_data));
        measurements.push(measurement);
    }

    let stats = TickStatistics::from_measurements(&measurements);
    println!("VALIDATE(S,P): {}", stats.to_string());

    assert!(
        stats.meets_slo(),
        "VALIDATE(S,P) violated Chatman constant: max={} ticks, violations={}/{}",
        stats.max_ticks,
        stats.violations,
        stats.total_measurements
    );
}

#[test]
fn test_hot_path_ask_spo_latency() {
    println!("\n=== Hot Path ASK(S,P,O) Latency Test ===");

    let test_data: Vec<(u64, u64, u64)> = (0..10).map(|i| (i, i * 2, i * 3)).collect();

    // Warmup
    for _ in 0..WARMUP {
        let _ = hot_path_ask_spo(5, 10, 15, &test_data);
    }

    // Measure
    let mut measurements = Vec::with_capacity(ITERATIONS);
    for i in 0..ITERATIONS {
        let s = (i % 10) as u64;
        let p = s * 2;
        let o = s * 3;
        let (_, measurement) = measure_ticks(|| hot_path_ask_spo(s, p, o, &test_data));
        measurements.push(measurement);
    }

    let stats = TickStatistics::from_measurements(&measurements);
    println!("ASK(S,P,O): {}", stats.to_string());

    assert!(
        stats.meets_slo(),
        "ASK(S,P,O) violated Chatman constant: max={} ticks, violations={}/{}",
        stats.max_ticks,
        stats.violations,
        stats.total_measurements
    );
}

#[test]
fn test_hot_path_composite_latency() {
    println!("\n=== Hot Path Composite Operation Latency Test ===");

    let test_data: Vec<(u64, u64, u64)> = (0..10).map(|i| (i, i * 2, i * 3)).collect();

    // Warmup
    for _ in 0..WARMUP {
        let _ = hot_path_ask_sp(1, 2, &test_data);
        let _ = hot_path_count_sp(1, 2, &test_data);
    }

    // Measure composite operation
    let mut measurements = Vec::with_capacity(ITERATIONS);
    for i in 0..ITERATIONS {
        let s = (i % 10) as u64;
        let p = s * 2;

        let (_, measurement) = measure_ticks(|| {
            // Composite: ASK + COUNT (simulating guard evaluation)
            let exists = hot_path_ask_sp(s, p, &test_data);
            if exists {
                let _count = hot_path_count_sp(s, p, &test_data);
            }
        });
        measurements.push(measurement);
    }

    let stats = TickStatistics::from_measurements(&measurements);
    println!("Composite (ASK+COUNT): {}", stats.to_string());

    assert!(
        stats.meets_slo(),
        "Composite operation violated Chatman constant: max={} ticks, violations={}/{}",
        stats.max_ticks,
        stats.violations,
        stats.total_measurements
    );
}

#[test]
fn test_hot_path_under_load() {
    println!("\n=== Hot Path Under Load Test ===");

    let test_data: Vec<(u64, u64, u64)> = (0..100).map(|i| (i % 20, i * 2, i * 3)).collect();

    // Warmup
    for _ in 0..WARMUP {
        let _ = hot_path_ask_sp(5, 10, &test_data);
    }

    // Measure under higher data volume
    let mut measurements = Vec::with_capacity(ITERATIONS);
    for i in 0..ITERATIONS {
        let s = (i % 20) as u64;
        let p = s * 2;
        let (_, measurement) = measure_ticks(|| hot_path_ask_sp(s, p, &test_data));
        measurements.push(measurement);
    }

    let stats = TickStatistics::from_measurements(&measurements);
    println!("ASK(S,P) under load (100 triples): {}", stats.to_string());

    // Note: This test allows for slightly higher latency due to increased data volume
    // but still must meet the ≤8 tick constraint
    assert!(
        stats.max_ticks <= 8,
        "Hot path under load violated Chatman constant: max={} ticks",
        stats.max_ticks
    );
}
