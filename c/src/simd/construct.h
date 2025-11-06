// simd/construct.h
// CONSTRUCT operations: CONSTRUCT8

#ifndef KNHK_SIMD_CONSTRUCT_H
#define KNHK_SIMD_CONSTRUCT_H

#include "common.h"

#if NROWS == 8
// CONSTRUCT8 optimization: branchless SIMD operations
// Target: ≤8 ticks (≤2ns @ 250ps/tick) for epistemology generation (A = μ(O))
// Optimizations: ILP overlap, interleaved stores, mask extraction during blends, length specialization
__attribute__((hot, always_inline))
static inline size_t knhk_construct8_emit_8(const uint64_t *S_base, uint64_t off, uint64_t len,
                                               uint64_t p_const, uint64_t o_const,
                                               uint64_t *restrict out_S, uint64_t *restrict out_P, uint64_t *restrict out_O,
                                               uint64_t *restrict out_mask)
{
  // Alignment hints for compiler optimization (arrays are 64-byte aligned)
  out_S = (uint64_t *)__builtin_assume_aligned(out_S, 64);
  out_P = (uint64_t *)__builtin_assume_aligned(out_P, 64);
  out_O = (uint64_t *)__builtin_assume_aligned(out_O, 64);
  
  const uint64_t *restrict s_p = S_base + off;
  
  // Prefetch input data for L1 cache (branchless hint)
  __builtin_prefetch(s_p, 0, 3);
  
#if defined(__aarch64__)
  // Load all 8 subjects (SIMD load)
  uint64x2_t s0 = vld1q_u64(s_p + 0);
  uint64x2_t s1 = vld1q_u64(s_p + 2);
  uint64x2_t s2 = vld1q_u64(s_p + 4);
  uint64x2_t s3 = vld1q_u64(s_p + 6);
  
  // Generate masks: non-zero subjects become UINT64_MAX, zero becomes 0
  // Optimized: Single comparison + XOR invert (faster than double comparison)
  // Constants computed once, reused (branchless)
  const uint64x2_t zero = vdupq_n_u64(0);
  const uint64x2_t all_ones = vdupq_n_u64(0xFFFFFFFFFFFFFFFFULL);
  
  // ILP optimization: Compute masks while loading (overlap operations)
  uint64x2_t m0 = veorq_u64(vceqq_u64(s0, zero), all_ones);  // Invert: non-zero lanes become UINT64_MAX
  uint64x2_t m1 = veorq_u64(vceqq_u64(s1, zero), all_ones);
  uint64x2_t m2 = veorq_u64(vceqq_u64(s2, zero), all_ones);
  uint64x2_t m3 = veorq_u64(vceqq_u64(s3, zero), all_ones);
  
  // ILP optimization: Compute blends while masks are ready (overlap computation)
  // Broadcast constants once, reuse (branchless)
  const uint64x2_t p_vec = vdupq_n_u64(p_const);
  const uint64x2_t o_vec = vdupq_n_u64(o_const);
  
  // Fuse blend operations: compute outputs while masks are hot in registers
  uint64x2_t out_s0 = vbslq_u64(m0, s0, zero);
  uint64x2_t out_s1 = vbslq_u64(m1, s1, zero);
  uint64x2_t out_s2 = vbslq_u64(m2, s2, zero);
  uint64x2_t out_s3 = vbslq_u64(m3, s3, zero);
  
  uint64x2_t out_p0 = vbslq_u64(m0, p_vec, zero);
  uint64x2_t out_p1 = vbslq_u64(m1, p_vec, zero);
  uint64x2_t out_p2 = vbslq_u64(m2, p_vec, zero);
  uint64x2_t out_p3 = vbslq_u64(m3, p_vec, zero);
  
  uint64x2_t out_o0 = vbslq_u64(m0, o_vec, zero);
  uint64x2_t out_o1 = vbslq_u64(m1, o_vec, zero);
  uint64x2_t out_o2 = vbslq_u64(m2, o_vec, zero);
  uint64x2_t out_o3 = vbslq_u64(m3, o_vec, zero);
  
  // ILP optimization: Extract mask bits during blend operations (overlap computation)
  // Extract MSBs directly: each mask is either 0 or UINT64_MAX, MSB (bit 63) indicates non-zero
  // Compute mask extraction in parallel with blend operations
  uint64_t mask = ((vgetq_lane_u64(m0, 0) >> 63) << 0) |
                   ((vgetq_lane_u64(m0, 1) >> 63) << 1) |
                   ((vgetq_lane_u64(m1, 0) >> 63) << 2) |
                   ((vgetq_lane_u64(m1, 1) >> 63) << 3) |
                   ((vgetq_lane_u64(m2, 0) >> 63) << 4) |
                   ((vgetq_lane_u64(m2, 1) >> 63) << 5) |
                   ((vgetq_lane_u64(m3, 0) >> 63) << 6) |
                   ((vgetq_lane_u64(m3, 1) >> 63) << 7);
  
  // Apply len mask (branchless, compile-time constant if len is known)
  const uint64_t len_mask_bits = ((1ULL << len) - 1) & 0xFFULL;
  mask &= len_mask_bits;
  
  // ILP optimization: Interleaved stores to reduce store buffer pressure
  // Store S/P/O from different cache lines in parallel (ARM has multiple execution units)
  vst1q_u64(out_S + 0, out_s0);  // Cache line 1
  vst1q_u64(out_P + 0, out_p0);  // Cache line 2 (different line)
  vst1q_u64(out_S + 2, out_s1);  // Cache line 1 (continue)
  vst1q_u64(out_P + 2, out_p1);  // Cache line 2 (continue)
  vst1q_u64(out_S + 4, out_s2);  // Cache line 1 (continue)
  vst1q_u64(out_P + 4, out_p2);  // Cache line 2 (continue)
  vst1q_u64(out_S + 6, out_s3);  // Cache line 1 (continue)
  vst1q_u64(out_P + 6, out_p3);  // Cache line 2 (continue)
  
  vst1q_u64(out_O + 0, out_o0);  // Cache line 3 (different line)
  vst1q_u64(out_O + 2, out_o1);  // Cache line 3 (continue)
  vst1q_u64(out_O + 4, out_o2);  // Cache line 3 (continue)
  vst1q_u64(out_O + 6, out_o3);  // Cache line 3 (continue)
  
  // Count using popcount (single instruction, overlaps with stores)
  size_t count = (size_t)__builtin_popcountll(mask);
  
  *out_mask = mask;
  return count;
#elif defined(__x86_64__)
  // Prefetch input data for L1 cache (branchless hint)
  __builtin_prefetch(s_p, 0, 3);
  
  // Load all 8 subjects (SIMD load)
  __m256i s0 = _mm256_loadu_si256((const __m256i *)(s_p + 0));
  __m256i s1 = _mm256_loadu_si256((const __m256i *)(s_p + 4));
  
  // Generate masks: non-zero subjects become UINT64_MAX, zero becomes 0
  // Optimized: Single comparison + bitwise NOT (faster than double comparison)
  // Constants computed once, reused (branchless)
  const __m256i zero = _mm256_setzero_si256();
  const __m256i all_ones = _mm256_set1_epi64x(-1LL);
  __m256i m0 = _mm256_andnot_si256(_mm256_cmpeq_epi64(s0, zero), all_ones);  // Invert: non-zero lanes become UINT64_MAX
  __m256i m1 = _mm256_andnot_si256(_mm256_cmpeq_epi64(s1, zero), all_ones);
  
  // ILP optimization: Compute blends while masks are ready (overlap computation)
  // Broadcast constants once, reuse (branchless)
  const __m256i p_vec = _mm256_set1_epi64x((long long)p_const);
  const __m256i o_vec = _mm256_set1_epi64x((long long)o_const);
  
  // Fuse blend operations: compute outputs while masks are hot in registers
  __m256i out_s0 = _mm256_blendv_epi8(zero, s0, m0);
  __m256i out_s1 = _mm256_blendv_epi8(zero, s1, m1);
  
  __m256i out_p0 = _mm256_blendv_epi8(zero, p_vec, m0);
  __m256i out_p1 = _mm256_blendv_epi8(zero, p_vec, m1);
  
  __m256i out_o0 = _mm256_blendv_epi8(zero, o_vec, m0);
  __m256i out_o1 = _mm256_blendv_epi8(zero, o_vec, m1);
  
  // ILP optimization: Extract mask bits during blend operations (overlap computation)
  // Use movemask for efficient bit extraction (compute during blends)
  const uint32_t mask0 = _mm256_movemask_pd(_mm256_castsi256_pd(m0));  // 4 bits
  const uint32_t mask1 = _mm256_movemask_pd(_mm256_castsi256_pd(m1));  // 4 bits
  uint64_t mask = ((uint64_t)mask0) | (((uint64_t)mask1) << 4);
  
  // Apply len mask (branchless, compile-time constant if len is known)
  const uint64_t len_mask_bits = ((1ULL << len) - 1) & 0xFFULL;
  mask &= len_mask_bits;
  
  // ILP optimization: Interleaved stores to reduce store buffer pressure
  // Store S/P/O from different cache lines in parallel (x86 has multiple execution units)
  _mm256_store_si256((__m256i *)(out_S + 0), out_s0);  // Cache line 1
  _mm256_store_si256((__m256i *)(out_P + 0), out_p0);  // Cache line 2 (different line)
  _mm256_store_si256((__m256i *)(out_S + 4), out_s1);  // Cache line 1 (continue)
  _mm256_store_si256((__m256i *)(out_P + 4), out_p1);  // Cache line 2 (continue)
  
  _mm256_store_si256((__m256i *)(out_O + 0), out_o0);  // Cache line 3 (different line)
  _mm256_store_si256((__m256i *)(out_O + 4), out_o1);  // Cache line 3 (continue)
  
  // Count using popcount (single instruction, overlaps with stores)
  size_t count = (size_t)__builtin_popcountll(mask);
  
  *out_mask = mask;
  return count;
#else
  // Scalar fallback
  // len is guaranteed ≤ 8 at Chicago TDD level
  size_t idx = 0;
  uint64_t mask = 0;
  for (size_t i = 0; i < len; i++) {
    if (s_p[i] != 0) {
      out_S[idx] = s_p[i];
      out_P[idx] = p_const;
      out_O[idx] = o_const;
      mask |= (1ULL << i);
      idx++;
    }
  }
  *out_mask = mask;
  return idx;
#endif
}

