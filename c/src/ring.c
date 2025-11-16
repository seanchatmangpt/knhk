// ring.c
// Ring buffer implementation: Δ-ring (input) and A-ring (output)
// Branchless enqueue/dequeue with atomic operations

#define _DEFAULT_SOURCE
#define _POSIX_C_SOURCE 200112L

#include "knhk/ring.h"
#include "knhk/types.h"
#include <stdlib.h>
#include <string.h>
#include <stdatomic.h>

// Check if size is power-of-2
static int is_power_of_2(uint64_t n)
{
  return n && !(n & (n - 1));
}

// Aligned allocation helper (64-byte alignment)
static void* aligned_alloc_64(size_t size)
{
  void* ptr = NULL;
  if (posix_memalign(&ptr, 64, size) != 0) {
    return NULL;
  }
  return ptr;
}

// Initialize Δ-ring
int knhk_ring_init_delta(knhk_delta_ring_t *ring, uint64_t size)
{
  if (!ring || !is_power_of_2(size) || size < 8) {
    return -1;
  }

  // Allocate SoA arrays with 64-byte alignment
  size_t array_size = size * sizeof(uint64_t);
  
  ring->S = (uint64_t*)aligned_alloc_64(array_size);
  ring->P = (uint64_t*)aligned_alloc_64(array_size);
  ring->O = (uint64_t*)aligned_alloc_64(array_size);
  ring->cycle_ids = (uint64_t*)aligned_alloc_64(array_size);
  ring->flags = (_Atomic(uint64_t)*)aligned_alloc_64(array_size);
  
  if (!ring->S || !ring->P || !ring->O || !ring->cycle_ids || !ring->flags) {
    knhk_ring_cleanup_delta(ring);
    return -1;
  }

  // Initialize arrays to zero
  memset(ring->S, 0, array_size);
  memset(ring->P, 0, array_size);
  memset(ring->O, 0, array_size);
  memset(ring->cycle_ids, 0, array_size);
  memset(ring->flags, 0, array_size);

  ring->size = size;
  ring->size_mask = size - 1;

  // Initialize per-tick indices to 0
  for (int i = 0; i < 8; i++) {
    atomic_init(&ring->write_idx[i], 0);
    atomic_init(&ring->read_idx[i], 0);
  }

  return 0;
}

// Initialize A-ring
int knhk_ring_init_assertion(knhk_assertion_ring_t *ring, uint64_t size)
{
  if (!ring || !is_power_of_2(size) || size < 8) {
    return -1;
  }

  size_t array_size = size * sizeof(uint64_t);
  size_t receipt_size = size * sizeof(knhk_receipt_t);
  
  ring->S = (uint64_t*)aligned_alloc_64(array_size);
  ring->P = (uint64_t*)aligned_alloc_64(array_size);
  ring->O = (uint64_t*)aligned_alloc_64(array_size);
  ring->receipts = (knhk_receipt_t*)aligned_alloc_64(receipt_size);
  
  if (!ring->S || !ring->P || !ring->O || !ring->receipts) {
    knhk_ring_cleanup_assertion(ring);
    return -1;
  }

  memset(ring->S, 0, array_size);
  memset(ring->P, 0, array_size);
  memset(ring->O, 0, array_size);
  memset(ring->receipts, 0, receipt_size);

  ring->size = size;
  ring->size_mask = size - 1;

  for (int i = 0; i < 8; i++) {
    atomic_init(&ring->write_idx[i], 0);
    atomic_init(&ring->read_idx[i], 0);
  }

  return 0;
}

// Cleanup Δ-ring
void knhk_ring_cleanup_delta(knhk_delta_ring_t *ring)
{
  if (!ring) return;
  
  free(ring->S);
  free(ring->P);
  free(ring->O);
  free(ring->cycle_ids);
  free(ring->flags);
  
  ring->S = NULL;
  ring->P = NULL;
  ring->O = NULL;
  ring->cycle_ids = NULL;
  ring->flags = NULL;
  ring->size = 0;
}

// Cleanup A-ring
void knhk_ring_cleanup_assertion(knhk_assertion_ring_t *ring)
{
  if (!ring) return;
  
  free(ring->S);
  free(ring->P);
  free(ring->O);
  free(ring->receipts);
  
  ring->S = NULL;
  ring->P = NULL;
  ring->O = NULL;
  ring->receipts = NULL;
  ring->size = 0;
}

// Enqueue delta to ring at tick slot
int knhk_ring_enqueue_delta(knhk_delta_ring_t *ring, uint64_t tick,
                            const uint64_t *S, const uint64_t *P, const uint64_t *O,
                            uint64_t count, uint64_t cycle_id)
{
  if (!ring || !S || !P || !O || count == 0 || count > KNHK_NROWS || tick >= 8) {
    return -1;
  }

  // Get write index atomically
  uint64_t write_idx = atomic_fetch_add(&ring->write_idx[tick], count);
  uint64_t base_idx = write_idx & ring->size_mask;

  // Check for overflow (simple check: if write_idx + count > read_idx + size)
  uint64_t read_idx = atomic_load(&ring->read_idx[tick]);
  if ((write_idx + count) > (read_idx + ring->size)) {
    // Rollback: subtract count
    atomic_fetch_sub(&ring->write_idx[tick], count);
    return -1; // Ring full
  }

  // Write data (SoA layout)
  for (uint64_t i = 0; i < count; i++) {
    uint64_t idx = (base_idx + i) & ring->size_mask;
    ring->S[idx] = S[i];
    ring->P[idx] = P[i];
    ring->O[idx] = O[i];
    ring->cycle_ids[idx] = cycle_id;
    atomic_store(&ring->flags[idx], KNHK_RING_FLAG_VALID);
  }

  return 0;
}

