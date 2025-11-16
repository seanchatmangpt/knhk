//! Policy Lattice Kernel for MAPE-K Framework
//!
//! Implements a **doctrine-bound policy lattice** for formal constraint enforcement
//! in autonomic adaptations. Based on lattice theory, this module provides:
//!
//! - **Policy Atoms**: Strongly-typed primitives (LatencyBound, FailureRateBound, etc.)
//! - **Lattice Operations**: Meet (⊓), Join (⊔), partial ordering
//! - **Bottom Element (⊥)**: Represents "no actions allowed"
//! - **Zero-cost abstractions**: Compile-time checked bounds where possible
//!
//! # Lattice Theory Background
//!
//! A lattice (L, ≤, ⊓, ⊔) is a partially ordered set where every two elements have:
//! - A **meet (⊓)**: Greatest lower bound (stricter policy)
//! - A **join (⊔)**: Least upper bound (most permissive valid policy)
//!
//! **Laws satisfied**:
//! - Commutativity: a ⊓ b = b ⊓ a, a ⊔ b = b ⊔ a
//! - Associativity: (a ⊓ b) ⊓ c = a ⊓ (b ⊓ c)
//! - Idempotence: a ⊓ a = a, a ⊔ a = a
//! - Absorption: a ⊓ (a ⊔ b) = a, a ⊔ (a ⊓ b) = a
//!
//! # Integration with MAPE-K
//!
//! ```text
//! Plan Component → Generates actions with PolicyElement
//!                  ↓
//! Execute Component → Projects action against Doctrine (Q ∧ policy)
//!                  ↓
//! If policy' ≠ ⊥ → Execute action (lawful)
//! If policy' = ⊥ → Reject action (violates doctrine)
//! ```

use crate::error::{WorkflowError, WorkflowResult};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use uuid::Uuid;

// ============================================================================
// Policy Atom Types
// ============================================================================

/// Policy identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PolicyId(#[serde(with = "uuid::serde::compact")] pub Uuid);

impl PolicyId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PolicyId {
    fn default() -> Self {
        Self::new()
    }
}

/// Strictness level for policy enforcement
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Strictness {
    /// Soft constraint - can be violated if necessary
    Soft,
    /// Hard constraint - must never be violated
    Hard,
}

impl Strictness {
    /// Meet operation: stricter of two strictness levels
    pub fn meet(self, other: Self) -> Self {
        self.max(other)
    }

    /// Join operation: more relaxed of two strictness levels
    pub fn join(self, other: Self) -> Self {
        self.min(other)
    }
}

/// Latency bound policy atom
///
/// Constrains actions to maintain latency within bounds.
/// Forms a lattice ordered by target latency (lower target = stricter).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatencyBound {
    pub policy_id: PolicyId,
    /// Target P99 latency in milliseconds
    pub target_p99_ms: f64,
    /// Strictness of the bound
    pub strictness: Strictness,
}

impl LatencyBound {
    pub fn new(target_p99_ms: f64, strictness: Strictness) -> WorkflowResult<Self> {
        if target_p99_ms <= 0.0 {
            return Err(WorkflowError::Validation(
                "Latency bound must be positive".to_string(),
            ));
        }

        Ok(Self {
            policy_id: PolicyId::new(),
            target_p99_ms,
            strictness,
        })
    }

    /// Check if this bound is stricter than another (partial order)
    pub fn is_stricter_than(&self, other: &Self) -> bool {
        self.target_p99_ms < other.target_p99_ms
            || (self.target_p99_ms == other.target_p99_ms
                && self.strictness > other.strictness)
    }
}

/// Failure rate bound policy atom
///
/// Constrains actions to maintain error rate within acceptable limits.
/// Forms a lattice ordered by max error rate (lower rate = stricter).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FailureRateBound {
    pub policy_id: PolicyId,
    /// Maximum acceptable error rate (0.0 to 1.0)
    pub max_error_rate: f64,
}

impl FailureRateBound {
    pub fn new(max_error_rate: f64) -> WorkflowResult<Self> {
        if !(0.0..=1.0).contains(&max_error_rate) {
            return Err(WorkflowError::Validation(
                "Error rate must be between 0.0 and 1.0".to_string(),
            ));
        }

        Ok(Self {
            policy_id: PolicyId::new(),
            max_error_rate,
        })
    }

