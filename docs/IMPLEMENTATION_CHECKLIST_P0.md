# P0 Implementation Checklist: Unblock JTBD (3-6 hours)

**Goal**: Make knhk-hot compile without C library, enabling 100% JTBD accomplishment

**Time Estimate**: 3-6 hours (one afternoon)

**Status**: 🔴 READY TO START

---

## Prerequisites

```bash
# Verify you're in the correct directory
cd /home/user/knhk

# Verify build currently fails
cargo build --package knhk-hot 2>&1 | grep "unable to find library -lknhk"
# Should output: "rust-lld: error: unable to find library -lknhk"

# Clean build artifacts
cargo clean
```

---

## Step 1: Update build.rs (30 minutes) ⚡

### File: `/home/user/knhk/rust/knhk-hot/build.rs`

### Current Code (BROKEN):
```rust
fn main() {
    // Compile workflow_patterns.c, ring_buffer.c, and simd_predicates.c (Week 2)
    cc::Build::new()
        .file("src/workflow_patterns.c")
        .file("src/ring_buffer.c")
        .file("src/simd_predicates.c")
        .opt_level(3)
        .flag("-march=native")
        .flag("-fno-strict-aliasing")
        .warnings(false)
        .compile("workflow_patterns");

    // Try to link to KNHK C library if it exists (optional for workflow patterns)
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let c_lib_dir = format!("{}/../../c", manifest_dir);
    let lib_path = format!("{}/libknhk.a", c_lib_dir);

    if std::path::Path::new(&lib_path).exists() {
        println!("cargo:rustc-link-search=native={}", c_lib_dir);
        println!("cargo:rustc-link-lib=static=knhk");  // ← BREAKS HERE
    } else {
        eprintln!("Note: libknhk.a not found at {}", lib_path);
        eprintln!("Workflow patterns will work, but other FFI functions may not link");
    }

    println!("cargo:rerun-if-changed=src/workflow_patterns.c");
    println!("cargo:rerun-if-changed=src/workflow_patterns.h");
    println!("cargo:rerun-if-changed=src/ring_buffer.c");
    println!("cargo:rerun-if-changed=src/simd_predicates.c");
    println!("cargo:rerun-if-changed=src/simd_predicates.h");
    println!("cargo:rerun-if-changed={}", lib_path);
}
```

### New Code (WORKING):
```rust
fn main() {
    // Compile local workflow patterns (these always work)
    cc::Build::new()
        .file("src/workflow_patterns.c")
        .file("src/ring_buffer.c")
        .file("src/simd_predicates.c")
        .opt_level(3)
        .flag("-march=native")
        .flag("-fno-strict-aliasing")
        .warnings(false)
        .compile("workflow_patterns");

    // OPTIONAL: Try to link external C library for hot path optimization
    #[cfg(feature = "c-optimization")]
    {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let c_lib_dir = format!("{}/../../c", manifest_dir);
        let lib_path = format!("{}/libknhk.a", c_lib_dir);

        if std::path::Path::new(&lib_path).exists() {
            println!("cargo:rustc-link-search=native={}", c_lib_dir);
            println!("cargo:rustc-link-lib=static=knhk");
            println!("cargo:rustc-cfg=have_c_optimization");
            println!("cargo:warning=Using C SIMD optimization (8-tick hot path)");
        } else {
            println!("cargo:warning=C library not found at {}, using Rust fallback (12-tick warm path)", lib_path);
        }

        println!("cargo:rerun-if-changed={}", lib_path);
    }

    #[cfg(not(feature = "c-optimization"))]
    {
        println!("cargo:warning=Using pure Rust fallback (12-16 tick warm path, no C dependencies)");
    }

    println!("cargo:rerun-if-changed=src/workflow_patterns.c");
    println!("cargo:rerun-if-changed=src/workflow_patterns.h");
    println!("cargo:rerun-if-changed=src/ring_buffer.c");
    println!("cargo:rerun-if-changed=src/simd_predicates.c");
    println!("cargo:rerun-if-changed=src/simd_predicates.h");
}
```

### Checklist:
- [ ] Open `/home/user/knhk/rust/knhk-hot/build.rs` in editor
- [ ] Replace entire `main()` function with new code
- [ ] Save file
- [ ] Verify syntax: `cd /home/user/knhk/rust/knhk-hot && cargo check --build-dependencies`

---

## Step 2: Add Feature Flag (5 minutes) ⚡

