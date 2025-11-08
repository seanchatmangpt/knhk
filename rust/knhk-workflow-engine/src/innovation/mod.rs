//! Innovation module
//!
//! Provides innovative features without AI dependencies:
//! - Deterministic execution guarantees
//! - Zero-copy optimizations
//! - Hardware acceleration (SIMD, GPU)
//! - Formal verification
//! - Advanced scheduling
//! - Performance profiling

mod deterministic;
mod formal;
mod hardware;
mod zero_copy;

pub use deterministic::{
    DeltaLogEntry, DeterministicContext, DeterministicExecutor, ExecutionStep,
};
pub use formal::{FormalVerifier, Property, VerificationResult, Violation};
pub use hardware::{HardwareAcceleration, HardwareAccelerator};
pub use zero_copy::{ZeroCopyBytes, ZeroCopyStr, ZeroCopyTriple, ZeroCopyTripleBatch};
