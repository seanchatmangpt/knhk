# C API Reference

## Types

### knhk_op_t
Operation types for queries:
```c
typedef enum {
  KNHK_OP_ASK_SP = 1,          // ASK existence check
  KNHK_OP_COUNT_SP_GE = 2,     // COUNT >= k
  KNHK_OP_ASK_SPO = 3,         // Triple matching
  KNHK_OP_COUNT_SP_LE = 5,     // COUNT <= k
  KNHK_OP_COUNT_SP_EQ = 6,     // COUNT == k
  KNHK_OP_ASK_OP = 7,          // ASK(O,P) - reverse lookup
  KNHK_OP_UNIQUE_SP = 8,       // UNIQUE(S,P) - exactly one value
  KNHK_OP_COUNT_OP_GE = 9,     // COUNT(O,P) >= k
  KNHK_OP_COUNT_OP_LE = 10,    // COUNT(O,P) <= k
  KNHK_OP_COUNT_OP_EQ = 11,    // COUNT(O,P) == k
  KNHK_OP_COMPARE_O_EQ = 12,   // O == value (exact match)
  KNHK_OP_COMPARE_O_GT = 13,   // O > value (greater than)
  KNHK_OP_COMPARE_O_LT = 14,   // O < value (less than)
  KNHK_OP_COMPARE_O_GE = 15,   // O >= value (greater or equal)
  KNHK_OP_COMPARE_O_LE = 16,   // O <= value (less or equal)
  KNHK_OP_VALIDATE_DATATYPE_SP = 17,  // Validate datatype
  KNHK_OP_VALIDATE_DATATYPE_SPO = 18, // Validate datatype
  KNHK_OP_CONSTRUCT8 = 32      // CONSTRUCT8 - fixed-template emit
} knhk_op_t;
```

### knhk_context_t
Context holding SoA arrays and metadata:
```c
typedef struct {
  const uint64_t *S;        // Subject array (64-byte aligned)
  const uint64_t *P;        // Predicate array
  const uint64_t *O;        // Object array
  size_t triple_count;      // Number of loaded triples
  knhk_pred_run_t run;     // Predicate run metadata
} knhk_context_t;
```

### knhk_hook_ir_t
Query representation (Hook IR):
```c
typedef struct {
  knhk_op_t op;            // Operation type
  uint64_t s, p, o, k;      // Subject, predicate, object, threshold
  uint64_t *out_S;          // CONSTRUCT8 output buffers (may be NULL)
  uint64_t *out_P;
  uint64_t *out_O;
  uint64_t out_mask;        // per-lane bitmask result
} knhk_hook_ir_t;
```

### knhk_receipt_t
Receipt structure for timing and provenance:
```c
typedef struct {
  uint32_t ticks;    // ≤ 8 (Chatman Constant)
  uint32_t lanes;    // SIMD width used
  uint64_t span_id;  // OTEL-compatible id
  uint64_t a_hash;   // hash(A) = hash(μ(O)) fragment
} knhk_receipt_t;
```

## Functions

### Context Management

#### knhk_init_ctx
Initialize context with arrays:
```c
void knhk_init_ctx(knhk_context_t *ctx, const uint64_t *S, const uint64_t *P, const uint64_t *O);
```

#### knhk_pin_run
Set the active predicate run:
```c
static inline void knhk_pin_run(knhk_context_t *ctx, knhk_pred_run_t run);
```
**Guard**: `run.len` must be ≤8

### Query Evaluation

#### knhk_eval_bool
Evaluate boolean query (inline, hot path):
```c
static inline int knhk_eval_bool(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt);
```
- Returns: 1 if true, 0 if false
- All operations ≤8 ticks
- Supports all boolean operations in H_hot set

#### knhk_eval_construct8
Emit up to 8 triples using a fixed template:
```c
static inline int knhk_eval_construct8(const knhk_context_t *ctx, knhk_hook_ir_t *ir, knhk_receipt_t *rcpt);
```
- Returns: Number of lanes written
- Hot path operation: ≤8 ticks

### Receipt Operations

#### knhk_receipt_merge
Merge receipts (⊕):
```c
void knhk_receipt_merge(const knhk_receipt_t *rcpt1, const knhk_receipt_t *rcpt2, knhk_receipt_t *merged);
```

### Utilities

#### knhk_generate_span_id
Generate OTEL-compatible span ID:
```c
uint64_t knhk_generate_span_id(void);
```

## See Also

- [Rust API](rust-api.md) - Rust API reference
- [Erlang API](erlang-api.md) - Erlang API reference

