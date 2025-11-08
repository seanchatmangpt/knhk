# SoA Layout

Structure of Arrays (SoA) layout for SIMD optimization.

## Overview

SoA (Structure of Arrays) layout stores data in separate arrays:
- **Subjects**: Array of subject values
- **Predicates**: Array of predicate values
- **Objects**: Array of object values

## Structure

```c
typedef struct {
    uint64_t s[8];  // Subjects
    uint64_t p[8];  // Predicates
    uint64_t o[8];  // Objects
} knhk_soa_t;
```

## Benefits

### SIMD Optimization

SoA enables vectorized operations:

```c
// Load 4 predicates at once
__m256i predicates = _mm256_load_si256((__m256i*)soa->p);
```

### Cache Optimization

Sequential access patterns:

```c
// Access all subjects sequentially
for (int i = 0; i < 8; i++) {
    process(soa->s[i]);
}
```

### Memory Alignment

64-byte alignment for cache lines:

```c
alignas(64) uint64_t s[8];
```

## Conversion

### RawTriple to SoA

```rust
let soa = ring_conversion::triples_to_soa(triples)?;
```

### SoA to RawTriple

```rust
let triples = ring_conversion::soa_to_triples(&soa)?;
```

## Related Documentation

- [Ring Buffers](../ring-buffers.md) - Overview
- [Delta Ring](delta-ring.md) - Input buffer
- [Assertion Ring](assertion-ring.md) - Output buffer
