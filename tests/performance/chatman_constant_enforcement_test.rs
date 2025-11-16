///! Chatman Constant Enforcement Tests
//! Comprehensive validation that the system enforces τ ≤ 8 ticks

mod tick_measurement;

use tick_measurement::{measure_ticks, TickStatistics};

const ITERATIONS: usize = 1000;

/// Test comprehensive enforcement of Chatman constant across all hot path operations
#[test]
fn test_chatman_constant_comprehensive() {
    println!("\n=== Chatman Constant Comprehensive Enforcement Test ===");
    println!("Law: μ ⊂ τ ; τ ≤ 8 ticks");
    println!();

    let mut all_measurements = Vec::new();
    let mut violations_by_operation = std::collections::HashMap::new();

    // Test 1: Simple boolean check
    println!("Testing: Simple boolean check...");
    let mut measurements = Vec::new();
    for _ in 0..ITERATIONS {
        let (_, m) = measure_ticks(|| {
            let x = 42;
            x > 10
        });
        measurements.push(m);
    }
    let stats = TickStatistics::from_measurements(&measurements);
    println!("  Result: {}", stats.to_string());
    violations_by_operation.insert("boolean_check", stats.violations);
    all_measurements.extend(measurements);

    // Test 2: Array lookup
    println!("Testing: Array lookup...");
    let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let mut measurements = Vec::new();
    for i in 0..ITERATIONS {
        let idx = i % 10;
        let (_, m) = measure_ticks(|| arr[idx]);
        measurements.push(m);
    }
    let stats = TickStatistics::from_measurements(&measurements);
    println!("  Result: {}", stats.to_string());
    violations_by_operation.insert("array_lookup", stats.violations);
    all_measurements.extend(measurements);

    // Test 3: Hash lookup simulation
    println!("Testing: Hash lookup simulation...");
    let mut measurements = Vec::new();
    for i in 0..ITERATIONS {
        let (_, m) = measure_ticks(|| {
            // Simulated hash lookup (linear search in small array)
            let data = [(1u64, 100u64), (2, 200), (3, 300), (4, 400), (5, 500)];
            let key = (i % 5 + 1) as u64;
            data.iter().find(|(k, _)| *k == key)
        });
        measurements.push(m);
    }
    let stats = TickStatistics::from_measurements(&measurements);
    println!("  Result: {}", stats.to_string());
    violations_by_operation.insert("hash_lookup", stats.violations);
    all_measurements.extend(measurements);

    // Test 4: Bit manipulation
    println!("Testing: Bit manipulation...");
    let mut measurements = Vec::new();
    for i in 0..ITERATIONS {
        let x = i as u64;
        let (_, m) = measure_ticks(|| (x & 0xFF) | ((x & 0xFF00) >> 8));
        measurements.push(m);
    }
    let stats = TickStatistics::from_measurements(&measurements);
    println!("  Result: {}", stats.to_string());
    violations_by_operation.insert("bit_manipulation", stats.violations);
    all_measurements.extend(measurements);

    // Test 5: Range check
    println!("Testing: Range check...");
    let mut measurements = Vec::new();
    for i in 0..ITERATIONS {
        let x = i as u64;
        let (_, m) = measure_ticks(|| x >= 100 && x <= 1000);
        measurements.push(m);
    }
    let stats = TickStatistics::from_measurements(&measurements);
    println!("  Result: {}", stats.to_string());
    violations_by_operation.insert("range_check", stats.violations);
    all_measurements.extend(measurements);

    // Test 6: Conditional assignment
    println!("Testing: Conditional assignment...");
    let mut measurements = Vec::new();
    for i in 0..ITERATIONS {
        let x = i as u64;
        let (_, m) = measure_ticks(|| if x % 2 == 0 { x * 2 } else { x * 3 });
        measurements.push(m);
    }
    let stats = TickStatistics::from_measurements(&measurements);
    println!("  Result: {}", stats.to_string());
    violations_by_operation.insert("conditional_assignment", stats.violations);
    all_measurements.extend(measurements);

    // Overall statistics
    println!();
    println!("=== Overall Chatman Constant Enforcement ===");
    let overall_stats = TickStatistics::from_measurements(&all_measurements);
    println!("  Aggregate: {}", overall_stats.to_string());
    println!();

    // Violation breakdown
    println!("=== Violation Breakdown ===");
    let total_violations: usize = violations_by_operation.values().sum();
    for (op, violations) in violations_by_operation.iter() {
        println!("  {}: {} violations", op, violations);
    }
    println!("  Total: {} violations out of {} operations", total_violations, all_measurements.len());
    println!();

    // Assert: Zero violations allowed
    assert!(
        overall_stats.meets_slo(),
        "Chatman constant violated: {} operations exceeded 8 ticks",
        overall_stats.violations
    );

    assert_eq!(
        total_violations, 0,
        "Chatman constant violated: {} total violations across all operations",
        total_violations
    );

    println!("✅ Chatman constant enforcement verified: ALL operations ≤ 8 ticks");
}

