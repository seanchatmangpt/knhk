// beat.c
// 8-Beat Epoch Scheduler implementation
// Branchless cycle/tick/pulse generation for deterministic Λ ordering

#include "knhk/beat.h"
#include <stdatomic.h>

// Global cycle counter (initialized to 0)
_Atomic(uint64_t) knhk_global_cycle = ATOMIC_VAR_INIT(0);

// Initialize beat scheduler
// Called once at startup to ensure counter starts at 0
void knhk_beat_init(void)
{
  atomic_store(&knhk_global_cycle, 0);
}

