// kernels.c
// Branchless SIMD kernel implementations for μ(Δ) reconciliation
// Target: ≤2ns per operation (≤8 ticks @ 250ps/tick)
// Implements 6 hot-path kernels with zero branches

#include "knhk/kernels.h"
#include "knhk/pmu.h"
#include <stdint.h>
#include <stddef.h>

#if defined(__x86_64__) || defined(_M_X64)
#include <immintrin.h>  // AVX2
#elif defined(__aarch64__) || defined(_M_ARM64)
#include <arm_neon.h>   // NEON
#endif

// Kernel 1: ASK(S,P) - Check if (s,p) pattern exists
// Returns: CPU cycles, sets out_mask with matching lanes
uint64_t knhk_kernel_ask_sp_impl(
    const uint64_t *s_lane,
    const uint64_t *p_lane,
    const uint64_t *o_lane,
    size_t n_rows,
    uint64_t *out_mask
) {
    (void)o_lane;  // Unused for ASK(S,P) - only checks subject and predicate
    uint64_t start = knhk_pmu_rdtsc();

    // We need to extract S and P targets from first element (convention)
    // For ASK(S,P), we check if ANY row matches both subject and predicate
    uint64_t target_s = s_lane[0];
    uint64_t target_p = p_lane[0];

#if defined(__x86_64__) || defined(_M_X64)
    // Load targets into SIMD registers
    __m256i s_target = _mm256_set1_epi64x((long long)target_s);
    __m256i p_target = _mm256_set1_epi64x((long long)target_p);

    // Load first 4 rows
    __m256i s_vec0 = _mm256_loadu_si256((const __m256i *)(s_lane + 0));
    __m256i p_vec0 = _mm256_loadu_si256((const __m256i *)(p_lane + 0));

    // Compare subjects and predicates
    __m256i s_match0 = _mm256_cmpeq_epi64(s_vec0, s_target);
    __m256i p_match0 = _mm256_cmpeq_epi64(p_vec0, p_target);

    // Both must match (AND)
    __m256i match0 = _mm256_and_si256(s_match0, p_match0);

    // Load next 4 rows
    __m256i s_vec1 = _mm256_loadu_si256((const __m256i *)(s_lane + 4));
    __m256i p_vec1 = _mm256_loadu_si256((const __m256i *)(p_lane + 4));

    __m256i s_match1 = _mm256_cmpeq_epi64(s_vec1, s_target);
    __m256i p_match1 = _mm256_cmpeq_epi64(p_vec1, p_target);

    __m256i match1 = _mm256_and_si256(s_match1, p_match1);

    // Extract mask bits
    uint32_t mask0 = _mm256_movemask_pd(_mm256_castsi256_pd(match0));
    uint32_t mask1 = _mm256_movemask_pd(_mm256_castsi256_pd(match1));

    uint64_t mask = ((uint64_t)mask0) | (((uint64_t)mask1) << 4);

    // Apply row count mask (branchless)
    uint64_t len_mask = ((1ULL << n_rows) - 1) & 0xFFULL;
    mask &= len_mask;

    *out_mask = mask;

#elif defined(__aarch64__) || defined(_M_ARM64)
    // NEON implementation
    uint64x2_t s_target = vdupq_n_u64(target_s);
    uint64x2_t p_target = vdupq_n_u64(target_p);

    uint64x2_t s_vec0 = vld1q_u64(s_lane + 0);
    uint64x2_t p_vec0 = vld1q_u64(p_lane + 0);
    uint64x2_t s_vec1 = vld1q_u64(s_lane + 2);
    uint64x2_t p_vec1 = vld1q_u64(p_lane + 2);
    uint64x2_t s_vec2 = vld1q_u64(s_lane + 4);
    uint64x2_t p_vec2 = vld1q_u64(p_lane + 4);
    uint64x2_t s_vec3 = vld1q_u64(s_lane + 6);
    uint64x2_t p_vec3 = vld1q_u64(p_lane + 6);

    uint64x2_t s_match0 = vceqq_u64(s_vec0, s_target);
    uint64x2_t p_match0 = vceqq_u64(p_vec0, p_target);
    uint64x2_t match0 = vandq_u64(s_match0, p_match0);

    uint64x2_t s_match1 = vceqq_u64(s_vec1, s_target);
    uint64x2_t p_match1 = vceqq_u64(p_vec1, p_target);
    uint64x2_t match1 = vandq_u64(s_match1, p_match1);

    uint64x2_t s_match2 = vceqq_u64(s_vec2, s_target);
    uint64x2_t p_match2 = vceqq_u64(p_vec2, p_target);
    uint64x2_t match2 = vandq_u64(s_match2, p_match2);

    uint64x2_t s_match3 = vceqq_u64(s_vec3, s_target);
    uint64x2_t p_match3 = vceqq_u64(p_vec3, p_target);
    uint64x2_t match3 = vandq_u64(s_match3, p_match3);

    // Extract mask bits
    uint64_t mask = ((vgetq_lane_u64(match0, 0) >> 63) << 0) |
                     ((vgetq_lane_u64(match0, 1) >> 63) << 1) |
                     ((vgetq_lane_u64(match1, 0) >> 63) << 2) |
                     ((vgetq_lane_u64(match1, 1) >> 63) << 3) |
                     ((vgetq_lane_u64(match2, 0) >> 63) << 4) |
                     ((vgetq_lane_u64(match2, 1) >> 63) << 5) |
                     ((vgetq_lane_u64(match3, 0) >> 63) << 6) |
                     ((vgetq_lane_u64(match3, 1) >> 63) << 7);

    uint64_t len_mask = ((1ULL << n_rows) - 1) & 0xFFULL;
    mask &= len_mask;

    *out_mask = mask;

#else
    // Scalar fallback
    uint64_t mask = 0;
    for (size_t i = 0; i < n_rows && i < 8; i++) {
        uint64_t match = (s_lane[i] == target_s) & (p_lane[i] == target_p);
        mask |= (match << i);
    }
    *out_mask = mask;
#endif

    uint64_t end = knhk_pmu_rdtsc();
    return end - start;
}

