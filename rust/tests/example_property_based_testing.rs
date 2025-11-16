//! Property-Based Testing Examples with chicago-tdd-tools v1.3.0
//! Requires: features = ["testing-extras"]
//!
//! Property-based testing: Generate random test data and verify properties hold
//! Examples: commutativity, associativity, distributivity, idempotence

#[cfg(feature = "testing-extras")]
mod property_tests {
    use chicago_tdd_tools::prelude::*;

    // ========================================================================
    // 1. ARITHMETIC PROPERTIES
    // ========================================================================

    test!(test_addition_commutativity, {
        // Property: a + b == b + a (commutativity)
        let test_cases = vec![
            (5, 3),
            (10, 20),
            (0, 100),
            (i32::MAX - 1, 1),
        ];

        for (a, b) in test_cases {
            assert_eq!(a + b, b + a, "Commutativity failed for {} + {}", a, b);
        }
    });

    test!(test_addition_associativity, {
        // Property: (a + b) + c == a + (b + c) (associativity)
        let test_cases = vec![
            (5, 3, 2),
            (10, 20, 30),
            (1, 2, 3),
        ];

        for (a, b, c) in test_cases {
            let left = (a + b) + c;
            let right = a + (b + c);
            assert_eq!(left, right, "Associativity failed for ({} + {}) + {} vs {} + ({} + {})",
                a, b, c, a, b, c);
        }
    });

    test!(test_multiplication_commutativity, {
        // Property: a * b == b * a
        let test_cases = vec![
            (5, 3),
            (10, 2),
            (0, 100),
            (7, 8),
        ];

        for (a, b) in test_cases {
            assert_eq!(a * b, b * a, "Multiplication commutativity failed");
        }
    });

    // ========================================================================
    // 2. DISTRIBUTIVITY PROPERTIES
    // ========================================================================

    test!(test_distributivity, {
        // Property: a * (b + c) == (a * b) + (a * c)
        let test_cases = vec![
            (2, 3, 4),
            (5, 1, 2),
            (10, 2, 3),
        ];

        for (a, b, c) in test_cases {
            let left = a * (b + c);
            let right = (a * b) + (a * c);
            assert_eq!(left, right, "Distributivity failed for {} * ({} + {})", a, b, c);
        }
    });

    // ========================================================================
    // 3. COLLECTION PROPERTIES
    // ========================================================================

    test!(test_collection_reverse_idempotence, {
        // Property: reverse(reverse(list)) == list (idempotence)
        let lists = vec![
            vec![1, 2, 3],
            vec![],
            vec![42],
            vec!["a", "b", "c"],
        ];

        for mut list in lists {
            let original = list.clone();
            list.reverse();
            list.reverse();
            assert_eq!(list, original, "Double reverse should equal original");
        }
    });

    test!(test_collection_length_after_operations, {
        // Property: length(list) remains same after non-modifying operations
        let mut list = vec![1, 2, 3, 4, 5];
        let original_len = list.len();

        let _ = list.iter().map(|x| x * 2).collect::<Vec<_>>();
        assert_eq!(list.len(), original_len, "Length should not change");

        let filtered: Vec<_> = list.iter().filter(|x| x % 2 == 0).copied().collect();
        assert_eq!(list.len(), original_len, "Length should remain unchanged");
    });

    test!(test_concatenation_properties, {
        // Property: (a ++ b) ++ c == a ++ (b ++ c)
        let mut a = vec![1, 2];
        let mut b = vec![3, 4];
        let c = vec![5, 6];

        // Left associative
        let mut left = a.clone();
        left.extend(b.clone());
        left.extend(c.clone());

        // Right associative
        let mut right = a.clone();
        let mut temp = b.clone();
        temp.extend(c.clone());
        right.extend(temp);

        assert_eq!(left, right, "Concatenation should be associative");
    });

    // ========================================================================
    // 4. STRING PROPERTIES
    // ========================================================================

    test!(test_string_length_properties, {
        // Property: "ab".len() == "a".len() + "b".len()
        let test_cases = vec![
            ("hello", "world"),
            ("a", "b"),
            ("", "test"),
            ("test", ""),
        ];

        for (a, b) in test_cases {
            let concatenated = format!("{}{}", a, b);
            let individual_len = a.len() + b.len();
            assert_eq!(
                concatenated.len(), individual_len,
                "Length property failed for '{}' + '{}'", a, b
            );
        }
    });

