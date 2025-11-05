// knhk-warm: Warm path operations (≤500ms budget)
// CONSTRUCT8 and other emit operations moved from hot path

#![no_std]

extern crate alloc;

pub mod ffi;
pub mod warm_path;

pub use warm_path::*;

