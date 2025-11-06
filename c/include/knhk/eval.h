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
static inline int knhk_eval_bool(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
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

// Emit up to 8 triples using a fixed template (CONSTRUCT8)
// Returns number of lanes written, fills rcpt with user knowledge only
// WARM PATH: CONSTRUCT8 exceeds 8-tick budget (41-83 ticks), moved to warm path (≤500ms)
// This function is kept for backward compatibility but should be routed through warm path API
// Use knhk_warm_execute_construct8() from knhk/warm_path.h for warm path routing
static inline int knhk_eval_construct8(const knhk_context_t *ctx, knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  if (!ctx || !ir || ir->op != KNHK_OP_CONSTRUCT8)
    return 0;
  
  if (!ir->out_S || !ir->out_P || !ir->out_O)
    return 0;
  
  if (ir->p != ctx->run.pred)
    return 0;
  
  // Hot path: Pure CONSTRUCT logic only (branchless SIMD)
  // AOT optimization: Route to specialized functions based on len and pattern
  // Pattern detection happens at warm path, len is known at runtime
  // Note: Specialized variants available (len1-len8, all_nonzero) for future routing
#if NROWS == 8
  // Branchless: Always call generic function (len is guaranteed ≤ 8 by guard constraints)
  // Future optimization: Route to length-specialized variants via function table
  // Specialized functions: knhk_construct8_emit_8_len1() through len8()
  // Pattern-specialized: knhk_construct8_emit_8_all_nonzero() (skips mask generation)
  size_t written = knhk_construct8_emit_8(ctx->S, ctx->run.off, ctx->run.len,
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