    /// Check if this bound is stricter than another
    pub fn is_stricter_than(&self, other: &Self) -> bool {
        self.max_error_rate < other.max_error_rate
    }
}

/// Guard strictness level policy atom
///
/// Controls how strictly guards are evaluated.
/// Forms a lattice with two elements: Relax < Tighten.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GuardStrictnessLevel {
    /// Relaxed guard evaluation
    Relax,
    /// Tightened guard evaluation
    Tighten,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuardStrictness {
    pub policy_id: PolicyId,
    pub level: GuardStrictnessLevel,
}

impl GuardStrictness {
    pub fn new(level: GuardStrictnessLevel) -> Self {
        Self {
            policy_id: PolicyId::new(),
            level,
        }
    }

    /// Check if this guard is stricter than another
    pub fn is_stricter_than(&self, other: &Self) -> bool {
        self.level > other.level
    }
}

/// Capacity envelope policy atom
///
/// Constrains maximum concurrency and parallelism.
/// Forms a lattice ordered by capacity limits (lower limit = stricter).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapacityEnvelope {
    pub policy_id: PolicyId,
    /// Maximum concurrent tasks
    pub max_concurrency: u32,
    /// Maximum parallel workers
    pub max_parallelism: u32,
}

impl CapacityEnvelope {
    pub fn new(max_concurrency: u32, max_parallelism: u32) -> WorkflowResult<Self> {
        if max_concurrency == 0 || max_parallelism == 0 {
            return Err(WorkflowError::Validation(
                "Capacity limits must be positive".to_string(),
            ));
        }

        Ok(Self {
            policy_id: PolicyId::new(),
            max_concurrency,
            max_parallelism,
        })
    }

    /// Check if this envelope is stricter than another
    pub fn is_stricter_than(&self, other: &Self) -> bool {
        self.max_concurrency < other.max_concurrency
            || self.max_parallelism < other.max_parallelism
    }
}

// ============================================================================
// Policy Element (Sum Type of All Atoms)
// ============================================================================

/// Policy element in the lattice (sum type over all policy atoms)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PolicyElement {
    /// Bottom element (⊥) - no actions allowed
    Bottom,
    /// Latency constraint
    Latency(LatencyBound),
    /// Failure rate constraint
    FailureRate(FailureRateBound),
    /// Guard strictness constraint
    GuardStrictness(GuardStrictness),
    /// Capacity constraint
    Capacity(CapacityEnvelope),
    /// Conjunction of multiple policies (intersection)
    Conjunction(Vec<PolicyElement>),
}

impl PolicyElement {
    /// Check if this is the bottom element
    pub fn is_bottom(&self) -> bool {
        matches!(self, PolicyElement::Bottom)
    }

    /// Check if this policy is stricter than another (partial order)
    pub fn is_stricter_than(&self, other: &Self) -> Option<bool> {
        match (self, other) {
            // Bottom is strictest (rejects everything)
            (PolicyElement::Bottom, PolicyElement::Bottom) => Some(false),
            (PolicyElement::Bottom, _) => Some(true),
            (_, PolicyElement::Bottom) => Some(false),

            // Same type comparisons
            (PolicyElement::Latency(a), PolicyElement::Latency(b)) => {
                Some(a.is_stricter_than(b))
            }
            (PolicyElement::FailureRate(a), PolicyElement::FailureRate(b)) => {
                Some(a.is_stricter_than(b))
            }
            (PolicyElement::GuardStrictness(a), PolicyElement::GuardStrictness(b)) => {
                Some(a.is_stricter_than(b))
            }
            (PolicyElement::Capacity(a), PolicyElement::Capacity(b)) => {
                Some(a.is_stricter_than(b))
            }

            // Conjunction comparison
            (PolicyElement::Conjunction(a), PolicyElement::Conjunction(b)) => {
                // A conjunction is stricter if it has more constraints
                // or if all corresponding elements are stricter
                if a.len() > b.len() {
                    Some(true)
                } else if a.len() < b.len() {
                    Some(false)
                } else {
                    // Compare element-wise
                    let all_stricter = a
                        .iter()
                        .zip(b.iter())
                        .all(|(x, y)| x.is_stricter_than(y).unwrap_or(false));
                    Some(all_stricter)
                }
            }

            // Different types: incomparable
            _ => None,
        }
    }
}