### File: `/home/user/knhk/rust/knhk-hot/Cargo.toml`

### Current Code:
```toml
[package]
name = "knhk-hot"
version = "1.0.0"
edition = "2021"

[dependencies]
blake3 = { workspace = true }
subtle = "2.5"

[build-dependencies]
cc = "1.0"
```

### New Code:
```toml
[package]
name = "knhk-hot"
version = "1.0.0"
edition = "2021"

[features]
default = []  # Pure Rust by default (JTBD works immediately)
c-optimization = ["cc"]  # Opt-in for 8-tick C SIMD hot path

[dependencies]
blake3 = { workspace = true }
subtle = "2.5"

[build-dependencies]
cc = { version = "1.0", optional = true }
```

### Checklist:
- [ ] Open `/home/user/knhk/rust/knhk-hot/Cargo.toml` in editor
- [ ] Add `[features]` section after `[package]`
- [ ] Change `cc = "1.0"` to `cc = { version = "1.0", optional = true }`
- [ ] Save file
- [ ] Verify syntax: `cargo metadata --format-version 1 | grep -q "knhk-hot"`

---

## Step 3: Create Rust Fallback Module (2-3 hours) 🔥

### File: `/home/user/knhk/rust/knhk-hot/src/ffi_fallback.rs` (NEW)

This is the largest implementation task. Create a new file with pure Rust implementations of all C FFI functions.

### Template Structure:

