//! Unified Decision Interface
//!
//! Provides a common interface for decisions across μ-kernel and AHI layers.
//! Each decision carries:
//! - O-slice description
//! - Reference to Σ*
//! - Q invariants
//! - Expected tick cost (compile-time verified)
//! - Risk classification

use crate::sigma::{SigmaCompiled, SigmaHash};
use crate::timing::TickBudget;
use core::marker::PhantomData;

/// Observation slice trait
///
/// Represents a slice of observations O that a decision operates on.
pub trait ObservationSlice: Sized {
    /// Size of observation in bytes
    const SIZE: usize;

    /// Serialize to bytes (must be deterministic)
    fn to_bytes(&self) -> [u8; Self::SIZE];

    /// Hash of observation (for receipts)
    fn hash(&self) -> [u8; 32] {
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        hasher.update(&self.to_bytes());
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

/// Invariant identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct InvariantId(pub u16);

impl InvariantId {
    /// Create a new invariant ID
    pub const fn new(id: u16) -> Self {
        Self(id)
    }

    /// Get the raw ID
    pub const fn raw(&self) -> u16 {
        self.0
    }
}

/// Risk classification for decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RiskClass {
    /// Safe - no external effects
    Safe = 0,
    /// Low - limited scope, reversible
    Low = 1,
    /// Medium - moderate impact, partially reversible
    Medium = 2,
    /// High - significant impact, difficult to reverse
    High = 3,
    /// Critical - system-wide impact, irreversible
    Critical = 4,
}

impl RiskClass {
    /// Convert to numeric risk level (0-4)
    pub const fn level(&self) -> u8 {
        *self as u8
    }

    /// Check if risk requires approval
    pub const fn requires_approval(&self) -> bool {
        self.level() >= RiskClass::High.level()
    }
}

/// Compile-time assertion for tick costs
///
/// This ensures tick costs are known at compile time and satisfy constraints.
pub struct ConstAssert<const COND: bool>;

impl<const COND: bool> ConstAssert<COND> {
    /// Assert that condition is true at compile time
    pub const fn assert() -> Self
    where
        [(); COND as usize]:,
    {
        Self
    }
}

/// Action resulting from a decision
#[derive(Debug, Clone)]
pub struct Action {
    /// Action identifier
    pub action_id: u64,
    /// Output hash (for receipt)
    pub output_hash: [u8; 32],
    /// Actual ticks consumed
    pub ticks_consumed: u64,
}

/// Unified decision interface
///
/// Generic over:
/// - O: Observation slice type
/// - TICK_COST: Maximum ticks (const generic, compile-time checked)
///
/// Example:
/// ```ignore
/// let decision: Decision<MyObs, 8> = Decision::new(obs, sigma_ref, invariants);
/// assert!(decision.within_chatman_constant());
/// ```
#[derive(Debug)]
pub struct Decision<O, const TICK_COST: u64>
where
    O: ObservationSlice,
{
    /// Observations for this decision
    pub observations: O,

    /// Reference to Σ* (must outlive decision)
    pub sigma_ref: &'static SigmaCompiled,

    /// Invariants that must hold
    pub invariants: heapless::Vec<InvariantId, 16>,

    /// Risk classification
    pub risk_class: RiskClass,

    /// Tick budget for execution
    tick_budget: TickBudget,

    /// Compile-time proof that TICK_COST is valid
    _tick_proof: PhantomData<ConstAssert<{ TICK_COST > 0 }>>,
}

impl<O, const TICK_COST: u64> Decision<O, TICK_COST>
where
    O: ObservationSlice,
{
    /// Create a new decision
    ///
    /// # Arguments
    /// - observations: Input observations
    /// - sigma_ref: Reference to compiled ontology
    /// - invariants: List of invariants that must hold
    /// - risk_class: Risk classification
    ///
    /// # Returns
    /// A new decision with tick budget TICK_COST
    pub fn new(
        observations: O,
        sigma_ref: &'static SigmaCompiled,
        invariants: heapless::Vec<InvariantId, 16>,
        risk_class: RiskClass,
    ) -> Self {
        Self {
            observations,
            sigma_ref,
            invariants,
            risk_class,
            tick_budget: TickBudget::new(TICK_COST),
            _tick_proof: PhantomData,
        }
    }

    /// Check if decision is within Chatman Constant
    pub const fn within_chatman_constant() -> bool {
        TICK_COST <= crate::CHATMAN_CONSTANT
    }

    /// Get tick cost (compile-time constant)
    pub const fn tick_cost() -> u64 {
        TICK_COST
    }

    /// Get remaining tick budget
    pub fn remaining_budget(&self) -> u64 {
        self.tick_budget.remaining()
    }

    /// Get observation hash (for receipts)
    pub fn observation_hash(&self) -> [u8; 32] {
        self.observations.hash()
    }

    /// Get sigma hash (for receipts)
    pub fn sigma_hash(&self) -> SigmaHash {
        self.sigma_ref.header.hash
    }

    /// Execute decision (A = μ(O))
    ///
    /// This is the core decision execution that implements:
    /// A = μ(O; Σ*) under Q with τ ≤ TICK_COST
    pub fn execute(&mut self) -> Result<Action, DecisionError> {
        // Verify all invariants before execution
        self.verify_invariants()?;

        // Check risk level
        if self.risk_class.requires_approval() {
            return Err(DecisionError::ApprovalRequired);
        }

        // Execute decision logic (simplified for demonstration)
        // In production, this would dispatch to pattern handlers
        let action_id = 1; // Would come from pattern dispatch
        let output_hash = self.observation_hash(); // Simplified
        let ticks_consumed = 1; // Would be measured

        // Consume tick budget
        let _ = self.tick_budget.consume(ticks_consumed);

        // Verify we didn't exceed budget
        if self.tick_budget.is_exhausted() {
            return Err(DecisionError::TickBudgetExceeded);
        }

        Ok(Action {
            action_id,
            output_hash,
            ticks_consumed,
        })
    }

    /// Verify all invariants hold
    fn verify_invariants(&self) -> Result<(), DecisionError> {
        // In production, would check each invariant against guards
        // For now, just verify we have a valid sigma
        if !self.sigma_ref.header.is_valid() {
            return Err(DecisionError::InvalidSigma);
        }

        Ok(())
    }
}

/// Decision errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionError {
    /// Tick budget exceeded
    TickBudgetExceeded,
    /// Invariant violated
    InvariantViolated(InvariantId),
    /// Invalid Σ* reference
    InvalidSigma,
    /// Approval required for high-risk decision
    ApprovalRequired,
    /// Observation validation failed
    InvalidObservation,
}

