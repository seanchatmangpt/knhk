// knhk/fiber.h
// Fiber execution interface: per-shard, per-hook execution units
// Executes μ on ≤8 items (run_len≤8), parks to W1 if needed

#ifndef KNHK_FIBER_H
#define KNHK_FIBER_H

#include "types.h"
#include "beat.h"
#include "ring.h"
#include "eval.h"
#include <stddef.h>

// Fiber execution result
typedef enum {
  KNHK_FIBER_SUCCESS = 0,      // Execution completed successfully
  KNHK_FIBER_PARKED = 1,       // Δ parked to W1 (L1 miss or ticks>8)
  KNHK_FIBER_ERROR = -1        // Execution error
} knhk_fiber_result_t;

// Execute μ on ≤8 items at tick slot
// Takes: ctx (with pinned run), ir (hook IR), tick slot, cycle_id, shard_id, hook_id
// Returns: receipt with cycle_id, shard_id, hook_id, ticks, span_id, a_hash
// Fills receipt with provenance information
// If execution exceeds 8 ticks or predicts L1 miss, parks Δ to W1
knhk_fiber_result_t knhk_fiber_execute(
  const knhk_context_t *ctx,
  knhk_hook_ir_t *ir,
  uint64_t tick,
  uint64_t cycle_id,
  uint64_t shard_id,
  uint64_t hook_id,
  knhk_receipt_t *receipt
);

// Park delta to W1 (warm path)
// Single atomic write sets PARKED flag in ring
void knhk_fiber_park(
  knhk_delta_ring_t *delta_ring,
  uint64_t tick,
  uint64_t ring_idx,
  uint64_t cycle_id
);

// Execute fiber from delta ring at tick slot
// Reads delta from ring, executes μ, writes assertion + receipt to output ring
// Returns number of deltas processed
size_t knhk_fiber_process_tick(
  knhk_delta_ring_t *delta_ring,
  knhk_assertion_ring_t *assertion_ring,
  knhk_context_t *ctx,
  knhk_hook_ir_t *ir,
  uint64_t tick,
  uint64_t shard_id,
  uint64_t hook_id
);

#endif // KNHK_FIBER_H

