//! Comprehensive tests for zero-cost proof abstractions
//!
//! This test suite validates that all proof types have zero runtime cost
//! and provide the expected compile-time guarantees.

use knhk_mu_kernel::proofs::*;
use core::mem::size_of;

#[test]
fn test_phantom_zero_cost() {
    // Proven<T, P> must be the same size as T
    assert_eq!(
        size_of::<Proven<u8, phantom::NonZeroPred>>(),
        size_of::<u8>(),
        "Proven<u8> should be 1 byte"
    );

    assert_eq!(
        size_of::<Proven<u64, phantom::NonZeroPred>>(),
        size_of::<u64>(),
        "Proven<u64> should be 8 bytes"
    );

    assert_eq!(
        size_of::<Proven<[u8; 100], phantom::NonZeroPred>>(),
        size_of::<[u8; 100]>(),
        "Proven<[u8; 100]> should be 100 bytes"
    );

    // Witness must be zero-sized
    assert_eq!(
        size_of::<Witness<()>>(),
        0,
        "Witness should be zero-sized"
    );

    assert_eq!(
        size_of::<Witness<u64>>(),
        0,
        "Witness<u64> should be zero-sized"
    );

    // Proof<T, P> should be the same size as T (witness is zero-sized)
    assert_eq!(
        size_of::<Proof<u64, ()>>(),
        size_of::<u64>(),
        "Proof<u64, ()> should be 8 bytes"
    );
}

#[test]
fn test_const_generic_zero_cost() {
    // Bounded<T, MIN, MAX> must be the same size as T
    assert_eq!(
        size_of::<Bounded<u64, 0, 100>>(),
        size_of::<u64>(),
        "Bounded<u64, 0, 100> should be 8 bytes"
    );

    assert_eq!(
        size_of::<ChatmanBounded<u64>>(),
        size_of::<u64>(),
        "ChatmanBounded<u64> should be 8 bytes"
    );

    // ConstNonZero<T> must be the same size as T
    assert_eq!(
        size_of::<ConstNonZero<u64>>(),
        size_of::<u64>(),
        "ConstNonZero<u64> should be 8 bytes"
    );

    // Aligned<T, ALIGN> must be the same size as T
    assert_eq!(
        size_of::<Aligned<u64, 8>>(),
        size_of::<u64>(),
        "Aligned<u64, 8> should be 8 bytes"
    );

    // ConstRange is zero-sized
    assert_eq!(
        size_of::<ConstRange<0, 10>>(),
        0,
        "ConstRange should be zero-sized"
    );

    // Proofs are zero-sized
    assert_eq!(
        size_of::<ChatmanProof<8>>(),
        0,
        "ChatmanProof should be zero-sized"
    );

    assert_eq!(
        size_of::<PowerOfTwoProof<16>>(),
        0,
        "PowerOfTwoProof should be zero-sized"
    );
}

#[test]
fn test_non_zero_proof() {
    // Valid non-zero values
    let nz_u64 = NonZero::<u64>::new(42).expect("42 is non-zero");
    assert_eq!(*nz_u64.get(), 42);

    let nz_i64 = phantom::Proven::<i64, phantom::NonZeroPred>::new(-5).expect("-5 is non-zero");
    assert_eq!(*nz_i64.get(), -5);

    // Zero should fail
    assert!(NonZero::<u64>::new(0).is_none(), "0 should not be non-zero");
}

#[test]
fn test_chatman_compliant() {
    // Valid Chatman values (0-8)
    for i in 0..=8 {
        let chatman = ChatmanCompliant::new(i).expect(&format!("{} should be ≤ 8", i));
        assert_eq!(*chatman.get(), i);
    }

    // Values > 8 should fail
    assert!(ChatmanCompliant::new(9).is_none(), "9 > 8");
    assert!(ChatmanCompliant::new(100).is_none(), "100 > 8");
}

#[test]
fn test_sorted_proof() {
    // Create sorted vector
    let sorted = Sorted::new(vec![1, 2, 3, 4, 5]).expect("Should be sorted");
    assert_eq!(sorted.get().len(), 5);
    assert_eq!(sorted.get(), &vec![1, 2, 3, 4, 5]);

    // Unsorted vector should fail
    assert!(Sorted::new(vec![1, 3, 2]).is_none(), "Not sorted");

    // Empty vector is sorted
    let empty = Sorted::<i32>::new(vec![]).expect("Empty is sorted");
    assert_eq!(empty.get().len(), 0);

    // Single element is sorted
    let single = Sorted::new(vec![42]).expect("Single element is sorted");
    assert_eq!(single.get().len(), 1);
}

