// knhk-hot v1.0 Rust wrapper
// Safe wrappers around C FFI for hot path execution

pub mod beat_ffi;
pub mod content_addr;
pub mod cpu_dispatch;
pub mod cycle_counter;
pub mod ffi;
pub mod fiber_ffi;
pub mod kernels;
pub mod receipt_convert;
pub mod receipt_kernels;
pub mod ring_ffi;
pub mod w1_pipeline;

pub use beat_ffi::BeatScheduler;
pub use content_addr::{content_hash, content_hash_128, ContentId};
pub use cpu_dispatch::{
    get_discriminator_fn, get_multi_choice_fn, get_parallel_split_fn, get_synchronization_fn,
    init_cpu_dispatch, BranchFn, CpuDispatcher, CpuFeatures, PatternContext, PatternResult,
};
pub use cycle_counter::{cycles_to_ticks, read_cycles, read_cycles_precise, TickMeasurement};
pub use ffi::*;
pub use fiber_ffi::{FiberExecutor, FiberResult};
pub use kernels::{KernelExecutor, KernelType};
pub use receipt_convert::{c_receipt_to_etl, etl_receipt_to_c, hot_receipt_to_etl};
pub use receipt_kernels::{
    DeltaComposer, Pruner, ReceiptDelta, ReceiptFold, ReceiptHasher, ReceiptKernelType,
    ReceiptPipeline, Verifier,
};
pub use ring_ffi::{AssertionRing, DeltaRing};
pub use w1_pipeline::{stage1_structural_index, StructuralIndex};
