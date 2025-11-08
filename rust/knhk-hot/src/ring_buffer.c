// knhk-hot/src/ring_buffer.c
// Ring buffer implementation with per-tick isolation
// Each tick gets its own segment: tick_offset = tick * (size/8)

#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <assert.h>

// ============================================================================
// KNHK_ASSUME: Compiler hint pattern from simdjson
// Validates at ingress, trusts in hot path
// ============================================================================

#if defined(_MSC_VER)
  #define KNHK_ASSUME(COND) __assume(COND)
#elif defined(__GNUC__) || defined(__clang__)
  #define KNHK_ASSUME(COND) do { if (!(COND)) __builtin_unreachable(); } while (0)
#else
  #define KNHK_ASSUME(COND) assert(COND)
#endif

// Debug mode: use assertions that fire if violated
// Release mode: use compiler hints for optimization
#ifndef NDEBUG
  #define KNHK_DEBUG_ASSERT(COND) assert(COND)
#else
  #define KNHK_DEBUG_ASSERT(COND) KNHK_ASSUME(COND)
#endif

// Match Rust FFI structures
typedef struct {
    uint8_t fiber_id[32];
    uint64_t parent;
    uint64_t cycle_id;
    uint64_t timestamp_ns;
    uint32_t status;
    uint32_t padding;
} Receipt;

typedef struct {
    uint64_t* S;            // Subject array (64B aligned)
    uint64_t* P;            // Predicate array
    uint64_t* O;            // Object array
    uint64_t* cycle_ids;    // Cycle IDs per entry
    uint64_t* flags;        // Entry flags (PARKED, VALID)
    uint64_t size;          // Power-of-2 size
    uint64_t size_mask;     // size - 1
    uint64_t write_idx[8];  // Per-tick write indices
    uint64_t read_idx[8];   // Per-tick read indices
} knhk_delta_ring_t;

typedef struct {
    uint64_t* S;            // Subject array (64B aligned)
    uint64_t* P;            // Predicate array
    uint64_t* O;            // Object array
    Receipt* receipts;      // Receipts array
    uint64_t size;          // Power-of-2 size
    uint64_t size_mask;     // size - 1
    uint64_t write_idx[8];  // Per-tick write indices
    uint64_t read_idx[8];   // Per-tick read indices
} knhk_assertion_ring_t;

// Ring flags
#define KNHK_RING_FLAG_PARKED 0x1
#define KNHK_RING_FLAG_VALID  0x2

// Number of ticks
#define KNHK_NUM_TICKS 8

// ============================================================================
// Helper: Get tick segment offset and size
// ============================================================================

static inline uint64_t get_tick_segment_size(uint64_t ring_size) {
    // Each tick gets 1/8 of the ring
    // Branchless: shift right by 3 (divide by 8)
    return ring_size >> 3;
}

// Hot path: unchecked version (called after validation at ingress)
static inline uint64_t get_tick_offset_unchecked(uint64_t tick, uint64_t ring_size) {
    // Compiler can optimize assuming tick < 8 (validated at ingress)
    KNHK_DEBUG_ASSERT(tick < KNHK_NUM_TICKS);

    // Branchless: multiply + shift (2-3 cycles)
    uint64_t segment_size = ring_size >> 3;  // Divide by 8 (branchless)
    return tick * segment_size;
}

// Public version: validates at ingress
static inline uint64_t get_tick_offset(uint64_t tick, uint64_t ring_size) {
    // Validate once at ingress
    if (tick >= KNHK_NUM_TICKS) {
        return 0;  // Invalid tick, return 0 (caller checks)
    }

    // Trust in hot path
    return get_tick_offset_unchecked(tick, ring_size);
}

// ============================================================================
// Delta Ring Functions
// ============================================================================

// ============================================================================
// Internal unchecked functions (called after validation at ingress)
// ============================================================================

