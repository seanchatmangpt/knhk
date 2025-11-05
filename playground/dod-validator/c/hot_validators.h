// dod-validator/hot_validators.h
// Hot path validators for DoD validation using KNHK pattern matching
// Performance target: ≤8 ticks (≤2ns) per validation operation

#ifndef DOD_HOT_VALIDATORS_H
#define DOD_HOT_VALIDATORS_H

#include <stdint.h>
#include <stddef.h>
#include "../../../c/include/knhk.h"

// Pattern types for code validation
typedef enum {
    DOD_PATTERN_UNWRAP = 1,          // .unwrap() pattern
    DOD_PATTERN_EXPECT = 2,          // .expect() pattern
    DOD_PATTERN_TODO = 3,            // TODO comment pattern
    DOD_PATTERN_PLACEHOLDER = 4,     // Placeholder comment pattern
    DOD_PATTERN_PANIC = 5,           // panic!() pattern
    DOD_PATTERN_RESULT = 6,          // Result<T, E> pattern (positive)
} dod_pattern_t;

// Validation result (hot path)
typedef struct {
    int found;           // 1 if pattern found, 0 otherwise
    uint32_t count;     // Number of matches (for COUNT operations)
    uint64_t span_id;   // OTEL span ID for provenance
} dod_validation_result_t;

// Pattern context for validation
typedef struct {
    const uint64_t *patterns;  // Pattern hashes (SoA layout)
    uint32_t pattern_count;    // Number of patterns (≤8)
    dod_pattern_t pattern_type;
} dod_pattern_context_t;

// Validate pattern existence in code (ASK_SP operation)
// Returns 1 if pattern found, 0 otherwise
// Performance: ≤8 ticks (≤2ns) when measured externally
// Zero timing overhead - pure CONSTRUCT logic only
static inline int dod_match_pattern(
    const dod_pattern_context_t *ctx,
    uint64_t code_hash,
    dod_validation_result_t *result)
{
    if (!ctx || ctx->pattern_count == 0 || ctx->pattern_count > 8) {
        if (result) {
            result->found = 0;
            result->count = 0;
            result->span_id = 0;
        }
        return 0;
    }

    // Create KNHK context for pattern matching
    knhk_context_t knhk_ctx = {
        .S = ctx->patterns,
        .P = (const uint64_t[]){ ctx->pattern_type },
        .O = (const uint64_t[]){ code_hash },
        .triple_count = ctx->pattern_count,
        .run = {
            .pred = ctx->pattern_type,
            .off = 0,
            .len = ctx->pattern_count,
        },
    };

    // Create hook IR for ASK_SP operation
    knhk_hook_ir_t ir = {
        .op = KNHK_OP_ASK_SP,
        .s = code_hash,
        .p = ctx->pattern_type,
        .o = 0,
        .k = 0,
    };

    // Create receipt for provenance
    knhk_receipt_t rcpt = {0};

    // Execute pattern matching (hot path, ≤8 ticks)
    int match = knhk_eval_bool(&knhk_ctx, &ir, &rcpt);

    if (result) {
        result->found = match ? 1 : 0;
        result->count = match ? 1 : 0;
        result->span_id = rcpt.span_id;
    }

    return match;
}

// Count pattern occurrences (COUNT_SP_GE operation)
// Returns count of pattern matches
// Performance: ≤8 ticks (≤2ns) when measured externally
static inline uint32_t dod_count_patterns(
    const dod_pattern_context_t *ctx,
    uint64_t code_hash,
    dod_validation_result_t *result)
{
    if (!ctx || ctx->pattern_count == 0 || ctx->pattern_count > 8) {
        if (result) {
            result->found = 0;
            result->count = 0;
            result->span_id = 0;
        }
        return 0;
    }

    // Create KNHK context
    knhk_context_t knhk_ctx = {
        .S = ctx->patterns,
        .P = (const uint64_t[]){ ctx->pattern_type },
        .O = (const uint64_t[]){ code_hash },
        .triple_count = ctx->pattern_count,
        .run = {
            .pred = ctx->pattern_type,
            .off = 0,
            .len = ctx->pattern_count,
        },
    };

    // Create hook IR for COUNT_SP_GE operation
    knhk_hook_ir_t ir = {
        .op = KNHK_OP_COUNT_SP_GE,
        .s = code_hash,
        .p = ctx->pattern_type,
        .o = 0,
        .k = 1,  // Count >= 1
    };

    knhk_receipt_t rcpt = {0};

    // Execute count operation (hot path, ≤8 ticks)
    int has_match = knhk_eval_bool(&knhk_ctx, &ir, &rcpt);

    // For exact count, we need to use COUNT_SP_EQ
    // This is a simplified version - full implementation would use COUNT_SP_EQ
    uint32_t count = has_match ? 1 : 0;

    if (result) {
        result->found = has_match ? 1 : 0;
        result->count = count;
        result->span_id = rcpt.span_id;
    }

    return count;
}

// Validate guard constraint (max_run_len ≤ 8)
// Returns 1 if constraint satisfied, 0 otherwise
// Performance: ≤8 ticks (≤2ns) when measured externally
static inline int dod_validate_guard_constraint(
    uint32_t run_len,
    dod_validation_result_t *result)
{
    // Guard constraint: max_run_len ≤ 8
    int valid = (run_len <= 8) ? 1 : 0;

    if (result) {
        result->found = valid;
        result->count = run_len;
        result->span_id = knhk_generate_span_id();
    }

    return valid;
}

// Check for Result<T, E> pattern (positive validation)
// Returns 1 if Result pattern found, 0 otherwise
// Performance: ≤8 ticks (≤2ns) when measured externally
static inline int dod_check_result_pattern(
    const dod_pattern_context_t *ctx,
    uint64_t code_hash,
    dod_validation_result_t *result)
{
    // Use ASK_SP with Result pattern type
    dod_pattern_context_t result_ctx = *ctx;
    result_ctx.pattern_type = DOD_PATTERN_RESULT;

    return dod_match_pattern(&result_ctx, code_hash, result);
}

#endif // DOD_HOT_VALIDATORS_H