// Dequeue delta from ring at tick slot
size_t knhk_ring_dequeue_delta(knhk_delta_ring_t *ring, uint64_t tick,
                                uint64_t *S, uint64_t *P, uint64_t *O,
                                uint64_t *cycle_ids, size_t capacity)
{
  if (!ring || !S || !P || !O || !cycle_ids || capacity == 0 || tick >= 8) {
    return 0;
  }

  uint64_t read_idx = atomic_load(&ring->read_idx[tick]);
  uint64_t write_idx = atomic_load(&ring->write_idx[tick]);

  // Check if empty
  if (read_idx >= write_idx) {
    return 0;
  }

  // Calculate available count
  uint64_t available = write_idx - read_idx;
  uint64_t count = (available < capacity) ? available : capacity;
  if (count > KNHK_NROWS) {
    count = KNHK_NROWS; // Limit to max run length
  }

  uint64_t base_idx = read_idx & ring->size_mask;

  // Read data
  for (uint64_t i = 0; i < count; i++) {
    uint64_t idx = (base_idx + i) & ring->size_mask;
    
    // Check if entry is valid
    uint64_t flags = atomic_load(&ring->flags[idx]);
    if (!(flags & KNHK_RING_FLAG_VALID)) {
      break; // Stop at first invalid entry
    }

    S[i] = ring->S[idx];
    P[i] = ring->P[idx];
    O[i] = ring->O[idx];
    cycle_ids[i] = ring->cycle_ids[idx];
    
    // Clear flag
    atomic_store(&ring->flags[idx], 0);
  }

  // Advance read index
  atomic_fetch_add(&ring->read_idx[tick], count);

  return count;
}

// Enqueue assertion + receipt to ring at tick slot
int knhk_ring_enqueue_assertion(knhk_assertion_ring_t *ring, uint64_t tick,
                                const uint64_t *S, const uint64_t *P, const uint64_t *O,
                                const knhk_receipt_t *receipt, uint64_t count)
{
  if (!ring || !S || !P || !O || !receipt || count == 0 || count > KNHK_NROWS || tick >= 8) {
    return -1;
  }

  uint64_t write_idx = atomic_fetch_add(&ring->write_idx[tick], count);
  uint64_t base_idx = write_idx & ring->size_mask;

  uint64_t read_idx = atomic_load(&ring->read_idx[tick]);
  if ((write_idx + count) > (read_idx + ring->size)) {
    atomic_fetch_sub(&ring->write_idx[tick], count);
    return -1; // Ring full
  }

  // Write data + receipt
  for (uint64_t i = 0; i < count; i++) {
    uint64_t idx = (base_idx + i) & ring->size_mask;
    ring->S[idx] = S[i];
    ring->P[idx] = P[i];
    ring->O[idx] = O[i];
    ring->receipts[idx] = *receipt; // Copy receipt
  }

  return 0;
}

// Dequeue assertion + receipt from ring at tick slot
size_t knhk_ring_dequeue_assertion(knhk_assertion_ring_t *ring, uint64_t tick,
                                   uint64_t *S, uint64_t *P, uint64_t *O,
                                   knhk_receipt_t *receipts, size_t capacity)
{
  if (!ring || !S || !P || !O || !receipts || capacity == 0 || tick >= 8) {
    return 0;
  }

  uint64_t read_idx = atomic_load(&ring->read_idx[tick]);
  uint64_t write_idx = atomic_load(&ring->write_idx[tick]);

  if (read_idx >= write_idx) {
    return 0;
  }

  uint64_t available = write_idx - read_idx;
  uint64_t count = (available < capacity) ? available : capacity;
  if (count > KNHK_NROWS) {
    count = KNHK_NROWS;
  }

  uint64_t base_idx = read_idx & ring->size_mask;

  for (uint64_t i = 0; i < count; i++) {
    uint64_t idx = (base_idx + i) & ring->size_mask;
    S[i] = ring->S[idx];
    P[i] = ring->P[idx];
    O[i] = ring->O[idx];
    receipts[i] = ring->receipts[idx];
  }

  atomic_fetch_add(&ring->read_idx[tick], count);
  return count;
}

// Mark delta entry as parked
void knhk_ring_park_delta(knhk_delta_ring_t *ring, uint64_t tick, uint64_t idx)
{
  if (!ring || tick >= 8 || idx >= ring->size) {
    return;
  }

  uint64_t actual_idx = idx & ring->size_mask;
  // Single atomic write sets PARKED flag
  atomic_fetch_or(&ring->flags[actual_idx], KNHK_RING_FLAG_PARKED);
}

// Check if ring slot is empty
int knhk_ring_is_empty_delta(const knhk_delta_ring_t *ring, uint64_t tick)
{
  if (!ring || tick >= 8) {
    return 1;
  }

  uint64_t read_idx = atomic_load(&ring->read_idx[tick]);
  uint64_t write_idx = atomic_load(&ring->write_idx[tick]);
  return (read_idx >= write_idx) ? 1 : 0;
}

int knhk_ring_is_empty_assertion(const knhk_assertion_ring_t *ring, uint64_t tick)
{
  if (!ring || tick >= 8) {
    return 1;
  }

  uint64_t read_idx = atomic_load(&ring->read_idx[tick]);
  uint64_t write_idx = atomic_load(&ring->write_idx[tick]);
  return (read_idx >= write_idx) ? 1 : 0;
}