// Kernel 2: COUNT(S,P) >= k - Cardinality check
uint64_t knhk_kernel_count_sp_ge_impl(
    const uint64_t *s_lane,
    const uint64_t *p_lane,
    const uint64_t *o_lane,
    size_t n_rows,
    uint64_t *out_mask
) {
    uint64_t start = knhk_pmu_rdtsc();

    uint64_t target_s = s_lane[0];
    uint64_t target_p = p_lane[0];
    uint64_t threshold = o_lane[0];  // k threshold in object slot

#if defined(__x86_64__) || defined(_M_X64)
    __m256i s_target = _mm256_set1_epi64x((long long)target_s);
    __m256i p_target = _mm256_set1_epi64x((long long)target_p);

    __m256i s_vec0 = _mm256_loadu_si256((const __m256i *)(s_lane + 0));
    __m256i p_vec0 = _mm256_loadu_si256((const __m256i *)(p_lane + 0));
    __m256i s_vec1 = _mm256_loadu_si256((const __m256i *)(s_lane + 4));
    __m256i p_vec1 = _mm256_loadu_si256((const __m256i *)(p_lane + 4));

    __m256i s_match0 = _mm256_cmpeq_epi64(s_vec0, s_target);
    __m256i p_match0 = _mm256_cmpeq_epi64(p_vec0, p_target);
    __m256i match0 = _mm256_and_si256(s_match0, p_match0);

    __m256i s_match1 = _mm256_cmpeq_epi64(s_vec1, s_target);
    __m256i p_match1 = _mm256_cmpeq_epi64(p_vec1, p_target);
    __m256i match1 = _mm256_and_si256(s_match1, p_match1);

    uint32_t mask0 = _mm256_movemask_pd(_mm256_castsi256_pd(match0));
    uint32_t mask1 = _mm256_movemask_pd(_mm256_castsi256_pd(match1));

    uint64_t match_mask = ((uint64_t)mask0) | (((uint64_t)mask1) << 4);
    uint64_t len_mask = ((1ULL << n_rows) - 1) & 0xFFULL;
    match_mask &= len_mask;

    // Count matches
    uint64_t count = (uint64_t)__builtin_popcountll(match_mask);

    // Check if count >= threshold (branchless)
    uint64_t result = (count >= threshold) ? UINT64_MAX : 0;
    *out_mask = result;

#elif defined(__aarch64__) || defined(_M_ARM64)
    uint64x2_t s_target = vdupq_n_u64(target_s);
    uint64x2_t p_target = vdupq_n_u64(target_p);

    uint64x2_t s_vec0 = vld1q_u64(s_lane + 0);
    uint64x2_t p_vec0 = vld1q_u64(p_lane + 0);
    uint64x2_t s_vec1 = vld1q_u64(s_lane + 2);
    uint64x2_t p_vec1 = vld1q_u64(p_lane + 2);
    uint64x2_t s_vec2 = vld1q_u64(s_lane + 4);
    uint64x2_t p_vec2 = vld1q_u64(p_lane + 4);
    uint64x2_t s_vec3 = vld1q_u64(s_lane + 6);
    uint64x2_t p_vec3 = vld1q_u64(p_lane + 6);

    uint64x2_t match0 = vandq_u64(vceqq_u64(s_vec0, s_target), vceqq_u64(p_vec0, p_target));
    uint64x2_t match1 = vandq_u64(vceqq_u64(s_vec1, s_target), vceqq_u64(p_vec1, p_target));
    uint64x2_t match2 = vandq_u64(vceqq_u64(s_vec2, s_target), vceqq_u64(p_vec2, p_target));
    uint64x2_t match3 = vandq_u64(vceqq_u64(s_vec3, s_target), vceqq_u64(p_vec3, p_target));

    uint64_t match_mask = ((vgetq_lane_u64(match0, 0) >> 63) << 0) |
                           ((vgetq_lane_u64(match0, 1) >> 63) << 1) |
                           ((vgetq_lane_u64(match1, 0) >> 63) << 2) |
                           ((vgetq_lane_u64(match1, 1) >> 63) << 3) |
                           ((vgetq_lane_u64(match2, 0) >> 63) << 4) |
                           ((vgetq_lane_u64(match2, 1) >> 63) << 5) |
                           ((vgetq_lane_u64(match3, 0) >> 63) << 6) |
                           ((vgetq_lane_u64(match3, 1) >> 63) << 7);

    uint64_t len_mask = ((1ULL << n_rows) - 1) & 0xFFULL;
    match_mask &= len_mask;

    uint64_t count = (uint64_t)__builtin_popcountll(match_mask);
    uint64_t result = (count >= threshold) ? UINT64_MAX : 0;
    *out_mask = result;

#else
    uint64_t mask = 0;
    for (size_t i = 0; i < n_rows && i < 8; i++) {
        uint64_t match = (s_lane[i] == target_s) & (p_lane[i] == target_p);
        mask |= (match << i);
    }
    uint64_t count = (uint64_t)__builtin_popcountll(mask);
    *out_mask = (count >= threshold) ? UINT64_MAX : 0;
#endif

    uint64_t end = knhk_pmu_rdtsc();
    return end - start;
}