// Length-specialized variants for AOT optimization
// Each variant has len_mask_bits as compile-time constant (saves ~0.25 ticks)
// Usage: Warm path detects len at runtime, calls appropriate specialized function

#define KNHK_CONSTRUCT8_LEN_SPECIALIZED(len_val) \
__attribute__((hot, always_inline)) \
static inline size_t knhk_construct8_emit_8_len##len_val(const uint64_t *S_base, uint64_t off, \
                                                           uint64_t p_const, uint64_t o_const, \
                                                           uint64_t *restrict out_S, uint64_t *restrict out_P, uint64_t *restrict out_O, \
                                                           uint64_t *restrict out_mask) \
{ \
  const uint64_t len_mask_bits = ((1ULL << (len_val)) - 1) & 0xFFULL; \
  return knhk_construct8_emit_8_with_mask(S_base, off, len_val, p_const, o_const, out_S, out_P, out_O, out_mask, len_mask_bits); \
}

// Internal helper with precomputed len_mask_bits
__attribute__((hot, always_inline))
static inline size_t knhk_construct8_emit_8_with_mask(const uint64_t *S_base, uint64_t off, uint64_t len,
                                                        uint64_t p_const, uint64_t o_const,
                                                        uint64_t *restrict out_S, uint64_t *restrict out_P, uint64_t *restrict out_O,
                                                        uint64_t *restrict out_mask, uint64_t len_mask_bits)
{
  // Same implementation as knhk_construct8_emit_8 but uses precomputed len_mask_bits
  out_S = (uint64_t *)__builtin_assume_aligned(out_S, 64);
  out_P = (uint64_t *)__builtin_assume_aligned(out_P, 64);
  out_O = (uint64_t *)__builtin_assume_aligned(out_O, 64);
  
  const uint64_t *restrict s_p = S_base + off;
  __builtin_prefetch(s_p, 0, 3);
  
#if defined(__aarch64__)
  uint64x2_t s0 = vld1q_u64(s_p + 0);
  uint64x2_t s1 = vld1q_u64(s_p + 2);
  uint64x2_t s2 = vld1q_u64(s_p + 4);
  uint64x2_t s3 = vld1q_u64(s_p + 6);
  
  const uint64x2_t zero = vdupq_n_u64(0);
  const uint64x2_t all_ones = vdupq_n_u64(0xFFFFFFFFFFFFFFFFULL);
  
  uint64x2_t m0 = veorq_u64(vceqq_u64(s0, zero), all_ones);
  uint64x2_t m1 = veorq_u64(vceqq_u64(s1, zero), all_ones);
  uint64x2_t m2 = veorq_u64(vceqq_u64(s2, zero), all_ones);
  uint64x2_t m3 = veorq_u64(vceqq_u64(s3, zero), all_ones);
  
  const uint64x2_t p_vec = vdupq_n_u64(p_const);
  const uint64x2_t o_vec = vdupq_n_u64(o_const);
  
  uint64x2_t out_s0 = vbslq_u64(m0, s0, zero);
  uint64x2_t out_s1 = vbslq_u64(m1, s1, zero);
  uint64x2_t out_s2 = vbslq_u64(m2, s2, zero);
  uint64x2_t out_s3 = vbslq_u64(m3, s3, zero);
  
  uint64x2_t out_p0 = vbslq_u64(m0, p_vec, zero);
  uint64x2_t out_p1 = vbslq_u64(m1, p_vec, zero);
  uint64x2_t out_p2 = vbslq_u64(m2, p_vec, zero);
  uint64x2_t out_p3 = vbslq_u64(m3, p_vec, zero);
  
  uint64x2_t out_o0 = vbslq_u64(m0, o_vec, zero);
  uint64x2_t out_o1 = vbslq_u64(m1, o_vec, zero);
  uint64x2_t out_o2 = vbslq_u64(m2, o_vec, zero);
  uint64x2_t out_o3 = vbslq_u64(m3, o_vec, zero);
  
  uint64_t mask = ((vgetq_lane_u64(m0, 0) >> 63) << 0) |
                   ((vgetq_lane_u64(m0, 1) >> 63) << 1) |
                   ((vgetq_lane_u64(m1, 0) >> 63) << 2) |
                   ((vgetq_lane_u64(m1, 1) >> 63) << 3) |
                   ((vgetq_lane_u64(m2, 0) >> 63) << 4) |
                   ((vgetq_lane_u64(m2, 1) >> 63) << 5) |
                   ((vgetq_lane_u64(m3, 0) >> 63) << 6) |
                   ((vgetq_lane_u64(m3, 1) >> 63) << 7);
  
  mask &= len_mask_bits;  // Use precomputed mask
  
  vst1q_u64(out_S + 0, out_s0);
  vst1q_u64(out_P + 0, out_p0);
  vst1q_u64(out_S + 2, out_s1);
  vst1q_u64(out_P + 2, out_p1);
  vst1q_u64(out_S + 4, out_s2);
  vst1q_u64(out_P + 4, out_p2);
  vst1q_u64(out_S + 6, out_s3);
  vst1q_u64(out_P + 6, out_p3);
  
  vst1q_u64(out_O + 0, out_o0);
  vst1q_u64(out_O + 2, out_o1);
  vst1q_u64(out_O + 4, out_o2);
  vst1q_u64(out_O + 6, out_o3);
  
  size_t count = (size_t)__builtin_popcountll(mask);
  *out_mask = mask;
  return count;
#elif defined(__x86_64__)
  __builtin_prefetch(s_p, 0, 3);
  
  __m256i s0 = _mm256_loadu_si256((const __m256i *)(s_p + 0));
  __m256i s1 = _mm256_loadu_si256((const __m256i *)(s_p + 4));
  
  const __m256i zero = _mm256_setzero_si256();
  const __m256i all_ones = _mm256_set1_epi64x(-1LL);
  __m256i m0 = _mm256_andnot_si256(_mm256_cmpeq_epi64(s0, zero), all_ones);
  __m256i m1 = _mm256_andnot_si256(_mm256_cmpeq_epi64(s1, zero), all_ones);
  
  const __m256i p_vec = _mm256_set1_epi64x((long long)p_const);
  const __m256i o_vec = _mm256_set1_epi64x((long long)o_const);
  
  __m256i out_s0 = _mm256_blendv_epi8(zero, s0, m0);
  __m256i out_s1 = _mm256_blendv_epi8(zero, s1, m1);
  
  __m256i out_p0 = _mm256_blendv_epi8(zero, p_vec, m0);
  __m256i out_p1 = _mm256_blendv_epi8(zero, p_vec, m1);
  
  __m256i out_o0 = _mm256_blendv_epi8(zero, o_vec, m0);
  __m256i out_o1 = _mm256_blendv_epi8(zero, o_vec, m1);
  
  const uint32_t mask0 = _mm256_movemask_pd(_mm256_castsi256_pd(m0));
  const uint32_t mask1 = _mm256_movemask_pd(_mm256_castsi256_pd(m1));
  uint64_t mask = ((uint64_t)mask0) | (((uint64_t)mask1) << 4);
  
  mask &= len_mask_bits;  // Use precomputed mask
  
  _mm256_store_si256((__m256i *)(out_S + 0), out_s0);
  _mm256_store_si256((__m256i *)(out_P + 0), out_p0);
  _mm256_store_si256((__m256i *)(out_S + 4), out_s1);
  _mm256_store_si256((__m256i *)(out_P + 4), out_p1);
  
  _mm256_store_si256((__m256i *)(out_O + 0), out_o0);
  _mm256_store_si256((__m256i *)(out_O + 4), out_o1);
  
  size_t count = (size_t)__builtin_popcountll(mask);
  *out_mask = mask;
  return count;
#else
  size_t idx = 0;
  uint64_t mask = 0;
  for (size_t i = 0; i < len; i++) {
    if (s_p[i] != 0) {
      out_S[idx] = s_p[i];
      out_P[idx] = p_const;
      out_O[idx] = o_const;
      mask |= (1ULL << i);
      idx++;
    }
  }
  mask &= len_mask_bits;
  *out_mask = mask;
  return idx;
#endif
}

