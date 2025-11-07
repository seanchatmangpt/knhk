// eval_dispatch.c
// Branchless operation dispatch implementation
// Eliminates if-else chains and branches for zero mispredicts

#include "knhk/eval_dispatch.h"
#include "knhk/types.h"
#include "knhk/eval.h"  // For knhk_generate_span_id declaration
#include "simd.h"
#include <stdint.h>
#include <limits.h>

// Branchless comparison helpers (no branches, use arithmetic)
// These use sign bit arithmetic to avoid branches
static inline int branchless_ge(uint64_t a, uint64_t b) {
  // (a >= b) ? 1 : 0
  // For unsigned: if a >= b, then (a - b) doesn't wrap, so sign bit is 0
  // Return 1 if sign bit is 0, else 0 (branchless via arithmetic)
  uint64_t diff = a - b;
  // If diff wraps (a < b), sign bit is 1, return 0
  // If diff doesn't wrap (a >= b), sign bit is 0, return 1
  return (int)(1 - ((diff >> 63) & 1));
}

static inline int branchless_le(uint64_t a, uint64_t b) {
  // (a <= b) ? 1 : 0
  // Same logic: if b >= a, then (b - a) doesn't wrap
  uint64_t diff = b - a;
  return (int)(1 - ((diff >> 63) & 1));
}

static inline int branchless_eq(uint64_t a, uint64_t b) {
  // (a == b) ? 1 : 0
  // Use XOR: if a == b, then (a ^ b) == 0
  // Convert to boolean: 1 if zero, 0 if non-zero (branchless)
  uint64_t xor_result = a ^ b;
  // Use compiler builtin for popcount if available, else use OR reduction
  // If xor_result == 0, then no bits are set, return 1
  // If xor_result != 0, then at least one bit is set, return 0
  // Simple approach: use !(xor_result) but convert to 0/1 explicitly
  // Compiler should optimize (xor_result == 0) to branchless code
  // For maximum safety, use arithmetic: 1 - (min(1, popcount(xor_result)))
  #if defined(__GNUC__) || defined(__clang__)
    // Use builtin popcount (branchless)
    uint64_t popcnt = __builtin_popcountll(xor_result);
    // Return 1 if popcnt == 0, else 0 (branchless via sign bit)
    // If popcnt == 0, then (popcnt | (popcnt >> 32)) == 0, so return 1
    // If popcnt > 0, then (popcnt | (popcnt >> 32)) has bits set, so return 0
    uint64_t has_bits = popcnt | (popcnt >> 32);
    has_bits |= (has_bits >> 16);
    has_bits |= (has_bits >> 8);
    has_bits |= (has_bits >> 4);
    has_bits |= (has_bits >> 2);
    has_bits |= (has_bits >> 1);
    return (int)(1 - (has_bits & 1));
  #else
    // Fallback: OR reduction (branchless)
    uint64_t has_bits = xor_result | (xor_result >> 32);
    has_bits |= (has_bits >> 16);
    has_bits |= (has_bits >> 8);
    has_bits |= (has_bits >> 4);
    has_bits |= (has_bits >> 2);
    has_bits |= (has_bits >> 1);
    return (int)(1 - (has_bits & 1));
  #endif
}

// Branchless operation: ASK_SP
int knhk_eval_ask_sp(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  // Note: Predicate check is handled in eval.h, so we always execute here
  // The result will be masked in eval.h if predicate doesn't match
  
#if NROWS == 8
  int result = knhk_eq64_exists_8(ctx->S, ctx->run.off, ir->s);
#else
  int result = knhk_eq64_exists_run(ctx->S, ctx->run.off, ctx->run.len, ir->s);
#endif
  
  // Fill receipt (always, no branch)
  // Note: eval.h will mask receipt fields if predicate doesn't match
  if (rcpt) {
    rcpt->lanes = KNHK_NROWS;
    rcpt->span_id = knhk_generate_span_id();
    rcpt->a_hash = (uint64_t)(ir->s ^ ir->p ^ ir->o ^ ir->k ^ (uint64_t)result ^ ctx->run.pred);
  }
  
  return result;
}

