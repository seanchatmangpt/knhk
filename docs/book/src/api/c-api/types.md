# C API Types and Structures

Complete reference for C API types and structures.

## Core Types

### knhk_soa_t

```c
typedef struct {
    uint64_t s[8];  // Subjects
    uint64_t p[8];  // Predicates
    uint64_t o[8];  // Objects
} knhk_soa_t;
```

### knhk_delta_t

```c
typedef struct {
    uint64_t cycle_id;
    knhk_soa_t soa;
} knhk_delta_t;
```

### knhk_assertion_t

```c
typedef struct {
    uint64_t cycle_id;
    knhk_receipt_t receipt;
    knhk_soa_t soa;
} knhk_assertion_t;
```

### knhk_fiber_t

```c
typedef struct {
    uint64_t shard_id;
    knhk_fiber_state_t state;
    // ... other fields
} knhk_fiber_t;
```

## Constants

### KNHK_TICK_BUDGET

```c
#define KNHK_TICK_BUDGET 8u
```

Maximum ticks per operation (Chatman Constant).

## Header File

`c/include/knhk/types.h`

## Related Documentation

- [C API](../c-api.md) - Overview
- [SoA Layout](../../architecture/ring-buffers/soa-layout.md) - Memory layout
