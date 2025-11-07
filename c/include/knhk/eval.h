// knhk/eval.h
// Query evaluation functions: boolean queries, CONSTRUCT8
// Branchless dispatch using function pointer table (zero branch mispredicts)

#ifndef KNHK_EVAL_H
#define KNHK_EVAL_H

#include "types.h"
#include "simd.h"
#include "eval_dispatch.h"

// Generate span ID (no timing dependency)
uint64_t knhk_generate_span_id(void);

// Evaluate boolean query (ASK, COUNT>=k, ASK_SPO)
// Branchless dispatch: uses function pointer table instead of if-else chains
// Eliminates branch mispredicts for zero mispredicts on hot path
// Fills receipt with provenance information (timing is caller's responsibility)
#ifndef KNHK_EVAL_BOOL_INLINE
#define KNHK_EVAL_BOOL_INLINE static inline __attribute__((always_inline))
#endif
KNHK_EVAL_BOOL_INLINE int knhk_eval_bool(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  // Branchless predicate check: use mask instead of if
  // If predicate doesn't match, mask result to zero (no branches)
  uint64_t pred_match = (ir->p == ctx->run.pred) ? UINT64_MAX : 0;
  
  // Branchless dispatch: table lookup (no branches, no mispredicts)
  // Bounds check: if op >= KNHK_OP_MAX, use noop (branchless via mask)
  uint64_t op_valid = (ir->op < KNHK_OP_MAX) ? UINT64_MAX : 0;
  uint64_t op_idx = (uint64_t)ir->op & op_valid;  // Zero if invalid
  
  const knhk_eval_fn_t* dispatch_table = knhk_get_eval_dispatch_table();
  knhk_eval_fn_t fn = dispatch_table[op_idx];
  
  // Call operation function (branchless)
  int result = fn(ctx, ir, rcpt);
  
  // Mask result if predicate doesn't match (branchless)
  result = (int)((uint64_t)result & pred_match);
  
  // Branchless receipt update: mask receipt fields if predicate doesn't match
  // Receipt is already filled by operation function, but we need to zero it if predicate mismatch
  if (rcpt) {
    // Mask receipt fields with pred_match (branchless)
    uint64_t pred_mask = pred_match;  // UINT64_MAX if match, 0 if mismatch
    rcpt->lanes = (uint32_t)((uint64_t)rcpt->lanes & pred_mask);
    rcpt->span_id = rcpt->span_id & pred_mask;
    
    // Update hash with final result (always, masked by pred_match)
    // Note: operation function already set a_hash, but we update it with final masked result
    uint64_t final_hash = (uint64_t)(ir->s ^ ir->p ^ ir->o ^ ir->k ^ (uint64_t)result ^ ctx->run.pred);
    rcpt->a_hash = final_hash & pred_mask;
  }

  return result;
}

// CONSTRUCT8 function pointer type for branchless dispatch
typedef size_t (*knhk_construct8_fn_t)(const uint64_t *S_base, uint64_t off, uint64_t len,
                                        uint64_t p_const, uint64_t o_const,
                                        uint64_t *restrict out_S, uint64_t *restrict out_P, uint64_t *restrict out_O,
                                        uint64_t *restrict out_mask);

// Branchless dispatch table for CONSTRUCT8 specialized functions
// Indexed by knhk_construct8_pattern_t (0 = GENERIC, 1 = ALL_NONZERO, 2-9 = LEN1-LEN8)
// Branchless lookup: fn = dispatch_table[ir->construct8_pattern_hint]
static inline __attribute__((always_inline)) const knhk_construct8_fn_t* knhk_get_construct8_dispatch_table(void)
{
  // Forward declarations for wrapper functions (defined in simd/construct.h)
  extern size_t knhk_construct8_emit_8_len1_wrapper(const uint64_t *S_base, uint64_t off, uint64_t len,
                                                     uint64_t p_const, uint64_t o_const,
                                                     uint64_t *restrict out_S, uint64_t *restrict out_P, uint64_t *restrict out_O,
                                                     uint64_t *restrict out_mask);
  extern size_t knhk_construct8_emit_8_len2_wrapper(const uint64_t *S_base, uint64_t off, uint64_t len,
                                                     uint64_t p_const, uint64_t o_const,
                                                     uint64_t *restrict out_S, uint64_t *restrict out_P, uint64_t *restrict out_O,
                                                     uint64_t *restrict out_mask);
  extern size_t knhk_construct8_emit_8_len3_wrapper(const uint64_t *S_base, uint64_t off, uint64_t len,
                                                     uint64_t p_const, uint64_t o_const,
                                                     uint64_t *restrict out_S, uint64_t *restrict out_P, uint64_t *restrict out_O,
                                                     uint64_t *restrict out_mask);
  extern size_t knhk_construct8_emit_8_len4_wrapper(const uint64_t *S_base, uint64_t off, uint64_t len,
                                                     uint64_t p_const, uint64_t o_const,
                                                     uint64_t *restrict out_S, uint64_t *restrict out_P, uint64_t *restrict out_O,
                                                     uint64_t *restrict out_mask);
  extern size_t knhk_construct8_emit_8_len5_wrapper(const uint64_t *S_base, uint64_t off, uint64_t len,
                                                     uint64_t p_const, uint64_t o_const,
                                                     uint64_t *restrict out_S, uint64_t *restrict out_P, uint64_t *restrict out_O,
                                                     uint64_t *restrict out_mask);
  extern size_t knhk_construct8_emit_8_len6_wrapper(const uint64_t *S_base, uint64_t off, uint64_t len,
                                                     uint64_t p_const, uint64_t o_const,
                                                     uint64_t *restrict out_S, uint64_t *restrict out_P, uint64_t *restrict out_O,
                                                     uint64_t *restrict out_mask);
  extern size_t knhk_construct8_emit_8_len7_wrapper(const uint64_t *S_base, uint64_t off, uint64_t len,
                                                     uint64_t p_const, uint64_t o_const,
                                                     uint64_t *restrict out_S, uint64_t *restrict out_P, uint64_t *restrict out_O,
                                                     uint64_t *restrict out_mask);
  extern size_t knhk_construct8_emit_8_len8_wrapper(const uint64_t *S_base, uint64_t off, uint64_t len,
                                                     uint64_t p_const, uint64_t o_const,
                                                     uint64_t *restrict out_S, uint64_t *restrict out_P, uint64_t *restrict out_O,
                                                     uint64_t *restrict out_mask);
  extern size_t knhk_construct8_emit_8_all_nonzero_wrapper(const uint64_t *S_base, uint64_t off, uint64_t len,
                                                            uint64_t p_const, uint64_t o_const,
                                                            uint64_t *restrict out_S, uint64_t *restrict out_P, uint64_t *restrict out_O,
                                                            uint64_t *restrict out_mask);
  
  static const knhk_construct8_fn_t dispatch_table[KNHK_CONSTRUCT8_PATTERN_MAX] = {
    [KNHK_CONSTRUCT8_PATTERN_GENERIC] = knhk_construct8_emit_8,
    [KNHK_CONSTRUCT8_PATTERN_ALL_NONZERO] = knhk_construct8_emit_8_all_nonzero_wrapper,
    [KNHK_CONSTRUCT8_PATTERN_LEN1] = knhk_construct8_emit_8_len1_wrapper,
    [KNHK_CONSTRUCT8_PATTERN_LEN2] = knhk_construct8_emit_8_len2_wrapper,
    [KNHK_CONSTRUCT8_PATTERN_LEN3] = knhk_construct8_emit_8_len3_wrapper,
    [KNHK_CONSTRUCT8_PATTERN_LEN4] = knhk_construct8_emit_8_len4_wrapper,
    [KNHK_CONSTRUCT8_PATTERN_LEN5] = knhk_construct8_emit_8_len5_wrapper,
    [KNHK_CONSTRUCT8_PATTERN_LEN6] = knhk_construct8_emit_8_len6_wrapper,
    [KNHK_CONSTRUCT8_PATTERN_LEN7] = knhk_construct8_emit_8_len7_wrapper,
    [KNHK_CONSTRUCT8_PATTERN_LEN8] = knhk_construct8_emit_8_len8_wrapper,
  };
  return dispatch_table;
}

