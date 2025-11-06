// knhk/admission.h
// Admission control for R1 Hot Path
// Enforces guard budget: if data misses L1, park to W1 and keep R1 SLO green
// If R1 cannot meet cache locality, it refuses, not degrades silently

#ifndef KNHK_ADMISSION_H
#define KNHK_ADMISSION_H

#include "types.h"
#include <stdint.h>
#include <stddef.h>

// Admission control result
typedef enum {
  KNHK_ADMIT_R1 = 0,    // Admit to R1 Hot Path (≤2ns)
  KNHK_ADMIT_W1 = 1,    // Park to W1 Warm Path (≤1ms)
  KNHK_ADMIT_C1 = 2,    // Route to C1 Cold Path (≤500ms)
  KNHK_REFUSE = 3       // Refuse: cannot meet cache locality, do not degrade silently
} knhk_admission_result_t;

// Cache locality check result
typedef struct {
  int is_l1_hot;         // 1 if data is L1-hot, 0 otherwise
  int is_l2_hot;         // 1 if data is L2-hot, 0 otherwise
  int is_llc_hot;        // 1 if data is LLC-hot, 0 otherwise
  uint64_t cache_line_addr; // Cache line address for prefetch hint
} knhk_cache_locality_t;

// Guard budget check
typedef struct {
  int can_meet_budget;   // 1 if can meet ≤8 tick budget, 0 otherwise
  uint64_t estimated_ticks; // Estimated ticks for operation
  knhk_admission_result_t admission; // Admission decision
} knhk_guard_budget_t;

// Check cache locality for S/P/O arrays
// Returns cache locality information for admission control
// Performance: Must be fast (measured externally, not in hot path)
static inline knhk_cache_locality_t knhk_check_cache_locality(
    const uint64_t *S,
    const uint64_t *P,
    const uint64_t *O,
    uint64_t run_off,
    uint64_t run_len
) {
  knhk_cache_locality_t loc = {0};
  
  // Check if arrays are 64-byte aligned (L1 cache line size)
  // For R1 admission, we require:
  // 1. Arrays are 64-byte aligned
  // 2. Run length ≤ 8 (fits in single cache line)
  // 3. Data is likely L1-hot (heuristic: recent access pattern)
  
  if (run_len > KNHK_NROWS) {
    // Run length exceeds hot path capacity
    loc.is_l1_hot = 0;
    loc.is_l2_hot = 1; // May be L2-hot
    return loc;
  }
  
  // Check alignment (64-byte aligned arrays are cache-friendly)
  uintptr_t s_addr = (uintptr_t)S;
  uintptr_t p_addr = (uintptr_t)P;
  uintptr_t o_addr = (uintptr_t)O;
  
  int aligned = ((s_addr % 64) == 0) && ((p_addr % 64) == 0) && ((o_addr % 64) == 0);
  
  if (!aligned) {
    // Arrays not aligned - cannot guarantee L1 locality
    loc.is_l1_hot = 0;
    loc.is_l2_hot = 1;
    return loc;
  }
  
  // For R1 admission, assume L1-hot if:
  // - Arrays are aligned
  // - Run length ≤ 8
  // - Data fits in single cache line (8 * 8 bytes = 64 bytes per array)
  loc.is_l1_hot = 1;
  loc.cache_line_addr = s_addr + (run_off * sizeof(uint64_t));
  
  return loc;
}

// Check guard budget for operation
// Returns admission decision based on cache locality and operation complexity
// Performance: Must be fast (measured externally, not in hot path)
static inline knhk_guard_budget_t knhk_check_guard_budget(
    const knhk_context_t *ctx,
    const knhk_hook_ir_t *ir,
    const knhk_cache_locality_t *locality
) {
  knhk_guard_budget_t budget = {0};
  
  // Guard: If R1 cannot meet cache locality, it refuses, not degrades silently
  if (!locality || !locality->is_l1_hot) {
    // Data is not L1-hot - refuse R1 admission
    budget.can_meet_budget = 0;
    budget.admission = KNHK_REFUSE; // Refuse, do not degrade silently
    return budget;
  }
  
  // Check operation complexity
  // Simple operations (ASK, COUNT, COMPARE, VALIDATE) can meet ≤8 tick budget
  // Complex operations (CONSTRUCT8, SELECT) may exceed budget
  
  switch (ir->op) {
    case KNHK_OP_ASK_SP:
    case KNHK_OP_ASK_SPO:
    case KNHK_OP_COUNT_SP_GE:
    case KNHK_OP_COUNT_SP_LE:
    case KNHK_OP_COUNT_SP_EQ:
    case KNHK_OP_COMPARE_O_EQ:
    case KNHK_OP_COMPARE_O_GT:
    case KNHK_OP_COMPARE_O_LT:
    case KNHK_OP_COMPARE_O_GE:
    case KNHK_OP_COMPARE_O_LE:
    case KNHK_OP_VALIDATE_DATATYPE_SP:
    case KNHK_OP_VALIDATE_DATATYPE_SPO:
    case KNHK_OP_UNIQUE_SP:
    case KNHK_OP_COUNT_OP:
    case KNHK_OP_COUNT_OP_LE:
    case KNHK_OP_COUNT_OP_EQ:
      // Simple operations: can meet ≤8 tick budget
      budget.can_meet_budget = 1;
      budget.estimated_ticks = 8; // ≤8 ticks
      budget.admission = KNHK_ADMIT_R1;
      break;
      
    case KNHK_OP_CONSTRUCT8:
      // CONSTRUCT8: may exceed 8-tick budget, route to W1
      budget.can_meet_budget = 0;
      budget.estimated_ticks = 41; // Known: 41-83 ticks
      budget.admission = KNHK_ADMIT_W1;
      break;
      
    case KNHK_OP_SELECT_SP:
      // SELECT: complex operation, route to W1 or C1
      budget.can_meet_budget = 0;
      budget.estimated_ticks = 100; // Estimated
      budget.admission = KNHK_ADMIT_W1;
      break;
      
    default:
      // Unknown operation: refuse
      budget.can_meet_budget = 0;
      budget.admission = KNHK_REFUSE;
      break;
  }
  
  return budget;
}

// Admission control: decide R1/W1/C1 routing
// Returns admission decision based on cache locality and guard budget
// Performance: Must be fast (measured externally, not in hot path)
static inline knhk_admission_result_t knhk_admission_control(
    const knhk_context_t *ctx,
    const knhk_hook_ir_t *ir
) {
  // Check cache locality
  knhk_cache_locality_t locality = knhk_check_cache_locality(
      ctx->S, ctx->P, ctx->O,
      ctx->run.off, ctx->run.len
  );
  
  // Check guard budget
  knhk_guard_budget_t budget = knhk_check_guard_budget(ctx, ir, &locality);
  
  // Admission decision: if data misses L1, park to W1 and keep R1 SLO green
  if (budget.admission == KNHK_ADMIT_R1 && locality.is_l1_hot) {
    return KNHK_ADMIT_R1;
  }
  
  if (budget.admission == KNHK_ADMIT_W1 || (!locality.is_l1_hot && locality.is_l2_hot)) {
    return KNHK_ADMIT_W1;
  }
  
  if (budget.admission == KNHK_ADMIT_C1 || (!locality.is_l2_hot && locality.is_llc_hot)) {
    return KNHK_ADMIT_C1;
  }
  
  // Refuse: cannot meet cache locality, do not degrade silently
  return KNHK_REFUSE;
}

#endif // KNHK_ADMISSION_H

