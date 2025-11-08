//! Timebase abstraction for deterministic time management
//!
//! Provides trait-based clock system with real (`SysClock`) and simulated (`SimClock`) implementations
//! for production and testing scenarios.

pub mod sim;
pub mod sys;
pub mod trait_impl;

pub use sim::SimClock;
pub use sys::SysClock;
pub use trait_impl::Timebase;
