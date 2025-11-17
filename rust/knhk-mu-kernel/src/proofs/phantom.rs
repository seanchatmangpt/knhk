//! Phantom Proof Types - Zero-Cost Compile-Time Proofs
//!
//! This module implements phantom types for carrying compile-time proof obligations
//! with zero runtime cost. All proof types use `PhantomData` and are zero-sized.

use alloc::vec::Vec;
use core::marker::PhantomData;

/// A predicate trait that types can implement to define compile-time properties
pub trait Predicate<T> {
    /// The proof witness type (typically zero-sized)
    type Proof;

    /// Check if the predicate holds for the given value
    ///
    /// This is used at construction time only, not at runtime
    fn check(value: &T) -> bool;

    /// Construct a proof witness (unsafe - caller must ensure predicate holds)
    ///
    /// # Safety
    ///
    /// The predicate must actually hold for the value
    unsafe fn prove_unchecked() -> Self::Proof;
}

/// A value with an attached compile-time proof
///
/// This type is guaranteed to be the same size as `T` in memory
pub struct Proven<T, P> {
    value: T,
    _proof: PhantomData<fn() -> P>,
}

impl<T, P> Proven<T, P>
where
    P: Predicate<T>,
{
    /// Construct a proven value by checking the predicate
    ///
    /// Returns `None` if the predicate doesn't hold
    pub fn new(value: T) -> Option<Self> {
        if P::check(&value) {
            Some(Self {
                value,
                _proof: PhantomData,
            })
        } else {
            None
        }
    }

    /// Construct a proven value without checking (unsafe)
    ///
    /// # Safety
    ///
    /// The caller must ensure the predicate actually holds for the value
    pub unsafe fn new_unchecked(value: T) -> Self {
        Self {
            value,
            _proof: PhantomData,
        }
    }

    /// Extract the inner value
    ///
    /// This is always safe because we have the proof
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Get a reference to the proven value
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Get a mutable reference to the proven value
    ///
    /// Note: This can break the invariant if you mutate to an invalid state!
    /// Use with caution or implement custom safety checks.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.value
    }

    /// Map a proven value through a function that preserves the proof
    pub fn map<U, F>(self, f: F) -> Proven<U, P>
    where
        F: FnOnce(T) -> U,
        P: Predicate<U>,
    {
        let new_value = f(self.value);
        // Safety: We trust that F preserves the predicate
        unsafe { Proven::new_unchecked(new_value) }
    }

    /// Try to map a proven value, re-checking the predicate
    pub fn try_map<U, F>(self, f: F) -> Option<Proven<U, P>>
    where
        F: FnOnce(T) -> U,
        P: Predicate<U>,
    {
        let new_value = f(self.value);
        Proven::new(new_value)
    }
}

impl<T: Clone, P> Clone for Proven<T, P> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _proof: PhantomData,
        }
    }
}

impl<T: Copy, P> Copy for Proven<T, P> {}

impl<T: core::fmt::Debug, P> core::fmt::Debug for Proven<T, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Proven")
            .field("value", &self.value)
            .field("proof", &"<zero-sized>")
            .finish()
    }
}

/// A zero-sized proof witness
///
/// This type carries proof information at compile-time but has no runtime representation
pub struct Witness<T> {
    _phantom: PhantomData<fn() -> T>,
}

impl<T> Witness<T> {
    /// Create a new witness (unsafe - caller must ensure the proof holds)
    ///
    /// # Safety
    ///
    /// The caller must ensure the witnessed property actually holds
    pub unsafe fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    /// Consume the witness
    pub fn consume(self) {
        // No-op - just drops the zero-sized type
    }
}

impl<T> Clone for Witness<T> {
    fn clone(&self) -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T> Copy for Witness<T> {}

impl<T> core::fmt::Debug for Witness<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Witness")
            .field("type", &core::any::type_name::<T>())
            .finish()
    }
}

/// A proof that carries a value and witness
pub struct Proof<T, P> {
    value: T,
    witness: Witness<P>,
}

impl<T, P> Proof<T, P> {
    /// Create a new proof with a witness
    ///
    /// # Safety
    ///
    /// The caller must ensure the proof actually holds
    pub unsafe fn new(value: T) -> Self {
        Self {
            value,
            witness: Witness::new(),
        }
    }

    /// Extract the value and witness
    pub fn into_parts(self) -> (T, Witness<P>) {
        (self.value, self.witness)
    }

    /// Get a reference to the value
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Get the witness
    pub fn witness(&self) -> Witness<P> {
        self.witness
    }
}

impl<T: Clone, P> Clone for Proof<T, P> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            witness: self.witness,
        }
    }
}

impl<T: Copy, P> Copy for Proof<T, P> {}

/// Predicate for non-zero values
pub struct NonZeroPred;

impl Predicate<u64> for NonZeroPred {
    type Proof = Witness<NonZeroPred>;

    fn check(value: &u64) -> bool {
        *value != 0
    }

    unsafe fn prove_unchecked() -> Self::Proof {
        Witness::new()
    }
}

impl Predicate<i64> for NonZeroPred {
    type Proof = Witness<NonZeroPred>;

    fn check(value: &i64) -> bool {
        *value != 0
    }

    unsafe fn prove_unchecked() -> Self::Proof {
        Witness::new()
    }
}

/// Type alias for proven non-zero values
pub type NonZero<T> = Proven<T, NonZeroPred>;

