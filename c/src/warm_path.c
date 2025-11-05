// warm_path.c
// C implementation for warm path operations
// FFI interface for warm path CONSTRUCT8 operations

#include "knhk/warm_path.h"
#include "knhk/eval.h"
#include "knhk/utils.h"
#include <stdint.h>
#include <time.h>

int knhk_warm_execute_construct8(
    const knhk_context_t *ctx,
    knhk_hook_ir_t *ir,
    knhk_warm_result_t *result
) {
    // Validate inputs
    if (!ctx || !ir || !result) {
        return -1;
    }

    // Validate operation type
    if (ir->op != KNHK_OP_CONSTRUCT8) {
        return -1;
    }

    // Validate guard constraints
    if (ctx->run.len > 8) {
        return -1;
    }

    // Measure execution time
    struct timespec start, end;
    clock_gettime(CLOCK_MONOTONIC, &start);

    // Execute CONSTRUCT8 via hot path
    knhk_receipt_t rcpt = {0};
    int lanes_written = knhk_eval_construct8(ctx, ir, &rcpt);

    clock_gettime(CLOCK_MONOTONIC, &end);

    // Calculate latency in milliseconds
    uint64_t start_ms = (uint64_t)(start.tv_sec * 1000 + start.tv_nsec / 1000000);
    uint64_t end_ms = (uint64_t)(end.tv_sec * 1000 + end.tv_nsec / 1000000);
    uint64_t latency_ms = end_ms - start_ms;

    // Check timeout (500ms budget)
    if (latency_ms > 500) {
        return -1;
    }

    // Fill result structure
    result->success = (lanes_written > 0) ? 1 : 0;
    result->latency_ms = latency_ms;
    result->lanes_written = (size_t)lanes_written;
    result->span_id = rcpt.span_id;

    return 0;
}

