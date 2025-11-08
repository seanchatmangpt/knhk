static inline uint64_t get_tick_offset_unchecked(uint64_t tick, uint64_t ring_size) {
    KNHK_DEBUG_ASSERT(tick < KNHK_NUM_TICKS);
    uint64_t segment_size = ring_size >> 3;
    return tick * segment_size;
}