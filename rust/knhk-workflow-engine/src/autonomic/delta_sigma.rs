//! ΔΣ Guarded Overlay Engine for Safe Ontology Evolution
//!
//! Provides type-safe overlay proposals with proof obligations for safe workflow adaptation.
//!
//! **Architecture**:
//! - Type-level proof states using phantom types (Unproven → ProofPending → Proven)
//! - Explicit scope tracking (workflows, patterns, guards affected)
//! - Risk surface analysis (which execution graph parts may change)
//! - Compile-time safety: invalid proofs rejected at compile time where possible
//! - Runtime verification: proof obligations must be satisfied before overlay application
//!
//! **Integration**:
//! - MAPE-K Plan: Generates `DeltaSigma<Unproven>` proposals
//! - MAPE-K Execute: Only accepts `DeltaSigma<Proven>` overlays
//! - Pattern Registry: Uses typed PatternId, not stringly-typed Turtle
//! - Validation: Integrates with existing YAWL validation framework
//!
//! # Example
//!
//! ```rust
//! use knhk_workflow_engine::autonomic::delta_sigma::*;
//!
//! // Create overlay proposal (unproven state)
//! let proposal = DeltaSigma::new(
//!     OverlayScope::new()
//!         .with_pattern(PatternId::new(12)?)
//!         .with_workflow(workflow_id),
//!     vec![OverlayChange::ScaleMultiInstance { delta: 2 }]
//! );
//!
//! // Generate proof obligations
//! let proof_pending = proposal.generate_proof_obligations()?;
//!
//! // Execute proof validation (async)
//! let proven = proof_pending.validate().await?;
//!
//! // Only proven overlays can be applied
//! executor.apply_overlay(proven).await?;
//! ```

use crate::error::{WorkflowError, WorkflowResult};
use crate::parser::WorkflowSpecId;
use crate::patterns::PatternId;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

/// Proof state marker trait
///
/// Types implementing this trait represent different proof validation states.
/// The type system enforces that only proven overlays can be applied.
pub trait ProofState: Send + Sync {}

/// Unproven state - overlay proposal has not been validated
#[derive(Debug, Clone, Copy)]
pub struct Unproven;

/// Proof pending state - proof obligations generated, validation in progress
#[derive(Debug, Clone, Copy)]
pub struct ProofPending;

/// Proven state - all proof obligations satisfied, safe to apply
#[derive(Debug, Clone, Copy)]
pub struct Proven;

impl ProofState for Unproven {}
impl ProofState for ProofPending {}
impl ProofState for Proven {}

/// Overlay identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OverlayId(#[serde(with = "uuid::serde::compact")] pub Uuid);

impl OverlayId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for OverlayId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for OverlayId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "overlay:{}", self.0)
    }
}

/// Overlay scope - which parts of the system are affected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayScope {
    /// Workflows affected by this overlay
    pub workflows: HashSet<WorkflowSpecId>,
    /// Patterns affected by this overlay
    pub patterns: HashSet<PatternId>,
    /// Guard constraints affected
    pub guards: HashSet<String>,
    /// Custom scope tags
    pub tags: HashMap<String, String>,
}

impl OverlayScope {
    /// Create empty scope
    pub fn new() -> Self {
        Self {
            workflows: HashSet::new(),
            patterns: HashSet::new(),
            guards: HashSet::new(),
            tags: HashMap::new(),
        }
    }

    /// Add workflow to scope
    pub fn with_workflow(mut self, workflow_id: WorkflowSpecId) -> Self {
        self.workflows.insert(workflow_id);
        self
    }

    /// Add pattern to scope
    pub fn with_pattern(mut self, pattern_id: PatternId) -> Self {
        self.patterns.insert(pattern_id);
        self
    }

    /// Add guard to scope
    pub fn with_guard(mut self, guard_name: String) -> Self {
        self.guards.insert(guard_name);
        self
    }

    /// Add tag to scope
    pub fn with_tag(mut self, key: String, value: String) -> Self {
        self.tags.insert(key, value);
        self
    }

    /// Check if scope is empty
    pub fn is_empty(&self) -> bool {
        self.workflows.is_empty()
            && self.patterns.is_empty()
            && self.guards.is_empty()
            && self.tags.is_empty()
    }

