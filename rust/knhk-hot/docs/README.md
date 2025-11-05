# knhk-hot Documentation

Hot path operations (≤2ns / ≤8 ticks) using SIMD-optimized C code.

## File Structure

```
rust/knhk-hot/
├── src/
│   ├── lib.rs              # Module exports
│   └── ffi.rs              # FFI-safe Rust wrappers around C hot path
├── build.rs                # Link to C library (libknhk.a)
└── Cargo.toml
```

## Core Components

### FFI Module (`src/ffi.rs`)
- **Engine**: Main Rust wrapper around C hot path
- **Types**: FFI-safe types (Ctx, Ir, Receipt, Run, Op)
- **Functions**: Safe wrappers for C functions
  - `eval_bool()` - Boolean query evaluation
  - `eval_construct8()` - CONSTRUCT8 operations
  - `eval_batch8()` - Batch execution

### Types
- **Run**: Predicate run metadata (pred, off, len ≤ 8)
- **Ctx**: Context with SoA arrays (S, P, O pointers)
- **Ir**: Hook IR representation (op, s, p, o, k)
- **Receipt**: Provenance receipt (ticks, lanes, span_id, a_hash)
- **Op**: Operation enum (AskSp, CountSpGe, Construct8, etc.)

## Dependencies

- C library: `libknhk.a` (linked via build.rs)
- No external Rust dependencies (pure FFI wrapper)

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
- [C Hot Path](../../../c/docs/README.md) - C implementation details

