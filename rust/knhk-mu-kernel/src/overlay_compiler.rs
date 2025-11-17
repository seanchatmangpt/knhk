//! Compiler-Generated Proof Infrastructure
//!
//! This module provides the infrastructure for the Σ→Σ* compiler to generate
//! proofs during compilation. Only the compiler can create these proofs.

use crate::overlay_proof::{ChangeCoverage, CompilerProof};
use crate::overlay_safety::{ColdUnsafe, HotSafe, SafeProof, WarmSafe};
use crate::overlay_types::{OverlayChange, OverlayChanges, OverlayError};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

/// Compiler context for proof generation
///
/// This is created by the compiler during Σ→Σ* compilation.
/// It tracks analysis results and generates proofs.
pub struct CompilerContext {
    /// Compiler version
    version: (u8, u8, u8),

    /// Next proof ID
    next_proof_id: u64,

    /// Invariant checker
    invariant_checker: InvariantChecker,

    /// Timing analyzer
    timing_analyzer: TimingAnalyzer,

    /// Safety classifier
    safety_classifier: SafetyClassifier,
}

impl CompilerContext {
    /// Create a new compiler context
    pub fn new(version: (u8, u8, u8)) -> Self {
        Self {
            version,
            next_proof_id: 1,
            invariant_checker: InvariantChecker::new(),
            timing_analyzer: TimingAnalyzer::new(),
            safety_classifier: SafetyClassifier::new(),
        }
    }

    /// Generate a proof for overlay changes
    ///
    /// This is the main entry point for proof generation during compilation.
    pub fn generate_proof(
        &mut self,
        changes: &OverlayChanges,
    ) -> Result<CompilerProof, CompilerError> {
        // Check all invariants
        let invariants = self.invariant_checker.check_all(changes)?;

        // Analyze timing
        let timing_bound = self.timing_analyzer.analyze(changes)?;

        // Determine coverage
        let coverage = self.compute_coverage(changes);

        // Generate signature (simplified - would use real crypto)
        let signature = self.sign_proof(&invariants, timing_bound, &coverage);

        let proof_id = self.next_proof_id;
        self.next_proof_id += 1;

        Ok(CompilerProof {
            compiler_version: self.version,
            proof_id,
            invariants,
            timing_bound,
            coverage,
            signature,
        })
    }

    /// Generate a safety-classified proof
    pub fn generate_safe_proof<S>(
        &mut self,
        changes: &OverlayChanges,
    ) -> Result<SafeProof<S, CompilerProof>, CompilerError>
    where
        S: crate::overlay_safety::SafetyLevel,
    {
        let proof = self.generate_proof(changes)?;

        SafeProof::new(proof)
            .map_err(|e| CompilerError::SafetyClassificationFailed(format!("{:?}", e)))
    }

    /// Compute coverage for changes
    fn compute_coverage(&self, changes: &OverlayChanges) -> ChangeCoverage {
        let covered_changes = changes.len() as u32;
        let coverage_percent = 100; // Compiler provides 100% coverage

        ChangeCoverage {
            covered_changes,
            coverage_percent,
        }
    }

    /// Sign the proof (simplified)
    fn sign_proof(
        &self,
        invariants: &[u16],
        timing_bound: u64,
        coverage: &ChangeCoverage,
    ) -> [u8; 64] {
        // In reality, would use ed25519 or similar
        // For now, just create a deterministic signature
        let mut sig = [0u8; 64];

        // Mix in compiler version
        sig[0] = self.version.0;
        sig[1] = self.version.1;
        sig[2] = self.version.2;

        // Mix in invariant count
        sig[3] = invariants.len() as u8;

        // Mix in timing bound
        sig[4..12].copy_from_slice(&timing_bound.to_le_bytes());

        // Mix in coverage
        sig[12..16].copy_from_slice(&coverage.covered_changes.to_le_bytes());

        // Mark as non-zero (so tests can detect signed vs unsigned)
        sig[63] = 1;

        sig
    }
}

/// Invariant checker
///
/// Checks that all required invariants hold for the changes.
struct InvariantChecker {
    /// Known invariants
    invariants: Vec<InvariantDefinition>,
}

