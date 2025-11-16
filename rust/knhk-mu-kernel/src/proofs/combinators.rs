//! Proof Combinators - Monad-Like Proof Composition
//!
//! This module provides combinator-based APIs for composing, transforming,
//! and manipulating proofs in a functional style.

use core::marker::PhantomData;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::collections::BTreeMap;
use super::phantom::{Proven, Predicate, Witness};

/// Result type for proof operations
pub type ProofResult<T, P> = Result<Proven<T, P>, ProofError>;

/// Errors that can occur during proof operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofError {
    /// The predicate doesn't hold
    PredicateFailed,
    /// An intermediate step failed
    CompositionFailed,
    /// The proof was invalidated
    ProofInvalidated,
    /// Custom error with message
    Custom(String),
}

impl core::fmt::Display for ProofError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ProofError::PredicateFailed => write!(f, "Predicate failed to hold"),
            ProofError::CompositionFailed => write!(f, "Proof composition failed"),
            ProofError::ProofInvalidated => write!(f, "Proof was invalidated"),
            ProofError::Custom(msg) => write!(f, "Proof error: {}", msg),
        }
    }
}

// Note: std::error::Error is not available in no_std

/// Extension trait for proof operations
pub trait ProofExt<T, P: Predicate<T>>: Sized {
    /// Map the proven value through a function
    fn map<U, F>(self, f: F) -> Proven<U, P>
    where
        F: FnOnce(T) -> U,
        P: Predicate<U>;

    /// Try to map the proven value, re-checking the predicate
    fn try_map<U, F>(self, f: F) -> ProofResult<U, P>
    where
        F: FnOnce(T) -> U,
        P: Predicate<U>;

    /// Chain proof operations sequentially
    fn and_then<U, Q, F>(self, f: F) -> ProofResult<U, Q>
    where
        Q: Predicate<U>,
        F: FnOnce(T) -> ProofResult<U, Q>;

    /// Combine with another proof
    fn zip<U, Q>(self, other: Proven<U, Q>) -> Proven<(T, U), (P, Q)>
    where
        Q: Predicate<U>,
        (P, Q): Predicate<(T, U)>;
}

impl<T, P: Predicate<T>> ProofExt<T, P> for Proven<T, P> {
    fn map<U, F>(self, f: F) -> Proven<U, P>
    where
        F: FnOnce(T) -> U,
        P: Predicate<U>,
    {
        self.map(f)
    }

    fn try_map<U, F>(self, f: F) -> ProofResult<U, P>
    where
        F: FnOnce(T) -> U,
        P: Predicate<U>,
    {
        self.try_map(f).ok_or(ProofError::PredicateFailed)
    }

    fn and_then<U, Q, F>(self, f: F) -> ProofResult<U, Q>
    where
        Q: Predicate<U>,
        F: FnOnce(T) -> ProofResult<U, Q>,
    {
        f(self.into_inner())
    }

    fn zip<U, Q>(self, other: Proven<U, Q>) -> Proven<(T, U), (P, Q)>
    where
        Q: Predicate<U>,
        (P, Q): Predicate<(T, U)>,
    {
        let value = (self.into_inner(), other.into_inner());
        // Safety: Both proofs hold, so the tuple proof holds
        unsafe { Proven::new_unchecked(value) }
    }
}

/// Split a proven pair into two proven values
pub fn split<T, U, P, Q>(proven: Proven<(T, U), (P, Q)>) -> (Proven<T, P>, Proven<U, Q>)
where
    P: Predicate<T>,
    Q: Predicate<U>,
    (P, Q): Predicate<(T, U)>,
{
    let (t, u) = proven.into_inner();
    // Safety: The pair proof implies both component proofs
    unsafe {
        (
            Proven::new_unchecked(t),
            Proven::new_unchecked(u),
        )
    }
}

/// Join two proven values into a proven pair
pub fn join<T, U, P, Q>(
    proven_t: Proven<T, P>,
    proven_u: Proven<U, Q>,
) -> Proven<(T, U), (P, Q)>
where
    P: Predicate<T>,
    Q: Predicate<U>,
    (P, Q): Predicate<(T, U)>,
{
    proven_t.zip(proven_u)
}

/// Proof builder for constructing proofs step-by-step
pub struct ProofBuilder<T, P> {
    value: Option<T>,
    _predicate: PhantomData<P>,
}

