// Unchecked internal function (hot path)
static inline int knhk_ring_enqueue_delta_unchecked(
    knhk_delta_ring_t* ring,
    uint64_t tick,
    // ...
) {
    // Compiler can optimize assuming these are true
    KNHK_DEBUG_ASSERT(ring != NULL);
    KNHK_DEBUG_ASSERT(tick < KNHK_NUM_TICKS);

    // No runtime checks in release mode
    uint64_t tick_offset = get_tick_offset_unchecked(tick, ring->size);
    // ... fast path
}

// Public API validates once at ingress
int knhk_ring_enqueue_delta(
    knhk_delta_ring_t* ring,
    uint64_t tick,
    // ...
) {
    // Validate ONCE at ingress
    if (ring == NULL) return -1;
    if (tick >= KNHK_NUM_TICKS) return -1;

    // Call unchecked version (no validation overhead)
    return knhk_ring_enqueue_delta_unchecked(ring, tick, ...);
}