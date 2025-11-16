# Chicago TDD Performance Harness - Quick Start

## Installation

```bash
cd /home/user/knhk/rust
cargo build -p chicago-tdd
```

## Run All Benchmarks

```bash
./scripts/bench-all.sh
```

This will:
- Run all 5 benchmark suites (25+ operations)
- Run integration tests
- Run bounds tests
- Generate timestamped results in `chicago-tdd-results/`
- **Exit with error if any operation exceeds 8 ticks**

## Run Individual Benchmarks

```bash
# Executor latency
cargo bench --bench executor_latency

# Task dispatch
cargo bench --bench task_dispatch

# Decision points
cargo bench --bench decision_point

# Join operations
cargo bench --bench join_operation

# MAPE-K loops
cargo bench --bench mape_k_latency
```

## Run Tests

```bash
# All tests
cargo test -p chicago-tdd

# Specific test suites
cargo test -p chicago-tdd --lib          # Unit tests
cargo test -p chicago-tdd --test integration_tests  # Integration
cargo test -p chicago-tdd bounds_tests::hot_path    # Hot path only
```

## Run Example

```bash
cargo run -p chicago-tdd --example basic_usage
```

## Use in Your Code

```rust
use chicago_tdd::{PerformanceHarness, OperationType};

fn main() {
    let mut harness = PerformanceHarness::new();

    // Measure a hot path operation
    let result = harness.measure("my_operation", OperationType::HotPath, || {
        // Your critical path code here
        some_fast_operation()
    });

    // Check if within bounds
    match result.assert_within_bounds() {
        Ok(_) => println!("✅ Operation within Chatman Constant (≤8 ticks)"),
        Err(e) => {
            eprintln!("❌ VIOLATION: {}", e);
            eprintln!("  P99: {} ticks", result.statistics.p99);
            std::process::exit(1);
        }
    }

    // Generate report
    let report = harness.report();
    chicago_tdd::Reporter::print_report(&report);
}
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Performance Gate
on: [push, pull_request]

jobs:
  performance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Chicago TDD Enforcement
        run: ./scripts/bench-all.sh
        # Build fails if any bounds violated
```

### Makefile

```makefile
.PHONY: test-chicago
test-chicago:
	@./scripts/bench-all.sh
```

## File Locations

```
Production Code:
  /home/user/knhk/rust/chicago-tdd/src/lib.rs       - Core harness
  /home/user/knhk/rust/chicago-tdd/src/timer.rs     - Precision timing
  /home/user/knhk/rust/chicago-tdd/src/reporter.rs  - Reporting

Benchmarks:
  /home/user/knhk/rust/chicago-tdd/benches/executor_latency.rs
  /home/user/knhk/rust/chicago-tdd/benches/task_dispatch.rs
  /home/user/knhk/rust/chicago-tdd/benches/decision_point.rs
  /home/user/knhk/rust/chicago-tdd/benches/join_operation.rs
  /home/user/knhk/rust/chicago-tdd/benches/mape_k_latency.rs

Tests:
  /home/user/knhk/rust/chicago-tdd/tests/integration_tests.rs
  /home/user/knhk/rust/chicago-tdd/tests/bounds_tests/hot_path.rs
  /home/user/knhk/rust/chicago-tdd/tests/bounds_tests/warm_path.rs
  /home/user/knhk/rust/chicago-tdd/tests/bounds_tests/cold_path.rs
  /home/user/knhk/rust/chicago-tdd/tests/bounds_tests/regression.rs

Scripts:
  /home/user/knhk/scripts/bench-all.sh              - Automation script

Documentation:
  /home/user/knhk/rust/chicago-tdd/README.md        - Complete guide
  /home/user/knhk/docs/CHICAGO_TDD_IMPLEMENTATION.md - Technical details
  /home/user/knhk/docs/CHICAGO_TDD_DELIVERY_SUMMARY.md - Delivery report
  /home/user/knhk/rust/chicago-tdd/QUICKSTART.md    - This file
```

## Key Concepts

### Operation Types

| Type | Bound | Purpose |
|------|-------|---------|
| **HotPath** | ≤8 ticks | Critical path operations (build-blocking) |
| **WarmPath** | ≤100ms | Non-critical operations (build-blocking) |
| **ColdPath** | None | Diagnostic only (informational) |

### Statistics Reported

- **p50**: Median latency
- **p75**: 75th percentile
- **p90**: 90th percentile
- **p95**: 95th percentile
- **p99**: 99th percentile (enforcement threshold)
- **p99.9**: 99.9th percentile
- **Mean**: Average latency
- **Std Dev**: Standard deviation
- **CV**: Coefficient of variation

### The Chatman Constant

**8 ticks** (approximately 2-4 nanoseconds on modern CPUs) is the hard latency bound for critical path operations.

**Why 8 ticks?**
- Operations fit in CPU L1 cache
- "Instant" relative to human perception
- Prevents unbounded complexity
- Forces architectural discipline
- Measurable with precision

## Troubleshooting

### Operation Exceeds 8 Ticks

**Problem**: `Hot path operation 'my_op' exceeded Chatman Constant: 15 ticks > 8 ticks`

**Solutions**:
1. **Move off critical path**: If operation isn't truly critical, use `WarmPath` or `ColdPath`
2. **Optimize algorithm**: Reduce computational complexity
3. **Cache results**: Precompute values
4. **Use better data structures**: Hash tables instead of linear search
5. **Minimize allocations**: Use stack instead of heap
6. **Avoid I/O**: Never do I/O on hot path

### Measurements Seem High

**Check**:
1. Are you running in release mode? (`cargo bench --release`)
2. Is CPU frequency scaling disabled?
3. Are other processes consuming CPU?
4. Is thermal throttling occurring?

### Tests Failing

**Common Issues**:
1. Overhead calibration needs longer warmup
2. System under heavy load
3. Running in debug mode (much slower)

**Fix**: Use release mode and quiet system:
```bash
cargo test --release -p chicago-tdd
```

## Next Steps

1. Read the complete documentation: `chicago-tdd/README.md`
2. Review benchmark implementations for examples
3. Run the example: `cargo run --example basic_usage`
4. Integrate into your project
5. Set up CI/CD enforcement

## Support

- **Documentation**: `/home/user/knhk/rust/chicago-tdd/README.md`
- **Implementation Guide**: `/home/user/knhk/docs/CHICAGO_TDD_IMPLEMENTATION.md`
- **Delivery Summary**: `/home/user/knhk/docs/CHICAGO_TDD_DELIVERY_SUMMARY.md`
- **Doctrine Reference**: `/home/user/knhk/DOCTRINE_COVENANT.md` (Covenant 5)

## Quick Commands

```bash
# Build
cargo build -p chicago-tdd

# Test
cargo test -p chicago-tdd

# Benchmark
./scripts/bench-all.sh

# Example
cargo run -p chicago-tdd --example basic_usage

# Check
cargo check -p chicago-tdd
```

---

**Remember**: The Chatman Constant is not a guideline—it's a hard invariant. Operations exceeding 8 ticks are not on the critical path.
