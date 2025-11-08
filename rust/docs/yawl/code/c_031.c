uint64_t get_tick_offset_unchecked(uint64_t tick, uint64_t ring_size) {
    KNHK_ASSUME(tick < KNHK_NUM_TICKS);  // Compiler hint
    return tick * (ring_size >> 3);  // Shift instead of division
}