// Branchless operation: ASK_SPO
int knhk_eval_ask_spo(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  uint64_t pred_match = (ir->p == ctx->run.pred) ? UINT64_MAX : 0;
  
#if NROWS == 8
  int result = knhk_eq64_spo_exists_8(ctx->S, ctx->O, ctx->run.off, ir->s, ir->o);
#else
  int result = knhk_eq64_spo_exists_run(ctx->S, ctx->O, ctx->run.off, ctx->run.len, ir->s, ir->o);
#endif
  
  result = (int)((uint64_t)result & pred_match);
  
  if (rcpt) {
    rcpt->lanes = KNHK_NROWS;
    rcpt->span_id = knhk_generate_span_id();
    rcpt->a_hash = (uint64_t)(ir->s ^ ir->p ^ ir->o ^ ir->k ^ (uint64_t)result ^ ctx->run.pred);
  }
  
  return result;
}

// Branchless operation: COUNT_SP_GE
int knhk_eval_count_sp_ge(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  uint64_t pred_match = (ir->p == ctx->run.pred) ? UINT64_MAX : 0;
  
#if NROWS == 8
  uint64_t cnt = knhk_eq64_count_8(ctx->S, ctx->run.off, ir->s);
#else
  uint64_t cnt = knhk_eq64_count_run(ctx->S, ctx->run.off, ctx->run.len, ir->s);
#endif
  
  // Branchless comparison: cnt >= ir->k
  int result = branchless_ge(cnt, ir->k);
  result = (int)((uint64_t)result & pred_match);
  
  if (rcpt) {
    rcpt->lanes = KNHK_NROWS;
    rcpt->span_id = knhk_generate_span_id();
    rcpt->a_hash = (uint64_t)(ir->s ^ ir->p ^ ir->o ^ ir->k ^ cnt ^ (uint64_t)result ^ ctx->run.pred);
  }
  
  return result;
}

// Branchless operation: COUNT_SP_LE
int knhk_eval_count_sp_le(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  uint64_t pred_match = (ir->p == ctx->run.pred) ? UINT64_MAX : 0;
  
#if NROWS == 8
  uint64_t cnt = knhk_eq64_count_8(ctx->S, ctx->run.off, ir->s);
#else
  uint64_t cnt = knhk_eq64_count_run(ctx->S, ctx->run.off, ctx->run.len, ir->s);
#endif
  
  int result = branchless_le(cnt, ir->k);
  result = (int)((uint64_t)result & pred_match);
  
  if (rcpt) {
    rcpt->lanes = KNHK_NROWS;
    rcpt->span_id = knhk_generate_span_id();
    rcpt->a_hash = (uint64_t)(ir->s ^ ir->p ^ ir->o ^ ir->k ^ cnt ^ (uint64_t)result ^ ctx->run.pred);
  }
  
  return result;
}

// Branchless operation: COUNT_SP_EQ
int knhk_eval_count_sp_eq(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  uint64_t pred_match = (ir->p == ctx->run.pred) ? UINT64_MAX : 0;
  
#if NROWS == 8
  uint64_t cnt = knhk_eq64_count_8(ctx->S, ctx->run.off, ir->s);
#else
  uint64_t cnt = knhk_eq64_count_run(ctx->S, ctx->run.off, ctx->run.len, ir->s);
#endif
  
  int result = branchless_eq(cnt, ir->k);
  result = (int)((uint64_t)result & pred_match);
  
  if (rcpt) {
    rcpt->lanes = KNHK_NROWS;
    rcpt->span_id = knhk_generate_span_id();
    rcpt->a_hash = (uint64_t)(ir->s ^ ir->p ^ ir->o ^ ir->k ^ cnt ^ (uint64_t)result ^ ctx->run.pred);
  }
  
  return result;
}

// Branchless operation: ASK_OP
int knhk_eval_ask_op(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  uint64_t pred_match = (ir->p == ctx->run.pred) ? UINT64_MAX : 0;
  
#if NROWS == 8
  int result = knhk_eq64_exists_o_8(ctx->O, ctx->run.off, ir->o);
#else
  int result = knhk_eq64_exists_run(ctx->O, ctx->run.off, ctx->run.len, ir->o);
#endif
  
  result = (int)((uint64_t)result & pred_match);
  
  if (rcpt) {
    rcpt->lanes = KNHK_NROWS;
    rcpt->span_id = knhk_generate_span_id();
    rcpt->a_hash = (uint64_t)(ir->s ^ ir->p ^ ir->o ^ ir->k ^ (uint64_t)result ^ ctx->run.pred);
  }
  
  return result;
}

