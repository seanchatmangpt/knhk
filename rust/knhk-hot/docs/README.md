# knhk-hot Documentation

Hot path operations (≤2ns / ≤8 ticks) using SIMD-optimized C code.

## Overview

The `knhk-hot` crate provides Rust FFI bindings for the C hot path:
- Boolean query evaluation (`knhk_eval_bool`)
- CONSTRUCT8 operations (`knhk_eval_construct8`)
- Batch execution (`knhk_eval_batch8`)
- Receipt generation and merging

## Architecture

- **Engine**: Main Rust wrapper around C hot path
- **Types**: FFI-safe types (Ctx, Ir, Receipt, Run, Op)
- **SIMD Operations**: Leverages C SIMD intrinsics for ≤8 tick execution

## Performance

- **Target**: ≤8 ticks (≤2ns)
- **Current**: Most operations ~1-1.5ns
- **Constraints**: max_run_len ≤ 8 (Chatman Constant)

## Key Features

- **SoA Layout**: Structure-of-Arrays with 64-byte alignment
- **Branchless**: Constant-time execution on hot path
- **SIMD**: Optimized with NEON/AVX2 intrinsics
- **Guard Validation**: Enforces run.len ≤ 8 constraint

## Usage

```rust
use knhk_hot::{Engine, Ir, Op, Receipt, Run};

let engine = Engine::new(s_array.as_ptr(), p_array.as_ptr(), o_array.as_ptr());
engine.pin_run(run)?;

let mut ir = Ir { op: Op::AskSp, s, p, o: 0, k: 0, ..Default::default() };
let mut receipt = Receipt::default();
let result = engine.eval_bool(&mut ir, &mut receipt);
```

## Related Documentation

- [Architecture](../../../docs/architecture.md) - System architecture
- [Performance](../../../docs/performance.md) - Performance guide
- [C Hot Path](../../../c/include/knhk/eval.h) - C header documentation