// Unchecked enqueue: assumes all pointers are valid and tick < 8
static inline int knhk_ring_enqueue_delta_unchecked(
    knhk_delta_ring_t* ring,
    uint64_t tick,
    const uint64_t* S,
    const uint64_t* P,
    const uint64_t* O,
    uint64_t count,
    uint64_t cycle_id
) {
    // Validated at ingress - compiler can optimize with these assumptions
    KNHK_DEBUG_ASSERT(ring != NULL);
    KNHK_DEBUG_ASSERT(S != NULL);
    KNHK_DEBUG_ASSERT(P != NULL);
    KNHK_DEBUG_ASSERT(O != NULL);
    KNHK_DEBUG_ASSERT(tick < KNHK_NUM_TICKS);

    if (count == 0) return 0;

    uint64_t tick_offset = get_tick_offset_unchecked(tick, ring->size);
    uint64_t segment_size = get_tick_segment_size(ring->size);
    uint64_t write_pos = ring->write_idx[tick];

    // Check if we have space in this tick's segment
    if (write_pos + count > segment_size) {
        return -1; // Segment full
    }

    // Hot path: write to tick's segment with no validation
    for (uint64_t i = 0; i < count; i++) {
        uint64_t idx = tick_offset + write_pos + i;
        ring->S[idx] = S[i];
        ring->P[idx] = P[i];
        ring->O[idx] = O[i];
        ring->cycle_ids[idx] = cycle_id;
        ring->flags[idx] = KNHK_RING_FLAG_VALID;
    }

    ring->write_idx[tick] += count;
    return 0;
}

// Unchecked dequeue: assumes all pointers are valid and tick < 8
static inline uint64_t knhk_ring_dequeue_delta_unchecked(
    knhk_delta_ring_t* ring,
    uint64_t tick,
    uint64_t* S,
    uint64_t* P,
    uint64_t* O,
    uint64_t* cycle_ids,
    uint64_t capacity
) {
    // Validated at ingress - compiler can optimize with these assumptions
    KNHK_DEBUG_ASSERT(ring != NULL);
    KNHK_DEBUG_ASSERT(S != NULL);
    KNHK_DEBUG_ASSERT(P != NULL);
    KNHK_DEBUG_ASSERT(O != NULL);
    KNHK_DEBUG_ASSERT(cycle_ids != NULL);
    KNHK_DEBUG_ASSERT(tick < KNHK_NUM_TICKS);

    uint64_t tick_offset = get_tick_offset_unchecked(tick, ring->size);
    uint64_t read_pos = ring->read_idx[tick];
    uint64_t write_pos = ring->write_idx[tick];

    // Calculate available entries in this tick's segment
    if (read_pos >= write_pos) {
        return 0; // Nothing to dequeue
    }

    uint64_t available = write_pos - read_pos;
    uint64_t to_read = available < capacity ? available : capacity;

    // Hot path: read from tick's segment with no validation
    for (uint64_t i = 0; i < to_read; i++) {
        uint64_t idx = tick_offset + read_pos + i;

        // Only read valid entries
        if (ring->flags[idx] & KNHK_RING_FLAG_VALID) {
            S[i] = ring->S[idx];
            P[i] = ring->P[idx];
            O[i] = ring->O[idx];
            cycle_ids[i] = ring->cycle_ids[idx];

            // Clear flag after reading
            ring->flags[idx] = 0;
        }
    }

    ring->read_idx[tick] += to_read;
    return to_read;
}

// ============================================================================
// Public API (validates at ingress, calls unchecked versions)
// ============================================================================

int knhk_ring_init_delta(knhk_delta_ring_t* ring, uint64_t size) {
    if (!ring) return -1;

    // Size must be power of 2 and divisible by 8 ticks
    if (size == 0 || (size & (size - 1)) != 0 || size < KNHK_NUM_TICKS) {
        return -1;
    }

    // Allocate 64-byte aligned arrays
    ring->S = aligned_alloc(64, size * sizeof(uint64_t));
    ring->P = aligned_alloc(64, size * sizeof(uint64_t));
    ring->O = aligned_alloc(64, size * sizeof(uint64_t));
    ring->cycle_ids = aligned_alloc(64, size * sizeof(uint64_t));
    ring->flags = aligned_alloc(64, size * sizeof(uint64_t));

    if (!ring->S || !ring->P || !ring->O || !ring->cycle_ids || !ring->flags) {
        free(ring->S);
        free(ring->P);
        free(ring->O);
        free(ring->cycle_ids);
        free(ring->flags);
        return -1;
    }

    ring->size = size;
    ring->size_mask = size - 1;

    // Initialize per-tick indices
    for (int i = 0; i < KNHK_NUM_TICKS; i++) {
        ring->write_idx[i] = 0;
        ring->read_idx[i] = 0;
    }

    // Clear all flags
    memset(ring->flags, 0, size * sizeof(uint64_t));

    return 0;
}

