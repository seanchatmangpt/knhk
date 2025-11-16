//! Basic usage example of Chicago TDD Performance Harness

use chicago_tdd::{PerformanceHarness, OperationType, Reporter};

fn main() {
    println!("Chicago TDD Performance Harness - Basic Usage Example\n");

    let mut harness = PerformanceHarness::new();

    // Example 1: Measure a hot path operation
    println!("=== Example 1: Hot Path Operation ===");
    let result1 = harness.measure("simple_arithmetic", OperationType::HotPath, || {
        let a = 42;
        let b = 58;
        a + b
    });

    Reporter::print_result(&result1);

    // Example 2: Measure a warm path operation
    println!("\n=== Example 2: Warm Path Operation ===");
    let result2 = harness.measure("string_format", OperationType::WarmPath, || {
        format!("Task_{}", 12345)
    });

    Reporter::print_result(&result2);

    // Example 3: Measure a cold path operation
    println!("\n=== Example 3: Cold Path Operation ===");
    let result3 = harness.measure("large_allocation", OperationType::ColdPath, || {
        Vec::<u64>::with_capacity(10000)
    });

    Reporter::print_result(&result3);

    // Example 4: Multiple operations and comprehensive report
    println!("\n=== Example 4: Multiple Operations ===");
    harness.clear(); // Clear previous results

    harness.measure("add", OperationType::HotPath, || 1 + 2);
    harness.measure("subtract", OperationType::HotPath, || 10 - 5);
    harness.measure("multiply", OperationType::HotPath, || 3 * 4);
    harness.measure("divide", OperationType::HotPath, || 20 / 4);
    harness.measure("comparison", OperationType::HotPath, || 42 > 10);

    let report = harness.report();
    Reporter::print_report(&report);

    // Example 5: Bounds enforcement
    println!("\n=== Example 5: Bounds Enforcement ===");
    match harness.assert_all_within_bounds() {
        Ok(_) => println!("✅ All operations within bounds!"),
        Err(e) => {
            eprintln!("❌ Bounds violation detected:");
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }

    // Example 6: Export results
    println!("\n=== Example 6: Export Results ===");

    let csv = Reporter::export_csv(&report);
    println!("CSV Export (first 5 lines):");
    for (i, line) in csv.lines().take(5).enumerate() {
        println!("{}: {}", i + 1, line);
    }

    let json = Reporter::export_json(&report);
    println!("\nJSON Export (truncated):");
    println!("{}", &json[..json.len().min(200)]);
    println!("...");

    println!("\n✅ Example completed successfully!");
}
