//! Java↔Rust Capability Parity Test Suite
//!
//! This module validates semantic equivalence between Java YAWL and Rust KNHK implementations
//! using Design for Lean Six Sigma (DFLSS) and TRIZ methodologies.
//!
//! ## DOCTRINE ALIGNMENT:
//! - Principle: Q (Hard Invariants) - Invariants are law
//! - Covenant: Covenant 2 (Invariants Are Law) + Covenant 5 (Chatman Constant)
//! - Why This Matters: Tests must prove semantic equivalence with sub-8-tick latency guarantee
//!
//! ## Test Organization:
//! - `pattern_tests` - 43 YAWL pattern parity tests using DOE-designed parameters
//! - `semantic_equivalence` - Behavioral equivalence proofs across critical scenarios
//! - `performance_comparison` - Latency and memory footprint comparison vs Java baseline
//! - `chaos_injection` - Failure handling and robustness equivalence tests
//!
//! ## Validation Strategy:
//! 1. Mock-based Java behavior simulation (London TDD)
//! 2. Property-based testing for parameter space exploration
//! 3. Chicago TDD for actual execution validation
//! 4. Weaver schema validation for telemetry assertions
//!
//! ## Critical-to-Quality (CTQ) Parameters:
//! - Behavioral parity: 100% semantic equivalence on control flow
//! - Performance: ≤8 ticks for hot path (Chatman Constant)
//! - Coverage: ≥95% of parameter space via DOE
//! - Robustness: Equivalent failure handling across all patterns

pub mod pattern_tests;
pub mod semantic_equivalence;
pub mod performance_comparison;
pub mod chaos_injection;
