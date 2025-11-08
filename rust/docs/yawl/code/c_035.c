// Branchless dispatch table (32 entries, cache-aligned)
static const PatternFn PATTERN_DISPATCH_TABLE[32] __attribute__((aligned(64))) = {
    NULL,                              // 0: unused
    pattern_sequence_dispatch,         // 1: Sequence
    pattern_parallel_dispatch,         // 2: Parallel Split
    pattern_sync_dispatch,             // 3: Synchronization
    pattern_choice_dispatch,           // 4: Exclusive Choice
    pattern_merge_dispatch,            // 5: Simple Merge
    pattern_multi_dispatch,            // 6: Multi-Choice
    NULL, NULL,                        // 7-8: unused
    pattern_discriminator_dispatch,    // 9: Discriminator
    pattern_cycles_dispatch,           // 10: Arbitrary Cycles
    pattern_implicit_dispatch,         // 11: Implicit Termination
    NULL, NULL, NULL, NULL,            // 12-15: unused
    pattern_deferred_dispatch,         // 16: Deferred Choice
    NULL, NULL, NULL,                  // 17-19: unused
    pattern_timeout_dispatch,          // 20: Timeout
    pattern_cancellation_dispatch,     // 21: Cancellation
    NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, NULL, // 22-31: unused
};

// Branchless dispatch (≤1 tick)
PatternResult knhk_dispatch_pattern(
    PatternType type,
    PatternContext* ctx,
    void* pattern_data,
    uint32_t data_size
) {
    // Direct array indexing - NO BRANCHES!
    PatternFn dispatch_fn = PATTERN_DISPATCH_TABLE[type];

    if (dispatch_fn == NULL) {
        return (PatternResult){.success = false, .error = "Invalid pattern"};
    }

    return dispatch_fn(ctx, pattern_data, data_size);
}