//! Advanced Validators
//!
//! Production-grade validation phases:
//! - Formal Soundness: Option to Complete, Proper Completion, No Dead Tasks
//! - Conformance Metrics: Real fitness/precision calculation
//! - Pattern Semantics: Inline Van der Aalst pattern verification
//! - Load Testing: 100-case stress testing

pub mod conformance;
pub mod formal_soundness;
pub mod load_testing;
pub mod pattern_semantics;

pub use conformance::ConformanceMetricsPhase;
pub use formal_soundness::FormalSoundnessPhase;
pub use load_testing::LoadTestingPhase;
pub use pattern_semantics::PatternSemanticsPhase;