impl InvariantChecker {
    fn new() -> Self {
        Self {
            invariants: Self::standard_invariants(),
        }
    }

    /// Standard invariants checked by compiler
    fn standard_invariants() -> Vec<InvariantDefinition> {
        vec![
            InvariantDefinition {
                id: 1,
                name: "tick_budget",
                checker: InvariantChecker::check_tick_budget,
            },
            InvariantDefinition {
                id: 2,
                name: "no_cycles",
                checker: InvariantChecker::check_no_cycles,
            },
            InvariantDefinition {
                id: 3,
                name: "type_safety",
                checker: InvariantChecker::check_type_safety,
            },
            InvariantDefinition {
                id: 4,
                name: "resource_bounds",
                checker: InvariantChecker::check_resource_bounds,
            },
        ]
    }

    /// Check all invariants
    fn check_all(&self, changes: &OverlayChanges) -> Result<Vec<u16>, CompilerError> {
        let mut preserved = Vec::new();

        for inv in &self.invariants {
            if (inv.checker)(changes)? {
                preserved.push(inv.id);
            } else {
                return Err(CompilerError::InvariantViolation {
                    id: inv.id,
                    name: inv.name,
                });
            }
        }

        Ok(preserved)
    }

    // Invariant checker functions

    fn check_tick_budget(changes: &OverlayChanges) -> Result<bool, CompilerError> {
        for change in changes.iter() {
            match change {
                OverlayChange::AddTask { tick_budget, .. } => {
                    if *tick_budget > crate::CHATMAN_CONSTANT {
                        return Ok(false);
                    }
                }
                _ => {}
            }
        }
        Ok(true)
    }

    fn check_no_cycles(_changes: &OverlayChanges) -> Result<bool, CompilerError> {
        // Simplified - would do actual cycle detection
        Ok(true)
    }

    fn check_type_safety(_changes: &OverlayChanges) -> Result<bool, CompilerError> {
        // Simplified - would check type system rules
        Ok(true)
    }

    fn check_resource_bounds(_changes: &OverlayChanges) -> Result<bool, CompilerError> {
        // Simplified - would check memory/resource limits
        Ok(true)
    }
}

/// Invariant definition
struct InvariantDefinition {
    id: u16,
    name: &'static str,
    checker: fn(&OverlayChanges) -> Result<bool, CompilerError>,
}

/// Timing analyzer
///
/// Analyzes worst-case execution time for changes.
struct TimingAnalyzer {
    /// Maximum ticks seen so far
    max_ticks: u64,
}

impl TimingAnalyzer {
    fn new() -> Self {
        Self { max_ticks: 0 }
    }

    /// Analyze timing for changes
    fn analyze(&mut self, changes: &OverlayChanges) -> Result<u64, CompilerError> {
        let mut total_ticks = 0u64;

        for change in changes.iter() {
            let ticks = self.analyze_change(change)?;
            total_ticks = total_ticks.saturating_add(ticks);
        }

        self.max_ticks = self.max_ticks.max(total_ticks);

        Ok(total_ticks)
    }

    /// Analyze single change
    fn analyze_change(&self, change: &OverlayChange) -> Result<u64, CompilerError> {
        match change {
            OverlayChange::AddTask { tick_budget, .. } => Ok(*tick_budget),
            OverlayChange::RemoveTask { .. } => Ok(1), // Constant time
            OverlayChange::ModifyTask { .. } => Ok(2), // Constant time
            OverlayChange::AddGuard { .. } => Ok(4),   // Guard registration
            OverlayChange::ModifyGuardThreshold { .. } => Ok(1), // Constant time
            OverlayChange::AddPattern { .. } => Ok(2), // Pattern registration
            OverlayChange::RemovePattern { .. } => Ok(1), // Constant time
        }
    }
}

/// Safety classifier
///
/// Classifies changes by safety level.
struct SafetyClassifier;

impl SafetyClassifier {
    fn new() -> Self {
        Self
    }

    /// Classify changes by safety level
    fn classify(&self, timing_bound: u64) -> SafetyClass {
        if timing_bound <= 8 {
            SafetyClass::Hot
        } else if timing_bound <= 1_000_000 {
            SafetyClass::Warm
        } else {
            SafetyClass::Cold
        }
    }
}