void knhk_ring_cleanup_delta(knhk_delta_ring_t* ring) {
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
}

int knhk_ring_enqueue_delta(
    knhk_delta_ring_t* ring,
    uint64_t tick,
    const uint64_t* S,
    const uint64_t* P,
    const uint64_t* O,
    uint64_t count,
    uint64_t cycle_id
) {
    // Validate ONCE at ingress
    if (!ring || !S || !P || !O || tick >= KNHK_NUM_TICKS) return -1;

    // Call unchecked version (no validation overhead in hot path)
    return knhk_ring_enqueue_delta_unchecked(ring, tick, S, P, O, count, cycle_id);
}

uint64_t knhk_ring_dequeue_delta(
    knhk_delta_ring_t* ring,
    uint64_t tick,
    uint64_t* S,
    uint64_t* P,
    uint64_t* O,
    uint64_t* cycle_ids,
    uint64_t capacity
) {
    // Validate ONCE at ingress
    if (!ring || !S || !P || !O || !cycle_ids || tick >= KNHK_NUM_TICKS) {
        return 0;
    }

    // Call unchecked version (no validation overhead in hot path)
    return knhk_ring_dequeue_delta_unchecked(ring, tick, S, P, O, cycle_ids, capacity);
}

// Unchecked park (internal)
static inline void knhk_ring_park_delta_unchecked(
    knhk_delta_ring_t* ring,
    uint64_t tick,
    uint64_t idx
) {
    KNHK_DEBUG_ASSERT(ring != NULL);
    KNHK_DEBUG_ASSERT(tick < KNHK_NUM_TICKS);

    uint64_t tick_offset = get_tick_offset_unchecked(tick, ring->size);
    uint64_t actual_idx = tick_offset + idx;

    if (actual_idx < ring->size) {
        ring->flags[actual_idx] |= KNHK_RING_FLAG_PARKED;
    }
}

void knhk_ring_park_delta(knhk_delta_ring_t* ring, uint64_t tick, uint64_t idx) {
    // Validate ONCE at ingress
    if (!ring || tick >= KNHK_NUM_TICKS) return;

    // Call unchecked version
    knhk_ring_park_delta_unchecked(ring, tick, idx);
}

int knhk_ring_is_empty_delta(const knhk_delta_ring_t* ring, uint64_t tick) {
    if (!ring || tick >= KNHK_NUM_TICKS) return 1;

    return ring->read_idx[tick] >= ring->write_idx[tick];
}

// ============================================================================
// Assertion Ring Functions
// ============================================================================

// ============================================================================
// Internal unchecked functions for assertion ring
// ============================================================================

// Unchecked enqueue: assumes all pointers are valid and tick < 8
static inline int knhk_ring_enqueue_assertion_unchecked(
    knhk_assertion_ring_t* ring,
    uint64_t tick,
    const uint64_t* S,
    const uint64_t* P,
    const uint64_t* O,
    const Receipt* receipt,
    uint64_t count
) {
    // Validated at ingress - compiler can optimize with these assumptions
    KNHK_DEBUG_ASSERT(ring != NULL);
    KNHK_DEBUG_ASSERT(S != NULL);
    KNHK_DEBUG_ASSERT(P != NULL);
    KNHK_DEBUG_ASSERT(O != NULL);
    KNHK_DEBUG_ASSERT(receipt != NULL);
    KNHK_DEBUG_ASSERT(tick < KNHK_NUM_TICKS);

    if (count == 0) return 0;

    uint64_t tick_offset = get_tick_offset_unchecked(tick, ring->size);
    uint64_t segment_size = get_tick_segment_size(ring->size);
    uint64_t write_pos = ring->write_idx[tick];

    // Check if we have space in this tick's segment
    if (write_pos + count > segment_size) {
        return -1; // Segment full
    }

    // Hot path: write to tick's segment with no validation
    for (uint64_t i = 0; i < count; i++) {
        uint64_t idx = tick_offset + write_pos + i;
        ring->S[idx] = S[i];
        ring->P[idx] = P[i];
        ring->O[idx] = O[i];
        ring->receipts[idx] = *receipt;
    }

    ring->write_idx[tick] += count;
    return 0;
}