impl fmt::Display for PolicyElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PolicyElement::Bottom => write!(f, "⊥"),
            PolicyElement::Latency(l) => {
                write!(f, "Latency({}ms, {:?})", l.target_p99_ms, l.strictness)
            }
            PolicyElement::FailureRate(fr) => {
                write!(f, "FailureRate({})", fr.max_error_rate)
            }
            PolicyElement::GuardStrictness(g) => {
                write!(f, "GuardStrictness({:?})", g.level)
            }
            PolicyElement::Capacity(c) => {
                write!(
                    f,
                    "Capacity(concurrency={}, parallelism={})",
                    c.max_concurrency, c.max_parallelism
                )
            }
            PolicyElement::Conjunction(policies) => {
                write!(f, "Conjunction(")?;
                for (i, p) in policies.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ∧ ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ")")
            }
        }
    }
}

// ============================================================================
// Lattice Trait and Operations
// ============================================================================

/// Lattice trait for policy elements
///
/// Defines the algebraic structure required for policy composition.
pub trait Lattice: Sized {
    /// Meet operation (⊓): Greatest lower bound (stricter policy)
    ///
    /// Returns the strictest policy that satisfies both constraints.
    fn meet(&self, other: &Self) -> Self;

    /// Join operation (⊔): Least upper bound (most permissive valid policy)
    ///
    /// Returns the most permissive policy that is still stricter than both.
    fn join(&self, other: &Self) -> Self;

    /// Bottom element (⊥): Strictest possible policy (rejects all actions)
    fn bottom() -> Self;

    /// Partial order relation
    fn partial_cmp_lattice(&self, other: &Self) -> Option<Ordering>;
}

impl Lattice for LatencyBound {
    fn meet(&self, other: &Self) -> Self {
        // Meet = stricter bound (lower latency)
        let target_p99_ms = self.target_p99_ms.min(other.target_p99_ms);
        let strictness = self.strictness.meet(other.strictness);

        Self {
            policy_id: PolicyId::new(),
            target_p99_ms,
            strictness,
        }
    }

    fn join(&self, other: &Self) -> Self {
        // Join = more relaxed bound (higher latency)
        let target_p99_ms = self.target_p99_ms.max(other.target_p99_ms);
        let strictness = self.strictness.join(other.strictness);

        Self {
            policy_id: PolicyId::new(),
            target_p99_ms,
            strictness,
        }
    }

    fn bottom() -> Self {
        Self {
            policy_id: PolicyId::new(),
            target_p99_ms: 0.0,
            strictness: Strictness::Hard,
        }
    }

    fn partial_cmp_lattice(&self, other: &Self) -> Option<Ordering> {
        if self.is_stricter_than(other) {
            Some(Ordering::Less)
        } else if other.is_stricter_than(self) {
            Some(Ordering::Greater)
        } else if self.target_p99_ms == other.target_p99_ms
            && self.strictness == other.strictness
        {
            Some(Ordering::Equal)
        } else {
            None // Incomparable
        }
    }
}

impl Lattice for FailureRateBound {
    fn meet(&self, other: &Self) -> Self {
        // Meet = stricter bound (lower error rate)
        let max_error_rate = self.max_error_rate.min(other.max_error_rate);

        Self {
            policy_id: PolicyId::new(),
            max_error_rate,
        }
    }

    fn join(&self, other: &Self) -> Self {
        // Join = more relaxed bound (higher error rate)
        let max_error_rate = self.max_error_rate.max(other.max_error_rate);

        Self {
            policy_id: PolicyId::new(),
            max_error_rate,
        }
    }

    fn bottom() -> Self {
        Self {
            policy_id: PolicyId::new(),
            max_error_rate: 0.0,
        }
    }

    fn partial_cmp_lattice(&self, other: &Self) -> Option<Ordering> {
        if self.is_stricter_than(other) {
            Some(Ordering::Less)
        } else if other.is_stricter_than(self) {
            Some(Ordering::Greater)
        } else {
            Some(Ordering::Equal)
        }
    }
}

impl Lattice for GuardStrictness {
    fn meet(&self, other: &Self) -> Self {
        // Meet = stricter guard level
        let level = self.level.max(other.level);

        Self {
            policy_id: PolicyId::new(),
            level,
        }
    }

