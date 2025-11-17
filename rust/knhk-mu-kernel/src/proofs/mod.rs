//! Zero-Cost Proof Abstractions
//!
//! This module provides compile-time proof obligations using advanced phantom types
//! and const generics. All proof types are designed for zero runtime cost.
//!
//! # Architecture
//!
//! The proof system is built on four core components:
//!
//! 1. **Phantom Types** - Zero-sized types that carry compile-time information
//! 2. **Const Generics** - Compile-time constants for bounds and verification
//! 3. **Type-Level Predicates** - Compile-time property checking
//! 4. **Proof Combinators** - Functional composition of proofs
//!
//! # Zero-Cost Guarantee
//!
//! All proof types use `PhantomData` and are guaranteed to have zero runtime overhead:
//!
//! ```rust
//! use knhk_mu_kernel::proofs::phantom::{Proven, NonZero};
//!
//! // Proven<T, P> is the same size as T
//! assert_eq!(
//!     std::mem::size_of::<Proven<u64, NonZero>>(),
//!     std::mem::size_of::<u64>()
//! );
//!
//! // Witnesses are zero-sized
//! assert_eq!(std::mem::size_of::<Witness<()>>(), 0);
//! ```
//!
//! # Usage Examples
//!
//! ## Phantom Proofs
//!
//! ```rust
//! use knhk_mu_kernel::proofs::phantom::{NonZero, ChatmanCompliant};
//!
//! // Proven non-zero value
//! let nz = NonZero::<u64>::new(42).expect("42 is non-zero");
//! assert_eq!(*nz.get(), 42);
//!
//! // Proven Chatman-compliant value (≤8)
//! let chatman = ChatmanCompliant::new(5).expect("5 ≤ 8");
//! assert_eq!(*chatman.get(), 5);
//! ```
//!
//! ## Const Generic Bounds
//!
//! ```rust
//! use knhk_mu_kernel::proofs::const_generic::{Bounded, ChatmanBounded};
//!
//! // Compile-time bounded value
//! let bounded = Bounded::<u64, 0, 100>::new(50).expect("50 in [0, 100]");
//! assert_eq!(*bounded.get(), 50);
//!
//! // Chatman-bounded value
//! let chatman = ChatmanBounded::<u64>::new(7).expect("7 in [0, 8]");
//! assert_eq!(*chatman.get(), 7);
//! ```
//!
//! ## Type-Level Predicates
//!
//! ```rust
//! use knhk_mu_kernel::proofs::predicates::{
//!     ProvenSorted, ProvenUnique, ProvenChatmanBounded
//! };
//!
//! // Proven sorted vector
//! let sorted = ProvenSorted::new(vec![3, 1, 2]);
//! assert_eq!(sorted.get(), &vec![1, 2, 3]);
//!
//! // Proven unique vector
//! let unique = ProvenUnique::new(vec![1, 2, 1, 3]);
//! assert_eq!(unique.get().len(), 3);
//!
//! // Proven Chatman-bounded vector
//! let chatman = ProvenChatmanBounded::new(vec![1, 2, 3]).expect("≤8 elements");
//! assert_eq!(chatman.len(), 3);
//! ```
//!
//! ## Proof Combinators
//!
//! ```rust
//! use knhk_mu_kernel::proofs::combinators::{ProofChain, ProofBuilder};
//! use knhk_mu_kernel::proofs::phantom::WithinChatmanPred;
//!
//! // Build a proof step-by-step
//! let proof = ProofBuilder::<u64, WithinChatmanPred>::new()
//!     .value(5)
//!     .build()
//!     .expect("5 ≤ 8");
//!
//! // Chain proof operations
//! let result = ProofChain::start(3u64)
//!     .map(|x| x * 2)
//!     .filter(|x| *x <= 8)
//!     .finish()
//!     .expect("6 ≤ 8");
//! ```
//!
//! # Performance
//!
//! All proof operations are zero-cost at runtime:
//!
//! - Proofs are checked at compile time or construction time
//! - No runtime overhead for holding proofs
//! - No runtime checks when using proven values
//! - Optimizes to the same assembly as unchecked code

pub mod combinators;
pub mod const_generic;
pub mod phantom;
pub mod predicates;

// Re-export commonly used types
pub use phantom::{
    ChatmanCompliant, NonZero, PowerOfTwo, Predicate, Proof, Proven, Sorted, Witness,
};

pub use const_generic::{
    Aligned, Bounded, ChatmanBounded, ChatmanProof, ConstNonZero, ConstRange, PowerOfTwoProof,
};

pub use predicates::{
    IsPowerOfTwo, IsSorted, IsWithinChatman, ProvenChatmanBounded, ProvenNonEmpty, ProvenSorted,
    ProvenUnique,
};

pub use combinators::{
    join, split, ProofBuilder, ProofChain, ProofError, ProofExt, ProofResult, ProofValidator,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_cost_phantom() {
        // Proven<T, P> should be the same size as T
        assert_eq!(
            std::mem::size_of::<Proven<u64, phantom::NonZeroPred>>(),
            std::mem::size_of::<u64>()
        );

        // Witness should be zero-sized
        assert_eq!(std::mem::size_of::<Witness<()>>(), 0);

        // Proof should be the same size as the value
        assert_eq!(
            std::mem::size_of::<Proof<u64, ()>>(),
            std::mem::size_of::<u64>()
        );
    }

    #[test]
    fn test_zero_cost_const_generic() {
        // Bounded should be the same size as T
        assert_eq!(
            std::mem::size_of::<Bounded<u64, 0, 100>>(),
            std::mem::size_of::<u64>()
        );

        // ConstNonZero should be the same size as T
        assert_eq!(
            std::mem::size_of::<ConstNonZero<u64>>(),
            std::mem::size_of::<u64>()
        );

        // Aligned should be the same size as T
        assert_eq!(
            std::mem::size_of::<Aligned<u64, 8>>(),
            std::mem::size_of::<u64>()
        );
    }

    #[test]
    fn test_chatman_constant() {
        // Test Chatman-bounded values
        let chatman = ChatmanCompliant::new(8).expect("8 ≤ 8");
        assert_eq!(*chatman.get(), 8);

        assert!(ChatmanCompliant::new(9).is_none());

        // Test Chatman-bounded collections
        let chatman_vec =
            ProvenChatmanBounded::new(vec![1, 2, 3, 4, 5, 6, 7, 8]).expect("8 elements ≤ 8");
        assert_eq!(chatman_vec.len(), 8);

        assert!(ProvenChatmanBounded::new(vec![1, 2, 3, 4, 5, 6, 7, 8, 9]).is_none());
    }

    #[test]
    fn test_proof_composition() {
        // Test split/join
        let nz1 = NonZero::<u64>::new(1).unwrap();
        let nz2 = NonZero::<u64>::new(2).unwrap();

        let pair = join(nz1, nz2);
        let (p1, p2) = split(pair);

        assert_eq!(*p1.get(), 1);
        assert_eq!(*p2.get(), 2);
    }

    #[test]
    fn test_predicate_composition() {
        use predicates::{AndCheck, Check, Even, Positive};

        // Test AND composition
        type EvenAndPositive = AndCheck<Even, Positive>;

        assert!(EvenAndPositive::check(&4));
        assert!(!EvenAndPositive::check(&3));
        assert!(!EvenAndPositive::check(&-4));
    }
}
