// knhk-hot v1.0 Rust wrapper
// Safe wrappers around C FFI for hot path execution

pub mod ffi;
pub mod beat_ffi;
pub mod ring_ffi;
pub mod fiber_ffi;
pub mod receipt_convert;
pub mod kernels;
pub use ffi::*;
pub use beat_ffi::BeatScheduler;
pub use ring_ffi::{DeltaRing, AssertionRing};
pub use fiber_ffi::{FiberExecutor, FiberResult};
pub use receipt_convert::{c_receipt_to_etl, etl_receipt_to_c, hot_receipt_to_etl};
pub use kernels::{KernelType, KernelExecutor};