    fn join(&self, other: &Self) -> Self {
        // Join = more relaxed guard level
        let level = self.level.min(other.level);

        Self {
            policy_id: PolicyId::new(),
            level,
        }
    }

    fn bottom() -> Self {
        Self {
            policy_id: PolicyId::new(),
            level: GuardStrictnessLevel::Tighten,
        }
    }

    fn partial_cmp_lattice(&self, other: &Self) -> Option<Ordering> {
        Some(self.level.cmp(&other.level))
    }
}

impl Lattice for CapacityEnvelope {
    fn meet(&self, other: &Self) -> Self {
        // Meet = stricter capacity (lower limits)
        let max_concurrency = self.max_concurrency.min(other.max_concurrency);
        let max_parallelism = self.max_parallelism.min(other.max_parallelism);

        Self {
            policy_id: PolicyId::new(),
            max_concurrency,
            max_parallelism,
        }
    }

    fn join(&self, other: &Self) -> Self {
        // Join = more relaxed capacity (higher limits)
        let max_concurrency = self.max_concurrency.max(other.max_concurrency);
        let max_parallelism = self.max_parallelism.max(other.max_parallelism);

        Self {
            policy_id: PolicyId::new(),
            max_concurrency,
            max_parallelism,
        }
    }

    fn bottom() -> Self {
        Self {
            policy_id: PolicyId::new(),
            max_concurrency: 0,
            max_parallelism: 0,
        }
    }

    fn partial_cmp_lattice(&self, other: &Self) -> Option<Ordering> {
        if self.is_stricter_than(other) {
            Some(Ordering::Less)
        } else if other.is_stricter_than(self) {
            Some(Ordering::Greater)
        } else if self.max_concurrency == other.max_concurrency
            && self.max_parallelism == other.max_parallelism
        {
            Some(Ordering::Equal)
        } else {
            None // Incomparable
        }
    }
}

impl Lattice for PolicyElement {
    fn meet(&self, other: &Self) -> Self {
        match (self, other) {
            // Bottom absorbs everything
            (PolicyElement::Bottom, _) | (_, PolicyElement::Bottom) => PolicyElement::Bottom,

            // Same type: use type-specific meet
            (PolicyElement::Latency(a), PolicyElement::Latency(b)) => {
                PolicyElement::Latency(a.meet(b))
            }
            (PolicyElement::FailureRate(a), PolicyElement::FailureRate(b)) => {
                PolicyElement::FailureRate(a.meet(b))
            }
            (PolicyElement::GuardStrictness(a), PolicyElement::GuardStrictness(b)) => {
                PolicyElement::GuardStrictness(a.meet(b))
            }
            (PolicyElement::Capacity(a), PolicyElement::Capacity(b)) => {
                PolicyElement::Capacity(a.meet(b))
            }

            // Different types: create conjunction
            (PolicyElement::Conjunction(a), PolicyElement::Conjunction(b)) => {
                let mut combined = a.clone();
                combined.extend(b.clone());
                PolicyElement::Conjunction(combined)
            }
            (PolicyElement::Conjunction(a), other) => {
                let mut combined = a.clone();
                combined.push(other.clone());
                PolicyElement::Conjunction(combined)
            }
            (other, PolicyElement::Conjunction(b)) => {
                let mut combined = vec![other.clone()];
                combined.extend(b.clone());
                PolicyElement::Conjunction(combined)
            }
            (a, b) => PolicyElement::Conjunction(vec![a.clone(), b.clone()]),
        }
    }

    fn join(&self, other: &Self) -> Self {
        match (self, other) {
            // Bottom is identity for join
            (PolicyElement::Bottom, other) | (other, PolicyElement::Bottom) => other.clone(),

            // Same type: use type-specific join
            (PolicyElement::Latency(a), PolicyElement::Latency(b)) => {
                PolicyElement::Latency(a.join(b))
            }
            (PolicyElement::FailureRate(a), PolicyElement::FailureRate(b)) => {
                PolicyElement::FailureRate(a.join(b))
            }
            (PolicyElement::GuardStrictness(a), PolicyElement::GuardStrictness(b)) => {
                PolicyElement::GuardStrictness(a.join(b))
            }
            (PolicyElement::Capacity(a), PolicyElement::Capacity(b)) => {
                PolicyElement::Capacity(a.join(b))
            }

            // Conjunction: take most permissive (shorter conjunction)
            (PolicyElement::Conjunction(a), PolicyElement::Conjunction(b)) => {
                if a.len() <= b.len() {
                    PolicyElement::Conjunction(a.clone())
                } else {
                    PolicyElement::Conjunction(b.clone())
                }
            }

            // Different types: return more permissive (self for now)
            // In a full implementation, would need domain-specific logic
            (a, _) => a.clone(),
        }
    }

