//! Performance tests for workflow engine
//!
//! Tests hot path operations, RDTSC measurements, and performance constraints.
//! All hot path operations must execute in ≤8 ticks (Chatman Constant).

mod hot_path;
mod rdtsc;
mod benchmarks;

