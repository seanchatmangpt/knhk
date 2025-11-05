// knhk-warm FFI bindings
// FFI-safe types for warm path operations

#![allow(non_camel_case_types)]

use knhk_hot::{Ctx, Ir, Receipt, Op};

// Re-export types from knhk-hot for convenience
pub use knhk_hot::Run;

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
    pub fn knhk_eval_construct8(
        ctx: *const Ctx,
        ir: *mut Ir,
        rcpt: *mut Receipt,
    ) -> i32;
}

// Safe wrapper for FFI calls
pub unsafe fn knhk_hot_eval_construct8(
    ctx: *const Ctx,
    ir: *mut Ir,
    rcpt: *mut Receipt,
) -> i32 {
    knhk_eval_construct8(ctx, ir, rcpt)
}

// Re-export Op for warm_path.rs
pub use knhk_hot::Op;