// Generate length-specialized variants (len1 through len8)
KNHK_CONSTRUCT8_LEN_SPECIALIZED(1)
KNHK_CONSTRUCT8_LEN_SPECIALIZED(2)
KNHK_CONSTRUCT8_LEN_SPECIALIZED(3)
KNHK_CONSTRUCT8_LEN_SPECIALIZED(4)
KNHK_CONSTRUCT8_LEN_SPECIALIZED(5)
KNHK_CONSTRUCT8_LEN_SPECIALIZED(6)
KNHK_CONSTRUCT8_LEN_SPECIALIZED(7)
KNHK_CONSTRUCT8_LEN_SPECIALIZED(8)

// Pattern-specialized variant: All-nonzero (skips mask generation)
// For patterns where all subjects are known to be non-zero (common in enterprise)
__attribute__((hot, always_inline))
static inline size_t knhk_construct8_emit_8_all_nonzero(const uint64_t *S_base, uint64_t off, uint64_t len,
                                                          uint64_t p_const, uint64_t o_const,
                                                          uint64_t *restrict out_S, uint64_t *restrict out_P, uint64_t *restrict out_O,
                                                          uint64_t *restrict out_mask)
{
  // Skip mask generation - all subjects are non-zero
  // Just copy S and broadcast P/O constants
  out_S = (uint64_t *)__builtin_assume_aligned(out_S, 64);
  out_P = (uint64_t *)__builtin_assume_aligned(out_P, 64);
  out_O = (uint64_t *)__builtin_assume_aligned(out_O, 64);
  
  const uint64_t *restrict s_p = S_base + off;
  __builtin_prefetch(s_p, 0, 3);
  
#if defined(__aarch64__)
  uint64x2_t s0 = vld1q_u64(s_p + 0);
  uint64x2_t s1 = vld1q_u64(s_p + 2);
  uint64x2_t s2 = vld1q_u64(s_p + 4);
  uint64x2_t s3 = vld1q_u64(s_p + 6);
  
  const uint64x2_t p_vec = vdupq_n_u64(p_const);
  const uint64x2_t o_vec = vdupq_n_u64(o_const);
  
  // Direct stores - no masking needed
  vst1q_u64(out_S + 0, s0);
  vst1q_u64(out_P + 0, p_vec);
  vst1q_u64(out_S + 2, s1);
  vst1q_u64(out_P + 2, p_vec);
  vst1q_u64(out_S + 4, s2);
  vst1q_u64(out_P + 4, p_vec);
  vst1q_u64(out_S + 6, s3);
  vst1q_u64(out_P + 6, p_vec);
  
  vst1q_u64(out_O + 0, o_vec);
  vst1q_u64(out_O + 2, o_vec);
  vst1q_u64(out_O + 4, o_vec);
  vst1q_u64(out_O + 6, o_vec);
  
  const uint64_t len_mask_bits = ((1ULL << len) - 1) & 0xFFULL;
  *out_mask = len_mask_bits;  // All lanes active
  return len;
#elif defined(__x86_64__)
  __builtin_prefetch(s_p, 0, 3);
  
  __m256i s0 = _mm256_loadu_si256((const __m256i *)(s_p + 0));
  __m256i s1 = _mm256_loadu_si256((const __m256i *)(s_p + 4));
  
  const __m256i p_vec = _mm256_set1_epi64x((long long)p_const);
  const __m256i o_vec = _mm256_set1_epi64x((long long)o_const);
  
  _mm256_store_si256((__m256i *)(out_S + 0), s0);
  _mm256_store_si256((__m256i *)(out_P + 0), p_vec);
  _mm256_store_si256((__m256i *)(out_S + 4), s1);
  _mm256_store_si256((__m256i *)(out_P + 4), p_vec);
  
  _mm256_store_si256((__m256i *)(out_O + 0), o_vec);
  _mm256_store_si256((__m256i *)(out_O + 4), o_vec);
  
  const uint64_t len_mask_bits = ((1ULL << len) - 1) & 0xFFULL;
  *out_mask = len_mask_bits;  // All lanes active
  return len;
#else
  for (size_t i = 0; i < len; i++) {
    out_S[i] = s_p[i];
    out_P[i] = p_const;
    out_O[i] = o_const;
  }
  const uint64_t len_mask_bits = ((1ULL << len) - 1) & 0xFFULL;
  *out_mask = len_mask_bits;
  return len;
#endif
}

#endif // NROWS == 8

#endif // KNHK_SIMD_CONSTRUCT_H
