# V1.0 FFI Compilation Fixes Required

**Status**: 🔴 **BLOCKING RELEASE**
**Estimated Fix Time**: 1-2 hours
**Location**: `rust/knhk-hot/src/ffi.rs`, `rust/knhk-hot/src/ring_ffi.rs`

---

## Fix 1: Mark Unsafe FFI Functions

**File**: `rust/knhk-hot/src/ffi.rs:126`

**Current Code**:
```rust
pub fn init_context(s: *const u64, p: *const u64, o: *const u64) -> HotContext {
    unsafe { knhk_init_ctx(&mut ctx, s, p, o) };
    ctx
}
```

**Fixed Code**:
```rust
pub unsafe fn init_context(s: *const u64, p: *const u64, o: *const u64) -> HotContext {
    let mut ctx = HotContext::default();
    knhk_init_ctx(&mut ctx, s, p, o);
    ctx
}
```

**Reason**: Functions that dereference raw pointers must be marked `unsafe`

---

## Fix 2: Replace len() == 0 with is_empty()

**File**: `rust/knhk-hot/src/ring_ffi.rs:146`

**Current Code**:
```rust
if S.len() == 0 || S.len() > 8 {
    return None;
}
```

**Fixed Code**:
```rust
if S.is_empty() || S.len() > 8 {
    return None;
}
```

**Reason**: Clippy prefers `is_empty()` for clarity

---

## Fix 3: Simplify Complex Return Types

**File**: `rust/knhk-hot/src/ring_ffi.rs:174`

**Current Code**:
```rust
pub fn dequeue_all(&self) -> Option<(Vec<u64>, Vec<u64>, Vec<u64>, Vec<u64>)> {
    // ...
}
```

**Fixed Code**:
```rust
type SoABuffers = (Vec<u64>, Vec<u64>, Vec<u64>, Vec<u64>);

pub fn dequeue_all(&self) -> Option<SoABuffers> {
    // ...
}
```

**Reason**: Reduce type complexity by using type alias

---

## Fix 4: Add FFI Naming Exceptions

**File**: `rust/knhk-hot/src/ffi.rs:26`

**Current Code**:
```rust
pub struct HotContext {
    pub S: *const u64,
    pub P: *const u64,
    pub O: *const u64,
}
```

**Fixed Code**:
```rust
#[allow(non_snake_case)] // FFI compatibility - matches C struct naming
pub struct HotContext {
    pub S: *const u64,
    pub P: *const u64,
    pub O: *const u64,
}
```

**Reason**: FFI requires uppercase field names to match C struct

---

## Verification Steps

After applying fixes:

1. **Build Test**:
   ```bash
   cd /Users/sac/knhk/rust/knhk-hot
   cargo build --release
   ```
   Expected: ✅ No errors

2. **Clippy Test**:
   ```bash
   cd /Users/sac/knhk/rust/knhk-hot
   cargo clippy -- -D warnings
   ```
   Expected: ✅ No warnings/errors

3. **Full Workspace Build**:
   ```bash
   cd /Users/sac/knhk
   cargo build --release --workspace
   ```
   Expected: ✅ All crates compile

4. **Full Workspace Clippy**:
   ```bash
   cd /Users/sac/knhk
   cargo clippy --workspace -- -D warnings
   ```
   Expected: ✅ Only acceptable dependency warnings

---

## Post-Fix Checklist

- [ ] All FFI functions marked `unsafe`
- [ ] All `len() == 0` replaced with `is_empty()`
- [ ] Complex types simplified with type aliases
- [ ] FFI naming exceptions added with comments
- [ ] `cargo build --release` succeeds
- [ ] `cargo clippy -- -D warnings` passes
- [ ] Tests still pass: `cargo test`

---

**Once Complete**: Update status to ✅ **GO FOR PRODUCTION**

---

*Generated: 2025-11-06*
*Code Review Agent: #10*
