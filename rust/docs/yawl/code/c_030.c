uint64_t get_tick_offset(uint64_t tick, uint64_t ring_size) {
    if (tick >= KNHK_NUM_TICKS) return 0;  // BRANCH!
    return tick * (ring_size / KNHK_NUM_TICKS);  // DIVISION!
}