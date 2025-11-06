// knhk/ring.h
// Ring buffers for Δ (input) and A (output) with SoA layout
// Branchless enqueue/dequeue with atomic operations
// Power-of-2 size for mod-8 indexing

#ifndef KNHK_RING_H
#define KNHK_RING_H

#include "types.h"
#include <stdatomic.h>
#include <stddef.h>

// Ring buffer entry flags
#define KNHK_RING_FLAG_PARKED 0x1ULL  // Entry parked to W1
#define KNHK_RING_FLAG_VALID  0x2ULL  // Entry contains valid data

// Δ-ring (input): SoA layout for deltas
typedef struct {
  uint64_t *S;              // Subject array (64B aligned)
  uint64_t *P;              // Predicate array
  uint64_t *O;              // Object array
  uint64_t *cycle_ids;      // Cycle IDs per entry
  _Atomic(uint64_t) *flags; // Entry flags (PARKED, VALID) - atomic array
  uint64_t size;            // Power-of-2 size (e.g., 256, 512, 1024)
  uint64_t size_mask;       // size - 1 (for mod operation)
  _Atomic(uint64_t) write_idx[8];  // Per-tick write indices
  _Atomic(uint64_t) read_idx[8];   // Per-tick read indices
} knhk_delta_ring_t;

// A-ring (output): SoA layout for assertions + receipts
typedef struct {
  uint64_t *S;              // Subject array (64B aligned)
  uint64_t *P;              // Predicate array
  uint64_t *O;              // Object array
  knhk_receipt_t *receipts; // Receipts array (parallel to S/P/O)
  uint64_t size;            // Power-of-2 size
  uint64_t size_mask;       // size - 1
  _Atomic(uint64_t) write_idx[8];  // Per-tick write indices
  _Atomic(uint64_t) read_idx[8];   // Per-tick read indices
} knhk_assertion_ring_t;

// Initialize Δ-ring
// Allocates SoA arrays with 64-byte alignment
// size must be power-of-2 (e.g., 256, 512, 1024)
// Returns 0 on success, -1 on failure
int knhk_ring_init_delta(knhk_delta_ring_t *ring, uint64_t size);

// Initialize A-ring
// Allocates SoA arrays with 64-byte alignment
// Returns 0 on success, -1 on failure
int knhk_ring_init_assertion(knhk_assertion_ring_t *ring, uint64_t size);

// Cleanup ring buffers (free allocated memory)
void knhk_ring_cleanup_delta(knhk_delta_ring_t *ring);
void knhk_ring_cleanup_assertion(knhk_assertion_ring_t *ring);

// Enqueue delta to ring at tick slot
// Non-blocking: uses atomic fetch-and-add
// Returns 0 on success, -1 if ring is full
int knhk_ring_enqueue_delta(knhk_delta_ring_t *ring, uint64_t tick,
                            const uint64_t *S, const uint64_t *P, const uint64_t *O,
                            uint64_t count, uint64_t cycle_id);

// Dequeue delta from ring at tick slot
// Returns number of entries read (0 if empty)
// Writes to provided buffers (must have capacity ≥ count)
size_t knhk_ring_dequeue_delta(knhk_delta_ring_t *ring, uint64_t tick,
                                uint64_t *S, uint64_t *P, uint64_t *O,
                                uint64_t *cycle_ids, size_t capacity);

// Enqueue assertion + receipt to ring at tick slot
// Non-blocking: uses atomic fetch-and-add
// Returns 0 on success, -1 if ring is full
int knhk_ring_enqueue_assertion(knhk_assertion_ring_t *ring, uint64_t tick,
                                const uint64_t *S, const uint64_t *P, const uint64_t *O,
                                const knhk_receipt_t *receipt, uint64_t count);

// Dequeue assertion + receipt from ring at tick slot
// Returns number of entries read (0 if empty)
size_t knhk_ring_dequeue_assertion(knhk_assertion_ring_t *ring, uint64_t tick,
                                   uint64_t *S, uint64_t *P, uint64_t *O,
                                   knhk_receipt_t *receipts, size_t capacity);

// Mark delta entry as parked (for W1 demotion)
// Single atomic write sets PARKED flag
void knhk_ring_park_delta(knhk_delta_ring_t *ring, uint64_t tick, uint64_t idx);

// Check if ring slot is empty at tick
// Returns 1 if empty, 0 if has data
int knhk_ring_is_empty_delta(const knhk_delta_ring_t *ring, uint64_t tick);
int knhk_ring_is_empty_assertion(const knhk_assertion_ring_t *ring, uint64_t tick);

#endif // KNHK_RING_H