// Kernel 3: ASK(S,P,O) - Exact triple match
uint64_t knhk_kernel_ask_spo_impl(
    const uint64_t *s_lane,
    const uint64_t *p_lane,
    const uint64_t *o_lane,
    size_t n_rows,
    uint64_t *out_mask
) {
    uint64_t start = knhk_pmu_rdtsc();

    uint64_t target_s = s_lane[0];
    uint64_t target_p = p_lane[0];
    uint64_t target_o = o_lane[0];

#if defined(__x86_64__) || defined(_M_X64)
    __m256i s_target = _mm256_set1_epi64x((long long)target_s);
    __m256i p_target = _mm256_set1_epi64x((long long)target_p);
    __m256i o_target = _mm256_set1_epi64x((long long)target_o);

    __m256i s_vec0 = _mm256_loadu_si256((const __m256i *)(s_lane + 0));
    __m256i p_vec0 = _mm256_loadu_si256((const __m256i *)(p_lane + 0));
    __m256i o_vec0 = _mm256_loadu_si256((const __m256i *)(o_lane + 0));

    __m256i s_vec1 = _mm256_loadu_si256((const __m256i *)(s_lane + 4));
    __m256i p_vec1 = _mm256_loadu_si256((const __m256i *)(p_lane + 4));
    __m256i o_vec1 = _mm256_loadu_si256((const __m256i *)(o_lane + 4));

    __m256i s_match0 = _mm256_cmpeq_epi64(s_vec0, s_target);
    __m256i p_match0 = _mm256_cmpeq_epi64(p_vec0, p_target);
    __m256i o_match0 = _mm256_cmpeq_epi64(o_vec0, o_target);
    __m256i match0 = _mm256_and_si256(_mm256_and_si256(s_match0, p_match0), o_match0);

    __m256i s_match1 = _mm256_cmpeq_epi64(s_vec1, s_target);
    __m256i p_match1 = _mm256_cmpeq_epi64(p_vec1, p_target);
    __m256i o_match1 = _mm256_cmpeq_epi64(o_vec1, o_target);
    __m256i match1 = _mm256_and_si256(_mm256_and_si256(s_match1, p_match1), o_match1);

    uint32_t mask0 = _mm256_movemask_pd(_mm256_castsi256_pd(match0));
    uint32_t mask1 = _mm256_movemask_pd(_mm256_castsi256_pd(match1));

    uint64_t mask = ((uint64_t)mask0) | (((uint64_t)mask1) << 4);
    uint64_t len_mask = ((1ULL << n_rows) - 1) & 0xFFULL;
    mask &= len_mask;

    *out_mask = mask;

#elif defined(__aarch64__) || defined(_M_ARM64)
    uint64x2_t s_target = vdupq_n_u64(target_s);
    uint64x2_t p_target = vdupq_n_u64(target_p);
    uint64x2_t o_target = vdupq_n_u64(target_o);

    uint64x2_t s_vec0 = vld1q_u64(s_lane + 0);
    uint64x2_t p_vec0 = vld1q_u64(p_lane + 0);
    uint64x2_t o_vec0 = vld1q_u64(o_lane + 0);

    uint64x2_t s_vec1 = vld1q_u64(s_lane + 2);
    uint64x2_t p_vec1 = vld1q_u64(p_lane + 2);
    uint64x2_t o_vec1 = vld1q_u64(o_lane + 2);

    uint64x2_t s_vec2 = vld1q_u64(s_lane + 4);
    uint64x2_t p_vec2 = vld1q_u64(p_lane + 4);
    uint64x2_t o_vec2 = vld1q_u64(o_lane + 4);

    uint64x2_t s_vec3 = vld1q_u64(s_lane + 6);
    uint64x2_t p_vec3 = vld1q_u64(p_lane + 6);
    uint64x2_t o_vec3 = vld1q_u64(o_lane + 6);

    uint64x2_t match0 = vandq_u64(vandq_u64(vceqq_u64(s_vec0, s_target), vceqq_u64(p_vec0, p_target)), vceqq_u64(o_vec0, o_target));
    uint64x2_t match1 = vandq_u64(vandq_u64(vceqq_u64(s_vec1, s_target), vceqq_u64(p_vec1, p_target)), vceqq_u64(o_vec1, o_target));
    uint64x2_t match2 = vandq_u64(vandq_u64(vceqq_u64(s_vec2, s_target), vceqq_u64(p_vec2, p_target)), vceqq_u64(o_vec2, o_target));
    uint64x2_t match3 = vandq_u64(vandq_u64(vceqq_u64(s_vec3, s_target), vceqq_u64(p_vec3, p_target)), vceqq_u64(o_vec3, o_target));

    uint64_t mask = ((vgetq_lane_u64(match0, 0) >> 63) << 0) |
                     ((vgetq_lane_u64(match0, 1) >> 63) << 1) |
                     ((vgetq_lane_u64(match1, 0) >> 63) << 2) |
                     ((vgetq_lane_u64(match1, 1) >> 63) << 3) |
                     ((vgetq_lane_u64(match2, 0) >> 63) << 4) |
                     ((vgetq_lane_u64(match2, 1) >> 63) << 5) |
                     ((vgetq_lane_u64(match3, 0) >> 63) << 6) |
                     ((vgetq_lane_u64(match3, 1) >> 63) << 7);

    uint64_t len_mask = ((1ULL << n_rows) - 1) & 0xFFULL;
    mask &= len_mask;

    *out_mask = mask;

#else
    uint64_t mask = 0;
    for (size_t i = 0; i < n_rows && i < 8; i++) {
        uint64_t match = (s_lane[i] == target_s) & (p_lane[i] == target_p) & (o_lane[i] == target_o);
        mask |= (match << i);
    }
    *out_mask = mask;
#endif

    uint64_t end = knhk_pmu_rdtsc();
    return end - start;
}

