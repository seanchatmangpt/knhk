# Eval Dispatch

Eval dispatch provides branchless kernel dispatch for hot path operations.

## Overview

Eval dispatch routes operations to appropriate kernels:
- ASK operations
- COUNT operations
- COMPARE operations
- VALIDATE operations
- SELECT operations
- UNIQUE operations

## Kernel Dispatch

### Operation Routing

```c
knhk_result_t knhk_eval_dispatch(knhk_operation_t op, knhk_soa_t *soa, knhk_query_t *query) {
    switch (op) {
        case KNHK_OP_ASK: return knhk_eval_ask(soa, query);
        case KNHK_OP_COUNT: return knhk_eval_count(soa, query);
        case KNHK_OP_COMPARE: return knhk_eval_compare(soa, query);
        // ... other operations
    }
}
```

### Branchless Dispatch

For hot path, use function pointer table:

```c
typedef knhk_result_t (*knhk_kernel_fn)(knhk_soa_t *, knhk_query_t *);

static const knhk_kernel_fn kernels[] = {
    [KNHK_OP_ASK] = knhk_eval_ask,
    [KNHK_OP_COUNT] = knhk_eval_count,
    // ... other kernels
};

knhk_result_t knhk_eval_dispatch(knhk_operation_t op, knhk_soa_t *soa, knhk_query_t *query) {
    return kernels[op](soa, query);
}
```

## Related Documentation

- [Hot Path](../hot-path.md) - Overview
- [Kernel Operations](kernels.md) - Individual kernels
- [Performance Targets](performance.md) - Performance goals
