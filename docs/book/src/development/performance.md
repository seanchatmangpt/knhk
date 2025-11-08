# Performance

KNHK is optimized for hot path operations with ≤2ns latency (8 ticks).

## Performance Targets

### Hot Path

- **Single Hook Execution**: <2ns (8 ticks) per admitted unit
- **Ring Buffer Operations**: Branchless enqueue/dequeue
- **Fiber Execution**: ≤8 ticks per tick slot
- **Memory Layout**: Zero-copy, SIMD-aware, 64-byte alignment

### ETL Pipeline

- **Beat Advancement**: Continuous cycle/tick/pulse generation
- **Delta Admission**: Cycle ID stamping on admission
- **Fiber Rotation**: Per-shard execution with tick-based rotation
- **Commit Boundary**: Pulse-triggered commit (every 8 ticks)

### Cold Path

- **100 hooks**: <100ms (parallel)
- **1000 hooks**: <1s (parallel)
- **Throughput**: 1000+ hooks/sec

## Optimization Techniques

### Branchless Operations

All hot path operations are branchless:

```c
// Branchless tick extraction
uint64_t tick = cycle & 0x7;

// Branchless pulse detection
uint64_t pulse = !(tick & 0x7);
```

### SoA Layout

Structure of Arrays (SoA) for SIMD optimization:

```rust
struct SoAArrays {
    s: [u64; 8],  // Subjects
    p: [u64; 8],  // Predicates
    o: [u64; 8],  // Objects
}
```

### Cache Optimization

- 64-byte alignment prevents false sharing
- SoA layout optimizes cache line usage
- Prefetch-friendly access patterns

## Performance Monitoring

### PMU Benchmarks

Performance monitoring unit (PMU) benchmarks verify ≤8 tick budget:

```bash
make test-performance-v04
```

### SLO Tracking

Service level objectives (SLOs) are tracked per runtime class:
- R1: ≤2ns (8 ticks)
- W1: ≤1ms
- C1: ≤500ms

## Related Documentation

- [Hot Path](../architecture/hot-path.md) - Hot path operations
- [8-Beat System](../architecture/8beat-system.md) - Epoch system
- [Branchless C Engine](../architecture/branchless-engine.md) - C implementation
