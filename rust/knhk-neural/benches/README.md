# KNHK Phase 6: Neural Integration Benchmarks

## Overview

The Phase 6 benchmark suite provides comprehensive performance validation for KNHK's advanced neural integration system. It measures training performance, inference speed, reinforcement learning efficiency, and validates the claimed 20%+ daily improvement.

**File**: `phase_6_neural_benchmarks.rs` (767 lines)

## Architecture

The benchmark suite is organized into four key groups:

### 1. Training Benchmarks (`training_benches`)
- **Epoch Time Analysis**: Measures wall-clock time per training epoch across dataset sizes (100, 1000, 10,000 samples)
- **Convergence Speed**: Tracks epochs needed to reach target loss (0.1)
- **Optimizer Comparison**: SGD vs Adam vs AdamW performance

**Target Metrics**:
- 100 samples: <100ms per epoch
- 1000 samples: <500ms per epoch
- 10,000 samples: <2 seconds per epoch
- Linear scaling validation: `epoch_time(n) ≤ 1.1 × epoch_time(n/10)`

### 2. Inference Benchmarks (`inference_benches`)
- **Latency Measurement**: Single prediction latency with target <100µs
- **Throughput Testing**: Batch prediction performance across batch sizes (1, 10, 100, 1000)

**Target Metrics**:
- Latency: <100µs per prediction (hot path: ≤8 ticks @ 3.5GHz ≈ 91ns)
- Throughput: ≥10,000 predictions/second in batch mode
- Batch efficiency: 100 predictions in <10ms

### 3. Reinforcement Learning Benchmarks (`rl_benches`)
- **Action Selection**: ε-greedy policy performance
- **Q-Learning Convergence**: Episodes to convergence vs state space size (10, 100, 1000 states)

**Target Metrics**:
- Action selection: <1µs per operation (hot path)
- Convergence: <1000 episodes for 100-state space
- Memory overhead: <100KB per 1000 Q-table entries

### 4. Validation Benchmarks (`validation_benches`)
- **Memory Usage**: Peak RSS monitoring during training
- **Daily Improvement**: Validates ≥20% performance gain in 5 epochs
- **Adaptive Workflow**: End-to-end workflow execution with learning

**Target Metrics**:
- Model memory: <10MB per instance
- Training batch memory: <50MB for 1000 samples @ 128 features
- Daily improvement: ≥20% within 5 training epochs
- Consistent improvement across iterations

## Running the Benchmarks

### Build the Benchmark Suite
```bash
cd /home/user/knhk
cargo build -p knhk-neural --benches
```

### Run All Benchmarks
```bash
cargo bench -p knhk-neural
```

### Run Specific Benchmark Groups

**Training Performance** (measures epoch time and convergence):
```bash
cargo bench -p knhk-neural -- --test-threads=1 training_benches
```

**Inference Performance** (measures latency and throughput):
```bash
cargo bench -p knhk-neural -- --test-threads=1 inference_benches
```

**Reinforcement Learning** (measures Q-Learning performance):
```bash
cargo bench -p knhk-neural -- --test-threads=1 rl_benches
```

**Validation Benchmarks** (memory, improvement, workflow):
```bash
cargo bench -p knhk-neural -- --test-threads=1 validation_benches
```

### Run Single Benchmark Function
```bash
# Training epoch time
cargo bench -p knhk-neural -- bench_training_epoch_time

# Inference latency (critical hot path)
cargo bench -p knhk-neural -- bench_inference_latency

# Q-Learning action selection (critical hot path)
cargo bench -p knhk-neural -- bench_ql_action_selection

# Daily improvement validation
cargo bench -p knhk-neural -- bench_daily_improvement
```

### Generate HTML Reports
Criterion automatically generates detailed HTML reports in:
```
target/criterion/
```

Open `index.html` to view interactive performance graphs and statistical analysis.

## Benchmark Details

### Training Epoch Time Benchmark
```rust
bench_training_epoch_time(c: &mut Criterion)
```

**What it measures**:
- Wall-clock time for one complete training epoch
- Tests with 100, 1000, and 10,000 sample datasets
- Uses 128 input features, 64 hidden layer
- Batch size: 32 samples
- Learning rate: 0.001

**Expected output** (in target/criterion/):
```
training_epoch_time/100       [time]   [variance]
training_epoch_time/1000      [time]   [variance]
training_epoch_time/10000     [time]   [variance]
```

### Convergence Speed Benchmark
```rust
bench_convergence_speed(c: &mut Criterion)
```