    fn bottom() -> Self {
        PolicyElement::Bottom
    }

    fn partial_cmp_lattice(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (PolicyElement::Bottom, PolicyElement::Bottom) => Some(Ordering::Equal),
            (PolicyElement::Bottom, _) => Some(Ordering::Less),
            (_, PolicyElement::Bottom) => Some(Ordering::Greater),

            (PolicyElement::Latency(a), PolicyElement::Latency(b)) => a.partial_cmp_lattice(b),
            (PolicyElement::FailureRate(a), PolicyElement::FailureRate(b)) => {
                a.partial_cmp_lattice(b)
            }
            (PolicyElement::GuardStrictness(a), PolicyElement::GuardStrictness(b)) => {
                a.partial_cmp_lattice(b)
            }
            (PolicyElement::Capacity(a), PolicyElement::Capacity(b)) => a.partial_cmp_lattice(b),

            // Conjunction ordering
            (PolicyElement::Conjunction(a), PolicyElement::Conjunction(b)) => {
                if a.len() < b.len() {
                    Some(Ordering::Greater)
                } else if a.len() > b.len() {
                    Some(Ordering::Less)
                } else {
                    Some(Ordering::Equal)
                }
            }

            // Different types are incomparable
            _ => None,
        }
    }
}

// ============================================================================
// Policy Lattice State
// ============================================================================

/// Policy lattice state
///
/// Maintains the current policy constraints in the knowledge base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyLattice {
    /// Current policy element
    pub current: PolicyElement,
    /// History of policy changes
    history: Vec<PolicyElement>,
    /// Maximum history size
    max_history: usize,
}

impl PolicyLattice {
    /// Create new policy lattice with no constraints (top element)
    pub fn new() -> Self {
        Self {
            current: PolicyElement::Conjunction(vec![]),
            history: Vec::new(),
            max_history: 1000,
        }
    }

    /// Strengthen policy (meet with new constraint)
    pub fn strengthen(&mut self, constraint: PolicyElement) {
        let new_policy = self.current.meet(&constraint);
        self.history.push(self.current.clone());
        self.current = new_policy;

        // Trim history
        if self.history.len() > self.max_history {
            self.history.drain(0..self.history.len() - self.max_history);
        }
    }

    /// Relax policy (join with less strict constraint)
    pub fn relax(&mut self, constraint: PolicyElement) {
        let new_policy = self.current.join(&constraint);
        self.history.push(self.current.clone());
        self.current = new_policy;

        // Trim history
        if self.history.len() > self.max_history {
            self.history.drain(0..self.history.len() - self.max_history);
        }
    }

    /// Check if policy is bottom (no actions allowed)
    pub fn is_bottom(&self) -> bool {
        self.current.is_bottom()
    }

    /// Get policy history
    pub fn get_history(&self) -> &[PolicyElement] {
        &self.history
    }

    /// Reset to no constraints
    pub fn reset(&mut self) {
        self.history.push(self.current.clone());
        self.current = PolicyElement::Conjunction(vec![]);
    }
}

impl Default for PolicyLattice {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_bound_creation() {
        let bound = LatencyBound::new(100.0, Strictness::Hard).unwrap();
        assert_eq!(bound.target_p99_ms, 100.0);
        assert_eq!(bound.strictness, Strictness::Hard);

        // Invalid bound
        assert!(LatencyBound::new(-10.0, Strictness::Soft).is_err());
    }

    #[test]
    fn test_latency_bound_ordering() {
        let strict = LatencyBound::new(50.0, Strictness::Hard).unwrap();
        let relaxed = LatencyBound::new(100.0, Strictness::Soft).unwrap();

        assert!(strict.is_stricter_than(&relaxed));
        assert!(!relaxed.is_stricter_than(&strict));
    }