#[test]
fn test_power_of_two_proof() {
    // Valid powers of two
    let pot_values = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];
    for &value in &pot_values {
        let pot = PowerOfTwo::new(value).expect(&format!("{} is power of 2", value));
        assert_eq!(*pot.get(), value);
    }

    // Non-powers of two should fail
    let non_pot = [0, 3, 5, 6, 7, 9, 15, 100];
    for &value in &non_pot {
        assert!(PowerOfTwo::new(value).is_none(), "{} is not power of 2", value);
    }
}

#[test]
fn test_bounded() {
    // Valid bounded values
    let b1 = Bounded::<u64, 0, 100>::new(50).expect("50 in [0, 100]");
    assert_eq!(*b1.get(), 50);

    let b2 = Bounded::<u64, 10, 20>::new(15).expect("15 in [10, 20]");
    assert_eq!(*b2.get(), 15);

    // Out of bounds should fail
    assert!(Bounded::<u64, 0, 100>::new(101).is_none(), "101 > 100");
    assert!(Bounded::<u64, 10, 20>::new(5).is_none(), "5 < 10");

    // Boundary values
    let b_min = Bounded::<u64, 0, 100>::new(0).expect("0 is min");
    assert_eq!(*b_min.get(), 0);

    let b_max = Bounded::<u64, 0, 100>::new(100).expect("100 is max");
    assert_eq!(*b_max.get(), 100);
}

#[test]
fn test_chatman_bounded() {
    // Valid Chatman-bounded values
    for i in 0..=8 {
        let cb = ChatmanBounded::<u64>::new(i).expect(&format!("{} ≤ 8", i));
        assert_eq!(*cb.get(), i);
    }

    // Out of Chatman bound
    assert!(ChatmanBounded::<u64>::new(9).is_none(), "9 > 8");

    // Check compile-time property
    assert!(ChatmanBounded::<u64>::is_chatman_bounded());
}

#[test]
fn test_aligned() {
    // Aligned values
    let aligned = unsafe { Aligned::<u64, 8>::new_unchecked(42) };
    assert_eq!(*aligned.get(), 42);
    assert_eq!(Aligned::<u64, 8>::alignment(), 8);

    // Different alignments
    let _ = unsafe { Aligned::<u64, 16>::new_unchecked(100) };
    let _ = unsafe { Aligned::<u64, 32>::new_unchecked(200) };
}

#[test]
fn test_const_range() {
    let range = ConstRange::<0, 10>::new();

    assert_eq!(ConstRange::<0, 10>::start(), 0);
    assert_eq!(ConstRange::<0, 10>::end(), 10);
    assert_eq!(ConstRange::<0, 10>::len(), 10);

    assert!(ConstRange::<0, 10>::contains(5));
    assert!(!ConstRange::<0, 10>::contains(10));
    assert!(!ConstRange::<0, 10>::contains(15));

    // Chatman range
    assert!(ConstRange::<0, 8>::is_chatman_bounded());
    assert!(!ConstRange::<0, 100>::is_chatman_bounded());
}

#[test]
fn test_type_level_predicates() {
    use predicates::TypePredicate;

    assert!(predicates::True::HOLDS);
    assert!(!predicates::False::HOLDS);

    type AndTrueTrue = predicates::And<predicates::True, predicates::True>;
    assert!(AndTrueTrue::HOLDS);

    type AndTrueFalse = predicates::And<predicates::True, predicates::False>;
    assert!(!AndTrueFalse::HOLDS);

    type OrTrueFalse = predicates::Or<predicates::True, predicates::False>;
    assert!(OrTrueFalse::HOLDS);

    type NotTrue = predicates::Not<predicates::True>;
    assert!(!NotTrue::HOLDS);
}

#[test]
fn test_is_within_chatman_trait() {
    // These should compile
    fn assert_chatman<const N: u64>() where (): IsWithinChatman<N> {}

    assert_chatman::<0>();
    assert_chatman::<1>();
    assert_chatman::<8>();

    // This would fail to compile:
    // assert_chatman::<9>();
}

