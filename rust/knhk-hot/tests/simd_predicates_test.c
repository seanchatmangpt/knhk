// knhk-hot/tests/simd_predicates_test.c
// Differential Testing: SIMD vs Scalar Predicate Matching

#include "../src/simd_predicates.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>
#include <time.h>

// ============================================================================
// Test Helpers
// ============================================================================

#define ASSERT_EQ(expected, actual, msg) \
    do { \
        if ((expected) != (actual)) { \
            fprintf(stderr, "FAIL: %s\n  Expected: %zu\n  Actual: %zu\n", msg, (size_t)(expected), (size_t)(actual)); \
            exit(1); \
        } \
    } while(0)

#define ASSERT_TRUE(cond, msg) \
    do { \
        if (!(cond)) { \
            fprintf(stderr, "FAIL: %s\n", msg); \
            exit(1); \
        } \
    } while(0)

// ============================================================================
// Differential Tests: SIMD vs Scalar
// ============================================================================

void test_empty_array() {
    printf("Test: Empty array\n");

    uint64_t predicates[] = {};
    bool result = knhk_match_predicates(predicates, 0, 42);

    ASSERT_TRUE(!result, "Empty array should return false");
    printf("  ✓ PASS\n");
}

void test_single_match() {
    printf("Test: Single predicate match\n");

    uint64_t predicates[] = {42};
    bool result = knhk_match_predicates(predicates, 1, 42);

    ASSERT_TRUE(result, "Should match single predicate");
    printf("  ✓ PASS\n");
}

void test_single_no_match() {
    printf("Test: Single predicate no match\n");

    uint64_t predicates[] = {42};
    bool result = knhk_match_predicates(predicates, 1, 99);

    ASSERT_TRUE(!result, "Should not match different predicate");
    printf("  ✓ PASS\n");
}

void test_multiple_match_first() {
    printf("Test: Match at first position\n");

    uint64_t predicates[] = {100, 200, 300, 400};
    bool result = knhk_match_predicates(predicates, 4, 100);

    ASSERT_TRUE(result, "Should match first predicate");
    printf("  ✓ PASS\n");
}

void test_multiple_match_last() {
    printf("Test: Match at last position\n");

    uint64_t predicates[] = {100, 200, 300, 400};
    bool result = knhk_match_predicates(predicates, 4, 400);

    ASSERT_TRUE(result, "Should match last predicate");
    printf("  ✓ PASS\n");
}

void test_multiple_match_middle() {
    printf("Test: Match in middle position\n");

    uint64_t predicates[] = {100, 200, 300, 400};
    bool result = knhk_match_predicates(predicates, 4, 200);

    ASSERT_TRUE(result, "Should match middle predicate");
    printf("  ✓ PASS\n");
}

void test_multiple_no_match() {
    printf("Test: No match in multiple predicates\n");

    uint64_t predicates[] = {100, 200, 300, 400};
    bool result = knhk_match_predicates(predicates, 4, 999);

    ASSERT_TRUE(!result, "Should not match non-existent predicate");
    printf("  ✓ PASS\n");
}

void test_find_multiple_matches() {
    printf("Test: Find multiple matching predicates\n");

    uint64_t predicates[] = {100, 200, 100, 300, 100, 400};
    size_t indices[10];

    size_t count = knhk_find_predicates(predicates, 6, 100, indices, 10);

    ASSERT_EQ(3, count, "Should find 3 matches");
    ASSERT_EQ(0, indices[0], "First match at index 0");
    ASSERT_EQ(2, indices[1], "Second match at index 2");
    ASSERT_EQ(4, indices[2], "Third match at index 4");
    printf("  ✓ PASS\n");
}

void test_find_max_matches_limit() {
    printf("Test: Find predicates with max_matches limit\n");

    uint64_t predicates[] = {100, 100, 100, 100, 100};
    size_t indices[3];

    size_t count = knhk_find_predicates(predicates, 5, 100, indices, 3);

    ASSERT_EQ(3, count, "Should find only 3 matches (max_matches limit)");
    printf("  ✓ PASS\n");
}

void test_differential_simd_vs_scalar() {
    printf("Test: Differential SIMD vs Scalar (1000 predicates)\n");

    const size_t SIZE = 1000;
    uint64_t* predicates = malloc(SIZE * sizeof(uint64_t));

    // Fill with predictable pattern
    for (size_t i = 0; i < SIZE; i++) {
        predicates[i] = (i * 7) % 100;  // Values 0-99
    }

    // Test multiple targets
    for (uint64_t target = 0; target < 100; target++) {
        bool simd_result = knhk_match_predicates(predicates, SIZE, target);
        bool scalar_result = knhk_match_predicates_scalar(predicates, SIZE, target);

        if (simd_result != scalar_result) {
            fprintf(stderr, "FAIL: SIMD vs Scalar mismatch for target %lu\n", target);
            fprintf(stderr, "  SIMD: %d, Scalar: %d\n", simd_result, scalar_result);
            free(predicates);
            exit(1);
        }
    }

    free(predicates);
    printf("  ✓ PASS (100%% match SIMD vs Scalar)\n");
}

