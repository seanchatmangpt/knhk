# Hooks Engine Stress Tests and Benchmarks

## Overview

Comprehensive stress tests and benchmarks for the Rust-native hooks engine, focusing on the 2 primary use cases: single hook execution and batch evaluation.

## Stress Tests

Located in `rust/knhk-unrdf/src/hooks_native_stress.rs`

### Test Coverage

1. **Concurrent Hook Execution** (`test_concurrent_hook_execution`)
   - Executes 1000 hooks (10 threads × 100 hooks/thread) concurrently
   - Validates thread safety and correctness under concurrent load
   - Measures throughput (hooks/sec)

2. **Large Batch Evaluation** (`test_large_batch_evaluation`)
   - Evaluates 1000 hooks in a single batch
   - Tests parallel execution via Rayon
   - Validates all hooks fire correctly

3. **Registry Concurrent Access** (`test_registry_concurrent_access`)
   - 20 threads, each registering 50 hooks
   - Tests thread-safe registry operations
   - Validates all hooks registered correctly

4. **Memory Pressure** (`test_memory_pressure`)
   - Executes 100 hooks against 10,000 triples
   - Tests memory efficiency with large datasets
   - Measures throughput under memory pressure

5. **Hook Receipt Uniqueness** (`test_hook_receipt_uniqueness`)
   - Generates 1000 receipts and validates uniqueness
   - Tests cryptographic receipt generation
   - Ensures no collisions (uses timestamp + counter)

6. **Batch with Varying Complexity** (`test_batch_with_varying_complexity`)
   - Tests batch execution with hooks of different query complexities
   - Validates handling of simple vs complex queries

7. **Error Handling Under Load** (`test_error_handling_under_load`)
   - Tests graceful error handling with invalid hooks
   - Validates error propagation

## Benchmarks

Located in `rust/knhk-unrdf/benches/hooks_native_bench.rs`

### Benchmark Groups

1. **Single Hook Execution** (`benchmark_single_hook_execution`)
   - Benchmarks with data sizes: 10, 100, 1000, 10000 triples
   - Measures execution time per hook
   - Provides throughput metrics

2. **Batch Hook Evaluation** (`benchmark_batch_hook_evaluation`)
   - Benchmarks with batch sizes: 1, 10, 100, 1000 hooks
   - Measures parallel execution efficiency
   - Compares single vs batch performance

3. **Hook Registry Operations** (`benchmark_hook_registry_operations`)
   - Benchmarks `register()`, `get()`, `list()` operations
   - Measures registry operation overhead

4. **Hook Receipt Generation** (`benchmark_hook_receipt_generation`)
   - Benchmarks cryptographic receipt generation
   - Measures SHA-256 hashing overhead

5. **Query Complexity** (`benchmark_query_complexity`)
   - Benchmarks different query complexities:
     - Simple: `ASK { ?s ?p ?o }`
     - With filter: `ASK { ?s ?p ?o . FILTER(?o > "100") }`
     - Multi-pattern: `ASK { ?s ?p ?o . ?s ?p2 ?o2 }`
     - Complex: `ASK { ?s ?p ?o . ?s ?p2 ?o2 . FILTER(?o != ?o2) }`

## Running Tests

```bash
# Run all stress tests
cargo test --features native stress_tests

# Run specific stress test
cargo test --features native stress_tests::test_concurrent_hook_execution

# Run benchmarks (requires --release)
cargo bench --features native

# Run with output
cargo bench --features native -- --nocapture
```

## Performance Targets

### Hot Path Requirements
- Single hook execution: ≤8 ticks (2ns) for hot path operations
- Batch evaluation: Parallel execution should scale linearly with cores
- Memory: Zero-copy where possible, efficient allocation

### Expected Results
- Single hook (100 triples): <1ms
- Batch (100 hooks): <100ms (parallel)
- Receipt generation: <10μs
- Registry operations: <1μs

## Test Results

All stress tests pass:
- ✅ Concurrent hook execution
- ✅ Large batch evaluation
- ✅ Registry concurrent access
- ✅ Memory pressure
- ✅ Hook receipt uniqueness (fixed with counter)
- ✅ Batch with varying complexity
- ✅ Error handling under load

## Benchmark Output

Benchmarks generate HTML reports in `target/criterion/hooks_native_bench/`:
- Visual performance comparisons
- Throughput metrics
- Statistical analysis

## Notes

- Receipt uniqueness uses high-resolution timestamp (nanoseconds) + atomic counter
- Parallel execution uses Rayon for efficient thread pool management
- All tests use production-ready implementations (no placeholders)
- Error handling validates graceful degradation under load