#[test]
fn test_is_power_of_two_trait() {
    fn assert_pot<const N: usize>() where (): IsPowerOfTwo<N> {}

    assert_pot::<1>();
    assert_pot::<2>();
    assert_pot::<1024>();

    // This would fail to compile:
    // assert_pot::<15>();
}

#[test]
fn test_proven_sorted() {
    let sorted = ProvenSorted::new(vec![3, 1, 2]);
    assert_eq!(sorted.get(), &vec![1, 2, 3]);

    let already_sorted = vec![1, 2, 3];
    let proven = ProvenSorted::try_prove(already_sorted).expect("Already sorted");
    assert_eq!(proven.get(), &vec![1, 2, 3]);

    let unsorted = vec![1, 3, 2];
    assert!(ProvenSorted::try_prove(unsorted).is_none());
}

#[test]
fn test_proven_unique() {
    let unique = ProvenUnique::new(vec![1, 2, 1, 3, 2]);
    assert_eq!(unique.get().len(), 3);
    assert!(unique.get().contains(&1));
    assert!(unique.get().contains(&2));
    assert!(unique.get().contains(&3));

    let already_unique = vec![1, 2, 3];
    let proven = ProvenUnique::try_prove(already_unique).expect("Already unique");
    assert_eq!(proven.get().len(), 3);
}

#[test]
fn test_proven_non_empty() {
    let ne = ProvenNonEmpty::new(vec![1, 2, 3]).expect("Not empty");
    assert_eq!(ne.get().len(), 3);

    let singleton = ProvenNonEmpty::singleton(42);
    assert_eq!(singleton.get(), &vec![42]);

    let empty: Vec<i32> = vec![];
    assert!(ProvenNonEmpty::new(empty).is_none());
}

#[test]
fn test_proven_chatman_bounded() {
    // Valid Chatman-bounded vectors
    for len in 0..=8 {
        let vec: Vec<i32> = (0..len).collect();
        let cb = ProvenChatmanBounded::new(vec.clone()).expect(&format!("{} ≤ 8", len));
        assert_eq!(cb.len(), len as u8);
        assert_eq!(cb.get().len(), len as usize);
    }

    // Too many elements
    let too_many: Vec<i32> = (0..9).collect();
    assert!(ProvenChatmanBounded::new(too_many).is_none());
}

#[test]
fn test_proof_builder() {
    use phantom::NonZeroPred;

    let proof = ProofBuilder::<u64, NonZeroPred>::new()
        .value(42)
        .build()
        .expect("42 is non-zero");

    assert_eq!(*proof.get(), 42);

    let failed = ProofBuilder::<u64, NonZeroPred>::new()
        .value(0)
        .build();

    assert!(failed.is_err());
}

#[test]
fn test_proof_chain() {
    use phantom::WithinChatmanPred;

    let result = ProofChain::start(3u64)
        .map(|x| x * 2)
        .filter(|x| *x <= 8)
        .finish()
        .expect("3 * 2 = 6 ≤ 8");

    assert_eq!(*result.get(), 6);

    let failed = ProofChain::start(5u64)
        .map(|x| x * 2)
        .filter(|x| *x <= 8)
        .finish();

    assert!(failed.is_err()); // 5 * 2 = 10 > 8
}

#[test]
fn test_split_join() {
    use phantom::NonZeroPred;

    let nz1 = Proven::<u64, NonZeroPred>::new(1).unwrap();
    let nz2 = Proven::<u64, NonZeroPred>::new(2).unwrap();

    let pair = join(nz1, nz2);
    let (p1, p2) = split(pair);

    assert_eq!(*p1.get(), 1);
    assert_eq!(*p2.get(), 2);
}

#[test]
fn test_proof_validator() {
    use phantom::WithinChatmanPred;

    let validator = ProofValidator::<u64, WithinChatmanPred>::new()
        .check(|x| *x % 2 == 0) // Must be even
        .check(|x| *x > 0);     // Must be positive

    assert!(validator.validate(2).is_ok());
    assert!(validator.validate(4).is_ok());
    assert!(validator.validate(6).is_ok());
    assert!(validator.validate(8).is_ok());

    assert!(validator.validate(0).is_err()); // Not positive
    assert!(validator.validate(1).is_err()); // Odd
    assert!(validator.validate(3).is_err()); // Odd
    assert!(validator.validate(10).is_err()); // > 8
}

