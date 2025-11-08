# Cache Optimization

Cache optimization for hot path performance.

## Overview

KNHK optimizes for cache performance:
- 64-byte alignment (cache line size)
- SoA layout (cache-friendly access patterns)
- Prefetch-friendly data structures
- Minimize cache misses

## Cache Line Alignment

### 64-Byte Alignment

```c
typedef struct {
    alignas(64) uint64_t s[8];  // Cache-aligned subjects
    alignas(64) uint64_t p[8];   // Cache-aligned predicates
    alignas(64) uint64_t o[8];   // Cache-aligned objects
} knhk_soa_t;
```

### Benefits

- Prevents false sharing
- Optimizes cache line usage
- Enables SIMD operations
- Reduces cache misses

## Access Patterns

### Sequential Access

SoA layout enables sequential access:

```c
// Access all subjects sequentially
for (int i = 0; i < 8; i++) {
    process(soa->s[i]);
}
```

### Prefetch

Prefetch next cache line:

```c
__builtin_prefetch(&soa->s[8], 1, 3);  // Prefetch for write
```

## Related Documentation

- [Hot Path Implementation](hot-path.md) - Hot path operations
- [SIMD Optimization](simd.md) - SIMD implementation
- [SoA Layout](../ring-buffers/soa-layout.md) - Memory layout