// Kernel 4: VALIDATE(S,P) - Datatype validation
uint64_t knhk_kernel_validate_sp_impl(
    const uint64_t *s_lane,
    const uint64_t *p_lane,
    const uint64_t *o_lane,
    size_t n_rows,
    uint64_t *out_mask
) {
    uint64_t start = knhk_pmu_rdtsc();

    uint64_t target_s = s_lane[0];
    uint64_t target_p = p_lane[0];
    uint64_t datatype_hash = o_lane[0];  // Expected datatype hash

#if defined(__x86_64__) || defined(_M_X64)
    __m256i s_target = _mm256_set1_epi64x((long long)target_s);
    __m256i p_target = _mm256_set1_epi64x((long long)target_p);
    __m256i dt_target = _mm256_set1_epi64x((long long)datatype_hash);

    __m256i s_vec0 = _mm256_loadu_si256((const __m256i *)(s_lane + 0));
    __m256i p_vec0 = _mm256_loadu_si256((const __m256i *)(p_lane + 0));
    __m256i o_vec0 = _mm256_loadu_si256((const __m256i *)(o_lane + 0));

    __m256i s_vec1 = _mm256_loadu_si256((const __m256i *)(s_lane + 4));
    __m256i p_vec1 = _mm256_loadu_si256((const __m256i *)(p_lane + 4));
    __m256i o_vec1 = _mm256_loadu_si256((const __m256i *)(o_lane + 4));

    // Match S and P, validate O against datatype
    __m256i sp_match0 = _mm256_and_si256(_mm256_cmpeq_epi64(s_vec0, s_target), _mm256_cmpeq_epi64(p_vec0, p_target));
    __m256i dt_match0 = _mm256_cmpeq_epi64(o_vec0, dt_target);
    __m256i match0 = _mm256_and_si256(sp_match0, dt_match0);

    __m256i sp_match1 = _mm256_and_si256(_mm256_cmpeq_epi64(s_vec1, s_target), _mm256_cmpeq_epi64(p_vec1, p_target));
    __m256i dt_match1 = _mm256_cmpeq_epi64(o_vec1, dt_target);
    __m256i match1 = _mm256_and_si256(sp_match1, dt_match1);

    uint32_t mask0 = _mm256_movemask_pd(_mm256_castsi256_pd(match0));
    uint32_t mask1 = _mm256_movemask_pd(_mm256_castsi256_pd(match1));

    uint64_t mask = ((uint64_t)mask0) | (((uint64_t)mask1) << 4);
    uint64_t len_mask = ((1ULL << n_rows) - 1) & 0xFFULL;
    mask &= len_mask;

    *out_mask = mask;

#elif defined(__aarch64__) || defined(_M_ARM64)
    uint64x2_t s_target = vdupq_n_u64(target_s);
    uint64x2_t p_target = vdupq_n_u64(target_p);
    uint64x2_t dt_target = vdupq_n_u64(datatype_hash);

    uint64x2_t s_vec0 = vld1q_u64(s_lane + 0);
    uint64x2_t p_vec0 = vld1q_u64(p_lane + 0);
    uint64x2_t o_vec0 = vld1q_u64(o_lane + 0);

    uint64x2_t s_vec1 = vld1q_u64(s_lane + 2);
    uint64x2_t p_vec1 = vld1q_u64(p_lane + 2);
    uint64x2_t o_vec1 = vld1q_u64(o_lane + 2);

    uint64x2_t s_vec2 = vld1q_u64(s_lane + 4);
    uint64x2_t p_vec2 = vld1q_u64(p_lane + 4);
    uint64x2_t o_vec2 = vld1q_u64(o_lane + 4);

    uint64x2_t s_vec3 = vld1q_u64(s_lane + 6);
    uint64x2_t p_vec3 = vld1q_u64(p_lane + 6);
    uint64x2_t o_vec3 = vld1q_u64(o_lane + 6);

    uint64x2_t match0 = vandq_u64(vandq_u64(vceqq_u64(s_vec0, s_target), vceqq_u64(p_vec0, p_target)), vceqq_u64(o_vec0, dt_target));
    uint64x2_t match1 = vandq_u64(vandq_u64(vceqq_u64(s_vec1, s_target), vceqq_u64(p_vec1, p_target)), vceqq_u64(o_vec1, dt_target));
    uint64x2_t match2 = vandq_u64(vandq_u64(vceqq_u64(s_vec2, s_target), vceqq_u64(p_vec2, p_target)), vceqq_u64(o_vec2, dt_target));
    uint64x2_t match3 = vandq_u64(vandq_u64(vceqq_u64(s_vec3, s_target), vceqq_u64(p_vec3, p_target)), vceqq_u64(o_vec3, dt_target));

    uint64_t mask = ((vgetq_lane_u64(match0, 0) >> 63) << 0) |
                     ((vgetq_lane_u64(match0, 1) >> 63) << 1) |
                     ((vgetq_lane_u64(match1, 0) >> 63) << 2) |
                     ((vgetq_lane_u64(match1, 1) >> 63) << 3) |
                     ((vgetq_lane_u64(match2, 0) >> 63) << 4) |
                     ((vgetq_lane_u64(match2, 1) >> 63) << 5) |
                     ((vgetq_lane_u64(match3, 0) >> 63) << 6) |
                     ((vgetq_lane_u64(match3, 1) >> 63) << 7);

    uint64_t len_mask = ((1ULL << n_rows) - 1) & 0xFFULL;
    mask &= len_mask;

    *out_mask = mask;

#else
    uint64_t mask = 0;
    for (size_t i = 0; i < n_rows && i < 8; i++) {
        uint64_t match = (s_lane[i] == target_s) & (p_lane[i] == target_p) & (o_lane[i] == datatype_hash);
        mask |= (match << i);
    }
    *out_mask = mask;
#endif

    uint64_t end = knhk_pmu_rdtsc();
    return end - start;
}