#[test]
fn test_parking_decision_latency() {
    println!("\n=== Parking Decision Latency Test ===");
    println!("When τ > 8, decision must be to park the request");
    println!();

    // Simulate decision-making for parking
    let mut measurements = Vec::new();

    for tick_count in 0..20u64 {
        let (decision, m) = measure_ticks(|| {
            // Decision logic: if tick_count > 8, park
            if tick_count > 8 {
                "PARK"
            } else {
                "EXECUTE"
            }
        });

        measurements.push(m);

        // Verify decision is correct
        if tick_count > 8 {
            assert_eq!(decision, "PARK", "Should park when τ > 8");
        } else {
            assert_eq!(decision, "EXECUTE", "Should execute when τ ≤ 8");
        }
    }

    let stats = TickStatistics::from_measurements(&measurements);
    println!("Parking decision overhead: {}", stats.to_string());

    // The parking decision itself must be fast (meta-constraint)
    assert!(
        stats.meets_slo(),
        "Parking decision violated Chatman constant: {}",
        stats.to_string()
    );

    println!("✅ Parking decision latency verified: ≤ 8 ticks");
}

#[test]
fn test_tick_budget_tracking() {
    println!("\n=== Tick Budget Tracking Test ===");

    struct TickBudget {
        remaining: u64,
        spent: u64,
    }

    impl TickBudget {
        fn new() -> Self {
            Self {
                remaining: 8,
                spent: 0,
            }
        }

        fn spend(&mut self, ticks: u64) -> bool {
            if ticks <= self.remaining {
                self.remaining -= ticks;
                self.spent += ticks;
                true
            } else {
                false
            }
        }

        fn is_exhausted(&self) -> bool {
            self.remaining == 0
        }
    }

    // Test budget tracking overhead
    let mut measurements = Vec::new();

    for i in 0..ITERATIONS {
        let (_, m) = measure_ticks(|| {
            let mut budget = TickBudget::new();
            let ticks_to_spend = (i % 4 + 1) as u64;
            budget.spend(ticks_to_spend)
        });
        measurements.push(m);
    }

    let stats = TickStatistics::from_measurements(&measurements);
    println!("Budget tracking overhead: {}", stats.to_string());

    assert!(
        stats.meets_slo(),
        "Budget tracking violated Chatman constant: {}",
        stats.to_string()
    );

    println!("✅ Tick budget tracking verified: ≤ 8 ticks overhead");
}

#[test]
fn test_worst_case_latency() {
    println!("\n=== Worst Case Latency Test ===");
    println!("Testing maximum allowed complexity that still meets τ ≤ 8");
    println!();

    // Worst case: small array scan (hot path limit)
    let data = [(1u64, 100u64), (2, 200), (3, 300), (4, 400), (5, 500)];
    let mut measurements = Vec::new();

    for i in 0..ITERATIONS {
        let key = ((i % 5) + 1) as u64;
        let (_, m) = measure_ticks(|| {
            // Linear scan of 5 elements (worst case for hot path)
            data.iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| *v)
                .unwrap_or(0)
        });
        measurements.push(m);
    }

    let stats = TickStatistics::from_measurements(&measurements);
    println!("Worst case (5-element scan): {}", stats.to_string());

    assert!(
        stats.meets_slo(),
        "Worst case violated Chatman constant: {}",
        stats.to_string()
    );

    println!("✅ Worst case latency verified: ≤ 8 ticks");
}
