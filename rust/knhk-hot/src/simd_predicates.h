// knhk-hot/src/simd_predicates.h
// Week 2: SIMD Predicate Matching - Public API

#ifndef KNHK_SIMD_PREDICATES_H
#define KNHK_SIMD_PREDICATES_H

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// ============================================================================
// Public API
// ============================================================================

/**
 * Match a target predicate against an array of predicates (SIMD-optimized)
 *
 * Returns true if ANY predicate in the array matches the target.
 *
 * Performance:
 * - ARM64 NEON: ≤0.5 ticks (2 predicates/cycle)
 * - x86_64 AVX2: ≤0.25 ticks (4 predicates/cycle)
 * - Scalar fallback: ≤2 ticks (1 predicate/cycle)
 *
 * @param predicates Array of predicates to search (must be 64-byte aligned for best perf)
 * @param count Number of predicates in array
 * @param target Target predicate to match
 * @return true if match found, false otherwise
 */
bool knhk_match_predicates(
    const uint64_t* predicates,
    size_t count,
    uint64_t target
);

/**
 * Find ALL matching predicates and return their indices (SIMD-optimized)
 *
 * Fills `indices` array with positions of all matches (up to max_matches).
 *
 * @param predicates Array of predicates to search
 * @param count Number of predicates in array
 * @param target Target predicate to match
 * @param indices Output array for match indices (must be allocated with size >= max_matches)
 * @param max_matches Maximum number of matches to return
 * @return Number of matches found (≤ max_matches)
 */
size_t knhk_find_predicates(
    const uint64_t* predicates,
    size_t count,
    uint64_t target,
    size_t* indices,
    size_t max_matches
);

// ============================================================================
// Platform-Specific Implementations (for testing/benchmarking)
// ============================================================================

#ifdef __aarch64__
bool knhk_match_predicates_simd_arm64(const uint64_t* predicates, size_t count, uint64_t target);
size_t knhk_find_predicates_simd_arm64(const uint64_t* predicates, size_t count, uint64_t target, size_t* indices, size_t max_matches);
#endif

#ifdef __x86_64__
bool knhk_match_predicates_simd_x86(const uint64_t* predicates, size_t count, uint64_t target);
size_t knhk_find_predicates_simd_x86(const uint64_t* predicates, size_t count, uint64_t target, size_t* indices, size_t max_matches);
#endif

bool knhk_match_predicates_scalar(const uint64_t* predicates, size_t count, uint64_t target);
size_t knhk_find_predicates_scalar(const uint64_t* predicates, size_t count, uint64_t target, size_t* indices, size_t max_matches);

#ifdef __cplusplus
}
#endif

#endif // KNHK_SIMD_PREDICATES_H
