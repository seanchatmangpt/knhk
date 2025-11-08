// Branchless tick offset calculation
static inline uint64_t get_tick_segment_size(uint64_t ring_size) {
    // Each tick gets 1/8 of the ring
    // Branchless: shift right by 3 (divide by 8)
    return ring_size >> 3;  // NO BRANCHES!
}

static inline uint64_t get_tick_offset_unchecked(uint64_t tick, uint64_t ring_size) {
    KNHK_DEBUG_ASSERT(tick < KNHK_NUM_TICKS);

    // Branchless: multiply + shift (2-3 cycles)
    uint64_t segment_size = ring_size >> 3;  // Divide by 8 (branchless)
    return tick * segment_size;  // Multiply (branchless)
}