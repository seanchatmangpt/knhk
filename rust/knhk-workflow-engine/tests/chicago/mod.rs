//! Chicago TDD Test Suite for KNHK Self-Executing Workflows
//!
//! Comprehensive test suite following Chicago TDD methodology:
//! - State-based testing with real collaborators
//! - Behavior verification (test what, not how)
//! - AAA pattern (Arrange, Act, Assert)
//! - No mocking of domain logic
//!
//! ## Test Organization
//!
//! - `pattern_sequence_test` - Pattern 1: Sequence
//! - `pattern_parallel_split_test` - Pattern 2: Parallel Split
//! - `hook_engine_test` - Hook lifecycle and execution
//! - `guard_enforcement_test` - Guard constraints and validation
//! - `receipt_generation_test` - Lockchain receipts and validation
//! - `mape_k_monitor_test` - MAPE-K Monitor phase
//! - `mape_k_analyze_test` - MAPE-K Analyze phase
//! - `snapshot_system_test` - State persistence and recovery
//!
//! ## Running Tests
//!
//! ```bash
//! # Run all Chicago TDD tests
//! cargo test --test chicago
//!
//! # Run specific test module
//! cargo test --test pattern_sequence_test
//! cargo test --test hook_engine_test
//!
//! # Run with output
//! cargo test --test chicago -- --nocapture
//! ```

mod guard_enforcement_test;
mod hook_engine_test;
mod mape_k_analyze_test;
mod mape_k_monitor_test;
mod pattern_parallel_split_test;
mod pattern_sequence_test;
mod receipt_generation_test;
mod snapshot_system_test;
