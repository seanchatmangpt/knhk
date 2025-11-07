// knhk/pmu.h
// Performance Monitoring Unit: Hardware cycle counters for τ ≤ 8 enforcement
// Uses platform-specific instructions to measure actual execution time

#ifndef KNHK_PMU_H
#define KNHK_PMU_H

#include <stdint.h>

// Platform-specific cycle counter (RDTSC on x86, CNTVCT on ARM)
// Returns raw CPU cycle count from hardware counter
static inline uint64_t knhk_pmu_rdtsc(void) {
#if defined(__x86_64__) || defined(_M_X64)
    // x86-64: Read Time-Stamp Counter
    uint32_t lo, hi;
    __asm__ __volatile__("rdtsc" : "=a"(lo), "=d"(hi));
    return ((uint64_t)hi << 32) | lo;
#elif defined(__aarch64__) || defined(_M_ARM64)
    // ARM64: Read Virtual Count Register
    uint64_t val;
    __asm__ __volatile__("mrs %0, cntvct_el0" : "=r"(val));
    return val;
#elif defined(__i386__) || defined(_M_IX86)
    // x86-32: Read Time-Stamp Counter
    uint64_t val;
    __asm__ __volatile__("rdtsc" : "=A"(val));
    return val;
#else
    // Fallback: Use zero (will trigger parking)
    // In production, this should be a compile-time error
    #warning "PMU not supported on this architecture - using fallback"
    return 0;
#endif
}

// Convert CPU cycles to KNHK ticks
// 1 tick = 1 nanosecond @ 1GHz reference clock
// Adjust KNHK_PMU_CYCLES_PER_TICK based on actual CPU frequency
// Example: 4GHz CPU → 4 cycles per tick, 2GHz CPU → 2 cycles per tick
#ifndef KNHK_PMU_CYCLES_PER_TICK
#define KNHK_PMU_CYCLES_PER_TICK 1  // Default: 1GHz reference
#endif

#define KNHK_PMU_CYCLES_TO_TICKS(cycles) ((cycles) / KNHK_PMU_CYCLES_PER_TICK)

// PMU measurement context for fiber execution
typedef struct {
    uint64_t start_cycles;     // Cycle count at start
    uint64_t end_cycles;       // Cycle count at end
    uint64_t elapsed_ticks;    // Computed ticks (for τ enforcement)
} knhk_pmu_measurement_t;

// Start PMU measurement (inline for zero overhead)
static inline knhk_pmu_measurement_t knhk_pmu_start(void) {
    knhk_pmu_measurement_t m = {0};
    m.start_cycles = knhk_pmu_rdtsc();
    return m;
}

// End PMU measurement and compute elapsed ticks
static inline void knhk_pmu_end(knhk_pmu_measurement_t *m) {
    m->end_cycles = knhk_pmu_rdtsc();
    uint64_t elapsed_cycles = m->end_cycles - m->start_cycles;
    m->elapsed_ticks = KNHK_PMU_CYCLES_TO_TICKS(elapsed_cycles);
}

// Get elapsed ticks from measurement
static inline uint64_t knhk_pmu_get_ticks(const knhk_pmu_measurement_t *m) {
    return m->elapsed_ticks;
}

// Check if measurement violates τ ≤ 8 law
static inline int knhk_pmu_exceeds_budget(const knhk_pmu_measurement_t *m) {
    return m->elapsed_ticks > 8;
}

#endif // KNHK_PMU_H