// Kernel 5: UNIQUE(S,P) - Verify single value (cardinality exactly 1)
uint64_t knhk_kernel_unique_sp_impl(
    const uint64_t *s_lane,
    const uint64_t *p_lane,
    const uint64_t *o_lane,
    size_t n_rows,
    uint64_t *out_mask
) {
    (void)o_lane;  // Unused for UNIQUE(S,P) - only checks subject and predicate cardinality
    uint64_t start = knhk_pmu_rdtsc();

    uint64_t target_s = s_lane[0];
    uint64_t target_p = p_lane[0];

#if defined(__x86_64__) || defined(_M_X64)
    __m256i s_target = _mm256_set1_epi64x((long long)target_s);
    __m256i p_target = _mm256_set1_epi64x((long long)target_p);

    __m256i s_vec0 = _mm256_loadu_si256((const __m256i *)(s_lane + 0));
    __m256i p_vec0 = _mm256_loadu_si256((const __m256i *)(p_lane + 0));
    __m256i s_vec1 = _mm256_loadu_si256((const __m256i *)(s_lane + 4));
    __m256i p_vec1 = _mm256_loadu_si256((const __m256i *)(p_lane + 4));

    __m256i match0 = _mm256_and_si256(_mm256_cmpeq_epi64(s_vec0, s_target), _mm256_cmpeq_epi64(p_vec0, p_target));
    __m256i match1 = _mm256_and_si256(_mm256_cmpeq_epi64(s_vec1, s_target), _mm256_cmpeq_epi64(p_vec1, p_target));

    uint32_t mask0 = _mm256_movemask_pd(_mm256_castsi256_pd(match0));
    uint32_t mask1 = _mm256_movemask_pd(_mm256_castsi256_pd(match1));

    uint64_t match_mask = ((uint64_t)mask0) | (((uint64_t)mask1) << 4);
    uint64_t len_mask = ((1ULL << n_rows) - 1) & 0xFFULL;
    match_mask &= len_mask;

    // Count matches - must be exactly 1 for UNIQUE
    uint64_t count = (uint64_t)__builtin_popcountll(match_mask);
    uint64_t result = (count == 1) ? UINT64_MAX : 0;
    *out_mask = result;

#elif defined(__aarch64__) || defined(_M_ARM64)
    uint64x2_t s_target = vdupq_n_u64(target_s);
    uint64x2_t p_target = vdupq_n_u64(target_p);

    uint64x2_t s_vec0 = vld1q_u64(s_lane + 0);
    uint64x2_t p_vec0 = vld1q_u64(p_lane + 0);
    uint64x2_t s_vec1 = vld1q_u64(s_lane + 2);
    uint64x2_t p_vec1 = vld1q_u64(p_lane + 2);
    uint64x2_t s_vec2 = vld1q_u64(s_lane + 4);
    uint64x2_t p_vec2 = vld1q_u64(p_lane + 4);
    uint64x2_t s_vec3 = vld1q_u64(s_lane + 6);
    uint64x2_t p_vec3 = vld1q_u64(p_lane + 6);

    uint64x2_t match0 = vandq_u64(vceqq_u64(s_vec0, s_target), vceqq_u64(p_vec0, p_target));
    uint64x2_t match1 = vandq_u64(vceqq_u64(s_vec1, s_target), vceqq_u64(p_vec1, p_target));
    uint64x2_t match2 = vandq_u64(vceqq_u64(s_vec2, s_target), vceqq_u64(p_vec2, p_target));
    uint64x2_t match3 = vandq_u64(vceqq_u64(s_vec3, s_target), vceqq_u64(p_vec3, p_target));

    uint64_t match_mask = ((vgetq_lane_u64(match0, 0) >> 63) << 0) |
                           ((vgetq_lane_u64(match0, 1) >> 63) << 1) |
                           ((vgetq_lane_u64(match1, 0) >> 63) << 2) |
                           ((vgetq_lane_u64(match1, 1) >> 63) << 3) |
                           ((vgetq_lane_u64(match2, 0) >> 63) << 4) |
                           ((vgetq_lane_u64(match2, 1) >> 63) << 5) |
                           ((vgetq_lane_u64(match3, 0) >> 63) << 6) |
                           ((vgetq_lane_u64(match3, 1) >> 63) << 7);

    uint64_t len_mask = ((1ULL << n_rows) - 1) & 0xFFULL;
    match_mask &= len_mask;

    uint64_t count = (uint64_t)__builtin_popcountll(match_mask);
    uint64_t result = (count == 1) ? UINT64_MAX : 0;
    *out_mask = result;

#else
    uint64_t mask = 0;
    for (size_t i = 0; i < n_rows && i < 8; i++) {
        uint64_t match = (s_lane[i] == target_s) & (p_lane[i] == target_p);
        mask |= (match << i);
    }
    uint64_t count = (uint64_t)__builtin_popcountll(mask);
    *out_mask = (count == 1) ? UINT64_MAX : 0;
#endif

    uint64_t end = knhk_pmu_rdtsc();
    return end - start;
}