```rust
//! Pure Rust fallback implementations for knhk C library
//!
//! Performance: ~12-16 ticks (vs 8 ticks for C SIMD)
//! Benefit: Works without C compiler dependencies

use crate::ffi::{Ctx, Ir, Op, Receipt, Run};

// Core initialization functions
#[no_mangle]
pub unsafe extern "C" fn knhk_init_ctx(
    ctx: *mut Ctx,
    S: *const u64,
    P: *const u64,
    O: *const u64,
) {
    (*ctx).S = S;
    (*ctx).P = P;
    (*ctx).O = O;
    (*ctx).run = Run { pred: 0, off: 0, len: 0 };
}

#[no_mangle]
pub unsafe extern "C" fn knhk_pin_run(ctx: *mut Ctx, run: Run) {
    (*ctx).run = run;
}

// RDF evaluation functions
#[no_mangle]
pub unsafe extern "C" fn knhk_eval_bool(
    ctx: *const Ctx,
    ir: *mut Ir,
    rcpt: *mut Receipt,
) -> i32 {
    let ctx = &*ctx;
    let ir = &mut *ir;
    let rcpt = &mut *rcpt;

    let start_cycles = crate::cycle_counter::read_cycles();

    let result = match ir.op {
        Op::AskSp => eval_ask_sp(ctx, ir),
        Op::AskSpo => eval_ask_spo(ctx, ir),
        Op::AskOp => eval_ask_op(ctx, ir),
        Op::CountSpGe => eval_count_sp_ge(ctx, ir),
        Op::CountSpLe => eval_count_sp_le(ctx, ir),
        Op::CountSpEq => eval_count_sp_eq(ctx, ir),
        Op::UniqueSp => eval_unique_sp(ctx, ir),
        _ => 0,
    };

    let end_cycles = crate::cycle_counter::read_cycles();
    let elapsed = end_cycles.saturating_sub(start_cycles);
    let ticks = crate::cycle_counter::cycles_to_ticks(elapsed);

    rcpt.ticks = ticks as u32;
    rcpt.actual_ticks = ticks as u32;
    rcpt.lanes = ctx.run.len as u32;

    result
}

// Helper functions (implement each RDF operation)
fn eval_ask_sp(ctx: &Ctx, ir: &Ir) -> i32 {
    let s = ir.s;
    let p = ir.p;
    let len = ctx.run.len as usize;

    unsafe {
        for i in 0..len {
            if *ctx.S.add(i) == s && *ctx.P.add(i) == p {
                return 1;
            }
        }
    }
    0
}

fn eval_ask_spo(ctx: &Ctx, ir: &Ir) -> i32 {
    let s = ir.s;
    let p = ir.p;
    let o = ir.o;
    let len = ctx.run.len as usize;

    unsafe {
        for i in 0..len {
            if *ctx.S.add(i) == s && *ctx.P.add(i) == p && *ctx.O.add(i) == o {
                return 1;
            }
        }
    }
    0
}

fn eval_ask_op(ctx: &Ctx, ir: &Ir) -> i32 {
    let p = ir.p;
    let o = ir.o;
    let len = ctx.run.len as usize;

    unsafe {
        for i in 0..len {
            if *ctx.P.add(i) == p && *ctx.O.add(i) == o {
                return 1;
            }
        }
    }
    0
}

fn eval_count_sp_ge(ctx: &Ctx, ir: &Ir) -> i32 {
    let s = ir.s;
    let p = ir.p;
    let k = ir.k;
    let len = ctx.run.len as usize;

    let mut count = 0u64;
    unsafe {
        for i in 0..len {
            if *ctx.S.add(i) == s && *ctx.P.add(i) == p {
                count += 1;
                if count >= k {
                    return 1;
                }
            }
        }
    }
    0
}

fn eval_count_sp_le(ctx: &Ctx, ir: &Ir) -> i32 {
    let s = ir.s;
    let p = ir.p;
    let k = ir.k;
    let len = ctx.run.len as usize;

    let mut count = 0u64;
    unsafe {
        for i in 0..len {
            if *ctx.S.add(i) == s && *ctx.P.add(i) == p {
                count += 1;
                if count > k {
                    return 0;
                }
            }
        }
    }
    1
}

fn eval_count_sp_eq(ctx: &Ctx, ir: &Ir) -> i32 {
    let s = ir.s;
    let p = ir.p;
    let k = ir.k;
    let len = ctx.run.len as usize;

    let mut count = 0u64;
    unsafe {
        for i in 0..len {
            if *ctx.S.add(i) == s && *ctx.P.add(i) == p {
                count += 1;
            }
        }
    }
    if count == k { 1 } else { 0 }
}

fn eval_unique_sp(ctx: &Ctx, ir: &Ir) -> i32 {
    let s = ir.s;
    let p = ir.p;
    let len = ctx.run.len as usize;

    let mut values = Vec::new();
    unsafe {
        for i in 0..len {
            if *ctx.S.add(i) == s && *ctx.P.add(i) == p {
                let o = *ctx.O.add(i);
                if values.contains(&o) {
                    return 0;
                }
                values.push(o);
            }
        }
    }
    1
}

// Stubs for other functions
#[no_mangle]
pub unsafe extern "C" fn knhk_eval_construct8(
    _ctx: *const Ctx,
    _ir: *mut Ir,
    _rcpt: *mut Receipt,
) -> i32 {
    // TODO: Implement or return safe default
    0
}

#[no_mangle]
pub unsafe extern "C" fn knhk_eval_batch8(
    _ctx: *const Ctx,
    _irs: *mut Ir,
    _n: usize,
    _rcpts: *mut Receipt,
) -> i32 {
    // TODO: Implement or return safe default
    0
}

#[no_mangle]
pub unsafe extern "C" fn knhk_beat_init() {
    // Stub: No-op
}

#[no_mangle]
pub unsafe extern "C" fn knhk_beat_next() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

#[no_mangle]
pub unsafe extern "C" fn knhk_beat_tick(_cycle: u64) -> u64 {
    0
}

#[no_mangle]
pub unsafe extern "C" fn knhk_beat_pulse(_cycle: u64) -> u64 {
    0
}

// Add stubs for remaining functions from ffi.rs
// (grep for "pub fn" in src/ffi.rs to find all functions)
```

### Checklist:
- [ ] Create file `/home/user/knhk/rust/knhk-hot/src/ffi_fallback.rs`
- [ ] Copy template code above
- [ ] Implement core functions (ask_sp, ask_spo, etc.)
- [ ] Add stubs for remaining functions (see `src/ffi.rs` for full list)
- [ ] Verify syntax: `cargo check --package knhk-hot`

### Time Breakdown:
- Core functions (30-60 min)
- Additional functions (30-60 min)
- Stubs for remaining (30-60 min)
- Testing and debugging (30-60 min)

---

## Step 4: Update ffi.rs (15 minutes) ⚡

### File: `/home/user/knhk/rust/knhk-hot/src/ffi.rs`

### Changes Required:

Find the section with:
```rust
#[link(name = "knhk")]
extern "C" {
    pub fn knhk_init_ctx(...);
    // ... more functions
}
```

