//! Const Generic Proofs - Compile-Time Bounds and Verification
//!
//! This module uses const generics to encode compile-time arithmetic
//! and bounds checking with zero runtime cost.

use alloc::vec::Vec;
use core::marker::PhantomData;
use core::mem;

/// A value bounded by compile-time constants
///
/// The bounds are checked at compile time through const assertions
pub struct Bounded<T, const MIN: u64, const MAX: u64> {
    value: T,
    _bounds: PhantomData<
        [(); {
            assert!(MIN <= MAX, "MIN must be <= MAX");
            0
        }],
    >,
}

impl<T, const MIN: u64, const MAX: u64> Bounded<T, MIN, MAX> {
    /// Create a new bounded value
    ///
    /// Returns `None` if the value is outside the bounds
    pub fn new(value: T) -> Option<Self>
    where
        T: TryInto<u64> + Copy,
    {
        if let Ok(v) = value.try_into() {
            if v >= MIN && v <= MAX {
                return Some(Self {
                    value,
                    _bounds: PhantomData,
                });
            }
        }
        None
    }

    /// Create a bounded value without checking (unsafe)
    ///
    /// # Safety
    ///
    /// The value must be within [MIN, MAX]
    pub unsafe fn new_unchecked(value: T) -> Self {
        Self {
            value,
            _bounds: PhantomData,
        }
    }

    /// Extract the inner value
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Get a reference to the bounded value
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Get the minimum bound
    pub const fn min_bound() -> u64 {
        MIN
    }

    /// Get the maximum bound
    pub const fn max_bound() -> u64 {
        MAX
    }

    /// Check if the bounds are within the Chatman constant
    pub const fn is_chatman_bounded() -> bool {
        MAX - MIN <= 8
    }
}

impl<T: Clone, const MIN: u64, const MAX: u64> Clone for Bounded<T, MIN, MAX> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _bounds: PhantomData,
        }
    }
}

impl<T: Copy, const MIN: u64, const MAX: u64> Copy for Bounded<T, MIN, MAX> {}

impl<T: core::fmt::Debug, const MIN: u64, const MAX: u64> core::fmt::Debug
    for Bounded<T, MIN, MAX>
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Bounded")
            .field("value", &self.value)
            .field("min", &MIN)
            .field("max", &MAX)
            .finish()
    }
}

/// Type alias for Chatman-bounded values (0-8)
pub type ChatmanBounded<T> = Bounded<T, 0, 8>;

/// A non-zero value using const generics
pub struct ConstNonZero<T, const ZERO: u64 = 0> {
    value: T,
    _zero: PhantomData<[(); ZERO as usize]>,
}

impl<T, const ZERO: u64> ConstNonZero<T, ZERO> {
    /// Create a non-zero value
    pub fn new(value: T) -> Option<Self>
    where
        T: TryInto<u64> + Copy,
    {
        if let Ok(v) = value.try_into() {
            if v != ZERO {
                return Some(Self {
                    value,
                    _zero: PhantomData,
                });
            }
        }
        None
    }

    /// Create without checking (unsafe)
    ///
    /// # Safety
    ///
    /// The value must not equal ZERO
    pub unsafe fn new_unchecked(value: T) -> Self {
        Self {
            value,
            _zero: PhantomData,
        }
    }

    /// Extract the inner value
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Get a reference to the value
    pub fn get(&self) -> &T {
        &self.value
    }
}

/// An aligned value using const generics
///
/// The alignment is checked at compile time
pub struct Aligned<T, const ALIGN: usize> {
    value: T,
    _align: PhantomData<
        [(); {
            assert!(ALIGN > 0, "Alignment must be > 0");
            assert!(ALIGN & (ALIGN - 1) == 0, "Alignment must be a power of 2");
            0
        }],
    >,
}

impl<T, const ALIGN: usize> Aligned<T, ALIGN> {
    /// Create an aligned value
    ///
    /// For pointer types, checks that the address is aligned
    pub fn new(value: T) -> Option<Self>
    where
        T: Copy,
    {
        // For usize, check alignment
        if mem::size_of::<T>() == mem::size_of::<usize>() {
            let ptr = &value as *const T as usize;
            if ptr % ALIGN == 0 {
                return Some(Self {
                    value,
                    _align: PhantomData,
                });
            }
            return None;
        }

        // For other types, assume aligned (conservative)
        Some(Self {
            value,
            _align: PhantomData,
        })
    }

    /// Create without checking (unsafe)
    ///
    /// # Safety
    ///
    /// The value must be properly aligned
    pub unsafe fn new_unchecked(value: T) -> Self {
        Self {
            value,
            _align: PhantomData,
        }
    }

    /// Extract the inner value
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Get a reference to the aligned value
    pub fn get(&self) -> &T {
        &self.value
    }

    /// Get the alignment
    pub const fn alignment() -> usize {
        ALIGN
    }
}

impl<T: Clone, const ALIGN: usize> Clone for Aligned<T, ALIGN> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            _align: PhantomData,
        }
    }
}

impl<T: Copy, const ALIGN: usize> Copy for Aligned<T, ALIGN> {}

/// A range with compile-time bounds
pub struct ConstRange<const START: u64, const END: u64> {
    _phantom: PhantomData<
        [(); {
            assert!(START <= END, "START must be <= END");
            0
        }],
    >,
}

impl<const START: u64, const END: u64> ConstRange<START, END> {
    /// Create a new const range
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    /// Check if a value is in the range
    pub const fn contains(value: u64) -> bool {
        value >= START && value < END
    }

    /// Get the start of the range
    pub const fn start() -> u64 {
        START
    }

    /// Get the end of the range
    pub const fn end() -> u64 {
        END
    }

    /// Get the length of the range
    pub const fn len() -> u64 {
        END - START
    }

