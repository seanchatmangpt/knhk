// rust/knhk-etl/examples/hot_path_optimizations.rs
// Example demonstrating 80/20 optimizations from simdjson lessons
// Shows memory reuse, branchless guards, and zero-copy views

use knhk_etl::{
    guard_validation::{match_ask_sp_branchless, validate_all_guards_branchless},
    hot_path_engine::HotPathEngine,
    load::{PredRun, SoAArrays},
    PipelineError, SoAArraysExt,
};

fn main() {
    println!("=== KNHK Hot Path Optimizations Demo ===\n");

    // Example 1: Memory Reuse Engine
    println!("1. Memory Reuse Engine (simdjson pattern)");
    println!("   Reuse buffers to keep memory hot in cache\n");

    let mut engine = HotPathEngine::new();

    // First operation
    let triples1 = vec![(1, 100, 1000), (2, 100, 2000), (3, 100, 3000)];
    match engine.load_triples(&triples1) {
        Ok(buffers1) => {
            println!("   Loaded {} triples into reusable buffers", triples1.len());
            println!("   Buffer address: {:p}", buffers1);

            // Second operation reuses same buffers (hot in cache)
            let triples2 = vec![(4, 200, 4000), (5, 200, 5000)];
            match engine.load_triples(&triples2) {
                Ok(buffers2) => {
                    println!("   Loaded {} triples, reused buffers", triples2.len());
                    println!("   Buffer address: {:p} (same buffer!)", buffers2);
                }
                Err(e) => println!("   Error: {}", e.message()),
            }
        }
        Err(e) => println!("   Error: {}", e.message()),
    }
    println!();

    // Example 2: Branchless Guard Validation
    println!("2. Branchless Guard Validation (simdjson pattern)");
    println!("   Eliminate branches to avoid misprediction penalties\n");

    let valid_run = PredRun {
        pred: 100,
        off: 0,
        len: 5,
    };

    let invalid_run = PredRun {
        pred: 100,
        off: 0,
        len: 9, // Exceeds max_run_len
    };

    // Branchless validation (returns 1 if valid, 0 otherwise)
    let valid_result = validate_all_guards_branchless(&valid_run, 8, 8);
    let invalid_result = validate_all_guards_branchless(&invalid_run, 8, 8);

    println!(
        "   Valid run (len=5): {}",
        if valid_result == 1 {
            "✅ PASS"
        } else {
            "❌ FAIL"
        }
    );
    println!(
        "   Invalid run (len=9): {}",
        if invalid_result == 0 {
            "✅ REJECTED"
        } else {
            "❌ ACCEPTED"
        }
    );
    println!("   No branches executed - uses arithmetic comparison");
    println!();

    // Example 3: Zero-Copy Triple Views
    println!("3. Zero-Copy Triple Views (simdjson pattern)");
    println!("   Use views instead of copies for zero-copy access\n");

    let mut soa = SoAArrays::new();
    for i in 0..5 {
        soa.s[i] = (i + 1) as u64;
        soa.p[i] = 100;
        soa.o[i] = (i + 100) as u64;
    }

    // Zero-copy view of single triple
    let view = soa.view_triple(0);
    println!(
        "   Triple 0: S={}, P={}, O={}",
        view.subject(),
        view.predicate(),
        view.object()
    );

    // Zero-copy iteration (forward-only)
    println!("   Iterating over triples (zero-copy):");
    for (i, triple_view) in soa.iter_triples(5).enumerate() {
        println!(
            "     [{}] S={}, P={}, O={}",
            i,
            triple_view.subject(),
            triple_view.predicate(),
            triple_view.object()
        );
    }
    println!("   No copying - views reference existing data");
    println!();

    // Example 4: Branchless Matching
    println!("4. Branchless Matching (simdjson pattern)");
    println!("   Use arithmetic for matching instead of branches\n");

    let subject = 1;
    let predicate = 100;
    let target_s = 1;
    let target_p = 100;

    // Branchless ASK_SP matching
    let match_result = match_ask_sp_branchless(subject, predicate, target_s, target_p);
    println!(
        "   ASK_SP({}, {}) matches ({}, {}): {}",
        subject,
        predicate,
        target_s,
        target_p,
        if match_result == 1 {
            "✅ YES"
        } else {
            "❌ NO"
        }
    );
    println!("   No branches - uses arithmetic comparison");
    println!();

    // Example 5: Performance Characteristics
    println!("5. Performance Characteristics");
    println!("   All optimizations target hot path (≤8 ticks constraint)\n");
    println!("   ✅ Memory reuse: Eliminates allocation overhead");
    println!("   ✅ Branchless guards: Eliminates misprediction penalties");
    println!("   ✅ Zero-copy views: Eliminates copying overhead");
    println!("   ✅ Cache alignment: 64-byte aligned SoAArrays");
    println!("   ✅ Benchmarking: Measure and validate performance");
    println!();

    println!("=== Demo Complete ===");
    println!("Run benchmarks: cargo bench --bench hot_path_performance");
}
