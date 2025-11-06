// core.c
// Core evaluation logic (v1.0)
// Updated to use branchless dispatch table

#include "core.h"
#include "knhk/eval.h"  // For knhk_eval_bool (branchless dispatch)
#include "simd.h"
#include "clock.h"
#include <string.h>

// Implementation moved to inline in header (knhk.h)

// Initialize context with arrays
void knhk_init_ctx(knhk_context_t *ctx, const uint64_t *S, const uint64_t *P, const uint64_t *O)
{
  if (!ctx) return;
  ctx->S = S;
  ctx->P = P;
  ctx->O = O;
  ctx->triple_count = 0;
  ctx->run.pred = 0;
  ctx->run.off = 0;
  ctx->run.len = 0;
}

// Evaluate boolean query (ASK, COUNT>=k, ASK_SPO)
// Now uses branchless dispatch table (zero branch mispredicts)
int knhk_core_eval_bool(const knhk_context_t *ctx, const knhk_hook_ir_t *ir)
{
  // Use branchless dispatch table via knhk_eval_bool()
  // This eliminates if-else chains and achieves zero branch mispredicts
  knhk_receipt_t rcpt = {0};
  return knhk_eval_bool(ctx, ir, &rcpt);
}

// CONSTRUCT8: Fixed-template emit (≤8 triples)
// Emits (S[i], P_template, O_template) for matching lanes
// Returns number of lanes written, fills receipt
// knhk_eval_construct8 is defined inline in header (knhk.h)

// Batch execution with Λ ordering (deterministic, ≤8 hooks)
// CONSTRUCT8 is inline in header, batch calls it
int knhk_eval_batch8(const knhk_context_t *ctx, knhk_hook_ir_t *irs, size_t n, knhk_receipt_t *rcpts)
{
  if (!ctx || !irs || n == 0 || n > KNHK_NROWS)
    return 0;
  
  if (!rcpts)
    return 0;
  
  int executed = 0;
  
  // Λ ordering: execute hooks in deterministic order (by index)
  // Each hook executes independently, receipts merge via ⊕
  for (size_t i = 0; i < n; i++) {
    knhk_receipt_t rcpt = {0};
    int result = 0;
    
    if (irs[i].op == KNHK_OP_CONSTRUCT8) {
      result = knhk_eval_construct8(ctx, &irs[i], &rcpt);
    } else {
      result = knhk_eval_bool(ctx, &irs[i], &rcpt);
    }
    
    rcpts[i] = rcpt;
    executed++;
    
    // If a hook fails (result == 0), continue to next hook (batch continues)
    // Guard H validation happens at Chicago TDD level, not in hot path
  }
  
  return executed;
}

// Evaluate SELECT query and return count of results (legacy, cold path)
size_t knhk_core_eval_select(const knhk_context_t *ctx, const knhk_hook_ir_t *ir)
{
  if (ir->p != ctx->run.pred || ir->op != KNHK_OP_SELECT_SP)
    return 0;

  if (!ir->select_out || ir->select_capacity == 0)
    return 0;

#if NROWS == 8
  // Use optimized unrolled version for NROWS=8
  return knhk_select_gather_8(ctx->S, ctx->O, ctx->run.off, ir->s, ir->select_out, ir->select_capacity);
#else
  return knhk_select_gather(ctx->S, ctx->O, ctx->run.off, ctx->run.len, ir->s, ir->select_out, ir->select_capacity);
#endif
}

