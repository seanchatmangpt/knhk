# SIMD Optimization

SIMD (Single Instruction, Multiple Data) optimization for vectorized operations.

## Overview

KNHK uses SIMD intrinsics for:
- Parallel triple matching
- Vectorized comparisons
- Batch operations
- Performance-critical paths

## SIMD Support

### NEON (ARM)

```c
#include <arm_neon.h>

// Load 4 triples at once
uint64x2_t subjects = vld1q_u64(soa->s);
uint64x2_t predicates = vld1q_u64(soa->p);
```

### AVX2 (x86_64)

```c
#include <immintrin.h>

// Load 4 triples at once
__m256i subjects = _mm256_load_si256((__m256i*)soa->s);
__m256i predicates = _mm256_load_si256((__m256i*)soa->p);
```

## SoA Layout

Structure of Arrays (SoA) enables SIMD:

```c
typedef struct {
    uint64_t s[8];  // Subjects (aligned to 64 bytes)
    uint64_t p[8];  // Predicates
    uint64_t o[8];  // Objects
} knhk_soa_t;
```

## Vectorized Operations

### Parallel Matching

```c
// Compare 4 predicates at once
__m256i mask = _mm256_cmpeq_epi64(predicates, target);
int matches = _mm256_movemask_pd(_mm256_castsi256_pd(mask));
```

## Related Documentation

- [Hot Path Implementation](hot-path.md) - Hot path operations
- [Cache Optimization](cache.md) - Cache optimization
- [SoA Layout](../ring-buffers/soa-layout.md) - Memory layout
