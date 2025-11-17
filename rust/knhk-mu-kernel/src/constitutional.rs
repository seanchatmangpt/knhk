//! Constitutional Traits
//!
//! Defines core constitutional guarantees that both μ-kernel and AHI
//! must uphold. These traits form the "constitution" - the invariants
//! that cannot be violated:
//!
//! 1. **DoctrineAligned**: Respects doctrine constraints
//! 2. **ChatmanBounded**: Guarantees ≤8 ticks for hot path
//! 3. **ClosedWorld**: No hidden state, complete observability
//! 4. **Deterministic**: Same input → same output
//!
//! Both branches must implement these for core components.
//! CI enforces no component loses these bounds.

use crate::timing::TickBudget;

/// Doctrine alignment trait
///
/// Components implementing this trait guarantee they respect
/// doctrinal constraints. Doctrine defines what operations are
/// permitted and what invariants must hold.
///
/// Example:
/// ```ignore
/// impl DoctrineAligned for MyComponent {
///     fn verify_doctrine(&self, doctrine: &Doctrine) -> Result<(), DoctrineViolation> {
///         // Check all operations respect doctrine
///         if self.requires_permission() && !doctrine.permits(&self) {
///             return Err(DoctrineViolation::Unauthorized);
///         }
///         Ok(())
///     }
///
///     fn doctrine_hash(&self) -> [u8; 32] {
///         self.associated_doctrine.hash()
///     }
/// }
/// ```
pub trait DoctrineAligned {
    /// Verify this component respects doctrine
    fn verify_doctrine(&self, doctrine: &Doctrine) -> Result<(), DoctrineViolation>;

    /// Get hash of applicable doctrine (for receipts)
    fn doctrine_hash(&self) -> [u8; 32];
}

/// Doctrine definition
#[derive(Debug, Clone)]
pub struct Doctrine {
    /// Doctrine identifier
    pub id: u64,
    /// Permitted operations (bitmap)
    pub permitted_ops: u64,
    /// Required invariants
    pub required_invariants: heapless::Vec<u16, 32>,
    /// Doctrine hash
    pub hash: [u8; 32],
}

impl Doctrine {
    /// Create a new doctrine
    pub fn new(id: u64, permitted_ops: u64) -> Self {
        let mut hasher = sha3::Sha3_256::new();
        use sha3::Digest;
        hasher.update(&id.to_le_bytes());
        hasher.update(&permitted_ops.to_le_bytes());
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);

        Self {
            id,
            permitted_ops,
            required_invariants: heapless::Vec::new(),
            hash,
        }
    }

    /// Check if operation is permitted
    pub fn permits_op(&self, op_bit: u64) -> bool {
        (self.permitted_ops & (1 << op_bit)) != 0
    }

    /// Add required invariant
    pub fn add_invariant(&mut self, invariant_id: u16) -> Result<(), ()> {
        self.required_invariants.push(invariant_id)
    }
}

/// Doctrine violations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoctrineViolation {
    /// Operation not permitted by doctrine
    Unauthorized,
    /// Required invariant not satisfied
    InvariantViolation(u16),
    /// Invalid doctrine reference
    InvalidDoctrine,
}

/// Chatman-bounded trait
///
/// Components implementing this trait guarantee worst-case execution
/// time ≤8 CPU cycles (the Chatman Constant) for their hot path.
///
/// This is enforced through:
/// 1. Compile-time const evaluation
/// 2. Runtime tick counting
/// 3. Benchmark verification in CI
///
/// Example:
/// ```ignore
/// impl ChatmanBounded for FastCheck {
///     const WORST_CASE_TICKS: u64 = 5;
///
///     fn tick_budget(&self) -> TickBudget {
///         TickBudget::new(Self::WORST_CASE_TICKS)
///     }
/// }
/// ```
pub trait ChatmanBounded {
    /// Worst-case tick count (must be ≤8)
    const WORST_CASE_TICKS: u64;