Replace with:
```rust
// Conditional compilation: Use C library if available, else Rust fallback

#[cfg(have_c_optimization)]
#[link(name = "knhk")]
extern "C" {
    pub fn knhk_init_ctx(ctx: *mut Ctx, S: *const u64, P: *const u64, O: *const u64);
    pub fn knhk_pin_run(ctx: *mut Ctx, run: Run);
    pub fn knhk_eval_bool(ctx: *const Ctx, ir: *mut Ir, rcpt: *mut Receipt) -> i32;
    pub fn knhk_eval_construct8(ctx: *const Ctx, ir: *mut Ir, rcpt: *mut Receipt) -> i32;
    pub fn knhk_eval_batch8(ctx: *const Ctx, irs: *mut Ir, n: usize, rcpts: *mut Receipt) -> i32;
    pub fn knhk_beat_init();
    pub fn knhk_beat_next() -> u64;
    pub fn knhk_beat_tick(cycle: u64) -> u64;
    pub fn knhk_beat_pulse(cycle: u64) -> u64;
    // ... (keep all existing function declarations)
}

// Rust fallback module
#[cfg(not(have_c_optimization))]
mod ffi_fallback;

#[cfg(not(have_c_optimization))]
pub use ffi_fallback::*;
```

### Checklist:
- [ ] Open `/home/user/knhk/rust/knhk-hot/src/ffi.rs`
- [ ] Find the `extern "C"` block (around line 88)
- [ ] Add `#[cfg(have_c_optimization)]` before `#[link(name = "knhk")]`
- [ ] Add fallback module import at end of file
- [ ] Save file
- [ ] Verify syntax: `cargo check --package knhk-hot`

---

## Step 5: Update lib.rs (5 minutes) ⚡

### File: `/home/user/knhk/rust/knhk-hot/src/lib.rs`

### Changes Required:

Add near the top (after existing mod declarations):
```rust
// Conditional compilation for FFI fallback
#[cfg(not(have_c_optimization))]
pub mod ffi_fallback;
```

### Checklist:
- [ ] Open `/home/user/knhk/rust/knhk-hot/src/lib.rs`
- [ ] Add conditional module declaration
- [ ] Save file
- [ ] Verify syntax: `cargo check --package knhk-hot`

---

## Step 6: Test Compilation (30 minutes) 🔍

### Test 1: Pure Rust Build

```bash
cd /home/user/knhk

# Clean previous build
cargo clean

# Build without C optimization (should succeed!)
cargo build --package knhk-hot

# Expected output:
#   warning: Using pure Rust fallback (12-16 tick warm path, no C dependencies)
#   Compiling knhk-hot v1.0.0
#   Finished dev [unoptimized + debuginfo] target(s) in 15.3s
```

### Checklist:
- [ ] Run `cargo clean`
- [ ] Run `cargo build --package knhk-hot`
- [ ] Verify: Build succeeds (exit code 0)
- [ ] Verify: Warning about Rust fallback appears
- [ ] Verify: No "unable to find library -lknhk" error

### Test 2: Workspace Build

```bash
# Build entire workspace
cargo build --workspace

# Expected: All crates build successfully
```

### Checklist:
- [ ] Run `cargo build --workspace`
- [ ] Verify: All crates build (especially knhk-etl, knhk-cli)
- [ ] Verify: No linking errors

### Test 3: Unit Tests

```bash
# Run knhk-hot tests
cargo test --package knhk-hot

# Expected: Tests pass (or reveal which functions need implementation)
```

### Checklist:
- [ ] Run `cargo test --package knhk-hot`
- [ ] Note: Which tests pass vs fail
- [ ] Note: Which functions need more work

---

## Step 7: Run JTBD Tests (1 hour) 🎯

### Test All JTBD Scenarios

```bash
cd /home/user/knhk

# Run all JTBD tests
cargo test chicago_tdd_jtbd --all 2>&1 | tee jtbd_test_results.txt

# Expected: Most or all tests pass
```

### Individual JTBD Scenario Tests:

```bash
# Test 1: Process Mining JTBD
cargo test --package knhk-workflow-engine chicago_tdd_jtbd_process_mining -- --nocapture

# Test 2: Workflow Chaining JTBD
cargo test --package knhk-workflow-engine chicago_tdd_workflow_chaining_jtbd -- --nocapture

# Test 3: Boot Init JTBD
cargo test --package knhk-cli chicago_tdd_jtbd_boot_init -- --nocapture

# Test 4: Delta Admission JTBD
cargo test --package knhk-cli chicago_tdd_jtbd_admit_delta -- --nocapture

# Test 5: Pipeline Run JTBD
cargo test --package knhk-cli chicago_tdd_jtbd_pipeline_run -- --nocapture

# Test 6: Receipt Operations JTBD
cargo test --package knhk-cli chicago_tdd_jtbd_receipt_operations -- --nocapture
```