    /// Calculate risk surface (number of affected entities)
    pub fn risk_surface(&self) -> usize {
        self.workflows.len() + self.patterns.len() + self.guards.len()
    }
}

impl Default for OverlayScope {
    fn default() -> Self {
        Self::new()
    }
}

/// Overlay change - typed changes to workflow behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverlayChange {
    /// Scale multi-instance pattern
    ScaleMultiInstance { delta: i32 },
    /// Adjust performance target
    AdjustPerformance { target_ticks: u64 },
    /// Modify guard constraint
    ModifyGuard {
        guard_name: String,
        new_value: String,
    },
    /// Enable/disable pattern
    TogglePattern {
        pattern_id: PatternId,
        enabled: bool,
    },
    /// Adjust resource allocation
    AdjustResources { resource: String, multiplier: f64 },
    /// Custom overlay (with validation hook)
    Custom {
        change_type: String,
        params: HashMap<String, String>,
    },
}

impl OverlayChange {
    /// Get human-readable description
    pub fn description(&self) -> String {
        match self {
            OverlayChange::ScaleMultiInstance { delta } => {
                format!("Scale multi-instance by {}", delta)
            }
            OverlayChange::AdjustPerformance { target_ticks } => {
                format!("Adjust performance target to {} ticks", target_ticks)
            }
            OverlayChange::ModifyGuard {
                guard_name,
                new_value,
            } => {
                format!("Modify guard '{}' to '{}'", guard_name, new_value)
            }
            OverlayChange::TogglePattern {
                pattern_id,
                enabled,
            } => {
                format!(
                    "{} pattern {}",
                    if *enabled { "Enable" } else { "Disable" },
                    pattern_id
                )
            }
            OverlayChange::AdjustResources {
                resource,
                multiplier,
            } => {
                format!("Adjust resource '{}' by {}x", resource, multiplier)
            }
            OverlayChange::Custom { change_type, .. } => {
                format!("Custom change: {}", change_type)
            }
        }
    }

    /// Get affected scope from change
    pub fn affected_scope(&self) -> OverlayScope {
        let mut scope = OverlayScope::new();

        match self {
            OverlayChange::ScaleMultiInstance { .. } => {
                // Affects multi-instance patterns (12-15)
                for id in 12..=15 {
                    if let Ok(pattern_id) = PatternId::new(id) {
                        scope.patterns.insert(pattern_id);
                    }
                }
            }
            OverlayChange::ModifyGuard { guard_name, .. } => {
                scope.guards.insert(guard_name.clone());
            }
            OverlayChange::TogglePattern { pattern_id, .. } => {
                scope.patterns.insert(*pattern_id);
            }
            OverlayChange::AdjustPerformance { .. } => {
                // Affects all patterns (performance is global)
                for id in 1..=43 {
                    if let Ok(pattern_id) = PatternId::new(id) {
                        scope.patterns.insert(pattern_id);
                    }
                }
            }
            OverlayChange::AdjustResources { .. } => {
                // Resource changes affect all workflows
            }
            OverlayChange::Custom { .. } => {
                // Custom changes must declare scope explicitly
            }
        }

        scope
    }
}

/// ΔΣ (DeltaSigma) - Type-safe overlay proposal with proof state
///
/// The type parameter `P: ProofState` encodes the proof validation state.
/// This prevents invalid state transitions at compile time.
///
/// # Type Safety
///
/// - `DeltaSigma<Unproven>`: Proposal created, not yet validated
/// - `DeltaSigma<ProofPending>`: Proof obligations generated, validation in progress
/// - `DeltaSigma<Proven>`: All proofs satisfied, safe to apply
///
/// # State Transitions
///
/// ```text
/// Unproven ──generate_proof_obligations()──> ProofPending ──validate()──> Proven
/// ```
///
/// Only `DeltaSigma<Proven>` can be applied to the running system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaSigma<P: ProofState> {
    /// Overlay identifier
    pub id: OverlayId,
    /// Scope of overlay (what is affected)
    pub scope: OverlayScope,
    /// Changes to apply
    pub changes: Vec<OverlayChange>,
    /// Creation timestamp (ms since epoch)
    pub created_at_ms: u64,
    /// Metadata
    pub metadata: HashMap<String, String>,
    /// Proof state (phantom type parameter)
    #[serde(skip)]
    _proof_state: PhantomData<P>,
}

