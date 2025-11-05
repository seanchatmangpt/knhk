# C Hot Path Documentation

C hot path implementation for ≤2ns query execution.

## Overview

The C hot path provides SIMD-optimized operations:
- Boolean query evaluation (`knhk_eval_bool`)
- CONSTRUCT8 operations (`knhk_eval_construct8`)
- Batch execution (`knhk_eval_batch8`)
- SIMD intrinsics (NEON/AVX2)

## Architecture

- **SoA Layout**: Structure-of-Arrays with 64-byte alignment
- **SIMD Operations**: Optimized with SIMD intrinsics
- **Branchless**: Constant-time execution
- **Unrolled**: Fully unrolled for NROWS=8

## Performance

- **Target**: ≤8 ticks (≤2ns)
- **Current**: Most operations ~1-1.5ns
- **Measurement**: External timing by Rust framework

## Key Files

- `include/knhk/eval.h` - Query evaluation functions
- `include/knhk/types.h` - Type definitions
- `include/knhk/simd.h` - SIMD operations
- `src/simd.c` - SIMD implementations

## Related Documentation

- [Architecture](../../docs/architecture.md) - System architecture
- [Performance](../../docs/performance.md) - Performance guide
- [knhk-hot Rust Bindings](../rust/knhk-hot/docs/README.md) - Rust wrapper