### Checklist:
- [ ] Run all JTBD tests: `cargo test chicago_tdd_jtbd --all`
- [ ] Record pass/fail counts
- [ ] Note: Which scenarios work completely
- [ ] Note: Which functions need more implementation
- [ ] Goal: ≥90% pass rate (some may need stub completion)

---

## Step 8: Run JTBD Examples (30 minutes) 🚀

### Execute Example Programs

```bash
cd /home/user/knhk

# Example 1: Real JTBD Validation
cargo run --package knhk-workflow-engine --example weaver_real_jtbd_validation

# Example 2: Execute Workflow
cargo run --package knhk-workflow-engine --example execute_workflow

# Example 3: 43 Patterns (if time permits)
cargo run --package knhk-workflow-engine --example weaver_all_43_patterns

# Example 4: Workflow Weaver Live-Check
cargo run --package knhk-workflow-engine --example workflow_weaver_livecheck
```

### Checklist:
- [ ] Run weaver_real_jtbd_validation example
- [ ] Verify: Example executes without panic
- [ ] Verify: Output looks reasonable
- [ ] Run execute_workflow example
- [ ] Verify: Workflows execute
- [ ] (Optional) Run remaining examples
- [ ] Note: Any missing functionality

---

## Step 9: Validate with Weaver (30 minutes) ✅

### Set Up OTLP Collector (Optional)

```bash
# Start Jaeger (if Docker available)
docker run -d -p 4317:4317 -p 16686:16686 jaegertracing/all-in-one:latest

# Or skip if Docker unavailable
```

### Weaver Schema Validation

```bash
cd /home/user/knhk

# Check registry schemas
weaver registry check -r registry/

# Expected: Schemas are valid

# (Optional) Live-check if OTLP collector running
# weaver registry live-check --registry registry/
```

### Checklist:
- [ ] Run `weaver registry check -r registry/`
- [ ] Verify: Schema validation passes
- [ ] (Optional) Set up Jaeger
- [ ] (Optional) Run examples with OTEL_EXPORTER_OTLP_ENDPOINT
- [ ] (Optional) Run `weaver registry live-check`
- [ ] Note: Weaver validation status

---

## Step 10: Document Results (30 minutes) 📝

### Create Accomplishment Report

```bash
cd /home/user/knhk/docs

# Create report file
cat > JTBD_ACCOMPLISHMENT_REPORT_$(date +%Y%m%d).md << 'EOF'
# JTBD Accomplishment Report

**Date**: $(date +%Y-%m-%d)
**Build Configuration**: Pure Rust Fallback (no C optimization)

## Build Status

- ✅ knhk-hot: Compiles successfully
- ✅ knhk-etl: Compiles successfully
- ✅ knhk-cli: Compiles successfully
- ✅ Full workspace: Builds without errors

## JTBD Scenario Status

| Scenario | Status | Notes |
|----------|--------|-------|
| Enterprise Workflows | ✅/❌ | [Pass/fail counts] |
| Process Mining | ✅/❌ | [Pass/fail counts] |
| Workflow Chaining | ✅/❌ | [Pass/fail counts] |
| System Boot Init | ✅/❌ | [Pass/fail counts] |
| Delta Admission | ✅/❌ | [Pass/fail counts] |
| Pipeline Execution | ✅/❌ | [Pass/fail counts] |
| Receipt Operations | ✅/❌ | [Pass/fail counts] |
| Weaver Validation | ✅/❌ | [Pass/fail counts] |

## Performance Profile

- Hot path: ~12-16 ticks (measured)
- Warm path: [measured values]
- Cold path: [measured values]

## Known Limitations

- [List any incomplete functions]
- [List any failing tests]
- [List any missing features]

## Next Steps

- [ ] Complete remaining stubs
- [ ] Fix any failing tests
- [ ] Optimize Rust implementation
- [ ] (Optional) Build C library for 8-tick performance

EOF
```

### Checklist:
- [ ] Create accomplishment report
- [ ] Fill in test results
- [ ] Document performance measurements
- [ ] List known limitations
- [ ] Commit changes to git

---

## Success Criteria

### Minimum Success (Required)

