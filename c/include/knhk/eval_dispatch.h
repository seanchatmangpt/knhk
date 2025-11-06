// knhk/eval_dispatch.h
// Branchless operation dispatch table for hot path execution
// Eliminates if-else chains to achieve zero branch mispredicts

#ifndef KNHK_EVAL_DISPATCH_H
#define KNHK_EVAL_DISPATCH_H

#include "types.h"

// Function pointer type for branchless operation dispatch
typedef int (*knhk_eval_fn_t)(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt);

// Branchless operation implementations (forward declarations)
// These are implemented in eval_dispatch.c
int knhk_eval_ask_sp(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt);
int knhk_eval_ask_spo(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt);
int knhk_eval_count_sp_ge(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt);
int knhk_eval_count_sp_le(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt);
int knhk_eval_count_sp_eq(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt);
int knhk_eval_ask_op(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt);
int knhk_eval_unique_sp(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt);
int knhk_eval_count_op(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt);
int knhk_eval_count_op_le(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt);
int knhk_eval_count_op_eq(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt);
int knhk_eval_compare_o_eq(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt);
int knhk_eval_compare_o_gt(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt);
int knhk_eval_compare_o_lt(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt);
int knhk_eval_compare_o_ge(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt);
int knhk_eval_compare_o_le(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt);
int knhk_eval_validate_datatype_sp(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt);
int knhk_eval_validate_datatype_spo(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt);
int knhk_eval_noop(const knhk_context_t *ctx, const knhk_hook_ir_t *ir, knhk_receipt_t *rcpt); // Invalid/unsupported op

// Maximum operation code (for bounds checking)
#define KNHK_OP_MAX 33  // KNHK_OP_CONSTRUCT8 = 32, so max valid op is 32

// Get dispatch table (const, read-only, cache-friendly)
// Returns pointer to dispatch table array
const knhk_eval_fn_t* knhk_get_eval_dispatch_table(void);

#endif // KNHK_EVAL_DISPATCH_H

