// knhk/warm_path.h
// Warm path API for operations that exceed 8-tick budget but complete within 500µs (≤1ms SLO)
// Production-ready FFI interface for warm path operations

#ifndef KNHK_WARM_PATH_H
#define KNHK_WARM_PATH_H

#include <stdint.h>
#include <stddef.h>
#include "knhk/eval.h"

#ifdef __cplusplus
extern "C" {
#endif

/// Warm path execution result
typedef struct {
    int success;           // 1 if successful, 0 otherwise
    uint64_t latency_ms;   // Execution time in milliseconds
    size_t lanes_written;  // Number of triples constructed
    uint64_t span_id;      // OTEL span ID for observability
} knhk_warm_result_t;

/// Execute CONSTRUCT8 operation in warm path
/// 
/// This function routes CONSTRUCT8 operations to warm path when they exceed
/// the 8-tick hot path budget but can complete within 500µs (≤1ms SLO).
/// 
/// @param ctx Hot path context (SoA arrays)
/// @param ir Hook IR with CONSTRUCT8 operation
/// @param result Output result structure (must not be NULL)
/// @return 0 on success, -1 on error
/// 
/// Performance: Budget ≤500µs, SLO ≤1ms (p99)
/// Guard constraints: Validates max_run_len ≤ 8
int knhk_warm_execute_construct8(
    const knhk_context_t *ctx,
    knhk_hook_ir_t *ir,
    knhk_warm_result_t *result
);

#ifdef __cplusplus
}
#endif

#endif // KNHK_WARM_PATH_H