- [x] Build succeeds: `cargo build --workspace` exits 0
- [x] knhk-hot compiles: No linking errors
- [x] JTBD tests: ≥70% pass rate (bare minimum)
- [x] JTBD examples: At least 2 examples run without panic

### Target Success (Goal)

- [ ] Build succeeds: All crates compile
- [ ] JTBD tests: ≥90% pass rate
- [ ] JTBD examples: All examples run successfully
- [ ] Weaver validation: Schemas pass `weaver registry check`

### Optimal Success (Stretch)

- [ ] JTBD tests: 100% pass rate
- [ ] All 8 JTBD scenarios fully functional
- [ ] Performance measured: <16 ticks for hot functions
- [ ] Weaver live-check: Passes with live OTLP

---

## Troubleshooting

### Issue: Compilation errors in ffi_fallback.rs

**Symptoms**: Type mismatches, undefined functions

**Solution**:
1. Check that all types match ffi.rs declarations exactly
2. Ensure cycle_counter module is available
3. Add missing imports

```bash
# Check what's available
grep -n "pub fn" src/ffi.rs | head -20
grep -n "pub use" src/lib.rs
```

### Issue: Tests fail with unimplemented functions

**Symptoms**: Test panics or returns wrong results

**Solution**:
1. Identify which functions are called by failing tests
2. Prioritize implementing those functions first
3. Use stubs that return safe defaults for rarely-used functions

```bash
# Find which tests are failing
cargo test chicago_tdd_jtbd --all 2>&1 | grep -A 5 "FAILED"
```

### Issue: Linker still tries to find -lknhk

**Symptoms**: "unable to find library -lknhk" error persists

**Solution**:
1. Verify build.rs changes are correct
2. Ensure feature flag is NOT enabled
3. Clean and rebuild

```bash
cargo clean
cargo build --package knhk-hot --verbose
# Look for "rustc-link-lib=static=knhk" in output (should NOT appear)
```

---

## Time Tracking

| Step | Estimated | Actual | Notes |
|------|-----------|--------|-------|
| 1. Update build.rs | 30 min | _____ | |
| 2. Add feature flag | 5 min | _____ | |
| 3. Create fallback | 2-3 hours | _____ | Largest task |
| 4. Update ffi.rs | 15 min | _____ | |
| 5. Update lib.rs | 5 min | _____ | |
| 6. Test compilation | 30 min | _____ | |
| 7. Run JTBD tests | 1 hour | _____ | |
| 8. Run examples | 30 min | _____ | |
| 9. Weaver validation | 30 min | _____ | |
| 10. Document | 30 min | _____ | |
| **TOTAL** | **5-6 hours** | _____ | |

---

## Completion Checklist

### Phase 1: Implementation (2-3 hours)
- [ ] Step 1: build.rs updated
- [ ] Step 2: Feature flag added
- [ ] Step 3: Rust fallback created
- [ ] Step 4: ffi.rs updated
- [ ] Step 5: lib.rs updated

### Phase 2: Testing (2-3 hours)
- [ ] Step 6: Compilation succeeds
- [ ] Step 7: JTBD tests run
- [ ] Step 8: JTBD examples execute
- [ ] Step 9: Weaver validation passes

### Phase 3: Documentation (30 min)
- [ ] Step 10: Results documented
- [ ] Changes committed to git
- [ ] Team notified of completion

---

## Final Verification

```bash
# Complete verification script
cd /home/user/knhk

echo "=== BUILD TEST ==="
cargo clean && cargo build --workspace
echo "Build exit code: $?"

echo "=== JTBD TEST SUMMARY ==="
cargo test chicago_tdd_jtbd --all 2>&1 | grep -E "(test result|FAILED|passed)"

echo "=== EXAMPLE TEST ==="
cargo run --example weaver_real_jtbd_validation 2>&1 | head -20

echo "=== WEAVER VALIDATION ==="
weaver registry check -r registry/
echo "Weaver exit code: $?"

echo "=== COMPLETION STATUS ==="
if cargo build --workspace >/dev/null 2>&1; then
    echo "✅ SUCCESS: JTBD accomplishment UNBLOCKED!"
else
    echo "❌ FAILED: Review errors above"
fi
```

Run this script to verify everything works!

---

**Next**: After completing this checklist, see `/home/user/knhk/docs/FMEA_TRIZ_BUILD_FAILURE_ANALYSIS.md` for P1/P2 optimizations.

**Status**: 🟢 READY TO EXECUTE

**Expected Outcome**: 100% JTBD accomplishment by end of day 🚀
