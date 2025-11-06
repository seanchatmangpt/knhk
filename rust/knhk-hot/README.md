# knhk-hot

Hot path operations (≤2ns / ≤8 ticks) using SIMD-optimized C code.

## Overview

`knhk-hot` provides FFI-safe Rust wrappers around the C hot path implementation. It enables sub-nanosecond query execution with SIMD optimizations, branchless operations, and SoA (Structure-of-Arrays) layout.

## Quick Start

```rust
use knhk_hot::{Engine, Ir, Op, Receipt, Run};

// Create SoA arrays (64-byte aligned, length = 8)
let s_array = [0u64; 8];
let p_array = [0u64; 8];
let o_array = [0u64; 8];

// Create engine with SoA arrays
let mut engine = Engine::new(
    s_array.as_ptr(),
    p_array.as_ptr(),
    o_array.as_ptr(),
);

// Pin a predicate run (max_run_len ≤ 8)
let run = Run {
    pred: 0xC0FFEE,
    off: 0,
    len: 1,  // Must be ≤ 8
};

// Pin run (returns Result, but error is static str)
match engine.pin_run(run) {
    Ok(()) => {},
    Err(e) => panic!("Run pin failed: {}", e),
}

// Execute ASK_SP query
let mut ir = Ir {
    op: Op::AskSp,
    s: 0xA11CE,
    p: 0xC0FFEE,
    o: 0,
    k: 0,
    out_S: std::ptr::null_mut(),
    out_P: std::ptr::null_mut(),
    out_O: std::ptr::null_mut(),
    out_mask: 0,
};

let mut receipt = Receipt::default();
let result = engine.eval_bool(&mut ir, &mut receipt);  // Returns bool, not Result

// Verify receipt
assert!(receipt.ticks <= 8);
assert_eq!(receipt.lanes, 8);
```

## Key Features

- **Performance**: ≤8 ticks (≤2ns) execution target
- **SIMD Optimized**: NEON/AVX2 intrinsics
- **Branchless**: Constant-time execution
- **SoA Layout**: 64-byte aligned arrays
- **Guard Validation**: Enforces run.len ≤ 8

## Documentation

For detailed documentation, see [docs/README.md](docs/README.md).

## Related Documentation

- [Architecture](../../docs/architecture.md) - System architecture
- [Performance](../../docs/performance.md) - Performance guide
- [C Hot Path](../../c/docs/README.md) - C implementation details