/// Predicate for values within the Chatman constant (≤8)
pub struct WithinChatmanPred;

impl Predicate<u64> for WithinChatmanPred {
    type Proof = Witness<WithinChatmanPred>;

    fn check(value: &u64) -> bool {
        *value <= 8
    }

    unsafe fn prove_unchecked() -> Self::Proof {
        Witness::new()
    }
}

/// Type alias for proven Chatman-compliant values
pub type ChatmanCompliant = Proven<u64, WithinChatmanPred>;

/// Predicate for sorted sequences
pub struct SortedPred;

impl<T: Ord> Predicate<Vec<T>> for SortedPred {
    type Proof = Witness<SortedPred>;

    fn check(value: &Vec<T>) -> bool {
        value.windows(2).all(|w| w[0] <= w[1])
    }

    unsafe fn prove_unchecked() -> Self::Proof {
        Witness::new()
    }
}

/// Type alias for proven sorted vectors
pub type Sorted<T> = Proven<Vec<T>, SortedPred>;

/// Predicate for power-of-two values
pub struct PowerOfTwoPred;

impl Predicate<usize> for PowerOfTwoPred {
    type Proof = Witness<PowerOfTwoPred>;

    fn check(value: &usize) -> bool {
        *value > 0 && (*value & (*value - 1)) == 0
    }

    unsafe fn prove_unchecked() -> Self::Proof {
        Witness::new()
    }
}

/// Type alias for proven power-of-two values
pub type PowerOfTwo = Proven<usize, PowerOfTwoPred>;

/// Predicate conjunction - both P and Q must hold
pub struct And<P, Q> {
    _p: PhantomData<P>,
    _q: PhantomData<Q>,
}

impl<T, P, Q> Predicate<T> for And<P, Q>
where
    P: Predicate<T>,
    Q: Predicate<T>,
{
    type Proof = (P::Proof, Q::Proof);

    fn check(value: &T) -> bool {
        P::check(value) && Q::check(value)
    }

    unsafe fn prove_unchecked() -> Self::Proof {
        (P::prove_unchecked(), Q::prove_unchecked())
    }
}

/// Predicate disjunction - either P or Q must hold
pub struct Or<P, Q> {
    _p: PhantomData<P>,
    _q: PhantomData<Q>,
}

impl<T, P, Q> Predicate<T> for Or<P, Q>
where
    P: Predicate<T>,
    Q: Predicate<T>,
{
    type Proof = Either<P::Proof, Q::Proof>;

    fn check(value: &T) -> bool {
        P::check(value) || Q::check(value)
    }

    unsafe fn prove_unchecked() -> Self::Proof {
        // Default to left - this is unsafe anyway
        Either::Left(P::prove_unchecked())
    }
}

/// Either type for proof disjunction
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

/// Predicate negation
pub struct Not<P> {
    _p: PhantomData<P>,
}

impl<T, P> Predicate<T> for Not<P>
where
    P: Predicate<T>,
{
    type Proof = Witness<Not<P>>;

    fn check(value: &T) -> bool {
        !P::check(value)
    }

    unsafe fn prove_unchecked() -> Self::Proof {
        Witness::new()
    }
}

/// Refinement type - a value that satisfies a predicate at compile time
pub struct Refined<T, P: Predicate<T>> {
    value: T,
    _proof: PhantomData<P>,
}

impl<T, P: Predicate<T>> Refined<T, P> {
    /// Refine a value by checking the predicate
    pub fn new(value: T) -> Option<Self> {
        if P::check(&value) {
            Some(Self {
                value,
                _proof: PhantomData,
            })
        } else {
            None
        }
    }

    /// Refine without checking (unsafe)
    ///
    /// # Safety
    ///
    /// The predicate must actually hold
    pub unsafe fn new_unchecked(value: T) -> Self {
        Self {
            value,
            _proof: PhantomData,
        }
    }

    /// Extract the refined value
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Get a reference to the refined value
    pub fn get(&self) -> &T {
        &self.value
    }
}

impl<T: Clone, P: Predicate<T>> Clone for Refined<T, P> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _proof: PhantomData,
        }
    }
}

impl<T: Copy, P: Predicate<T>> Copy for Refined<T, P> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_witness_is_zero_sized() {
        assert_eq!(std::mem::size_of::<Witness<u64>>(), 0);
    }

    #[test]
    fn test_proven_same_size_as_inner() {
        assert_eq!(
            std::mem::size_of::<Proven<u64, NonZeroPred>>(),
            std::mem::size_of::<u64>()
        );
    }

    #[test]
    fn test_non_zero() {
        let nz = NonZero::<u64>::new(42).unwrap();
        assert_eq!(*nz.get(), 42);

        assert!(NonZero::<u64>::new(0).is_none());
    }

    #[test]
    fn test_chatman_compliant() {
        let c = ChatmanCompliant::new(8).unwrap();
        assert_eq!(*c.get(), 8);

        assert!(ChatmanCompliant::new(9).is_none());
    }

    #[test]
    fn test_sorted() {
        let sorted = Sorted::new(vec![1, 2, 3, 4]).unwrap();
        assert_eq!(sorted.get().len(), 4);

        assert!(Sorted::new(vec![1, 3, 2]).is_none());
    }

    #[test]
    fn test_power_of_two() {
        let pot = PowerOfTwo::new(16).unwrap();
        assert_eq!(*pot.get(), 16);

        assert!(PowerOfTwo::new(15).is_none());
    }
}