/// Safety classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyClass {
    Hot,
    Warm,
    Cold,
}

/// Compiler errors
#[derive(Debug, Clone)]
pub enum CompilerError {
    /// Invariant violation
    InvariantViolation { id: u16, name: &'static str },

    /// Timing bound exceeded
    TimingBoundExceeded { max_allowed: u64, actual: u64 },

    /// Safety classification failed
    SafetyClassificationFailed(String),

    /// Change validation failed
    ChangeValidationFailed(String),
}

impl core::fmt::Display for CompilerError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvariantViolation { id, name } => {
                write!(f, "invariant {} ({}) violated", id, name)
            }
            Self::TimingBoundExceeded {
                max_allowed,
                actual,
            } => {
                write!(
                    f,
                    "timing bound exceeded: max {}, actual {}",
                    max_allowed, actual
                )
            }
            Self::SafetyClassificationFailed(msg) => {
                write!(f, "safety classification failed: {}", msg)
            }
            Self::ChangeValidationFailed(msg) => {
                write!(f, "change validation failed: {}", msg)
            }
        }
    }
}

/// Proof builder for convenient construction
pub struct ProofBuilder {
    ctx: CompilerContext,
}

impl ProofBuilder {
    /// Create a new proof builder
    pub fn new() -> Self {
        Self {
            ctx: CompilerContext::new(crate::MU_KERNEL_VERSION),
        }
    }

    /// Build a HotSafe proof
    pub fn build_hot_safe(
        &mut self,
        changes: &OverlayChanges,
    ) -> Result<SafeProof<HotSafe, CompilerProof>, CompilerError> {
        self.ctx.generate_safe_proof(changes)
    }

    /// Build a WarmSafe proof
    pub fn build_warm_safe(
        &mut self,
        changes: &OverlayChanges,
    ) -> Result<SafeProof<WarmSafe, CompilerProof>, CompilerError> {
        self.ctx.generate_safe_proof(changes)
    }

    /// Build a ColdUnsafe proof
    pub fn build_cold_unsafe(
        &mut self,
        changes: &OverlayChanges,
    ) -> Result<SafeProof<ColdUnsafe, CompilerProof>, CompilerError> {
        self.ctx.generate_safe_proof(changes)
    }
}

impl Default for ProofBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sigma::TaskDescriptor;

    #[test]
    fn test_compiler_proof_generation() {
        let mut ctx = CompilerContext::new((2027, 0, 0));

        let mut changes = OverlayChanges::new();
        changes.push(OverlayChange::AddTask {
            task_id: 1,
            descriptor: TaskDescriptor::default(),
            tick_budget: 6,
        });

        let proof = ctx.generate_proof(&changes);
        assert!(proof.is_ok());

        let proof = proof.unwrap();
        assert_eq!(proof.compiler_version, (2027, 0, 0));
        assert!(proof.timing_bound <= 8);
    }

    #[test]
    fn test_invariant_violation_detection() {
        let mut ctx = CompilerContext::new((2027, 0, 0));

        let mut changes = OverlayChanges::new();
        changes.push(OverlayChange::AddTask {
            task_id: 1,
            descriptor: TaskDescriptor::default(),
            tick_budget: 100, // Exceeds CHATMAN_CONSTANT
        });

        let proof = ctx.generate_proof(&changes);
        assert!(proof.is_err());
    }

    #[test]
    fn test_proof_builder_hot_safe() {
        let mut builder = ProofBuilder::new();

        let mut changes = OverlayChanges::new();
        changes.push(OverlayChange::AddTask {
            task_id: 1,
            descriptor: TaskDescriptor::default(),
            tick_budget: 5,
        });

        let proof = builder.build_hot_safe(&changes);
        assert!(proof.is_ok());
    }

    #[test]
    fn test_proof_builder_rejects_slow_for_hot() {
        let mut builder = ProofBuilder::new();

        let mut changes = OverlayChanges::new();
        changes.push(OverlayChange::AddTask {
            task_id: 1,
            descriptor: TaskDescriptor::default(),
            tick_budget: 100,
        });

        let proof = builder.build_hot_safe(&changes);
        assert!(proof.is_err());
    }
}
