// knhk-hot v1.0 Rust wrapper
// Safe wrappers around C FFI for hot path execution

pub mod beat_ffi;
pub mod content_addr;
pub mod cpu_dispatch;
pub mod ffi;
pub mod fiber_ffi;
pub mod kernels;
pub mod receipt_convert;
pub mod ring_ffi;

pub use beat_ffi::BeatScheduler;
pub use content_addr::{content_hash, content_hash_128, ContentId};
pub use cpu_dispatch::{
    init_cpu_dispatch, get_discriminator_fn, get_parallel_split_fn,
    get_synchronization_fn, get_multi_choice_fn, CpuDispatcher, CpuFeatures,
    PatternContext, PatternResult, BranchFn,
};
pub use ffi::*;
pub use fiber_ffi::{FiberExecutor, FiberResult};
pub use kernels::{KernelExecutor, KernelType};
pub use receipt_convert::{c_receipt_to_etl, etl_receipt_to_c, hot_receipt_to_etl};
pub use ring_ffi::{AssertionRing, DeltaRing};