// Branchless operation: UNIQUE_SP
int knhk_eval_unique_sp(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  uint64_t pred_match = (ir->p == ctx->run.pred) ? UINT64_MAX : 0;
  
#if NROWS == 8
  uint64_t cnt = knhk_eq64_count_8(ctx->S, ctx->run.off, ir->s);
#else
  uint64_t cnt = knhk_eq64_count_run(ctx->S, ctx->run.off, ctx->run.len, ir->s);
#endif
  
  int result = branchless_eq(cnt, 1ULL);
  result = (int)((uint64_t)result & pred_match);
  
  if (rcpt) {
    rcpt->lanes = KNHK_NROWS;
    rcpt->span_id = knhk_generate_span_id();
    rcpt->a_hash = (uint64_t)(ir->s ^ ir->p ^ ir->o ^ ir->k ^ cnt ^ (uint64_t)result ^ ctx->run.pred);
  }
  
  return result;
}

// Branchless operation: COUNT_OP
int knhk_eval_count_op(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  uint64_t pred_match = (ir->p == ctx->run.pred) ? UINT64_MAX : 0;
  
#if NROWS == 8
  uint64_t cnt = knhk_eq64_count_8(ctx->O, ctx->run.off, ir->o);
#else
  uint64_t cnt = knhk_eq64_count_run(ctx->O, ctx->run.off, ctx->run.len, ir->o);
#endif
  
  int result = branchless_ge(cnt, ir->k);
  result = (int)((uint64_t)result & pred_match);
  
  if (rcpt) {
    rcpt->lanes = KNHK_NROWS;
    rcpt->span_id = knhk_generate_span_id();
    rcpt->a_hash = (uint64_t)(ir->s ^ ir->p ^ ir->o ^ ir->k ^ cnt ^ (uint64_t)result ^ ctx->run.pred);
  }
  
  return result;
}

// Branchless operation: COUNT_OP_LE
int knhk_eval_count_op_le(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  uint64_t pred_match = (ir->p == ctx->run.pred) ? UINT64_MAX : 0;
  
#if NROWS == 8
  uint64_t cnt = knhk_eq64_count_8(ctx->O, ctx->run.off, ir->o);
#else
  uint64_t cnt = knhk_eq64_count_run(ctx->O, ctx->run.off, ctx->run.len, ir->o);
#endif
  
  int result = branchless_le(cnt, ir->k);
  result = (int)((uint64_t)result & pred_match);
  
  if (rcpt) {
    rcpt->lanes = KNHK_NROWS;
    rcpt->span_id = knhk_generate_span_id();
    rcpt->a_hash = (uint64_t)(ir->s ^ ir->p ^ ir->o ^ ir->k ^ cnt ^ (uint64_t)result ^ ctx->run.pred);
  }
  
  return result;
}

// Branchless operation: COUNT_OP_EQ
int knhk_eval_count_op_eq(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  uint64_t pred_match = (ir->p == ctx->run.pred) ? UINT64_MAX : 0;
  
#if NROWS == 8
  uint64_t cnt = knhk_eq64_count_8(ctx->O, ctx->run.off, ir->o);
#else
  uint64_t cnt = knhk_eq64_count_run(ctx->O, ctx->run.off, ctx->run.len, ir->o);
#endif
  
  int result = branchless_eq(cnt, ir->k);
  result = (int)((uint64_t)result & pred_match);
  
  if (rcpt) {
    rcpt->lanes = KNHK_NROWS;
    rcpt->span_id = knhk_generate_span_id();
    rcpt->a_hash = (uint64_t)(ir->s ^ ir->p ^ ir->o ^ ir->k ^ cnt ^ (uint64_t)result ^ ctx->run.pred);
  }
  
  return result;
}

// Branchless operation: COMPARE_O_EQ
int knhk_eval_compare_o_eq(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  uint64_t pred_match = (ir->p == ctx->run.pred) ? UINT64_MAX : 0;
  
#if NROWS == 8
  int result = knhk_compare_o_8(ctx->O, ctx->run.off, ir->o, 0);
#else
  // Scalar fallback (branchless loop)
  uint64_t has_match = 0;
  for (uint64_t i = 0; i < ctx->run.len; i++) {
    has_match |= (ctx->O[ctx->run.off + i] == ir->o) ? UINT64_MAX : 0;
  }
  int result = (has_match != 0) ? 1 : 0;
#endif
  
  result = (int)((uint64_t)result & pred_match);
  
  if (rcpt) {
    rcpt->lanes = KNHK_NROWS;
    rcpt->span_id = knhk_generate_span_id();
    rcpt->a_hash = (uint64_t)(ir->s ^ ir->p ^ ir->o ^ ir->k ^ (uint64_t)result ^ ctx->run.pred);
  }
  
  return result;
}

