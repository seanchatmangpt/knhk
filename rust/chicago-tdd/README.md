# Chicago TDD Performance Harness

**Enforces the Chatman Constant: max_run_length ≤ 8 ticks for all critical path operations**

## Overview

The Chicago TDD Performance Harness is a comprehensive performance measurement and enforcement system that implements **Covenant 5** from `DOCTRINE_COVENANT.md`: "The Chatman Constant Guards All Complexity."

This harness provides:

- ✅ **Precision CPU tick measurement** using RDTSC (x86_64)
- ✅ **Statistical analysis** with warmup/cooldown
- ✅ **Hard bounds enforcement** (≤8 ticks for hot path)
- ✅ **Detailed bottleneck identification**
- ✅ **Regression detection**
- ✅ **CI/CD integration** for build blocking

## The Chatman Constant

**8 ticks** (approximately 2-4 nanoseconds on modern CPUs) is the hard latency bound for all critical path operations. This constant:

- **Enforces bounded complexity**: No unbounded loops or recursion
- **Ensures predictable latency**: Hot path operations are "instant" relative to human time
- **Guards against accidental slowdowns**: Build fails if bounds violated
- **Reflects physical constraints**: Operations must fit in CPU cache

## Quick Start

### Run All Benchmarks

```bash
# Run complete benchmark suite with enforcement
./scripts/bench-all.sh

# Run individual benchmarks
cargo bench --bench executor_latency
cargo bench --bench task_dispatch
cargo bench --bench decision_point
cargo bench --bench join_operation
cargo bench --bench mape_k_latency
```

### Run Tests

```bash
# All tests
cargo test --workspace

# Hot path bounds tests only
cargo test --test integration_tests bounds_tests::hot_path

# Integration tests
cargo test --test integration_tests
```

## Usage in Code

```rust
use chicago_tdd::{PerformanceHarness, OperationType};

fn main() {
    let mut harness = PerformanceHarness::new();

    // Measure a critical path operation
    let result = harness.measure("my_operation", OperationType::HotPath, || {
        // Your hot path code here
        42 + 58
    });

    // Assert it's within bounds (panics if violated)
    result.assert_within_bounds().expect("Operation too slow!");

    // Generate comprehensive report
    let report = harness.report();
    chicago_tdd::Reporter::print_report(&report);
}
```

## Operation Types

### Hot Path (≤8 ticks)

**Critical path operations that MUST be instant:**

- Task lookup in executor
- Case state access
- Pattern registry lookup
- Decision evaluation
- State transition checks
- Join synchronization checks
- MAPE-K monitor/analyze/plan/execute/knowledge

**Example:**
```rust
harness.measure("task_lookup", OperationType::HotPath, || {
    tasks.get(&task_id)
});
```

### Warm Path (≤100ms)

**Operations that can have some latency:**

- Small allocations
- String formatting
- HashMap operations
- Small JSON parsing
- File metadata access

**Example:**
```rust
harness.measure("json_parse", OperationType::WarmPath, || {
    serde_json::from_str::<Value>(&small_json)
});
```

### Cold Path (Diagnostic Only)

**No hard bound - measured for informational purposes:**

- Large allocations
- Thread spawning
- File I/O
- Network calls
- Large JSON parsing

**Example:**
```rust
harness.measure("large_alloc", OperationType::ColdPath, || {
    Vec::<u64>::with_capacity(1_000_000)
});
```

## Benchmarks

### Executor Latency (`executor_latency.rs`)

Measures workflow engine executor operations:
- Task lookup
- Case state access
- Pattern lookup
- Decision evaluation
- State transition checks

### Task Dispatch (`task_dispatch.rs`)

Measures task dispatch operations:
- Queue enqueue
- Priority calculation
- Resource availability check
- Task ID generation
- Dispatch decision

### Decision Point (`decision_point.rs`)

Measures split/join decision evaluation:
- AND-split (all branches)
- XOR-split (exclusive choice)
- OR-split (multiple branches)
- Guard condition evaluation
- Branch selection

### Join Operation (`join_operation.rs`)

Measures join synchronization:
- AND-join (wait for all)
- XOR-join (first wins)
- OR-join (wait for active)
- Join counter increment
- Join condition check

### MAPE-K Latency (`mape_k_latency.rs`)

