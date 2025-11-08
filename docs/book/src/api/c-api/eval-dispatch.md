# Eval Dispatch C API

Complete C API reference for eval dispatch.

## Functions

### knhk_eval_ask

```c
knhk_result_t knhk_eval_ask(knhk_soa_t *soa, knhk_query_t *query);
```

Branchless ASK operation.

**Parameters**:
- `soa`: SoA structure
- `query`: Query structure

**Returns**: Result (TRUE/FALSE)

### knhk_eval_count

```c
uint64_t knhk_eval_count(knhk_soa_t *soa, knhk_query_t *query);
```

Branchless COUNT operation.

**Parameters**:
- `soa`: SoA structure
- `query`: Query structure

**Returns**: Count of matching triples

### knhk_eval_compare

```c
int knhk_eval_compare(knhk_soa_t *soa, knhk_query_t *query);
```

Branchless COMPARE operation.

**Parameters**:
- `soa`: SoA structure
- `query`: Query structure

**Returns**: Comparison result

## Header File

`c/include/knhk/eval_dispatch.h`

## Related Documentation

- [C API](../c-api.md) - Overview
- [Eval Dispatch](../../architecture/hot-path/eval-dispatch.md) - Architecture
