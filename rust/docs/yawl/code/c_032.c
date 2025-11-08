// Branchless dispatch table (32 entries, aligned to cache line)
static const PatternFn PATTERN_DISPATCH_TABLE[32] __attribute__((aligned(64))) = {
    NULL,                        // 0: unused
    pattern_sequence_dispatch,   // 1: Sequence
    pattern_parallel_dispatch,   // 2: Parallel Split
    // ... all 32 entries in ONE cache line
};

// Pattern metadata (cache-aligned)
static const PatternMetadata PATTERN_METADATA[22] __attribute__((aligned(64))) = {
    {"Invalid", 0, false},
    {"Sequence", 1, false},
    {"Parallel Split", 2, true},
    // ... metadata in ONE cache line
};