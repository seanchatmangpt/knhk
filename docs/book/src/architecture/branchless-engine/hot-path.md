# Hot Path Implementation

The hot path provides ≤2ns latency (8 ticks) for critical operations.

## Overview

The hot path is implemented in C for minimal latency:
- Branchless operations
- SIMD optimization
- Cache-friendly memory layout
- Zero-copy operations

## Core Operations

### ASK Operations

```c
knhk_result_t knhk_eval_ask(knhk_soa_t *soa, knhk_query_t *query) {
    // Branchless ASK evaluation
    // Returns boolean result
}
```

### COUNT Operations

```c
uint64_t knhk_eval_count(knhk_soa_t *soa, knhk_query_t *query) {
    // Branchless COUNT evaluation
    // Returns count of matching triples
}
```

### COMPARE Operations

```c
int knhk_eval_compare(knhk_soa_t *soa, knhk_query_t *query) {
    // Branchless COMPARE evaluation
    // Returns comparison result
}
```

## Performance Targets

- **Single Operation**: <2ns (8 ticks)
- **Memory Access**: Cache-aligned (64-byte)
- **Branch Prediction**: Zero branches in hot path
- **SIMD**: Vectorized operations where possible

## Related Documentation

- [SIMD Optimization](simd.md) - SIMD implementation
- [Cache Optimization](cache.md) - Cache optimization
- [Hot Path](../hot-path.md) - Hot path overview
