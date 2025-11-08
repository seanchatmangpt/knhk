// knhk-hot/src/workflow_patterns.h
// Van der Aalst Critical 8 Workflow Patterns
// Zero-overhead implementation: Guards at ingress, trust in hot path

#ifndef KNHK_WORKFLOW_PATTERNS_H
#define KNHK_WORKFLOW_PATTERNS_H

#include <stdint.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// ============================================================================
// Pattern Types (8 Critical Patterns for 85% Coverage)
// ============================================================================

typedef enum {
    PATTERN_SEQUENCE = 1,          // Pattern 1: Sequential execution
    PATTERN_PARALLEL_SPLIT = 2,    // Pattern 2: AND-split (SIMD-capable)
    PATTERN_SYNCHRONIZATION = 3,   // Pattern 3: AND-join (SIMD-capable)
    PATTERN_EXCLUSIVE_CHOICE = 4,  // Pattern 4: XOR-split
    PATTERN_SIMPLE_MERGE = 5,      // Pattern 5: XOR-join
    PATTERN_MULTI_CHOICE = 6,      // Pattern 6: OR-split (SIMD-capable)
    PATTERN_ARBITRARY_CYCLES = 10, // Pattern 10: Retry/loop
    PATTERN_DEFERRED_CHOICE = 16,  // Pattern 16: Event-driven choice
} PatternType;

// ============================================================================
// Core Data Structures (Cache-Aligned for Performance)
// ============================================================================

// Pattern context (input/output data)
typedef struct {
    uint64_t* data;      // Generic data array
    uint32_t len;        // Data length
    uint64_t metadata;   // Pattern-specific metadata
} PatternContext;

// Pattern result
typedef struct {
    bool success;        // Execution succeeded
    uint32_t branches;   // Number of branches executed
    uint64_t result;     // Pattern-specific result
    const char* error;   // Error message (NULL if success)
} PatternResult;

// Branch function pointer (for parallel/choice patterns)
typedef bool (*BranchFn)(PatternContext* ctx);

// Condition function pointer (for choice patterns)
typedef bool (*ConditionFn)(const PatternContext* ctx);

// ============================================================================
// Pattern 1: Sequence (1 tick)
// Execute tasks in strict order: A → B → C
// ============================================================================

PatternResult knhk_pattern_sequence(
    PatternContext* ctx,
    BranchFn* branches,
    uint32_t num_branches
);

// ============================================================================
// Pattern 2: Parallel Split (2 ticks, SIMD-capable)
// Execute ALL branches concurrently (AND-split)
// ============================================================================

PatternResult knhk_pattern_parallel_split(
    PatternContext* ctx,
    BranchFn* branches,
    uint32_t num_branches
);

// SIMD-optimized version (ARM64 NEON, 4 branches in parallel)
PatternResult knhk_pattern_parallel_split_simd(
    PatternContext* ctx,
    BranchFn* branches,
    uint32_t num_branches
);

// ============================================================================
// Pattern 3: Synchronization (3 ticks, SIMD-capable)
// Wait for ALL branches to complete (AND-join)
// ============================================================================

PatternResult knhk_pattern_synchronization(
    PatternContext* ctx,
    uint64_t* branch_results,
    uint32_t num_branches
);

// SIMD-optimized version (vectorized result checking)
PatternResult knhk_pattern_synchronization_simd(
    PatternContext* ctx,
    uint64_t* branch_results,
    uint32_t num_branches
);

// ============================================================================
// Pattern 4: Exclusive Choice (2 ticks)
// Choose ONE branch based on condition (XOR-split)
// ============================================================================

PatternResult knhk_pattern_exclusive_choice(
    PatternContext* ctx,
    ConditionFn* conditions,
    BranchFn* branches,
    uint32_t num_branches
);

// ============================================================================
// Pattern 5: Simple Merge (1 tick)
// Continue after ANY branch completes (XOR-join)
// ============================================================================

PatternResult knhk_pattern_simple_merge(
    PatternContext* ctx,
    uint64_t branch_result
);

// ============================================================================
// Pattern 6: Multi-Choice (3 ticks, SIMD-capable)
// Execute 1+ branches based on conditions (OR-split)
// ============================================================================

PatternResult knhk_pattern_multi_choice(
    PatternContext* ctx,
    ConditionFn* conditions,
    BranchFn* branches,
    uint32_t num_branches
);

// SIMD-optimized version (vectorized condition evaluation)
PatternResult knhk_pattern_multi_choice_simd(
    PatternContext* ctx,
    ConditionFn* conditions,
    BranchFn* branches,
    uint32_t num_branches
);

// ============================================================================
// Pattern 10: Arbitrary Cycles (2 ticks)
// Retry branch until condition met or max attempts reached
// ============================================================================

PatternResult knhk_pattern_arbitrary_cycles(
    PatternContext* ctx,
    BranchFn branch,
    ConditionFn should_continue,
    uint32_t max_iterations
);

// ============================================================================
// Pattern 16: Deferred Choice (3 ticks)
// Wait for first event/condition, then execute corresponding branch
// ============================================================================

PatternResult knhk_pattern_deferred_choice(
    PatternContext* ctx,
    ConditionFn* conditions,
    BranchFn* branches,
    uint32_t num_branches,
    uint64_t timeout_ticks
);

// ============================================================================
// Branchless Pattern Dispatch (≤1 tick)
// Function pointer table for zero-overhead pattern selection
// ============================================================================

typedef PatternResult (*PatternFn)(PatternContext*, void*, uint32_t);

// Dispatch pattern with branchless lookup
PatternResult knhk_dispatch_pattern(
    PatternType type,
    PatternContext* ctx,
    void* pattern_data,
    uint32_t data_size
);

// ============================================================================
// Helper Functions
// ============================================================================

// Create pattern context (ingress validation here)
PatternContext* knhk_pattern_context_create(uint32_t capacity);

// Destroy pattern context
void knhk_pattern_context_destroy(PatternContext* ctx);

// Add data to context
bool knhk_pattern_context_add(PatternContext* ctx, uint64_t data);

// Get pattern name (for telemetry)
const char* knhk_pattern_name(PatternType type);

// Get pattern tick budget (for ingress validation)
uint32_t knhk_pattern_tick_budget(PatternType type);

// Validate pattern at ingress (guards enforce constraints ONCE)
bool knhk_pattern_validate_ingress(
    PatternType type,
    uint32_t num_branches,
    const char** error_msg
);

#ifdef __cplusplus
}
#endif

#endif // KNHK_WORKFLOW_PATTERNS_H