impl<T, P: Predicate<T>> ProofBuilder<T, P> {
    /// Create a new proof builder
    pub fn new() -> Self {
        Self {
            value: None,
            _predicate: PhantomData,
        }
    }

    /// Set the value to prove
    pub fn value(mut self, value: T) -> Self {
        self.value = Some(value);
        self
    }

    /// Build the proof
    pub fn build(self) -> ProofResult<T, P> {
        match self.value {
            Some(value) => {
                Proven::new(value).ok_or(ProofError::PredicateFailed)
            }
            None => Err(ProofError::Custom("No value provided".to_string())),
        }
    }

    /// Build the proof unsafely
    ///
    /// # Safety
    ///
    /// The predicate must hold for the value
    pub unsafe fn build_unchecked(self) -> Result<Proven<T, P>, ProofError> {
        match self.value {
            Some(value) => Ok(Proven::new_unchecked(value)),
            None => Err(ProofError::Custom("No value provided".to_string())),
        }
    }
}

impl<T, P: Predicate<T>> Default for ProofBuilder<T, P> {
    fn default() -> Self {
        Self::new()
    }
}

/// Proof chain for sequential proof composition
pub struct ProofChain<T, P> {
    result: ProofResult<T, P>,
}

impl<T, P: Predicate<T>> ProofChain<T, P> {
    /// Start a new proof chain
    pub fn start(value: T) -> Self {
        Self {
            result: Proven::new(value).ok_or(ProofError::PredicateFailed),
        }
    }

    /// Start with an existing proven value
    pub fn from_proven(proven: Proven<T, P>) -> Self {
        Self {
            result: Ok(proven),
        }
    }

    /// Map the value if the proof holds
    pub fn map<U, F>(self, f: F) -> ProofChain<U, P>
    where
        F: FnOnce(T) -> U,
        P: Predicate<U>,
    {
        ProofChain {
            result: self.result.and_then(|proven| {
                proven.try_map(f).map_err(|_| ProofError::CompositionFailed)
            }),
        }
    }

    /// Chain another proof operation
    pub fn and_then<U, Q, F>(self, f: F) -> ProofChain<U, Q>
    where
        Q: Predicate<U>,
        F: FnOnce(T) -> ProofResult<U, Q>,
    {
        ProofChain {
            result: self.result.and_then(|proven| {
                f(proven.into_inner())
            }),
        }
    }

    /// Filter the proof with an additional check
    pub fn filter<F>(self, f: F) -> Self
    where
        F: FnOnce(&T) -> bool,
    {
        ProofChain {
            result: self.result.and_then(|proven| {
                if f(proven.get()) {
                    Ok(proven)
                } else {
                    Err(ProofError::PredicateFailed)
                }
            }),
        }
    }

    /// Finalize the proof chain
    pub fn finish(self) -> ProofResult<T, P> {
        self.result
    }

    /// Unwrap the proof or panic
    pub fn unwrap(self) -> Proven<T, P> {
        self.result.unwrap()
    }

    /// Unwrap the proof or use a default
    pub fn unwrap_or(self, default: Proven<T, P>) -> Proven<T, P> {
        self.result.unwrap_or(default)
    }
}

/// Proof validator for runtime verification
pub struct ProofValidator<T, P> {
    checks: Vec<Box<dyn Fn(&T) -> bool>>,
    _predicate: PhantomData<P>,
}

impl<T, P: Predicate<T>> ProofValidator<T, P> {
    /// Create a new proof validator
    pub fn new() -> Self {
        Self {
            checks: Vec::new(),
            _predicate: PhantomData,
        }
    }

    /// Add a validation check
    pub fn check<F>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        self.checks.push(Box::new(f));
        self
    }

    /// Validate a value and construct a proof
    pub fn validate(&self, value: T) -> ProofResult<T, P> {
        // First check the predicate
        if !P::check(&value) {
            return Err(ProofError::PredicateFailed);
        }

        // Then check all additional validators
        for check in &self.checks {
            if !check(&value) {
                return Err(ProofError::PredicateFailed);
            }
        }

        // Safety: All checks passed
        Ok(unsafe { Proven::new_unchecked(value) })
    }
}

impl<T, P: Predicate<T>> Default for ProofValidator<T, P> {
    fn default() -> Self {
        Self::new()
    }
}

