// knhk/warm_path.h
// Warm path API for CONSTRUCT8 operations (≤500ms budget)
// CONSTRUCT8 moved from hot path (exceeds 8-tick budget)

#ifndef KNHK_WARM_PATH_H
#define KNHK_WARM_PATH_H

#include "knhk/types.h"

#ifdef __cplusplus
extern "C" {
#endif

/// Execute CONSTRUCT8 in warm path (≤500ms budget)
/// 
/// This function routes CONSTRUCT8 operations from hot path to warm path
/// since CONSTRUCT8 performs emit work (SIMD loads, blending, stores) which
/// exceeds the 8-tick hot path budget.
///
/// @param ctx Context with pinned run
/// @param ir Hook IR with CONSTRUCT8 operation and output buffers
/// @param rcpt Receipt to fill with execution results
/// @return Number of lanes written (0 on error)
int knhk_warm_execute_construct8(
    const knhk_context_t *ctx,
    knhk_hook_ir_t *ir,
    knhk_receipt_t *rcpt
);

#ifdef __cplusplus
}
#endif

#endif // KNHK_WARM_PATH_H

