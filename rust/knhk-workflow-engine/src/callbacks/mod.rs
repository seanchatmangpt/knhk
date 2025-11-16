//! Callback system using Higher-Ranked Trait Bounds
//!
//! Provides flexible callback registration and execution using HRTBs,
//! enabling callbacks that work with any lifetime.

pub mod hrtb;

pub use hrtb::{
    AsyncPatternCallback, CallbackExecutor, CallbackRegistry, CallbackRegistryBuilder,
    PatternCallback,
};
