// knhk/beat.h
// 8-Beat Epoch Scheduler: branchless cycle/tick/pulse generation
// Law: μ ⊂ τ (τ=8), Λ total order, branchless cadence

#ifndef KNHK_BEAT_H
#define KNHK_BEAT_H

#include <stdint.h>
#include <stdatomic.h>

// Global cycle counter (shared across all threads/pods)
extern _Atomic(uint64_t) knhk_global_cycle;

// Initialize beat scheduler (call once at startup)
void knhk_beat_init(void);

// Advance cycle counter atomically, return old cycle value then increment
// Branchless: single atomic operation
static inline uint64_t knhk_beat_next(void)
{
  return atomic_fetch_add(&knhk_global_cycle, 1);
}

// Extract tick from cycle (0..7)
// Branchless: bitwise mask operation
static inline uint64_t knhk_beat_tick(uint64_t cycle)
{
  return cycle & 0x7ULL;
}

// Compute pulse signal (1 when tick==0, else 0)
// Branchless: mask-based, no conditional branches
// Pulse indicates wrap boundary for commit operations
static inline uint64_t knhk_beat_pulse(uint64_t cycle)
{
  uint64_t tick = cycle & 0x7ULL;
  // Branchless: return 1 if tick==0, else 0
  // Use arithmetic underflow: when tick==0, (tick - 1) wraps to 0xFF...
  // Right-shift by 63 gives 1 when tick==0, else 0
  return ((tick - 1ULL) >> 63ULL) & 1ULL;
}

// Get current cycle without incrementing
static inline uint64_t knhk_beat_current(void)
{
  return atomic_load(&knhk_global_cycle);
}

#endif // KNHK_BEAT_H

