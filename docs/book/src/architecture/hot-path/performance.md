# Performance Targets

Hot path performance targets and validation.

## Overview

Hot path operations must complete within 8 ticks (Chatman Constant):
- **Single Hook Execution**: <2ns (8 ticks)
- **Ring Buffer Operations**: Branchless enqueue/dequeue
- **Fiber Execution**: ≤8 ticks per tick slot
- **Memory Layout**: Zero-copy, SIMD-aware, 64-byte alignment

## Performance Validation

### PMU Benchmarks

Performance monitoring unit (PMU) benchmarks verify ≤8 tick budget:

```bash
make test-performance-v04
```

### Tick Budget

All hot path operations must complete within 8 ticks:

```c
#define KNHK_TICK_BUDGET 8u
```

### SLO Tracking

Service level objectives (SLOs) are tracked:
- R1: ≤2ns (8 ticks)
- W1: ≤1ms
- C1: ≤500ms

## Related Documentation

- [Hot Path](../hot-path.md) - Overview
- [Branchless Operations](branchless.md) - Implementation
- [Eval Dispatch](eval-dispatch.md) - Kernel dispatch