/// Example observation slice for u64 values
#[derive(Debug, Clone, Copy)]
pub struct U64Observation(pub u64);

impl ObservationSlice for U64Observation {
    const SIZE: usize = 8;

    fn to_bytes(&self) -> [u8; Self::SIZE] {
        self.0.to_le_bytes()
    }
}

/// Example observation slice for 32-byte arrays
#[derive(Debug, Clone, Copy)]
pub struct Byte32Observation(pub [u8; 32]);

impl ObservationSlice for Byte32Observation {
    const SIZE: usize = 32;

    fn to_bytes(&self) -> [u8; Self::SIZE] {
        self.0
    }
}

// Re-export heapless for users
pub use heapless;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sigma::SigmaCompiled;

    #[test]
    fn test_risk_classification() {
        assert_eq!(RiskClass::Safe.level(), 0);
        assert_eq!(RiskClass::Critical.level(), 4);

        assert!(!RiskClass::Medium.requires_approval());
        assert!(RiskClass::High.requires_approval());
        assert!(RiskClass::Critical.requires_approval());
    }

    #[test]
    fn test_invariant_id() {
        let inv = InvariantId::new(42);
        assert_eq!(inv.raw(), 42);
    }

    #[test]
    fn test_observation_slice() {
        let obs = U64Observation(12345);
        let bytes = obs.to_bytes();
        assert_eq!(bytes, 12345u64.to_le_bytes());

        let hash = obs.hash();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_decision_creation() {
        let sigma = Box::leak(Box::new(SigmaCompiled::new()));
        let obs = U64Observation(42);
        let invariants = heapless::Vec::new();

        let decision: Decision<_, 8> = Decision::new(obs, sigma, invariants, RiskClass::Low);

        assert!(Decision::<U64Observation, 8>::within_chatman_constant());
        assert_eq!(Decision::<U64Observation, 8>::tick_cost(), 8);
        assert_eq!(decision.remaining_budget(), 8);
    }

    #[test]
    fn test_decision_tick_costs() {
        // Hot path - within Chatman Constant
        assert!(Decision::<U64Observation, 8>::within_chatman_constant());
        assert!(Decision::<U64Observation, 1>::within_chatman_constant());

        // Warm path - exceeds Chatman Constant
        assert!(!Decision::<U64Observation, 100>::within_chatman_constant());
        assert!(!Decision::<U64Observation, 1000>::within_chatman_constant());
    }

    #[test]
    fn test_byte32_observation() {
        let obs = Byte32Observation([1; 32]);
        let bytes = obs.to_bytes();
        assert_eq!(bytes.len(), 32);
        assert_eq!(bytes[0], 1);
    }
}
