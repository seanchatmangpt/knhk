// knhk-hot/src/simd_predicates.c
// Week 2: SIMD Predicate Matching Optimization
// Target: ≤0.5 ticks (4x speedup vs 2-tick sequential)

#include <stdint.h>
#include <stdbool.h>
#include <stddef.h>

// ============================================================================
// ARM64 NEON SIMD Implementation (Target: Apple Silicon, AWS Graviton)
// ============================================================================

#ifdef __aarch64__
#include <arm_neon.h>

// Match predicate against array using NEON (processes 2 × u64 per iteration)
// Returns true if ANY predicate matches target
bool knhk_match_predicates_simd_arm64(
    const uint64_t* predicates,
    size_t count,
    uint64_t target
) {
    // Broadcast target to both lanes (2 × u64)
    uint64x2_t target_vec = vdupq_n_u64(target);

    // Process 2 predicates at a time
    size_t i = 0;
    for (; i + 1 < count; i += 2) {
        // Load 2 predicates (128-bit = 2 × 64-bit)
        uint64x2_t p_vec = vld1q_u64(&predicates[i]);

        // Compare: result[lane] = 0xFFFF...FFFF if equal, else 0x0
        uint64x2_t cmp = vceqq_u64(p_vec, target_vec);

        // Extract comparison result: any non-zero means match
        // vgetq_lane extracts one 64-bit lane
        if (vgetq_lane_u64(cmp, 0) != 0 || vgetq_lane_u64(cmp, 1) != 0) {
            return true;
        }
    }

    // Handle remaining predicate (if count is odd)
    if (i < count && predicates[i] == target) {
        return true;
    }

    return false;
}

// Find ALL matching predicates (returns count of matches, fills indices)
// indices must be allocated with size >= count
size_t knhk_find_predicates_simd_arm64(
    const uint64_t* predicates,
    size_t count,
    uint64_t target,
    size_t* indices,
    size_t max_matches
) {
    uint64x2_t target_vec = vdupq_n_u64(target);
    size_t match_count = 0;

    size_t i = 0;
    for (; i + 1 < count && match_count < max_matches; i += 2) {
        uint64x2_t p_vec = vld1q_u64(&predicates[i]);
        uint64x2_t cmp = vceqq_u64(p_vec, target_vec);

        // Check each lane
        if (vgetq_lane_u64(cmp, 0) != 0) {
            indices[match_count++] = i;
            if (match_count >= max_matches) break;
        }
        if (vgetq_lane_u64(cmp, 1) != 0 && match_count < max_matches) {
            indices[match_count++] = i + 1;
        }
    }

    // Handle remaining predicate
    if (i < count && match_count < max_matches && predicates[i] == target) {
        indices[match_count++] = i;
    }

    return match_count;
}

#endif // __aarch64__

// ============================================================================
// x86_64 AVX2 SIMD Implementation (Target: Intel/AMD servers)
// ============================================================================

#ifdef __x86_64__
#include <immintrin.h>

// Match predicate against array using AVX2 (processes 4 × u64 per iteration)
bool knhk_match_predicates_simd_x86(
    const uint64_t* predicates,
    size_t count,
    uint64_t target
) {
    // Broadcast target to all 4 lanes (4 × u64 = 256 bits)
    __m256i target_vec = _mm256_set1_epi64x(target);

    // Process 4 predicates at a time
    size_t i = 0;
    for (; i + 3 < count; i += 4) {
        // Load 4 predicates (256-bit = 4 × 64-bit)
        __m256i p_vec = _mm256_loadu_si256((__m256i*)&predicates[i]);

        // Compare: result[lane] = 0xFFFF...FFFF if equal, else 0x0
        __m256i cmp = _mm256_cmpeq_epi64(p_vec, target_vec);

        // Extract comparison result as bitmask (1 bit per lane)
        int mask = _mm256_movemask_epi8(cmp);

        // If any bit set, we have a match
        if (mask != 0) {
            return true;
        }
    }

    // Handle remaining predicates (scalar fallback)
    for (; i < count; i++) {
        if (predicates[i] == target) {
            return true;
        }
    }

    return false;
}

// Find ALL matching predicates (AVX2 version)
size_t knhk_find_predicates_simd_x86(
    const uint64_t* predicates,
    size_t count,
    uint64_t target,
    size_t* indices,
    size_t max_matches
) {
    __m256i target_vec = _mm256_set1_epi64x(target);
    size_t match_count = 0;

    size_t i = 0;
    for (; i + 3 < count && match_count < max_matches; i += 4) {
        __m256i p_vec = _mm256_loadu_si256((__m256i*)&predicates[i]);
        __m256i cmp = _mm256_cmpeq_epi64(p_vec, target_vec);
        int mask = _mm256_movemask_epi8(cmp);

        // Check each lane (8 bytes per lane, so check every 8th bit)
        for (int lane = 0; lane < 4 && match_count < max_matches; lane++) {
            if (mask & (0xFF << (lane * 8))) {
                indices[match_count++] = i + lane;
            }
        }
    }

    // Handle remaining predicates
    for (; i < count && match_count < max_matches; i++) {
        if (predicates[i] == target) {
            indices[match_count++] = i;
        }
    }

    return match_count;
}

#endif // __x86_64__

// ============================================================================
// Scalar Fallback (for platforms without SIMD)
// ============================================================================

bool knhk_match_predicates_scalar(
    const uint64_t* predicates,
    size_t count,
    uint64_t target
) {
    for (size_t i = 0; i < count; i++) {
        if (predicates[i] == target) {
            return true;
        }
    }
    return false;
}

size_t knhk_find_predicates_scalar(
    const uint64_t* predicates,
    size_t count,
    uint64_t target,
    size_t* indices,
    size_t max_matches
) {
    size_t match_count = 0;
    for (size_t i = 0; i < count && match_count < max_matches; i++) {
        if (predicates[i] == target) {
            indices[match_count++] = i;
        }
    }
    return match_count;
}

// ============================================================================
// Public API (auto-dispatches to best available implementation)
// ============================================================================

bool knhk_match_predicates(
    const uint64_t* predicates,
    size_t count,
    uint64_t target
) {
    if (!predicates || count == 0) {
        return false;
    }

#ifdef __aarch64__
    return knhk_match_predicates_simd_arm64(predicates, count, target);
#elif defined(__x86_64__)
    return knhk_match_predicates_simd_x86(predicates, count, target);
#else
    return knhk_match_predicates_scalar(predicates, count, target);
#endif
}

size_t knhk_find_predicates(
    const uint64_t* predicates,
    size_t count,
    uint64_t target,
    size_t* indices,
    size_t max_matches
) {
    if (!predicates || count == 0 || !indices || max_matches == 0) {
        return 0;
    }

#ifdef __aarch64__
    return knhk_find_predicates_simd_arm64(predicates, count, target, indices, max_matches);
#elif defined(__x86_64__)
    return knhk_find_predicates_simd_x86(predicates, count, target, indices, max_matches);
#else
    return knhk_find_predicates_scalar(predicates, count, target, indices, max_matches);
#endif
}