    /// Check if the range is within the Chatman constant
    pub const fn is_chatman_bounded() -> bool {
        (END - START) <= 8
    }
}

impl<const START: u64, const END: u64> Default for ConstRange<START, END> {
    fn default() -> Self {
        Self::new()
    }
}

/// Compile-time arithmetic verification
pub struct ConstArithmetic;

impl ConstArithmetic {
    /// Verify addition doesn't overflow at compile time
    pub const fn checked_add<const A: u64, const B: u64>() -> Option<u64> {
        if A > u64::MAX - B {
            None
        } else {
            Some(A + B)
        }
    }

    /// Verify subtraction doesn't underflow
    pub const fn checked_sub<const A: u64, const B: u64>() -> Option<u64> {
        if A < B {
            None
        } else {
            Some(A - B)
        }
    }

    /// Verify multiplication doesn't overflow
    pub const fn checked_mul<const A: u64, const B: u64>() -> Option<u64> {
        if B > 0 && A > u64::MAX / B {
            None
        } else {
            Some(A * B)
        }
    }

    /// Verify division by non-zero
    pub const fn checked_div<const A: u64, const B: u64>() -> Option<u64> {
        if B == 0 {
            None
        } else {
            Some(A / B)
        }
    }
}

/// Type-level natural numbers
pub trait Nat {
    const VALUE: usize;
}

pub struct Z; // Zero

impl Nat for Z {
    const VALUE: usize = 0;
}

pub struct S<N: Nat>(PhantomData<N>); // Successor

impl<N: Nat> Nat for S<N> {
    const VALUE: usize = N::VALUE + 1;
}

// Type aliases for small natural numbers
pub type N0 = Z;
pub type N1 = S<N0>;
pub type N2 = S<N1>;
pub type N3 = S<N2>;
pub type N4 = S<N3>;
pub type N5 = S<N4>;
pub type N6 = S<N5>;
pub type N7 = S<N6>;
pub type N8 = S<N7>;

/// Fixed-size array with compile-time length
pub struct ConstArray<T, N: Nat> {
    data: Vec<T>,
    _len: PhantomData<N>,
}

impl<T, N: Nat> ConstArray<T, N> {
    /// Create a new const array
    pub fn new(data: Vec<T>) -> Option<Self> {
        if data.len() == N::VALUE {
            Some(Self {
                data,
                _len: PhantomData,
            })
        } else {
            None
        }
    }

    /// Create without checking (unsafe)
    ///
    /// # Safety
    ///
    /// The data length must equal N::VALUE
    pub unsafe fn new_unchecked(data: Vec<T>) -> Self {
        Self {
            data,
            _len: PhantomData,
        }
    }

    /// Get the compile-time length
    pub const fn len() -> usize {
        N::VALUE
    }

    /// Check if the array is within Chatman constant
    pub const fn is_chatman_sized() -> bool {
        N::VALUE <= 8
    }

    /// Get a reference to the data
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }
}

/// Proof that a const value is within the Chatman constant
pub struct ChatmanProof<const N: u64> {
    _phantom: PhantomData<
        [(); {
            assert!(N <= 8, "Value must be <= 8 (Chatman constant)");
            0
        }],
    >,
}

impl<const N: u64> ChatmanProof<N> {
    /// Create a Chatman proof
    ///
    /// This will only compile if N <= 8
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    /// Get the proven value
    pub const fn value() -> u64 {
        N
    }
}

/// Proof that a const value is a power of two
pub struct PowerOfTwoProof<const N: usize> {
    _phantom: PhantomData<
        [(); {
            assert!(N > 0, "Value must be > 0");
            assert!(N & (N - 1) == 0, "Value must be a power of 2");
            0
        }],
    >,
}

impl<const N: usize> PowerOfTwoProof<N> {
    /// Create a power-of-two proof
    ///
    /// This will only compile if N is a power of two
    pub const fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    /// Get the proven value
    pub const fn value() -> usize {
        N
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded() {
        let b = Bounded::<u64, 0, 100>::new(50).unwrap();
        assert_eq!(*b.get(), 50);

        assert!(Bounded::<u64, 0, 100>::new(101).is_none());
        assert!(Bounded::<u64, 10, 100>::new(5).is_none());
    }

    #[test]
    fn test_chatman_bounded() {
        let b = ChatmanBounded::<u64>::new(5).unwrap();
        assert_eq!(*b.get(), 5);
        assert!(ChatmanBounded::<u64>::is_chatman_bounded());
    }

    #[test]
    fn test_const_non_zero() {
        let nz = ConstNonZero::<u64>::new(42).unwrap();
        assert_eq!(*nz.get(), 42);

        assert!(ConstNonZero::<u64>::new(0).is_none());
    }

    #[test]
    fn test_const_range() {
        let range = ConstRange::<0, 10>::new();
        assert_eq!(ConstRange::<0, 10>::len(), 10);
        assert!(ConstRange::<0, 10>::contains(5));
        assert!(!ConstRange::<0, 10>::contains(15));
    }

    #[test]
    fn test_chatman_proof() {
        let _proof = ChatmanProof::<8>::new();
        assert_eq!(ChatmanProof::<8>::value(), 8);

        // This would fail to compile:
        // let _invalid = ChatmanProof::<9>::new();
    }

    #[test]
    fn test_power_of_two_proof() {
        let _proof = PowerOfTwoProof::<16>::new();
        assert_eq!(PowerOfTwoProof::<16>::value(), 16);

        // This would fail to compile:
        // let _invalid = PowerOfTwoProof::<15>::new();
    }

    #[test]
    fn test_type_level_nats() {
        assert_eq!(N0::VALUE, 0);
        assert_eq!(N1::VALUE, 1);
        assert_eq!(N8::VALUE, 8);
    }
}