    #[test]
    fn test_latency_bound_meet() {
        let a = LatencyBound::new(100.0, Strictness::Soft).unwrap();
        let b = LatencyBound::new(50.0, Strictness::Hard).unwrap();

        let meet = a.meet(&b);
        assert_eq!(meet.target_p99_ms, 50.0); // Stricter (lower)
        assert_eq!(meet.strictness, Strictness::Hard); // Stricter
    }

    #[test]
    fn test_latency_bound_join() {
        let a = LatencyBound::new(100.0, Strictness::Soft).unwrap();
        let b = LatencyBound::new(50.0, Strictness::Hard).unwrap();

        let join = a.join(&b);
        assert_eq!(join.target_p99_ms, 100.0); // More relaxed (higher)
        assert_eq!(join.strictness, Strictness::Soft); // More relaxed
    }

    #[test]
    fn test_failure_rate_bound() {
        let bound = FailureRateBound::new(0.05).unwrap();
        assert_eq!(bound.max_error_rate, 0.05);

        // Invalid bounds
        assert!(FailureRateBound::new(-0.1).is_err());
        assert!(FailureRateBound::new(1.5).is_err());
    }

    #[test]
    fn test_failure_rate_lattice() {
        let a = FailureRateBound::new(0.05).unwrap();
        let b = FailureRateBound::new(0.10).unwrap();

        let meet = a.meet(&b);
        assert_eq!(meet.max_error_rate, 0.05); // Stricter

        let join = a.join(&b);
        assert_eq!(join.max_error_rate, 0.10); // More relaxed
    }

    #[test]
    fn test_capacity_envelope() {
        let cap = CapacityEnvelope::new(10, 4).unwrap();
        assert_eq!(cap.max_concurrency, 10);
        assert_eq!(cap.max_parallelism, 4);

        // Invalid capacities
        assert!(CapacityEnvelope::new(0, 4).is_err());
        assert!(CapacityEnvelope::new(10, 0).is_err());
    }

    #[test]
    fn test_capacity_lattice() {
        let a = CapacityEnvelope::new(10, 4).unwrap();
        let b = CapacityEnvelope::new(20, 8).unwrap();

        let meet = a.meet(&b);
        assert_eq!(meet.max_concurrency, 10); // Stricter (lower)
        assert_eq!(meet.max_parallelism, 4);

        let join = a.join(&b);
        assert_eq!(join.max_concurrency, 20); // More relaxed (higher)
        assert_eq!(join.max_parallelism, 8);
    }

    #[test]
    fn test_policy_element_bottom() {
        let bottom = PolicyElement::bottom();
        assert!(bottom.is_bottom());

        let latency = PolicyElement::Latency(LatencyBound::new(100.0, Strictness::Soft).unwrap());
        assert!(!latency.is_bottom());
    }

    #[test]
    fn test_policy_element_meet_same_type() {
        let a = PolicyElement::Latency(LatencyBound::new(100.0, Strictness::Soft).unwrap());
        let b = PolicyElement::Latency(LatencyBound::new(50.0, Strictness::Hard).unwrap());

        let meet = a.meet(&b);
        match meet {
            PolicyElement::Latency(bound) => {
                assert_eq!(bound.target_p99_ms, 50.0);
                assert_eq!(bound.strictness, Strictness::Hard);
            }
            _ => panic!("Expected Latency policy"),
        }
    }

    #[test]
    fn test_policy_element_meet_different_types() {
        let latency = PolicyElement::Latency(LatencyBound::new(100.0, Strictness::Soft).unwrap());
        let failure = PolicyElement::FailureRate(FailureRateBound::new(0.05).unwrap());

        let meet = latency.meet(&failure);
        match meet {
            PolicyElement::Conjunction(policies) => {
                assert_eq!(policies.len(), 2);
            }
            _ => panic!("Expected Conjunction policy"),
        }
    }

    #[test]
    fn test_policy_element_bottom_absorption() {
        let bottom = PolicyElement::Bottom;
        let latency = PolicyElement::Latency(LatencyBound::new(100.0, Strictness::Soft).unwrap());

        let meet = bottom.meet(&latency);
        assert!(meet.is_bottom());

        let meet2 = latency.meet(&bottom);
        assert!(meet2.is_bottom());
    }