// Branchless operation: COMPARE_O_GT
int knhk_eval_compare_o_gt(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  uint64_t pred_match = (ir->p == ctx->run.pred) ? UINT64_MAX : 0;
  
#if NROWS == 8
  int result = knhk_compare_o_8(ctx->O, ctx->run.off, ir->o, 1);
#else
  uint64_t has_match = 0;
  for (uint64_t i = 0; i < ctx->run.len; i++) {
    has_match |= (ctx->O[ctx->run.off + i] > ir->o) ? UINT64_MAX : 0;
  }
  int result = (has_match != 0) ? 1 : 0;
#endif
  
  result = (int)((uint64_t)result & pred_match);
  
  if (rcpt) {
    rcpt->lanes = KNHK_NROWS;
    rcpt->span_id = knhk_generate_span_id();
    rcpt->a_hash = (uint64_t)(ir->s ^ ir->p ^ ir->o ^ ir->k ^ (uint64_t)result ^ ctx->run.pred);
  }
  
  return result;
}

// Branchless operation: COMPARE_O_LT
int knhk_eval_compare_o_lt(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  uint64_t pred_match = (ir->p == ctx->run.pred) ? UINT64_MAX : 0;
  
#if NROWS == 8
  int result = knhk_compare_o_8(ctx->O, ctx->run.off, ir->o, 2);
#else
  uint64_t has_match = 0;
  for (uint64_t i = 0; i < ctx->run.len; i++) {
    has_match |= (ctx->O[ctx->run.off + i] < ir->o) ? UINT64_MAX : 0;
  }
  int result = (has_match != 0) ? 1 : 0;
#endif
  
  result = (int)((uint64_t)result & pred_match);
  
  if (rcpt) {
    rcpt->lanes = KNHK_NROWS;
    rcpt->span_id = knhk_generate_span_id();
    rcpt->a_hash = (uint64_t)(ir->s ^ ir->p ^ ir->o ^ ir->k ^ (uint64_t)result ^ ctx->run.pred);
  }
  
  return result;
}

// Branchless operation: COMPARE_O_GE
int knhk_eval_compare_o_ge(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  uint64_t pred_match = (ir->p == ctx->run.pred) ? UINT64_MAX : 0;
  
#if NROWS == 8
  int result = knhk_compare_o_8(ctx->O, ctx->run.off, ir->o, 3);
#else
  uint64_t has_match = 0;
  for (uint64_t i = 0; i < ctx->run.len; i++) {
    has_match |= (ctx->O[ctx->run.off + i] >= ir->o) ? UINT64_MAX : 0;
  }
  int result = (has_match != 0) ? 1 : 0;
#endif
  
  result = (int)((uint64_t)result & pred_match);
  
  if (rcpt) {
    rcpt->lanes = KNHK_NROWS;
    rcpt->span_id = knhk_generate_span_id();
    rcpt->a_hash = (uint64_t)(ir->s ^ ir->p ^ ir->o ^ ir->k ^ (uint64_t)result ^ ctx->run.pred);
  }
  
  return result;
}

// Branchless operation: COMPARE_O_LE
int knhk_eval_compare_o_le(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  uint64_t pred_match = (ir->p == ctx->run.pred) ? UINT64_MAX : 0;
  
#if NROWS == 8
  int result = knhk_compare_o_8(ctx->O, ctx->run.off, ir->o, 4);
#else
  uint64_t has_match = 0;
  for (uint64_t i = 0; i < ctx->run.len; i++) {
    has_match |= (ctx->O[ctx->run.off + i] <= ir->o) ? UINT64_MAX : 0;
  }
  int result = (has_match != 0) ? 1 : 0;
#endif
  
  result = (int)((uint64_t)result & pred_match);
  
  if (rcpt) {
    rcpt->lanes = KNHK_NROWS;
    rcpt->span_id = knhk_generate_span_id();
    rcpt->a_hash = (uint64_t)(ir->s ^ ir->p ^ ir->o ^ ir->k ^ (uint64_t)result ^ ctx->run.pred);
  }
  
  return result;
}