#[test]
fn test_proof_memo() {
    use combinators::ProofMemo;
    use phantom::WithinChatmanPred;

    let mut memo = ProofMemo::<u64, WithinChatmanPred>::new();

    let p1 = memo.get_or_prove("key1".to_string(), 5).unwrap();
    let p2 = memo.get_or_prove("key1".to_string(), 5).unwrap();
    let p3 = memo.get_or_prove("key2".to_string(), 7).unwrap();

    assert_eq!(*p1.get(), 5);
    assert_eq!(*p2.get(), 5);
    assert_eq!(*p3.get(), 7);

    memo.clear();
    let p4 = memo.get_or_prove("key1".to_string(), 5).unwrap();
    assert_eq!(*p4.get(), 5);
}

#[test]
fn test_value_predicates() {
    use predicates::{Even, Odd, Positive, Negative, Check};

    assert!(Even::check(&4));
    assert!(!Even::check(&5));

    assert!(Odd::check(&5));
    assert!(!Odd::check(&4));

    assert!(Positive::check(&10i64));
    assert!(!Positive::check(&-10i64));

    assert!(Negative::check(&-10i64));
    assert!(!Negative::check(&10i64));
}

#[test]
fn test_predicate_composition() {
    use predicates::{Even, Positive, AndCheck, OrCheck, NotCheck, Check};

    type EvenAndPositive = AndCheck<Even, Positive>;
    assert!(EvenAndPositive::check(&4i64));
    assert!(!EvenAndPositive::check(&3i64));
    assert!(!EvenAndPositive::check(&-4i64));

    type EvenOrPositive = OrCheck<Even, Positive>;
    assert!(EvenOrPositive::check(&4i64));
    assert!(EvenOrPositive::check(&3i64));
    assert!(EvenOrPositive::check(&-4i64));
    assert!(!EvenOrPositive::check(&-3i64));

    type NotEven = NotCheck<Even>;
    assert!(!NotEven::check(&4i64));
    assert!(NotEven::check(&5i64));
}

#[test]
fn test_chatman_range() {
    use predicates::{ChatmanRange, Check};

    for i in 0..=8 {
        assert!(ChatmanRange::check(&i), "{} should be in [0, 8]", i);
    }

    assert!(!ChatmanRange::check(&9));
    assert!(!ChatmanRange::check(&-1));
    assert!(!ChatmanRange::check(&100));
}

#[test]
fn test_chatman_proof_const() {
    let _proof0 = ChatmanProof::<0>::new();
    let _proof8 = ChatmanProof::<8>::new();

    assert_eq!(ChatmanProof::<0>::value(), 0);
    assert_eq!(ChatmanProof::<8>::value(), 8);

    // This would fail to compile:
    // let _invalid = ChatmanProof::<9>::new();
}

#[test]
fn test_power_of_two_proof_const() {
    let _proof1 = PowerOfTwoProof::<1>::new();
    let _proof16 = PowerOfTwoProof::<16>::new();
    let _proof1024 = PowerOfTwoProof::<1024>::new();

    assert_eq!(PowerOfTwoProof::<16>::value(), 16);

    // This would fail to compile:
    // let _invalid = PowerOfTwoProof::<15>::new();
}

#[test]
fn test_proof_zero_cost_in_option() {
    // Option<Proven<T, P>> should be the same size as Option<T>
    assert_eq!(
        size_of::<Option<Proven<u64, phantom::NonZeroPred>>>(),
        size_of::<Option<u64>>(),
        "Option<Proven<u64>> should be same size as Option<u64>"
    );
}

#[test]
fn test_proof_zero_cost_in_result() {
    // Result<Proven<T, P>, E> should be the same size as Result<T, E>
    assert_eq!(
        size_of::<Result<Proven<u64, phantom::NonZeroPred>, ()>>(),
        size_of::<Result<u64, ()>>(),
        "Result<Proven<u64>, ()> should be same size as Result<u64, ()>"
    );
}

#[test]
fn test_proof_clone_copy() {
    let nz1 = NonZero::<u64>::new(42).unwrap();
    let nz2 = nz1; // Copy
    let nz3 = nz1.clone();

    assert_eq!(*nz1.get(), 42);
    assert_eq!(*nz2.get(), 42);
    assert_eq!(*nz3.get(), 42);
}