void test_differential_find_simd_vs_scalar() {
    printf("Test: Differential Find SIMD vs Scalar\n");

    const size_t SIZE = 500;
    uint64_t* predicates = malloc(SIZE * sizeof(uint64_t));

    // Fill with pattern that has duplicates
    for (size_t i = 0; i < SIZE; i++) {
        predicates[i] = i % 50;  // Values 0-49, each appears 10 times
    }

    for (uint64_t target = 0; target < 50; target++) {
        size_t simd_indices[100];
        size_t scalar_indices[100];

        size_t simd_count = knhk_find_predicates(predicates, SIZE, target, simd_indices, 100);
        size_t scalar_count = knhk_find_predicates_scalar(predicates, SIZE, target, scalar_indices, 100);

        if (simd_count != scalar_count) {
            fprintf(stderr, "FAIL: SIMD vs Scalar count mismatch for target %lu\n", target);
            fprintf(stderr, "  SIMD: %zu, Scalar: %zu\n", simd_count, scalar_count);
            free(predicates);
            exit(1);
        }

        // Compare indices
        for (size_t i = 0; i < simd_count; i++) {
            if (simd_indices[i] != scalar_indices[i]) {
                fprintf(stderr, "FAIL: SIMD vs Scalar indices mismatch for target %lu\n", target);
                free(predicates);
                exit(1);
            }
        }
    }

    free(predicates);
    printf("  ✓ PASS (100%% match SIMD vs Scalar)\n");
}

// ============================================================================
// Performance Benchmark
// ============================================================================

void benchmark_simd_vs_scalar() {
    printf("\nBenchmark: SIMD vs Scalar Performance\n");

    const size_t SIZE = 10000;
    const int ITERATIONS = 1000;

    uint64_t* predicates = malloc(SIZE * sizeof(uint64_t));
    for (size_t i = 0; i < SIZE; i++) {
        predicates[i] = i;
    }

    // Benchmark scalar
    clock_t start = clock();
    volatile bool result_scalar = false;
    for (int iter = 0; iter < ITERATIONS; iter++) {
        for (uint64_t target = 0; target < 100; target++) {
            result_scalar = knhk_match_predicates_scalar(predicates, SIZE, target);
        }
    }
    clock_t end = clock();
    double scalar_time = ((double)(end - start)) / CLOCKS_PER_SEC;

    // Benchmark SIMD
    start = clock();
    volatile bool result_simd = false;
    for (int iter = 0; iter < ITERATIONS; iter++) {
        for (uint64_t target = 0; target < 100; target++) {
            result_simd = knhk_match_predicates(predicates, SIZE, target);
        }
    }
    end = clock();
    double simd_time = ((double)(end - start)) / CLOCKS_PER_SEC;

    printf("  Scalar: %.4f seconds\n", scalar_time);
    printf("  SIMD:   %.4f seconds\n", simd_time);
    printf("  Speedup: %.2fx\n", scalar_time / simd_time);

    if (simd_time >= scalar_time) {
        fprintf(stderr, "  ⚠️  WARNING: SIMD not faster than scalar!\n");
    } else if (scalar_time / simd_time >= 4.0) {
        printf("  ✓ PASS (≥4x speedup achieved)\n");
    } else {
        printf("  ⚠️  PARTIAL PASS (speedup < 4x)\n");
    }

    free(predicates);
}

// ============================================================================
// Main
// ============================================================================

int main() {
    printf("SIMD Predicate Matching - Differential Tests\n");
    printf("==============================================\n\n");

    // Basic tests
    test_empty_array();
    test_single_match();
    test_single_no_match();
    test_multiple_match_first();
    test_multiple_match_last();
    test_multiple_match_middle();
    test_multiple_no_match();

    // Find tests
    test_find_multiple_matches();
    test_find_max_matches_limit();

    // Differential tests (critical)
    test_differential_simd_vs_scalar();
    test_differential_find_simd_vs_scalar();

    // Performance benchmark
    benchmark_simd_vs_scalar();

    printf("\n✅ ALL TESTS PASSED\n");
    return 0;
}