Measures autonomic loop operations:
- Monitor: Metric collection decision
- Analyze: Anomaly detection
- Plan: Policy lookup
- Execute: Action selection
- Knowledge: Pattern matching

## CI/CD Integration

### GitHub Actions

```yaml
- name: Chicago TDD Performance Enforcement
  run: |
    ./scripts/bench-all.sh
  # Fails build if any bounds violated
```

### Makefile Target

```makefile
.PHONY: test-chicago-v04
test-chicago-v04:
	@./scripts/bench-all.sh
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit
./scripts/bench-all.sh || {
    echo "❌ Performance bounds violated - commit blocked"
    exit 1
}
```

## Output Formats

### Terminal Report

Colorized report with:
- Summary statistics
- Per-operation results
- Bottleneck identification
- Recommendations

### CSV Export

```rust
let report = harness.report();
let csv = Reporter::export_csv(&report);
std::fs::write("results.csv", csv)?;
```

### JSON Export

```rust
let report = harness.report();
let json = Reporter::export_json(&report);
std::fs::write("results.json", json)?;
```

## Architecture

```
chicago-tdd/
├── src/
│   ├── lib.rs          # Core harness and measurement
│   ├── timer.rs        # Precision timing (RDTSC)
│   └── reporter.rs     # Reporting and visualization
├── benches/
│   ├── executor_latency.rs
│   ├── task_dispatch.rs
│   ├── decision_point.rs
│   ├── join_operation.rs
│   └── mape_k_latency.rs
├── tests/
│   ├── integration_tests.rs
│   └── bounds_tests/
│       ├── hot_path.rs
│       ├── warm_path.rs
│       ├── cold_path.rs
│       └── regression.rs
└── examples/
    └── basic_usage.rs
```

## Doctrine Alignment

This harness implements **Covenant 5** from `DOCTRINE_COVENANT.md`:

```
Covenant 5: The Chatman Constant Guards All Complexity (Q3 ⊨ Boundedness)

Principle: max_run_length ≤ 8 ticks

What This Means:
- 8 ticks is the hard latency bound for all critical path operations
- Exceeding 8 ticks means the operation is not on the critical path
- The constant is enforced at runtime and build time
- No unbounded recursion or iteration allowed

What Violates This Covenant:
- ❌ Any critical path operation exceeding 8 ticks
- ❌ Unbounded recursion or iteration
- ❌ Blocking I/O on the critical path
- ❌ Hot loop code that doesn't fit in CPU cache

What Embodies This Covenant:
- ✅ Chicago TDD harness measures every path in ticks
- ✅ max_run_length ≤ 8 enforced in MAPE-K Execute stage
- ✅ Hot path benchmarks run continuously
- ✅ Code violating the constant is rejected at build time
```

## Frequently Asked Questions

### Q: Why 8 ticks?

**A:** 8 ticks (≈2-4ns) is the point at which an operation is "instant" relative to human time but still measurable. It ensures operations fit in CPU cache and prevent unbounded complexity.

### Q: What if my operation needs more than 8 ticks?

**A:** Move it off the critical path. Use:
- Async execution for I/O
- Background threads for heavy computation
- Caching for repeated calculations
- Warm/Cold path classification

### Q: How accurate is RDTSC?

**A:** RDTSC provides sub-nanosecond precision on x86_64. We calibrate overhead and use compiler fences to prevent reordering. Results are statistically robust (10,000 samples).

### Q: Can I disable enforcement for development?

**A:** No. The Chatman Constant is a hard invariant (Covenant 5). If an operation exceeds bounds, optimize it or move it off the critical path.

### Q: What about non-x86 architectures?

**A:** The harness falls back to `Instant::now()` for nanosecond precision on other architectures. Bounds are still enforced.

## Contributing

All PRs must pass Chicago TDD benchmarks:

1. Run `./scripts/bench-all.sh` before committing
2. Fix any bounds violations
3. Document hot path optimizations
4. Update benchmarks if adding new critical paths

## License

MIT OR Apache-2.0 (same as KNHK project)

## See Also

- `DOCTRINE_2027.md` - Foundational principles
- `DOCTRINE_COVENANT.md` - Covenant 5 specification
- `CHATMAN_EQUATION_SPEC.md` - Formal derivation of constant
- `knhk-workflow-engine` - Production executor that uses this harness
