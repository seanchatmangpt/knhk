// knhk-warm FFI bindings
// FFI-safe types for warm path operations

#![allow(non_camel_case_types)]

// Re-export types from knhk-hot (they're already public via pub use ffi::* in knhk_hot)
pub use knhk_hot::{Ctx, Ir, Op, Receipt, Run};

// Type aliases for clarity
pub type HotContext = Ctx;
pub type HotHookIr = Ir;
pub type HotReceipt = Receipt;

// FFI function declarations
// These will call the C hot path functions but through warm path routing
#[link(name = "knhk")]
extern "C" {
    /// Execute CONSTRUCT8 operation
    /// Note: This is the same C function as hot path, but routed through warm path
    pub fn knhk_eval_construct8(ctx: *const Ctx, ir: *mut Ir, rcpt: *mut Receipt) -> i32;
}

/// Safe wrapper for FFI calls
///
/// # Safety
///
/// This function is unsafe because it:
/// - Dereferences raw pointers (`ctx`, `ir`, `rcpt`)
/// - Calls into C FFI which has no Rust safety guarantees
///
/// Callers must ensure:
/// - `ctx` points to a valid, initialized `Ctx` instance
/// - `ir` points to a valid, initialized `Ir` instance that can be mutated
/// - `rcpt` points to a valid, initialized `Receipt` instance that can be mutated
/// - All pointers remain valid for the duration of the function call
/// - No data races occur on the pointed-to data
pub unsafe fn knhk_hot_eval_construct8(ctx: *const Ctx, ir: *mut Ir, rcpt: *mut Receipt) -> i32 {
    knhk_eval_construct8(ctx, ir, rcpt)
}
