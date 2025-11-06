# knhk-hot

FFI-safe Rust wrapper for C hot path operations (≤2ns / ≤8 ticks).

## Overview

`knhk-hot` provides safe Rust abstractions over the C hot path implementation, enabling SIMD-optimized operations that execute in ≤8 ticks (≤2ns). The crate wraps C functions with FFI-safe types and enforces guard constraints at compile time.

## Quick Start

```rust
use knhk_hot::{Engine, Ir, Op, Receipt, Run};

// Initialize engine with SoA arrays (64-byte aligned)
let s_array = [0u64; 8];
let p_array = [0u64; 8];
let o_array = [0u64; 8];

let engine = Engine::new(
    s_array.as_ptr(),
    p_array.as_ptr(),
    o_array.as_ptr(),
);

// Pin a predicate run (max_run_len ≤ 8)
let run = Run {
    pred: 0xC0FFEE,
    off: 0,
    len: 1,
};
engine.pin_run(run)?;

// Execute ASK_SP operation
let mut ir = Ir {
    op: Op::AskSp,
    s: 0xA11CE,
    p: 0xC0FFEE,
    o: 0,
    k: 0,
    ..Default::default()
};

let mut receipt = Receipt::default();
let result = engine.eval_bool(&mut ir, &mut receipt)?;

// Verify receipt
assert!(receipt.ticks <= 8);
assert_eq!(receipt.lanes, 8);
```

## Key Features

- **FFI-Safe Wrappers**: Safe Rust abstractions over C functions
- **Performance**: ≤8 ticks (≤2ns) execution time
- **SoA Layout**: Structure-of-Arrays with 64-byte alignment for SIMD
- **Branchless Operations**: Constant-time execution on hot path
- **Guard Validation**: Enforces run.len ≤ 8 constraint
- **Receipt Generation**: Provenance tracking with span IDs

## Supported Operations

- **ASK_SP** - Existence checks
- **COUNT_SP_GE** - Cardinality checks (≥k)
- **COUNT_SP_EQ** - Cardinality checks (=k)
- **UNIQUE** - Uniqueness validation
- **COMPARE** - Value comparisons (EQ, NE, LT, LE, GT, GE)
- **CONSTRUCT8** - Triple construction (max 8 triples)

## Dependencies

- C library: `libknhk.a` (linked via build.rs)
- No external Rust dependencies (pure FFI wrapper)

## Performance

- **Target**: ≤8 ticks (≤2ns at ~250 ps/tick)
- **Current**: Most operations ~1-1.5ns
- **Constraints**: max_run_len ≤ 8 (Chatman Constant)
- **SIMD**: Optimized with NEON/AVX2 intrinsics

## Types

- **Run**: Predicate run metadata (pred, off, len ≤ 8)
- **Ctx**: Context with SoA arrays (S, P, O pointers)
- **Ir**: Hook IR representation (op, s, p, o, k)
- **Receipt**: Provenance receipt (ticks, lanes, span_id, a_hash)
- **Op**: Operation enum (AskSp, CountSpGe, Construct8, etc.)

## Related Documentation

- [Technical Documentation](docs/README.md) - Detailed API reference
- [Architecture](../../docs/architecture.md) - System architecture
- [Performance](../../docs/performance.md) - Performance guide
- [C Hot Path](../../c/docs/README.md) - C implementation details