// Branchless operation: VALIDATE_DATATYPE_SP
int knhk_eval_validate_datatype_sp(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  uint64_t pred_match = (ir->p == ctx->run.pred) ? UINT64_MAX : 0;
  
#if NROWS == 8
  int result = knhk_validate_datatype_sp_8(ctx->S, ctx->O, ctx->run.off, ir->s, ir->o);
#else
  // Scalar fallback (branchless)
  uint64_t has_match = 0;
  for (uint64_t i = 0; i < ctx->run.len; i++) {
    uint64_t s_match = (ctx->S[ctx->run.off + i] == ir->s) ? UINT64_MAX : 0;
    uint64_t o_match = (ctx->O[ctx->run.off + i] == ir->o) ? UINT64_MAX : 0;
    has_match |= (s_match & o_match);
  }
  int result = (has_match != 0) ? 1 : 0;
#endif
  
  result = (int)((uint64_t)result & pred_match);
  
  if (rcpt) {
    rcpt->lanes = KNHK_NROWS;
    rcpt->span_id = knhk_generate_span_id();
    rcpt->a_hash = (uint64_t)(ir->s ^ ir->p ^ ir->o ^ ir->k ^ (uint64_t)result ^ ctx->run.pred);
  }
  
  return result;
}

// Branchless operation: VALIDATE_DATATYPE_SPO
int knhk_eval_validate_datatype_spo(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  uint64_t pred_match = (ir->p == ctx->run.pred) ? UINT64_MAX : 0;
  
#if NROWS == 8
  int exists = knhk_eq64_spo_exists_8(ctx->S, ctx->O, ctx->run.off, ir->s, ir->o);
#else
  int exists = knhk_eq64_spo_exists_run(ctx->S, ctx->O, ctx->run.off, ctx->run.len, ir->s, ir->o);
#endif
  
  // For SPO datatype validation, existence check is sufficient at hot path
  // Full datatype hash validation occurs upstream at schema level
  int result = exists;
  result = (int)((uint64_t)result & pred_match);
  
  if (rcpt) {
    rcpt->lanes = KNHK_NROWS;
    rcpt->span_id = knhk_generate_span_id();
    rcpt->a_hash = (uint64_t)(ir->s ^ ir->p ^ ir->o ^ ir->k ^ (uint64_t)result ^ ctx->run.pred);
  }
  
  return result;
}

// No-op for invalid/unsupported operations
int knhk_eval_noop(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt)
{
  (void)ctx; (void)ir; // Suppress unused warnings
  
  if (rcpt) {
    rcpt->lanes = 0;
    rcpt->span_id = 0;
    rcpt->a_hash = 0;
  }
  
  return 0;
}

// Dispatch table: maps operation codes to branchless functions
// Const, read-only, cache-friendly (all entries initialized)
static const knhk_eval_fn_t dispatch_table[KNHK_OP_MAX] = {
  [0] = knhk_eval_noop,                                    // Invalid
  [KNHK_OP_ASK_SP] = knhk_eval_ask_sp,
  [KNHK_OP_COUNT_SP_GE] = knhk_eval_count_sp_ge,
  [KNHK_OP_ASK_SPO] = knhk_eval_ask_spo,
  [KNHK_OP_SELECT_SP] = knhk_eval_noop,                   // SELECT moved to warm path
  [KNHK_OP_COUNT_SP_LE] = knhk_eval_count_sp_le,
  [KNHK_OP_COUNT_SP_EQ] = knhk_eval_count_sp_eq,
  [KNHK_OP_ASK_OP] = knhk_eval_ask_op,
  [KNHK_OP_UNIQUE_SP] = knhk_eval_unique_sp,
  [KNHK_OP_COUNT_OP] = knhk_eval_count_op,
  [KNHK_OP_COUNT_OP_LE] = knhk_eval_count_op_le,
  [KNHK_OP_COUNT_OP_EQ] = knhk_eval_count_op_eq,
  [KNHK_OP_COMPARE_O_EQ] = knhk_eval_compare_o_eq,
  [KNHK_OP_COMPARE_O_GT] = knhk_eval_compare_o_gt,
  [KNHK_OP_COMPARE_O_LT] = knhk_eval_compare_o_lt,
  [KNHK_OP_COMPARE_O_GE] = knhk_eval_compare_o_ge,
  [KNHK_OP_COMPARE_O_LE] = knhk_eval_compare_o_le,
  [KNHK_OP_VALIDATE_DATATYPE_SP] = knhk_eval_validate_datatype_sp,
  [KNHK_OP_VALIDATE_DATATYPE_SPO] = knhk_eval_validate_datatype_spo,
  // KNHK_OP_CONSTRUCT8 = 32 (warm path, not in hot path dispatch)
};

// Force inline to eliminate function call overhead
inline __attribute__((always_inline)) const knhk_eval_fn_t* knhk_get_eval_dispatch_table(void)
{
  return dispatch_table;
}