// Unchecked dequeue: assumes all pointers are valid and tick < 8
static inline uint64_t knhk_ring_dequeue_assertion_unchecked(
    knhk_assertion_ring_t* ring,
    uint64_t tick,
    uint64_t* S,
    uint64_t* P,
    uint64_t* O,
    Receipt* receipts,
    uint64_t capacity
) {
    // Validated at ingress - compiler can optimize with these assumptions
    KNHK_DEBUG_ASSERT(ring != NULL);
    KNHK_DEBUG_ASSERT(S != NULL);
    KNHK_DEBUG_ASSERT(P != NULL);
    KNHK_DEBUG_ASSERT(O != NULL);
    KNHK_DEBUG_ASSERT(receipts != NULL);
    KNHK_DEBUG_ASSERT(tick < KNHK_NUM_TICKS);

    uint64_t tick_offset = get_tick_offset_unchecked(tick, ring->size);
    uint64_t read_pos = ring->read_idx[tick];
    uint64_t write_pos = ring->write_idx[tick];

    // Calculate available entries in this tick's segment
    if (read_pos >= write_pos) {
        return 0; // Nothing to dequeue
    }

    uint64_t available = write_pos - read_pos;
    uint64_t to_read = available < capacity ? available : capacity;

    // Hot path: read from tick's segment with no validation
    for (uint64_t i = 0; i < to_read; i++) {
        uint64_t idx = tick_offset + read_pos + i;
        S[i] = ring->S[idx];
        P[i] = ring->P[idx];
        O[i] = ring->O[idx];
        receipts[i] = ring->receipts[idx];
    }

    ring->read_idx[tick] += to_read;
    return to_read;
}

// ============================================================================
// Public API for assertion ring (validates at ingress)
// ============================================================================

int knhk_ring_init_assertion(knhk_assertion_ring_t* ring, uint64_t size) {
    if (!ring) return -1;

    // Size must be power of 2 and divisible by 8 ticks
    if (size == 0 || (size & (size - 1)) != 0 || size < KNHK_NUM_TICKS) {
        return -1;
    }

    // Allocate 64-byte aligned arrays
    ring->S = aligned_alloc(64, size * sizeof(uint64_t));
    ring->P = aligned_alloc(64, size * sizeof(uint64_t));
    ring->O = aligned_alloc(64, size * sizeof(uint64_t));
    ring->receipts = aligned_alloc(64, size * sizeof(Receipt));

    if (!ring->S || !ring->P || !ring->O || !ring->receipts) {
        free(ring->S);
        free(ring->P);
        free(ring->O);
        free(ring->receipts);
        return -1;
    }

    ring->size = size;
    ring->size_mask = size - 1;

    // Initialize per-tick indices
    for (int i = 0; i < KNHK_NUM_TICKS; i++) {
        ring->write_idx[i] = 0;
        ring->read_idx[i] = 0;
    }

    return 0;
}

void knhk_ring_cleanup_assertion(knhk_assertion_ring_t* ring) {
    if (!ring) return;

    free(ring->S);
    free(ring->P);
    free(ring->O);
    free(ring->receipts);

    ring->S = NULL;
    ring->P = NULL;
    ring->O = NULL;
    ring->receipts = NULL;
}

int knhk_ring_enqueue_assertion(
    knhk_assertion_ring_t* ring,
    uint64_t tick,
    const uint64_t* S,
    const uint64_t* P,
    const uint64_t* O,
    const Receipt* receipt,
    uint64_t count
) {
    // Validate ONCE at ingress
    if (!ring || !S || !P || !O || !receipt || tick >= KNHK_NUM_TICKS) {
        return -1;
    }

    // Call unchecked version (no validation overhead in hot path)
    return knhk_ring_enqueue_assertion_unchecked(ring, tick, S, P, O, receipt, count);
}

uint64_t knhk_ring_dequeue_assertion(
    knhk_assertion_ring_t* ring,
    uint64_t tick,
    uint64_t* S,
    uint64_t* P,
    uint64_t* O,
    Receipt* receipts,
    uint64_t capacity
) {
    // Validate ONCE at ingress
    if (!ring || !S || !P || !O || !receipts || tick >= KNHK_NUM_TICKS) {
        return 0;
    }

    // Call unchecked version (no validation overhead in hot path)
    return knhk_ring_dequeue_assertion_unchecked(ring, tick, S, P, O, receipts, capacity);
}

int knhk_ring_is_empty_assertion(const knhk_assertion_ring_t* ring, uint64_t tick) {
    if (!ring || tick >= KNHK_NUM_TICKS) return 1;

    return ring->read_idx[tick] >= ring->write_idx[tick];
}
