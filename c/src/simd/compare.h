// simd/compare.h
// Comparison operations: COMPARE_O_EQ/GT/LT/GE/LE

#ifndef KNHK_SIMD_COMPARE_H
#define KNHK_SIMD_COMPARE_H

#include "common.h"

#if NROWS == 8
static inline int knhk_compare_o_8(const uint64_t *O_base, uint64_t off, uint64_t threshold, int op_type)
{
  const uint64_t *o_p = O_base + off;
  
#if defined(__aarch64__)
  uint64x2_t K = vdupq_n_u64(threshold);
  uint64x2_t o0 = vld1q_u64(o_p + 0);
  uint64x2_t o1 = vld1q_u64(o_p + 2);
  uint64x2_t o2 = vld1q_u64(o_p + 4);
  uint64x2_t o3 = vld1q_u64(o_p + 6);
  
  // Branchless: compute all comparison types, then mask-select result
  // This eliminates the switch statement and all branches
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
  
  // Branchless mask selection based on op_type
  // Create masks for each op_type: 0xFFFFFFFFFFFFFFFF if match, 0 if not
  // Use arithmetic to avoid branches: mask = (op_type == N) ? UINT64_MAX : 0
  // Compiler should optimize these to branchless code (cmov or similar)
  uint64_t mask_eq = ((uint64_t)(op_type == 0)) * UINT64_MAX;
  uint64_t mask_gt = ((uint64_t)(op_type == 1)) * UINT64_MAX;
  uint64_t mask_lt = ((uint64_t)(op_type == 2)) * UINT64_MAX;
  uint64_t mask_ge = ((uint64_t)(op_type == 3)) * UINT64_MAX;
  uint64_t mask_le = ((uint64_t)(op_type == 4)) * UINT64_MAX;
  
  // Select result using masks (branchless)
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
  
  uint64_t t[2];
  uint64_t result = 0;
  vst1q_u64(t, m0);
  result |= (t[0] | t[1]);
  vst1q_u64(t, m1);
  result |= (t[0] | t[1]);
  vst1q_u64(t, m2);
  result |= (t[0] | t[1]);
  vst1q_u64(t, m3);
  result |= (t[0] | t[1]);
#elif defined(__x86_64__)
  __m256i K = _mm256_set1_epi64x((long long)threshold);
  __m256i o0 = _mm256_loadu_si256((const __m256i *)(o_p + 0));
  __m256i o1 = _mm256_loadu_si256((const __m256i *)(o_p + 4));
  
  // Branchless: compute all comparison types, then mask-select result
  __m256i m_eq_0 = _mm256_cmpeq_epi64(o0, K);
  __m256i m_eq_1 = _mm256_cmpeq_epi64(o1, K);
  
  __m256i K_signed = _mm256_set1_epi64x((long long)threshold);
  __m256i m_gt_0 = _mm256_cmpgt_epi64(o0, K_signed);
  __m256i m_gt_1 = _mm256_cmpgt_epi64(o1, K_signed);
  
  __m256i m_lt_0 = _mm256_cmpgt_epi64(K_signed, o0);
  __m256i m_lt_1 = _mm256_cmpgt_epi64(K_signed, o1);
  
  __m256i m_ge_0 = _mm256_or_si256(m_eq_0, m_gt_0);
  __m256i m_ge_1 = _mm256_or_si256(m_eq_1, m_gt_1);
  
  __m256i m_le_0 = _mm256_or_si256(m_eq_0, m_lt_0);
  __m256i m_le_1 = _mm256_or_si256(m_eq_1, m_lt_1);
  
  // Branchless mask selection based on op_type
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
  
  // Select result using masks (branchless)
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
  
  uint64_t t[4];
  uint64_t result = 0;
  _mm256_storeu_si256((__m256i *)t, m0);
  result |= (t[0] | t[1] | t[2] | t[3]);
  _mm256_storeu_si256((__m256i *)t, m1);
  result |= (t[0] | t[1] | t[2] | t[3]);
#else
  // Fallback: scalar comparison (branchless)
  // Compute all comparison types, then mask-select result
  uint64_t r_eq = (o_p[0] == threshold) | (o_p[1] == threshold) | (o_p[2] == threshold) | (o_p[3] == threshold) |
                  (o_p[4] == threshold) | (o_p[5] == threshold) | (o_p[6] == threshold) | (o_p[7] == threshold);
  uint64_t r_gt = (o_p[0] > threshold) | (o_p[1] > threshold) | (o_p[2] > threshold) | (o_p[3] > threshold) |
                  (o_p[4] > threshold) | (o_p[5] > threshold) | (o_p[6] > threshold) | (o_p[7] > threshold);
  uint64_t r_lt = (o_p[0] < threshold) | (o_p[1] < threshold) | (o_p[2] < threshold) | (o_p[3] < threshold) |
                  (o_p[4] < threshold) | (o_p[5] < threshold) | (o_p[6] < threshold) | (o_p[7] < threshold);
  uint64_t r_ge = (o_p[0] >= threshold) | (o_p[1] >= threshold) | (o_p[2] >= threshold) | (o_p[3] >= threshold) |
                  (o_p[4] >= threshold) | (o_p[5] >= threshold) | (o_p[6] >= threshold) | (o_p[7] >= threshold);
  uint64_t r_le = (o_p[0] <= threshold) | (o_p[1] <= threshold) | (o_p[2] <= threshold) | (o_p[3] <= threshold) |
                  (o_p[4] <= threshold) | (o_p[5] <= threshold) | (o_p[6] <= threshold) | (o_p[7] <= threshold);
  
  // Branchless mask selection
  uint64_t mask_eq = ((uint64_t)(op_type == 0)) * UINT64_MAX;
  uint64_t mask_gt = ((uint64_t)(op_type == 1)) * UINT64_MAX;
  uint64_t mask_lt = ((uint64_t)(op_type == 2)) * UINT64_MAX;
  uint64_t mask_ge = ((uint64_t)(op_type == 3)) * UINT64_MAX;
  uint64_t mask_le = ((uint64_t)(op_type == 4)) * UINT64_MAX;
  
  uint64_t result = (r_eq & mask_eq) | (r_gt & mask_gt) | (r_lt & mask_lt) | (r_ge & mask_ge) | (r_le & mask_le);
#endif
  
  return result != 0;
}

#endif // NROWS == 8

#endif // KNHK_SIMD_COMPARE_H