/// Proof adapter for converting between proof types
pub struct ProofAdapter<T, P, Q> {
    _phantom: PhantomData<(T, P, Q)>,
}

impl<T, P, Q> ProofAdapter<T, P, Q>
where
    P: Predicate<T>,
    Q: Predicate<T>,
{
    /// Try to adapt a proof from P to Q
    pub fn adapt(proven: Proven<T, P>) -> ProofResult<T, Q> {
        let value = proven.into_inner();
        Proven::new(value).ok_or(ProofError::PredicateFailed)
    }

    /// Adapt a proof unsafely
    ///
    /// # Safety
    ///
    /// Q must hold whenever P holds
    pub unsafe fn adapt_unchecked(proven: Proven<T, P>) -> Proven<T, Q> {
        let value = proven.into_inner();
        Proven::new_unchecked(value)
    }
}

/// Proof witness combinator
pub struct WitnessCombinator<P, Q> {
    _phantom: PhantomData<(P, Q)>,
}

impl<P, Q> WitnessCombinator<P, Q> {
    /// Combine two witnesses
    pub fn combine(w1: Witness<P>, w2: Witness<Q>) -> Witness<(P, Q)> {
        // Safety: Both witnesses exist, so the combined witness is valid
        unsafe { Witness::new() }
    }

    /// Split a combined witness
    pub fn split(w: Witness<(P, Q)>) -> (Witness<P>, Witness<Q>) {
        w.consume();
        // Safety: The combined witness implies both component witnesses
        unsafe { (Witness::new(), Witness::new()) }
    }
}

/// Proof memo for caching proof results
pub struct ProofMemo<T, P> {
    cache: BTreeMap<String, Proven<T, P>>,
    _phantom: PhantomData<P>,
}

impl<T: Clone, P: Predicate<T>> ProofMemo<T, P> {
    /// Create a new proof memo
    pub fn new() -> Self {
        Self {
            cache: BTreeMap::new(),
            _phantom: PhantomData,
        }
    }

    /// Get or create a proof for a value
    pub fn get_or_prove(&mut self, key: String, value: T) -> ProofResult<T, P> {
        if let Some(proven) = self.cache.get(&key) {
            return Ok(proven.clone());
        }

        let proven = Proven::new(value.clone())
            .ok_or(ProofError::PredicateFailed)?;

        self.cache.insert(key, proven.clone());
        Ok(proven)
    }

    /// Clear the memo cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }
}

impl<T: Clone, P: Predicate<T>> Default for ProofMemo<T, P> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::phantom::{NonZeroPred, ChatmanCompliant, WithinChatmanPred};

    #[test]
    fn test_proof_builder() {
        let proof = ProofBuilder::<u64, NonZeroPred>::new()
            .value(42)
            .build()
            .unwrap();

        assert_eq!(*proof.get(), 42);
    }

    #[test]
    fn test_proof_chain() {
        let result = ProofChain::start(5u64)
            .map(|x| x * 2)
            .filter(|x| *x <= 8)
            .finish();

        assert!(result.is_err()); // 10 > 8

        let result = ProofChain::start(3u64)
            .map(|x| x * 2)
            .filter(|x| *x <= 8)
            .finish();

        assert!(result.is_ok()); // 6 <= 8
    }

    #[test]
    fn test_split_join() {
        let nz1 = Proven::<u64, NonZeroPred>::new(1).unwrap();
        let nz2 = Proven::<u64, NonZeroPred>::new(2).unwrap();

        let pair = join(nz1, nz2);
        let (p1, p2) = split(pair);

        assert_eq!(*p1.get(), 1);
        assert_eq!(*p2.get(), 2);
    }

    #[test]
    fn test_proof_validator() {
        let validator = ProofValidator::<u64, WithinChatmanPred>::new()
            .check(|x| *x % 2 == 0); // Must be even

        assert!(validator.validate(4).is_ok());
        assert!(validator.validate(5).is_err()); // Odd
        assert!(validator.validate(10).is_err()); // > 8
    }

    #[test]
    fn test_proof_memo() {
        let mut memo = ProofMemo::<u64, WithinChatmanPred>::new();

        let p1 = memo.get_or_prove("key1".to_string(), 5).unwrap();
        let p2 = memo.get_or_prove("key1".to_string(), 5).unwrap();

        assert_eq!(*p1.get(), 5);
        assert_eq!(*p2.get(), 5);
    }
}