**What it measures**:
- Total training time to reach target loss (0.1)
- Number of epochs required for convergence
- Tests 100-sample and 1000-sample datasets

**Acceptance**: <50 epochs for 100 samples, <100 epochs for 1000 samples

### Inference Latency Benchmark
```rust
bench_inference_latency(c: &mut Criterion)
```

**What it measures**:
- Single forward pass latency: 128 input → 64 output layer
- Uses high-precision timing (criterion's statistical framework)
- Measures in microseconds (µs)

**Critical Target**: <100µs per prediction
- This is the hot path for inference
- Aligns with Chatman Constant (≤8 ticks @ 3.5GHz)

### Inference Throughput Benchmark
```rust
bench_inference_throughput(c: &mut Criterion)
```

**What it measures**:
- Batch prediction performance for batch sizes: 1, 10, 100, 1000
- Calculates predictions per second
- Tests linear scaling efficiency

**Expected output**:
```
inference_throughput/batch_predictions/1      [time/batch]
inference_throughput/batch_predictions/10     [time/batch]
inference_throughput/batch_predictions/100    [time/batch]
inference_throughput/batch_predictions/1000   [time/batch]
```

### Q-Learning Action Selection (Critical Hot Path)
```rust
bench_ql_action_selection(c: &mut Criterion)
```

**What it measures**:
- ε-greedy action selection latency
- Per-action selection time
- Randomness overhead

**Critical Target**: <1µs per action
- This is the innermost loop in RL training
- Must meet Chatman Constant constraints

### Q-Learning Convergence Benchmark
```rust
bench_ql_convergence(c: &mut Criterion)
```

**What it measures**:
- Episodes to convergence for state spaces of size 10, 100, 1000
- Q-table update latency
- Memory allocation patterns

**Expected behavior**:
- Linear growth in episodes with state space size
- Consistent per-update latency regardless of Q-table size

### Memory Usage Benchmark
```rust
bench_memory_usage(c: &mut Criterion)
```

**What it measures**:
- Per-model memory overhead
- Training batch memory allocation
- Memory leak detection

**Expected output**:
```
model_memory_128x64x32  [time to allocate]
training_memory_1000_samples  [time to train 1 epoch]
```

### Daily Improvement Validation
```rust
bench_daily_improvement(c: &mut Criterion)
```

**What it measures**:
- Loss reduction over 5 training epochs
- Improvement percentage: `(initial_loss - final_loss) / initial_loss × 100`

**Critical Target**: ≥20% improvement within 5 epochs
- Validates the core KNHK claim
- Uses 1000-sample dataset
- Batch size: 32

**Implementation detail**:
- Benchmark passes if improvement > 20%
- Automatically detects learning rate issues
- Verifies convergence stability

### Adaptive Workflow Execution
```rust
bench_adaptive_workflow(c: &mut Criterion)
```

**What it measures**:
- End-to-end workflow execution time (100 steps)
- Policy learning during execution
- Adaptive reward accumulation

**Expected behavior**:
- Should find optimal reward pattern around step 50
- Convergence to better policies over iterations

## DOCTRINE ALIGNMENT

### Principle: Q (Hard Invariants)
All benchmarks enforce the Q invariant: performance metrics are hard constraints that cannot be violated.

- **Q1 - No Retrocausation**: Timing measurements form immutable timeline
- **Q2 - Type Soundness**: All metrics have well-defined types
- **Q3 - Bounded Recursion**: Chatman Constant ≤8 ticks enforced for hot paths
- **Q4 - Latency SLOs**: Hard limits on inference (<100µs) and action selection (<1µs)
- **Q5 - Resource Bounds**: Memory budgets enforced per component

### Principle: Chatman Constant
The Chatman Constant (≤8 ticks for hot path operations) is validated:
- **Inference latency**: <100µs @ 3.5GHz ≈ 91ns (≤0.35 ticks overhead)
- **Action selection**: <1µs @ 3.5GHz ≈ 3.5 ticks (≤8 ticks hard limit)

### Principle: MAPE-K Integration
Benchmarks feed into MAPE-K learning loops:
- **Monitor**: Criterion collects detailed performance metrics
- **Analyze**: Statistical tests validate targets
- **Plan**: Failing benchmarks trigger optimization proposals
- **Execute**: Optimization changes tested against benchmarks
- **Knowledge**: Results inform performance models

## Validation with Weaver

After benchmarks pass, validate schema conformance:

```bash
# Validate OpenTelemetry schema
weaver registry check -r /home/user/knhk/registry/

# Live check: validate runtime telemetry matches schema
weaver registry live-check --registry /home/user/knhk/registry/
```

Benchmark results should emit:
- **Spans**: Training epoch timing, inference latency spans
- **Metrics**: Throughput (ops/sec), memory usage (bytes), loss trend
- **Logs**: Convergence tracking, optimization decisions

## Performance Analysis Workflow

### 1. Baseline Establishment
```bash
# Run full benchmark suite and save results
cargo bench -p knhk-neural > benchmark_baseline.txt 2>&1
```

### 2. Check Key Metrics
The benchmark output will show:
```
time:   [X.XXX ms X.XXX ms X.XXX ms]
        change: [-Y.YY% +Y.YY% +Y.YY%] (95% CI)
        Performance has improved.
```

### 3. Identify Bottlenecks
- If `training_epoch_time` > targets: Optimize matrix operations
- If `inference_latency` > 100µs: Profile neural layer forward pass
- If `ql_action_selection` > 1µs: Optimize random number generation or Q-table lookup
- If `daily_improvement` < 20%: Review learning rate scheduling

### 4. Optimization Loop
```
1. Run baseline benchmark
2. Make optimization change
3. Run benchmark again
4. Compare results (Criterion shows statistical significance)
5. If improvement < 5%, revert change
6. If improvement > 5%, keep and benchmark next target
```

## Integration with CI/CD

### GitHub Actions Workflow
Add to `.github/workflows/benchmark.yml`:

```yaml
name: Performance Benchmarks

on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - name: Run benchmarks
        run: cargo bench -p knhk-neural -- --test-threads=1
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: criterion-results
          path: target/criterion/
```

### Performance Regression Detection
Criterion automatically detects performance regressions:
- If measurement > last baseline + threshold, build fails
- Historical data stored in `target/criterion/`
- Compare branches: `cargo bench -- --baseline <baseline-name>`

## Troubleshooting

### Benchmark Builds But Won't Run
```bash
# Ensure dependencies are installed
cargo fetch -p knhk-neural

# Clean and rebuild
cargo clean -p knhk-neural
cargo build -p knhk-neural --benches
```

### Outliers in Measurements
Criterion handles outliers automatically, but if results are noisy:
- Increase sample size: `--sample-size 100`
- Increase measurement time: `--measurement-time 10`
- Disable power management on Linux: `sudo cpupower frequency-set -g performance`

### Memory Profiling
For detailed memory profiling beyond the benchmark:
```bash
# Using Valgrind
valgrind --tool=massif cargo bench -p knhk-neural -- bench_memory_usage

# View results
ms_print massif.out.<pid>
```

## References

- **KNHK Doctrine**: `/home/user/knhk/DOCTRINE_2027.md`
- **Covenant Enforcement**: `/home/user/knhk/DOCTRINE_COVENANT.md`
- **Criterion.rs Docs**: https://bheisler.github.io/criterion.rs/book/
- **OpenTelemetry Weaver**: https://github.com/open-telemetry/weaver
- **Chicago TDD**: Latency measurement harness in `chicago-tdd/`

## Success Criteria

All benchmarks pass when:
1. ✅ Training epochs meet timing targets
2. ✅ Convergence reaches target loss in specified epochs
3. ✅ Inference latency < 100µs (hot path)
4. ✅ Q-Learning action selection < 1µs
5. ✅ Daily improvement > 20% within 5 epochs
6. ✅ Memory usage < budgets
7. ✅ Optimizer comparison shows <15% overhead
8. ✅ Weaver schema validation passes
9. ✅ Statistical significance confirmed by Criterion

## File Structure

```
/home/user/knhk/rust/knhk-neural/
├── benches/
│   ├── phase_6_neural_benchmarks.rs   (767 lines - main benchmark suite)
│   └── README.md                       (this file)
├── src/
│   ├── lib.rs
│   ├── model.rs                        (NeuralModel + DenseLayer)
│   ├── reinforcement.rs                (QLearning + SARSA agents)
│   ├── optimizer.rs                    (SGD, Adam, AdamW)
│   ├── training.rs                     (Training loop)
│   └── workflow.rs                     (SelfLearningWorkflow)
├── Cargo.toml
└── target/
    └── criterion/                      (Benchmark reports)
```

## Contact & Issues

For benchmark failures or questions:
1. Check DOCTRINE_2027.md for principles
2. Review Covenant enforcement in DOCTRINE_COVENANT.md
3. Run Weaver validation
4. Check Criterion HTML reports in target/criterion/
