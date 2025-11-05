// knhk/warm_path.c
// Warm path implementation for CONSTRUCT8 operations

#include "knhk/warm_path.h"
#include "knhk/eval.h"

int knhk_warm_execute_construct8(
    const knhk_context_t *ctx,
    knhk_hook_ir_t *ir,
    knhk_receipt_t *rcpt
) {
    if (!ctx || !ir || !rcpt) {
        return 0;
    }

    // Validate CONSTRUCT8 operation
    if (ir->op != KNHK_OP_CONSTRUCT8) {
        return 0;
    }

    // Validate output buffers
    if (!ir->out_S || !ir->out_P || !ir->out_O) {
        return 0;
    }

    // Execute CONSTRUCT8 (same implementation as hot path, but routed through warm path)
    // Note: This still calls the inline function from eval.h, but we're routing
    // it through warm path for timing/budget purposes
    return knhk_eval_construct8(ctx, ir, rcpt);
}