// Emit up to 8 triples using a fixed template (CONSTRUCT8)
// Returns number of lanes written, fills rcpt with user knowledge only
// AOT optimization: Routes to specialized functions via branchless dispatch table
// Pattern hint set by warm path based on pattern detection (all-nonzero, len1-len8)
#ifndef KNHK_EVAL_CONSTRUCT8_INLINE
#define KNHK_EVAL_CONSTRUCT8_INLINE static inline __attribute__((always_inline))
#endif
KNHK_EVAL_CONSTRUCT8_INLINE int knhk_eval_construct8(const knhk_context_t *ctx, knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  if (!ctx || !ir || ir->op != KNHK_OP_CONSTRUCT8)
    return 0;
  
  if (!ir->out_S || !ir->out_P || !ir->out_O)
    return 0;
  
  if (ir->p != ctx->run.pred)
    return 0;
  
  // Hot path: Pure CONSTRUCT logic only (branchless SIMD)
  // Branchless routing: Use dispatch table indexed by pattern_hint
  // Pattern hint set by warm path (all-nonzero detection, length specialization)
#if NROWS == 8
  // Branchless dispatch: table lookup (no branches, no mispredicts)
  // Bounds check: if pattern_hint >= MAX, use generic (branchless via mask)
  uint8_t pattern_idx = ir->construct8_pattern_hint;
  uint8_t pattern_valid = (pattern_idx < KNHK_CONSTRUCT8_PATTERN_MAX) ? pattern_idx : KNHK_CONSTRUCT8_PATTERN_GENERIC;
  
  const knhk_construct8_fn_t* dispatch_table = knhk_get_construct8_dispatch_table();
  knhk_construct8_fn_t fn = dispatch_table[pattern_valid];
  
  // Call specialized function (branchless)
  // For length-specialized variants, len parameter is compile-time constant
  size_t written = fn(ctx->S, ctx->run.off, ctx->run.len,
                       ir->p, ir->o,
                       ir->out_S, ir->out_P, ir->out_O,
                       &ir->out_mask);
#else
  // Scalar fallback for non-8 configurations
  // ctx->run.len is guaranteed ≤ 8 at Chicago TDD level
  const uint64_t *s_p = ctx->S + ctx->run.off;
  size_t written = 0;
  uint64_t mask = 0;
  for (uint64_t i = 0; i < ctx->run.len; i++) {
    if (s_p[i] != 0) {
      ir->out_S[written] = s_p[i];
      ir->out_P[written] = ir->p;
      ir->out_O[written] = ir->o;
      mask |= (1ULL << i);
      written++;
    }
  }
  ir->out_mask = mask;
#endif
  
  // Fill receipt with user knowledge only (provenance, not timing)
  if (rcpt) {
    rcpt->lanes = (uint32_t)written;  // User knowledge: how many triples constructed
    rcpt->span_id = knhk_generate_span_id();  // User knowledge: provenance trace ID
    rcpt->a_hash = (uint64_t)(ir->s ^ ir->p ^ ir->o ^ ctx->run.pred ^ ir->out_mask);  // User knowledge: provenance hash
  }
  
  return (int)written;  // User knowledge: number of triples constructed
}

#endif // KNHK_EVAL_H

