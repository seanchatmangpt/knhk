# Guard Constraints

## Overview

KNHK enforces guard constraints to ensure ≤8 tick performance.

## Constraints

### max_run_len ≤ 8

Predicate run length must not exceed 8 elements.

**Enforced at**:
- Cover definition
- Reflex declaration
- Load stage (predicate run grouping)

**Validation**:
```c
if (run.len > 8) {
    return ERROR_GUARD_VIOLATION;
}
```

### τ ≤ 8

Epoch tick budget must not exceed 8 ticks.

**Enforced at**:
- Epoch creation
- Epoch execution

**Validation**:
```c
if (tau > 8) {
    return ERROR_GUARD_VIOLATION;
}
```

### Operation Validation

Reflex operations must be in H_hot set.

**Valid Operations**:
- ASK_SP, COUNT_SP_GE, COUNT_SP_LE, COUNT_SP_EQ
- ASK_SPO, ASK_OP, UNIQUE_SP
- COUNT_OP_GE, COUNT_OP_LE, COUNT_OP_EQ
- COMPARE_O_EQ, COMPARE_O_GT, COMPARE_O_LT, COMPARE_O_GE, COMPARE_O_LE
- CONSTRUCT8

## AOT Compilation Guard

Ahead-Of-Time validation ensures IR operations meet constraints:

```c
bool knhk_aot_validate_ir(knhk_op_t op, size_t run_len, size_t k);
bool knhk_aot_validate_run(knhk_pred_run_t run);
```

## Runtime Enforcement

All guard constraints are enforced at runtime:
- CLI commands validate inputs
- ETL pipeline stages validate data
- Hot path operations validate runs

## Error Handling

Guard violations return errors:
- `ERROR_GUARD_VIOLATION`: Constraint violated
- Descriptive error messages with context