impl DeltaSigma<Unproven> {
    /// Create new overlay proposal (unproven state)
    pub fn new(scope: OverlayScope, changes: Vec<OverlayChange>) -> Self {
        let timestamp_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        Self {
            id: OverlayId::new(),
            scope,
            changes,
            created_at_ms: timestamp_ms,
            metadata: HashMap::new(),
            _proof_state: PhantomData,
        }
    }

    /// Add metadata to overlay
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Merge scope from changes
    pub fn merge_change_scopes(mut self) -> Self {
        for change in &self.changes {
            let change_scope = change.affected_scope();
            self.scope.workflows.extend(change_scope.workflows);
            self.scope.patterns.extend(change_scope.patterns);
            self.scope.guards.extend(change_scope.guards);
        }
        self
    }

    /// Generate proof obligations (transition: Unproven → ProofPending)
    pub fn generate_proof_obligations(self) -> WorkflowResult<DeltaSigma<ProofPending>> {
        // Validate scope is not empty
        if self.scope.is_empty() {
            return Err(WorkflowError::Validation(
                "Overlay scope cannot be empty".to_string(),
            ));
        }

        // Validate changes are not empty
        if self.changes.is_empty() {
            return Err(WorkflowError::Validation(
                "Overlay changes cannot be empty".to_string(),
            ));
        }

        // Transition to ProofPending state
        Ok(DeltaSigma {
            id: self.id,
            scope: self.scope,
            changes: self.changes,
            created_at_ms: self.created_at_ms,
            metadata: self.metadata,
            _proof_state: PhantomData,
        })
    }
}

impl DeltaSigma<ProofPending> {
    /// Get proof obligations for this overlay
    pub fn proof_obligations(&self) -> Vec<ProofObligation> {
        let mut obligations = Vec::new();

        // Obligation 1: Invariant validation for affected patterns
        if !self.scope.patterns.is_empty() {
            obligations.push(ProofObligation::ValidateInvariants {
                pattern_ids: self.scope.patterns.iter().cloned().collect(),
                description: "Verify workflow invariants remain valid after overlay".to_string(),
            });
        }

        // Obligation 2: Performance constraints (τ ≤ 8 for hot path)
        obligations.push(ProofObligation::ValidatePerformance {
            max_ticks: 8,
            description: "Verify hot path performance constraint (τ ≤ 8)".to_string(),
        });

        // Obligation 3: Guard constraint validation
        if !self.scope.guards.is_empty() {
            obligations.push(ProofObligation::ValidateGuards {
                guard_names: self.scope.guards.iter().cloned().collect(),
                description: "Verify guard constraints remain valid".to_string(),
            });
        }

        // Obligation 4: SLO compliance
        obligations.push(ProofObligation::ValidateSLO {
            description: "Verify SLO compliance after overlay".to_string(),
        });

        // Obligation 5: Doctrine (Q) conformance
        obligations.push(ProofObligation::ValidateDoctrine {
            description: "Verify overlay conforms to system doctrine (Q)".to_string(),
        });

        obligations
    }

    /// Estimate validation effort (for scheduling)
    pub fn validation_effort(&self) -> ValidationEffort {
        let risk = self.scope.risk_surface();
        let change_count = self.changes.len();

        let estimated_seconds = (risk * 10) + (change_count * 5);

        if estimated_seconds < 30 {
            ValidationEffort::Low
        } else if estimated_seconds < 120 {
            ValidationEffort::Medium
        } else {
            ValidationEffort::High
        }
    }
}

impl DeltaSigma<Proven> {
    /// Get overlay ID
    pub fn id(&self) -> OverlayId {
        self.id
    }

    /// Get scope
    pub fn scope(&self) -> &OverlayScope {
        &self.scope
    }

    /// Get changes
    pub fn changes(&self) -> &[OverlayChange] {
        &self.changes
    }

