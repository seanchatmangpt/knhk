// knhk/receipts.h
// Receipt operations: merging and provenance tracking

#ifndef KNHK_RECEIPTS_H
#define KNHK_RECEIPTS_H

#include "types.h"

// Combine receipts via ⊕ (associative, branchless)
// Preserves cycle_id, shard_id, hook_id from first receipt
// Merges ticks (max), actual_ticks (max), lanes (sum), span_id and a_hash (XOR)
static inline knhk_receipt_t knhk_receipt_merge(knhk_receipt_t a, knhk_receipt_t b)
{
  knhk_receipt_t merged;
  // Preserve identifiers from first receipt (deterministic ordering)
  merged.cycle_id = a.cycle_id;
  merged.shard_id = a.shard_id;
  merged.hook_id = a.hook_id;
  // Merge metrics: max ticks (both estimated and actual)
  merged.ticks = (a.ticks > b.ticks) ? a.ticks : b.ticks;
  merged.actual_ticks = (a.actual_ticks > b.actual_ticks) ? a.actual_ticks : b.actual_ticks;
  merged.lanes = a.lanes + b.lanes;
  // Merge provenance: XOR (⊕ monoid)
  merged.span_id = a.span_id ^ b.span_id;
  merged.a_hash = a.a_hash ^ b.a_hash;
  return merged;
}

#endif // KNHK_RECEIPTS_H

