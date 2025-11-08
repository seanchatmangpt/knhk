# Branchless Operations

Branchless operations for constant-time execution in hot path.

## Overview

All hot path operations are branchless:
- No conditional branches
- Constant-time execution
- Predictable performance
- SIMD-friendly

## Branchless Patterns

### Tick Extraction

```c
// Branchless: extract bits 0-2
uint64_t tick = cycle & 0x7;
```

### Pulse Detection

```c
// Branchless: 1 when tick==0, 0 otherwise
uint64_t pulse = !(cycle & 0x7);
```

### Conditional Selection

```c
// Branchless: select value based on condition
uint64_t result = (condition ? value_a : value_b);
// Optimized to: result = (condition * value_a) | (!condition * value_b);
```

## Benefits

1. **Constant Time**: No branch prediction overhead
2. **Predictable**: Same input → same execution time
3. **SIMD-Friendly**: Vectorized operations possible
4. **Cache-Friendly**: No branch mispredictions

## Related Documentation

- [Hot Path](../hot-path.md) - Overview
- [Performance Targets](performance.md) - Performance goals
- [Eval Dispatch](eval-dispatch.md) - Kernel dispatch