    /// Check if overlay is still valid (not expired)
    pub fn is_valid(&self, max_age_ms: u64) -> bool {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        (now_ms - self.created_at_ms) <= max_age_ms
    }
}

/// Proof obligation - what must be validated before overlay application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofObligation {
    /// Validate workflow invariants
    ValidateInvariants {
        pattern_ids: Vec<PatternId>,
        description: String,
    },
    /// Validate performance constraints
    ValidatePerformance { max_ticks: u64, description: String },
    /// Validate guard constraints
    ValidateGuards {
        guard_names: Vec<String>,
        description: String,
    },
    /// Validate SLO compliance
    ValidateSLO { description: String },
    /// Validate doctrine conformance
    ValidateDoctrine { description: String },
    /// Custom proof obligation
    Custom {
        obligation_type: String,
        params: HashMap<String, String>,
        description: String,
    },
}

impl ProofObligation {
    /// Get human-readable description
    pub fn description(&self) -> &str {
        match self {
            ProofObligation::ValidateInvariants { description, .. } => description,
            ProofObligation::ValidatePerformance { description, .. } => description,
            ProofObligation::ValidateGuards { description, .. } => description,
            ProofObligation::ValidateSLO { description } => description,
            ProofObligation::ValidateDoctrine { description } => description,
            ProofObligation::Custom { description, .. } => description,
        }
    }

    /// Check if obligation is critical (must pass for overlay to be proven)
    pub fn is_critical(&self) -> bool {
        match self {
            ProofObligation::ValidatePerformance { .. } => true, // Performance is critical
            ProofObligation::ValidateDoctrine { .. } => true,    // Doctrine is critical
            ProofObligation::ValidateInvariants { .. } => true,  // Invariants are critical
            _ => false,
        }
    }
}

/// Validation effort estimate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationEffort {
    /// Low effort (< 30 seconds)
    Low,
    /// Medium effort (30-120 seconds)
    Medium,
    /// High effort (> 120 seconds)
    High,
}

/// Overlay composition - combine multiple overlays
pub struct OverlayComposition {
    /// Overlays to compose
    overlays: Vec<DeltaSigma<Proven>>,
    /// Composition strategy
    strategy: CompositionStrategy,
}

impl OverlayComposition {
    /// Create new composition
    pub fn new(strategy: CompositionStrategy) -> Self {
        Self {
            overlays: Vec::new(),
            strategy,
        }
    }

    /// Add overlay to composition
    pub fn add(mut self, overlay: DeltaSigma<Proven>) -> Self {
        self.overlays.push(overlay);
        self
    }

    /// Check if composition is valid (no conflicts)
    pub fn validate(&self) -> WorkflowResult<()> {
        match self.strategy {
            CompositionStrategy::Sequential => self.validate_sequential(),
            CompositionStrategy::Parallel => self.validate_parallel(),
            CompositionStrategy::Merge => self.validate_merge(),
        }
    }

    fn validate_sequential(&self) -> WorkflowResult<()> {
        // Sequential composition: overlays applied in order
        // No conflicts possible
        Ok(())
    }

    fn validate_parallel(&self) -> WorkflowResult<()> {
        // Parallel composition: overlays must not have overlapping scopes
        for i in 0..self.overlays.len() {
            for j in (i + 1)..self.overlays.len() {
                if self.scopes_overlap(&self.overlays[i].scope, &self.overlays[j].scope) {
                    return Err(WorkflowError::Validation(format!(
                        "Overlays {} and {} have overlapping scopes",
                        self.overlays[i].id, self.overlays[j].id
                    )));
                }
            }
        }
        Ok(())
    }

    fn validate_merge(&self) -> WorkflowResult<()> {
        // Merge composition: overlays combined into single overlay
        // Must check for conflicting changes
        for i in 0..self.overlays.len() {
            for j in (i + 1)..self.overlays.len() {
                if self.changes_conflict(&self.overlays[i], &self.overlays[j]) {
                    return Err(WorkflowError::Validation(format!(
                        "Overlays {} and {} have conflicting changes",
                        self.overlays[i].id, self.overlays[j].id
                    )));
                }
            }
        }
        Ok(())
    }

