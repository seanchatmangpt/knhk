// knhk/kernels.h
// Kernel dispatch table for μ(Δ) reconciliation
// Branchless function pointer dispatch for hot path execution

#ifndef KNHK_KERNELS_H
#define KNHK_KERNELS_H

#include "types.h"
#include <stddef.h>

// Kernel type enumeration (maps to knhk_op_t subset for hot path)
typedef enum {
    KNHK_KERNEL_ASK_SP = 0,       // ASK(S,P) - hot path
    KNHK_KERNEL_COUNT_SP_GE = 1,  // COUNT(S,P) >= k
    KNHK_KERNEL_ASK_SPO = 2,      // ASK(S,P,O) - exact match
    KNHK_KERNEL_VALIDATE_SP = 3,  // VALIDATE(S,P) - datatype validation
    KNHK_KERNEL_UNIQUE_SP = 4,    // UNIQUE(S,P) - single value check
    KNHK_KERNEL_COMPARE_O = 5,    // COMPARE O with value
    KNHK_KERNEL_MAX = 6           // Number of kernel types
} knhk_kernel_type_t;

// Kernel dispatch function signature
// Returns: CPU cycles consumed (for tick budget tracking)
// Parameters:
//   - s_lane: Subject array (aligned, ≤8 elements)
//   - p_lane: Predicate array (aligned, ≤8 elements)
//   - o_lane: Object array (aligned, ≤8 elements)
//   - n_rows: Number of rows (≤8, validated by guard)
//   - out_mask: Output bitmask (1 bit per validated row)
typedef uint64_t (*knhk_kernel_fn_t)(
    const uint64_t *s_lane,
    const uint64_t *p_lane,
    const uint64_t *o_lane,
    size_t n_rows,
    uint64_t *out_mask
);

// Kernel dispatch table entry
typedef struct {
    knhk_kernel_type_t type;
    knhk_kernel_fn_t execute;
} knhk_kernel_dispatch_t;

// Get dispatch table (const, cache-friendly)
// Returns: Pointer to static dispatch table
const knhk_kernel_dispatch_t* knhk_get_kernel_dispatch_table(void);

// Kernel implementations (forward declarations)
// These map to existing eval_dispatch.c implementations

// ASK(S,P) kernel - check if (s,p) exists
uint64_t knhk_kernel_ask_sp_impl(
    const uint64_t *s_lane,
    const uint64_t *p_lane,
    const uint64_t *o_lane,
    size_t n_rows,
    uint64_t *out_mask
);

// COUNT(S,P) >= k kernel
uint64_t knhk_kernel_count_sp_ge_impl(
    const uint64_t *s_lane,
    const uint64_t *p_lane,
    const uint64_t *o_lane,
    size_t n_rows,
    uint64_t *out_mask
);

// ASK(S,P,O) kernel - exact triple match
uint64_t knhk_kernel_ask_spo_impl(
    const uint64_t *s_lane,
    const uint64_t *p_lane,
    const uint64_t *o_lane,
    size_t n_rows,
    uint64_t *out_mask
);

// VALIDATE(S,P) datatype kernel
uint64_t knhk_kernel_validate_sp_impl(
    const uint64_t *s_lane,
    const uint64_t *p_lane,
    const uint64_t *o_lane,
    size_t n_rows,
    uint64_t *out_mask
);

// UNIQUE(S,P) kernel - verify single value
uint64_t knhk_kernel_unique_sp_impl(
    const uint64_t *s_lane,
    const uint64_t *p_lane,
    const uint64_t *o_lane,
    size_t n_rows,
    uint64_t *out_mask
);

// COMPARE O kernel - compare object values
uint64_t knhk_kernel_compare_o_impl(
    const uint64_t *s_lane,
    const uint64_t *p_lane,
    const uint64_t *o_lane,
    size_t n_rows,
    uint64_t *out_mask
);

// Dispatch helper: branchless kernel selection
// Returns: Kernel function pointer (or noop if invalid type)
#ifndef KNHK_SELECT_KERNEL_INLINE
#define KNHK_SELECT_KERNEL_INLINE static inline
#endif
KNHK_SELECT_KERNEL_INLINE knhk_kernel_fn_t knhk_select_kernel(knhk_kernel_type_t type) {
    const knhk_kernel_dispatch_t* table = knhk_get_kernel_dispatch_table();

    // Branchless bounds check: mask out invalid types
    uint64_t valid = (type < KNHK_KERNEL_MAX) ? UINT64_MAX : 0;
    size_t idx = ((size_t)type) & (size_t)valid;

    return table[idx].execute;
}

#endif // KNHK_KERNELS_H
