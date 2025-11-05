# C Hot Path Documentation

C hot path implementation for ≤2ns query execution.

## File Structure

```
c/
├── include/
│   ├── knhk.h              # Main umbrella header (includes all)
│   └── knhk/
│       ├── types.h         # Type definitions (enums, structs, constants)
│       ├── eval.h          # Query evaluation functions (eval_bool, eval_construct8)
│       ├── receipts.h      # Receipt operations (receipt_merge)
│       ├── utils.h         # Utility functions (init_ctx, load_rdf, clock)
│       ├── warm_path.h    # Warm path operations
│       └── unrdf.h         # unrdf FFI declarations
├── src/
│   ├── simd.c              # SIMD implementations (NEON/AVX2)
│   ├── simd/
│   │   ├── common.h       # Common SIMD infrastructure
│   │   ├── existence.h    # ASK operations (exists_8, spo_exists_8)
│   │   ├── count.h        # COUNT operations (count_8)
│   │   ├── compare.h      # Comparison operations (compare_o_8)
│   │   ├── construct.h    # CONSTRUCT8 operations (construct8_emit_8)
│   │   └── validate.h     # Datatype validation (validate_datatype_sp_8)
│   ├── core.c             # Core operations (batch execution)
│   ├── rdf.c              # RDF parsing (Turtle format)
│   ├── clock.c            # Timing utilities and span ID generation
│   └── warm_path.c        # Warm path operations
└── Makefile
```

## Core Components

### Evaluation Functions (`include/knhk/eval.h`)
- `knhk_eval_bool()` - Boolean query evaluation (ASK, COUNT)
- `knhk_eval_construct8()` - CONSTRUCT8 operations
- `knhk_eval_batch8()` - Batch execution (≤8 hooks)
- All functions are `static inline` for hot path performance

### SIMD Operations (`src/simd.c`, `src/simd/*.h`)
- **Existence**: `knhk_eq64_exists_8` - Check if value exists
- **Count**: `knhk_eq64_count_8` - Count matching values
- **Compare**: `knhk_compare_o_8` - Compare objects
- **Construct**: `knhk_construct8_emit_8` - Emit CONSTRUCT results
- **Validate**: `knhk_validate_datatype_sp_8` - Datatype validation

### Types (`include/knhk/types.h`)
- `knhk_context_t` - Context with SoA arrays
- `knhk_hook_ir_t` - Hook IR representation
- `knhk_receipt_t` - Provenance receipt
- `knhk_pred_run_t` - Predicate run metadata (len ≤ 8)

## Key Features

- **SoA Layout**: Structure-of-Arrays with 64-byte alignment
- **SIMD Optimized**: NEON (ARM) and AVX2 (x86) intrinsics
- **Branchless**: Constant-time execution on hot path
- **Unrolled**: Fully unrolled for NROWS=8
- **Hot Path**: ≤8 ticks (≤2ns) target

## Performance

- **Target**: ≤8 ticks (≤2ns)
- **Current**: Most operations ~1-1.5ns
- **Measurement**: External timing by Rust framework

## Related Documentation

- [Architecture](../../docs/architecture.md) - System architecture
- [Performance](../../docs/performance.md) - Performance guide
- [knhk-hot Rust Bindings](../rust/knhk-hot/docs/README.md) - Rust wrapper