// Kernel 6: COMPARE O - Compare object values (branchless op_type dispatch)
uint64_t knhk_kernel_compare_o_impl(
    const uint64_t *s_lane,
    const uint64_t *p_lane,
    const uint64_t *o_lane,
    size_t n_rows,
    uint64_t *out_mask
) {
    uint64_t start = knhk_pmu_rdtsc();

    uint64_t threshold = s_lane[0];  // Comparison threshold
    uint64_t op_type = p_lane[0];    // 0=EQ, 1=GT, 2=LT, 3=GE, 4=LE

#if defined(__x86_64__) || defined(_M_X64)
    __m256i K = _mm256_set1_epi64x((long long)threshold);
    __m256i o0 = _mm256_loadu_si256((const __m256i *)(o_lane + 0));
    __m256i o1 = _mm256_loadu_si256((const __m256i *)(o_lane + 4));

    // Branchless: compute all comparison types
    __m256i m_eq_0 = _mm256_cmpeq_epi64(o0, K);
    __m256i m_eq_1 = _mm256_cmpeq_epi64(o1, K);

    __m256i m_gt_0 = _mm256_cmpgt_epi64(o0, K);
    __m256i m_gt_1 = _mm256_cmpgt_epi64(o1, K);

    __m256i m_lt_0 = _mm256_cmpgt_epi64(K, o0);
    __m256i m_lt_1 = _mm256_cmpgt_epi64(K, o1);

    __m256i m_ge_0 = _mm256_or_si256(m_eq_0, m_gt_0);
    __m256i m_ge_1 = _mm256_or_si256(m_eq_1, m_gt_1);

    __m256i m_le_0 = _mm256_or_si256(m_eq_0, m_lt_0);
    __m256i m_le_1 = _mm256_or_si256(m_eq_1, m_lt_1);

    // Branchless mask selection
    uint64_t mask_eq = ((uint64_t)(op_type == 0)) * UINT64_MAX;
    uint64_t mask_gt = ((uint64_t)(op_type == 1)) * UINT64_MAX;
    uint64_t mask_lt = ((uint64_t)(op_type == 2)) * UINT64_MAX;
    uint64_t mask_ge = ((uint64_t)(op_type == 3)) * UINT64_MAX;
    uint64_t mask_le = ((uint64_t)(op_type == 4)) * UINT64_MAX;

    __m256i mask_eq_vec = _mm256_set1_epi64x((long long)mask_eq);
    __m256i mask_gt_vec = _mm256_set1_epi64x((long long)mask_gt);
    __m256i mask_lt_vec = _mm256_set1_epi64x((long long)mask_lt);
    __m256i mask_ge_vec = _mm256_set1_epi64x((long long)mask_ge);
    __m256i mask_le_vec = _mm256_set1_epi64x((long long)mask_le);

    // Select result using masks
    __m256i m0 = _mm256_or_si256(
        _mm256_or_si256(
            _mm256_and_si256(m_eq_0, mask_eq_vec),
            _mm256_and_si256(m_gt_0, mask_gt_vec)
        ),
        _mm256_or_si256(
            _mm256_or_si256(
                _mm256_and_si256(m_lt_0, mask_lt_vec),
                _mm256_and_si256(m_ge_0, mask_ge_vec)
            ),
            _mm256_and_si256(m_le_0, mask_le_vec)
        )
    );

    __m256i m1 = _mm256_or_si256(
        _mm256_or_si256(
            _mm256_and_si256(m_eq_1, mask_eq_vec),
            _mm256_and_si256(m_gt_1, mask_gt_vec)
        ),
        _mm256_or_si256(
            _mm256_or_si256(
                _mm256_and_si256(m_lt_1, mask_lt_vec),
                _mm256_and_si256(m_ge_1, mask_ge_vec)
            ),
            _mm256_and_si256(m_le_1, mask_le_vec)
        )
    );

    uint32_t mask0 = _mm256_movemask_pd(_mm256_castsi256_pd(m0));
    uint32_t mask1 = _mm256_movemask_pd(_mm256_castsi256_pd(m1));

    uint64_t mask = ((uint64_t)mask0) | (((uint64_t)mask1) << 4);
    uint64_t len_mask = ((1ULL << n_rows) - 1) & 0xFFULL;
    mask &= len_mask;

    *out_mask = mask;

#elif defined(__aarch64__) || defined(_M_ARM64)
    uint64x2_t K = vdupq_n_u64(threshold);
    uint64x2_t o0 = vld1q_u64(o_lane + 0);
    uint64x2_t o1 = vld1q_u64(o_lane + 2);
    uint64x2_t o2 = vld1q_u64(o_lane + 4);
    uint64x2_t o3 = vld1q_u64(o_lane + 6);

    // Compute all comparison types
    uint64x2_t m_eq_0 = vceqq_u64(o0, K);
    uint64x2_t m_eq_1 = vceqq_u64(o1, K);
    uint64x2_t m_eq_2 = vceqq_u64(o2, K);
    uint64x2_t m_eq_3 = vceqq_u64(o3, K);

    uint64x2_t m_gt_0 = vcgtq_u64(o0, K);
    uint64x2_t m_gt_1 = vcgtq_u64(o1, K);
    uint64x2_t m_gt_2 = vcgtq_u64(o2, K);
    uint64x2_t m_gt_3 = vcgtq_u64(o3, K);

    uint64x2_t m_lt_0 = vcltq_u64(o0, K);
    uint64x2_t m_lt_1 = vcltq_u64(o1, K);
    uint64x2_t m_lt_2 = vcltq_u64(o2, K);
    uint64x2_t m_lt_3 = vcltq_u64(o3, K);

    uint64x2_t m_ge_0 = vcgeq_u64(o0, K);
    uint64x2_t m_ge_1 = vcgeq_u64(o1, K);
    uint64x2_t m_ge_2 = vcgeq_u64(o2, K);
    uint64x2_t m_ge_3 = vcgeq_u64(o3, K);

    uint64x2_t m_le_0 = vcleq_u64(o0, K);
    uint64x2_t m_le_1 = vcleq_u64(o1, K);
    uint64x2_t m_le_2 = vcleq_u64(o2, K);
    uint64x2_t m_le_3 = vcleq_u64(o3, K);

    // Branchless mask selection
    uint64_t mask_eq = ((uint64_t)(op_type == 0)) * UINT64_MAX;
    uint64_t mask_gt = ((uint64_t)(op_type == 1)) * UINT64_MAX;
    uint64_t mask_lt = ((uint64_t)(op_type == 2)) * UINT64_MAX;
    uint64_t mask_ge = ((uint64_t)(op_type == 3)) * UINT64_MAX;
    uint64_t mask_le = ((uint64_t)(op_type == 4)) * UINT64_MAX;

    // Select result
    uint64x2_t m0 = vorrq_u64(
        vorrq_u64(
            vandq_u64(m_eq_0, vdupq_n_u64(mask_eq)),
            vandq_u64(m_gt_0, vdupq_n_u64(mask_gt))
        ),
        vorrq_u64(
            vorrq_u64(
                vandq_u64(m_lt_0, vdupq_n_u64(mask_lt)),
                vandq_u64(m_ge_0, vdupq_n_u64(mask_ge))
            ),
            vandq_u64(m_le_0, vdupq_n_u64(mask_le))
        )
    );

    uint64x2_t m1 = vorrq_u64(
        vorrq_u64(
            vandq_u64(m_eq_1, vdupq_n_u64(mask_eq)),
            vandq_u64(m_gt_1, vdupq_n_u64(mask_gt))
        ),
        vorrq_u64(
            vorrq_u64(
                vandq_u64(m_lt_1, vdupq_n_u64(mask_lt)),
                vandq_u64(m_ge_1, vdupq_n_u64(mask_ge))
            ),
            vandq_u64(m_le_1, vdupq_n_u64(mask_le))
        )
    );

    uint64x2_t m2 = vorrq_u64(
        vorrq_u64(
            vandq_u64(m_eq_2, vdupq_n_u64(mask_eq)),
            vandq_u64(m_gt_2, vdupq_n_u64(mask_gt))
        ),
        vorrq_u64(
            vorrq_u64(
                vandq_u64(m_lt_2, vdupq_n_u64(mask_lt)),
                vandq_u64(m_ge_2, vdupq_n_u64(mask_ge))
            ),
            vandq_u64(m_le_2, vdupq_n_u64(mask_le))
        )
    );

    uint64x2_t m3 = vorrq_u64(
        vorrq_u64(
            vandq_u64(m_eq_3, vdupq_n_u64(mask_eq)),
            vandq_u64(m_gt_3, vdupq_n_u64(mask_gt))
        ),
        vorrq_u64(
            vorrq_u64(
                vandq_u64(m_lt_3, vdupq_n_u64(mask_lt)),
                vandq_u64(m_ge_3, vdupq_n_u64(mask_ge))
            ),
            vandq_u64(m_le_3, vdupq_n_u64(mask_le))
        )
    );

    uint64_t mask = ((vgetq_lane_u64(m0, 0) >> 63) << 0) |
                     ((vgetq_lane_u64(m0, 1) >> 63) << 1) |
                     ((vgetq_lane_u64(m1, 0) >> 63) << 2) |
                     ((vgetq_lane_u64(m1, 1) >> 63) << 3) |
                     ((vgetq_lane_u64(m2, 0) >> 63) << 4) |
                     ((vgetq_lane_u64(m2, 1) >> 63) << 5) |
                     ((vgetq_lane_u64(m3, 0) >> 63) << 6) |
                     ((vgetq_lane_u64(m3, 1) >> 63) << 7);

    uint64_t len_mask = ((1ULL << n_rows) - 1) & 0xFFULL;
    mask &= len_mask;

    *out_mask = mask;

#else
    // Scalar fallback
    uint64_t r_eq = 0, r_gt = 0, r_lt = 0, r_ge = 0, r_le = 0;
    for (size_t i = 0; i < n_rows && i < 8; i++) {
        r_eq |= ((o_lane[i] == threshold) << i);
        r_gt |= ((o_lane[i] > threshold) << i);
        r_lt |= ((o_lane[i] < threshold) << i);
        r_ge |= ((o_lane[i] >= threshold) << i);
        r_le |= ((o_lane[i] <= threshold) << i);
    }

    uint64_t mask_eq = ((uint64_t)(op_type == 0)) * UINT64_MAX;
    uint64_t mask_gt = ((uint64_t)(op_type == 1)) * UINT64_MAX;
    uint64_t mask_lt = ((uint64_t)(op_type == 2)) * UINT64_MAX;
    uint64_t mask_ge = ((uint64_t)(op_type == 3)) * UINT64_MAX;
    uint64_t mask_le = ((uint64_t)(op_type == 4)) * UINT64_MAX;

    uint64_t result = (r_eq & mask_eq) | (r_gt & mask_gt) | (r_lt & mask_lt) | (r_ge & mask_ge) | (r_le & mask_le);
    *out_mask = result;
#endif

    uint64_t end = knhk_pmu_rdtsc();
    return end - start;
}

// Kernel dispatch table (const, cache-friendly)
static const knhk_kernel_dispatch_t kernel_dispatch_table[KNHK_KERNEL_MAX] = {
    { KNHK_KERNEL_ASK_SP,       knhk_kernel_ask_sp_impl },
    { KNHK_KERNEL_COUNT_SP_GE,  knhk_kernel_count_sp_ge_impl },
    { KNHK_KERNEL_ASK_SPO,      knhk_kernel_ask_spo_impl },
    { KNHK_KERNEL_VALIDATE_SP,  knhk_kernel_validate_sp_impl },
    { KNHK_KERNEL_UNIQUE_SP,    knhk_kernel_unique_sp_impl },
    { KNHK_KERNEL_COMPARE_O,    knhk_kernel_compare_o_impl }
};

// Get dispatch table
const knhk_kernel_dispatch_t* knhk_get_kernel_dispatch_table(void) {
    return kernel_dispatch_table;
}