    #[test]
    fn test_policy_lattice_strengthen() {
        let mut lattice = PolicyLattice::new();

        let constraint = PolicyElement::Latency(LatencyBound::new(100.0, Strictness::Soft).unwrap());
        lattice.strengthen(constraint);

        assert!(!lattice.is_bottom());
        assert_eq!(lattice.get_history().len(), 1);
    }

    #[test]
    fn test_policy_lattice_relax() {
        let mut lattice = PolicyLattice::new();

        let strict = PolicyElement::Latency(LatencyBound::new(50.0, Strictness::Hard).unwrap());
        lattice.strengthen(strict);

        let relaxed = PolicyElement::Latency(LatencyBound::new(100.0, Strictness::Soft).unwrap());
        lattice.relax(relaxed);

        assert_eq!(lattice.get_history().len(), 2);
    }

    #[test]
    fn test_strictness_meet_join() {
        let soft = Strictness::Soft;
        let hard = Strictness::Hard;

        assert_eq!(soft.meet(hard), Strictness::Hard);
        assert_eq!(hard.meet(soft), Strictness::Hard);

        assert_eq!(soft.join(hard), Strictness::Soft);
        assert_eq!(hard.join(soft), Strictness::Soft);
    }

    #[test]
    fn test_guard_strictness() {
        let relax = GuardStrictness::new(GuardStrictnessLevel::Relax);
        let tighten = GuardStrictness::new(GuardStrictnessLevel::Tighten);

        assert!(tighten.is_stricter_than(&relax));
        assert!(!relax.is_stricter_than(&tighten));

        let meet = relax.meet(&tighten);
        assert_eq!(meet.level, GuardStrictnessLevel::Tighten);

        let join = relax.join(&tighten);
        assert_eq!(join.level, GuardStrictnessLevel::Relax);
    }

    // Lattice law tests (commutativity, associativity, idempotence)

    #[test]
    fn test_lattice_commutativity() {
        let a = LatencyBound::new(100.0, Strictness::Soft).unwrap();
        let b = LatencyBound::new(50.0, Strictness::Hard).unwrap();

        let meet_ab = a.meet(&b);
        let meet_ba = b.meet(&a);
        assert_eq!(meet_ab.target_p99_ms, meet_ba.target_p99_ms);
        assert_eq!(meet_ab.strictness, meet_ba.strictness);

        let join_ab = a.join(&b);
        let join_ba = b.join(&a);
        assert_eq!(join_ab.target_p99_ms, join_ba.target_p99_ms);
        assert_eq!(join_ab.strictness, join_ba.strictness);
    }

    #[test]
    fn test_lattice_associativity() {
        let a = LatencyBound::new(100.0, Strictness::Soft).unwrap();
        let b = LatencyBound::new(50.0, Strictness::Hard).unwrap();
        let c = LatencyBound::new(75.0, Strictness::Soft).unwrap();

        let meet_ab_c = a.meet(&b).meet(&c);
        let meet_a_bc = a.meet(&b.meet(&c));
        assert_eq!(meet_ab_c.target_p99_ms, meet_a_bc.target_p99_ms);

        let join_ab_c = a.join(&b).join(&c);
        let join_a_bc = a.join(&b.join(&c));
        assert_eq!(join_ab_c.target_p99_ms, join_a_bc.target_p99_ms);
    }

    #[test]
    fn test_lattice_idempotence() {
        let a = LatencyBound::new(100.0, Strictness::Soft).unwrap();

        let meet_aa = a.meet(&a);
        assert_eq!(meet_aa.target_p99_ms, a.target_p99_ms);
        assert_eq!(meet_aa.strictness, a.strictness);

        let join_aa = a.join(&a);
        assert_eq!(join_aa.target_p99_ms, a.target_p99_ms);
        assert_eq!(join_aa.strictness, a.strictness);
    }

    #[test]
    fn test_lattice_absorption() {
        let a = LatencyBound::new(100.0, Strictness::Soft).unwrap();
        let b = LatencyBound::new(50.0, Strictness::Hard).unwrap();

        // a ⊓ (a ⊔ b) = a
        let a_join_b = a.join(&b);
        let meet = a.meet(&a_join_b);
        assert_eq!(meet.target_p99_ms, a.target_p99_ms);

        // a ⊔ (a ⊓ b) = a
        let a_meet_b = a.meet(&b);
        let join = a.join(&a_meet_b);
        assert_eq!(join.target_p99_ms, a.target_p99_ms);
    }
}