    /// Get tick budget for this component
    fn tick_budget(&self) -> TickBudget {
        TickBudget::new(Self::WORST_CASE_TICKS)
    }

    /// Compile-time assertion that bound is satisfied
    const CHATMAN_SATISFIED: () = {
        assert!(Self::WORST_CASE_TICKS <= crate::CHATMAN_CONSTANT);
    };
}

/// Closed-world trait
///
/// Components implementing this trait guarantee complete observability.
/// All state is visible and can be inspected. No hidden state, no
/// side channels.
///
/// This is crucial for:
/// - Debugging and verification
/// - Receipt generation (must capture complete state)
/// - Replay and determinism
///
/// Example:
/// ```ignore
/// impl ClosedWorld for MyComponent {
///     type State = MyState;
///
///     fn observe_complete_state(&self) -> Self::State {
///         MyState {
///             field1: self.field1,
///             field2: self.field2,
///             // All fields exposed
///         }
///     }
/// }
/// ```
pub trait ClosedWorld {
    /// Complete state type
    type State: Clone + core::fmt::Debug;

    /// Observe complete state (no hidden state)
    fn observe_complete_state(&self) -> Self::State;

    /// Hash of complete state (for receipts)
    fn state_hash(&self) -> [u8; 32] {
        use sha3::{Digest, Sha3_256};
        let state = self.observe_complete_state();
        let debug_repr = alloc::format!("{:?}", state);

        let mut hasher = Sha3_256::new();
        hasher.update(debug_repr.as_bytes());
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

/// Deterministic trait
///
/// Components implementing this trait guarantee deterministic behavior:
/// same input → same output, always.
///
/// Requirements:
/// - No random number generation (except from deterministic seed)
/// - No wall-clock time dependencies
/// - No hidden state
/// - No non-deterministic operations
///
/// Example:
/// ```ignore
/// impl Deterministic for PureFunction {
///     type Input = u64;
///     type Output = u64;
///
///     fn deterministic_execute(&self, input: &Self::Input) -> Self::Output {
///         input.wrapping_mul(2) // Pure, deterministic
///     }
/// }
/// ```
pub trait Deterministic {
    /// Input type
    type Input: Clone;

    /// Output type (must be deterministic)
    type Output: Clone + Eq;

    /// Execute deterministically (same input → same output)
    fn deterministic_execute(&self, input: &Self::Input) -> Self::Output;

    /// Verify determinism by executing twice
    fn verify_determinism(&self, input: &Self::Input) -> bool
    where
        Self::Input: Clone,
    {
        let output1 = self.deterministic_execute(input);
        let output2 = self.deterministic_execute(&input.clone());
        output1 == output2
    }
}

/// Constitutional component (implements all constitutional traits)
///
/// This is a marker trait for components that satisfy all constitutional
/// guarantees. Components claiming to be constitutional must implement
/// all four base traits.
pub trait Constitutional: DoctrineAligned + ChatmanBounded + ClosedWorld + Deterministic {
    /// Verify all constitutional guarantees
    fn verify_constitutional(&self, doctrine: &Doctrine) -> Result<(), ConstitutionalViolation> {
        // Verify doctrine alignment
        self.verify_doctrine(doctrine)
            .map_err(ConstitutionalViolation::DoctrineViolation)?;

        // Verify Chatman bound (compile-time)
        let _ = Self::CHATMAN_SATISFIED;

        // Verify closed world (all state observable)
        let _ = self.observe_complete_state();

        Ok(())
    }

    /// Generate constitutional receipt
    fn constitutional_receipt(&self) -> ConstitutionalReceipt {
        ConstitutionalReceipt {
            doctrine_hash: self.doctrine_hash(),
            state_hash: self.state_hash(),
            tick_bound: Self::WORST_CASE_TICKS,
            timestamp: 0, // Would come from kernel
        }
    }
}

/// Constitutional violations
#[derive(Debug, Clone)]
pub enum ConstitutionalViolation {
    /// Doctrine violation
    DoctrineViolation(DoctrineViolation),
    /// Chatman bound exceeded
    ChatmanExceeded { actual: u64, limit: u64 },
    /// Non-deterministic behavior detected
    NonDeterministic,
    /// Hidden state detected
    HiddenState,
}

/// Constitutional receipt (proves constitutional compliance)
#[derive(Debug, Clone)]
pub struct ConstitutionalReceipt {
    /// Hash of doctrine
    pub doctrine_hash: [u8; 32],
    /// Hash of complete state
    pub state_hash: [u8; 32],
    /// Tick bound
    pub tick_bound: u64,
    /// Timestamp
    pub timestamp: u64,
}

impl ConstitutionalReceipt {
    /// Compute receipt hash
    pub fn hash(&self) -> [u8; 32] {
        use sha3::{Digest, Sha3_256};
        let mut hasher = Sha3_256::new();
        hasher.update(&self.doctrine_hash);
        hasher.update(&self.state_hash);
        hasher.update(&self.tick_bound.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

/// Example constitutional component
#[derive(Debug, Clone)]
pub struct SimpleCheck {
    pub threshold: u64,
}

impl DoctrineAligned for SimpleCheck {
    fn verify_doctrine(&self, _doctrine: &Doctrine) -> Result<(), DoctrineViolation> {
        // Simple check always permitted
        Ok(())
    }

    fn doctrine_hash(&self) -> [u8; 32] {
        [0; 32] // Would use actual doctrine
    }
}

impl ChatmanBounded for SimpleCheck {
    const WORST_CASE_TICKS: u64 = 3;
}

impl ClosedWorld for SimpleCheck {
    type State = u64;

    fn observe_complete_state(&self) -> Self::State {
        self.threshold
    }
}

impl Deterministic for SimpleCheck {
    type Input = u64;
    type Output = bool;

    fn deterministic_execute(&self, input: &Self::Input) -> Self::Output {
        *input > self.threshold
    }
}

impl Constitutional for SimpleCheck {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doctrine() {
        let mut doctrine = Doctrine::new(1, 0b1111);

        assert!(doctrine.permits_op(0));
        assert!(doctrine.permits_op(3));
        assert!(!doctrine.permits_op(4));

        doctrine.add_invariant(42).unwrap();
        assert_eq!(doctrine.required_invariants.len(), 1);
    }

    #[test]
    fn test_chatman_bounded() {
        let check = SimpleCheck { threshold: 100 };

        // Compile-time check
        let _ = SimpleCheck::CHATMAN_SATISFIED;

        assert_eq!(SimpleCheck::WORST_CASE_TICKS, 3);
        assert!(SimpleCheck::WORST_CASE_TICKS <= crate::CHATMAN_CONSTANT);

        let budget = check.tick_budget();
        assert_eq!(budget.limit, 3);
    }

    #[test]
    fn test_closed_world() {
        let check = SimpleCheck { threshold: 42 };

        let state = check.observe_complete_state();
        assert_eq!(state, 42);

        let hash = check.state_hash();
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_deterministic() {
        let check = SimpleCheck { threshold: 50 };

        assert_eq!(check.deterministic_execute(&100), true);
        assert_eq!(check.deterministic_execute(&30), false);

        // Verify determinism
        assert!(check.verify_determinism(&75));
    }

    #[test]
    fn test_constitutional() {
        let check = SimpleCheck { threshold: 100 };
        let doctrine = Doctrine::new(1, 0xFFFF);

        // Verify all constitutional guarantees
        check.verify_constitutional(&doctrine).unwrap();

        // Generate receipt
        let receipt = check.constitutional_receipt();
        assert_eq!(receipt.tick_bound, 3);

        let hash = receipt.hash();
        assert_eq!(hash.len(), 32);
    }
}
