# Kernel Operations

Hot path kernel operations for ASK, COUNT, COMPARE, VALIDATE, SELECT, UNIQUE.

## Overview

Kernels are branchless operations optimized for hot path:
- Constant-time execution
- SIMD-optimized where possible
- Zero-copy operations
- Cache-friendly access patterns

## ASK Kernel

```c
knhk_result_t knhk_eval_ask(knhk_soa_t *soa, knhk_query_t *query) {
    // Branchless ASK evaluation
    // Returns boolean result
    uint64_t matches = 0;
    for (int i = 0; i < 8; i++) {
        matches |= (soa->p[i] == query->predicate);
    }
    return (matches > 0) ? KNHK_RESULT_TRUE : KNHK_RESULT_FALSE;
}
```

## COUNT Kernel

```c
uint64_t knhk_eval_count(knhk_soa_t *soa, knhk_query_t *query) {
    // Branchless COUNT evaluation
    uint64_t count = 0;
    for (int i = 0; i < 8; i++) {
        count += (soa->p[i] == query->predicate);
    }
    return count;
}
```

## COMPARE Kernel

```c
int knhk_eval_compare(knhk_soa_t *soa, knhk_query_t *query) {
    // Branchless COMPARE evaluation
    // Returns comparison result
}
```

## Related Documentation

- [Hot Path](../hot-path.md) - Overview
- [Eval Dispatch](eval-dispatch.md) - Kernel dispatch
- [Performance Targets](performance.md) - Performance goals