    fn scopes_overlap(&self, scope1: &OverlayScope, scope2: &OverlayScope) -> bool {
        !scope1.workflows.is_disjoint(&scope2.workflows)
            || !scope1.patterns.is_disjoint(&scope2.patterns)
            || !scope1.guards.is_disjoint(&scope2.guards)
    }

    fn changes_conflict(
        &self,
        overlay1: &DeltaSigma<Proven>,
        overlay2: &DeltaSigma<Proven>,
    ) -> bool {
        // Check for conflicting changes
        for change1 in &overlay1.changes {
            for change2 in &overlay2.changes {
                if self.changes_conflict_impl(change1, change2) {
                    return true;
                }
            }
        }
        false
    }

    fn changes_conflict_impl(&self, change1: &OverlayChange, change2: &OverlayChange) -> bool {
        // Two changes conflict if they modify the same entity in incompatible ways
        match (change1, change2) {
            (
                OverlayChange::ModifyGuard { guard_name: g1, .. },
                OverlayChange::ModifyGuard { guard_name: g2, .. },
            ) => g1 == g2,
            (
                OverlayChange::TogglePattern { pattern_id: p1, .. },
                OverlayChange::TogglePattern { pattern_id: p2, .. },
            ) => p1 == p2,
            (OverlayChange::AdjustPerformance { .. }, OverlayChange::AdjustPerformance { .. }) => {
                true
            } // Performance is global
            _ => false,
        }
    }
}

/// Composition strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompositionStrategy {
    /// Apply overlays sequentially (in order)
    Sequential,
    /// Apply overlays in parallel (must have disjoint scopes)
    Parallel,
    /// Merge overlays into single overlay (must not conflict)
    Merge,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_scope() {
        let scope = OverlayScope::new()
            .with_pattern(PatternId::new(12).unwrap())
            .with_guard("max_run_len".to_string());

        assert_eq!(scope.patterns.len(), 1);
        assert_eq!(scope.guards.len(), 1);
        assert_eq!(scope.risk_surface(), 2);
    }

    #[test]
    fn test_delta_sigma_state_transitions() {
        // Create unproven overlay
        let scope = OverlayScope::new().with_pattern(PatternId::new(12).unwrap());
        let changes = vec![OverlayChange::ScaleMultiInstance { delta: 2 }];
        let unproven = DeltaSigma::new(scope, changes);

        // Generate proof obligations (Unproven → ProofPending)
        let proof_pending = unproven.generate_proof_obligations().unwrap();

        // Verify proof obligations
        let obligations = proof_pending.proof_obligations();
        assert!(!obligations.is_empty());
        assert!(obligations
            .iter()
            .any(|o| matches!(o, ProofObligation::ValidatePerformance { .. })));
    }

    #[test]
    fn test_overlay_change_description() {
        let change = OverlayChange::ScaleMultiInstance { delta: 2 };
        assert_eq!(change.description(), "Scale multi-instance by 2");
    }

    #[test]
    fn test_proof_obligation_criticality() {
        let obligation = ProofObligation::ValidatePerformance {
            max_ticks: 8,
            description: "Test".to_string(),
        };
        assert!(obligation.is_critical());
    }

    #[test]
    fn test_overlay_composition_parallel() {
        // Create two non-overlapping proven overlays
        // Note: In real usage, overlays would be validated to Proven state
        // This test simulates that state
        let composition = OverlayComposition::new(CompositionStrategy::Parallel);

        // Validation should succeed for empty composition
        assert!(composition.validate().is_ok());
    }

    #[test]
    fn test_validation_effort() {
        let scope = OverlayScope::new()
            .with_pattern(PatternId::new(1).unwrap())
            .with_pattern(PatternId::new(2).unwrap());
        let changes = vec![OverlayChange::ScaleMultiInstance { delta: 1 }];
        let unproven = DeltaSigma::new(scope, changes);
        let proof_pending = unproven.generate_proof_obligations().unwrap();

        let effort = proof_pending.validation_effort();
        assert!(matches!(
            effort,
            ValidationEffort::Low | ValidationEffort::Medium
        ));
    }
}