    test!(test_string_case_conversion, {
        // Property: str.to_lowercase().to_uppercase().to_lowercase() == str.to_lowercase()
        let strings = vec!["HELLO", "World", "TeSt"];

        for s in strings {
            let lower = s.to_lowercase();
            let upper_lower = s.to_uppercase().to_lowercase();
            assert_eq!(lower, upper_lower, "Case conversion property failed for {}", s);
        }
    });

    // ========================================================================
    // 5. NUMERICAL PROPERTIES
    // ========================================================================

    test!(test_identity_property, {
        // Property: a + 0 == a (additive identity)
        let test_values = vec![0, 1, -1, 42, i32::MAX - 1];

        for a in test_values {
            assert_eq!(a + 0, a, "Additive identity failed for {}", a);
            assert_eq!(a * 1, a, "Multiplicative identity failed for {}", a);
        }
    });

    test!(test_zero_property, {
        // Property: a * 0 == 0
        let test_values = vec![0, 1, 42, 100, -5];

        for a in test_values {
            assert_eq!(a * 0, 0, "Multiplication by zero failed for {}", a);
        }
    });

    test!(test_order_preservation, {
        // Property: if a < b && b < c then a < c
        let test_cases = vec![
            (1, 5, 10),
            (0, 50, 100),
            (-10, 0, 10),
        ];

        for (a, b, c) in test_cases {
            if a < b && b < c {
                assert!(a < c, "Transitivity failed for {} < {} < {}", a, b, c);
            }
        }
    });

    // ========================================================================
    // 6. BOOLEAN LOGIC PROPERTIES
    // ========================================================================

    test!(test_boolean_negation, {
        // Property: !(!p) == p (double negation)
        let propositions = vec![true, false];

        for p in propositions {
            assert_eq!(!(!p), p, "Double negation failed for {}", p);
        }
    });

    test!(test_boolean_demorgan_laws, {
        // Property: !(a && b) == (!a || !b)
        let test_cases = vec![
            (true, true),
            (true, false),
            (false, true),
            (false, false),
        ];

        for (a, b) in test_cases {
            let left = !(a && b);
            let right = (!a) || (!b);
            assert_eq!(left, right, "De Morgan's law failed for ({}, {})", a, b);
        }
    });

    // ========================================================================
    // 7. FUNCTION PROPERTIES
    // ========================================================================

    test!(test_function_idempotence, {
        // Property: abs(abs(n)) == abs(n)
        let values = vec![-5, -1, 0, 1, 5];

        for n in values {
            let once = n.abs();
            let twice = once.abs();
            assert_eq!(once, twice, "abs() idempotence failed for {}", n);
        }
    });

    test!(test_function_monotonicity, {
        // Property: if a <= b then f(a) <= f(b) for monotonic function
        let f = |x: i32| x * x; // Square function is monotonic for positive numbers

        let test_cases = vec![
            (0, 1),
            (1, 2),
            (2, 3),
        ];

        for (a, b) in test_cases {
            if a <= b {
                assert!(f(a) <= f(b), "Monotonicity failed for f({}) vs f({})", a, b);
            }
        }
    });

    // ========================================================================
    // 8. COMPLEX PROPERTY CHAINS
    // ========================================================================

    #[derive(Debug, Clone)]
    struct TestData {
        values: Vec<i32>,
    }

    test!(test_complex_data_properties, {
        // Property: sorted(data) is always ascending
        let test_data = TestData {
            values: vec![3, 1, 4, 1, 5, 9, 2, 6],
        };

        let mut sorted = test_data.values.clone();
        sorted.sort();

        // Property: all pairs are in order
        for i in 0..sorted.len().saturating_sub(1) {
            assert!(sorted[i] <= sorted[i + 1], "Sorted property failed at index {}", i);
        }
    });

    test!(test_multiple_properties_in_sequence, {
        // Test multiple properties on same data
        let list = vec![1, 2, 3, 4, 5];

        // Property 1: length is correct
        assert_eq!(list.len(), 5);

        // Property 2: is sorted
        assert!(list.windows(2).all(|w| w[0] <= w[1]));

        // Property 3: sum is correct
        let sum: i32 = list.iter().sum();
        assert_eq!(sum, 15);

        // Property 4: contains expected elements
        assert!(list.contains(&3));
    });
}

#[cfg(not(feature = "testing-extras"))]
mod property_tests_disabled {
    test!(test_property_testing_requires_feature, {
        // This test runs when testing-extras is NOT enabled
        // It documents that property-based testing requires the feature
        assert!(true);
    });
}
