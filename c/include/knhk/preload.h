// knhk/preload.h
// Predictive preloading for R1 Hot Path
// Prefetches S/P/O runs into L1 using next-Δ hints and time-windowed heatmaps
// Used to keep hot data resident in L1 and avoid pointer chasing

#ifndef KNHK_PRELOAD_H
#define KNHK_PRELOAD_H

#include "types.h"
#include <stdint.h>
#include <stddef.h>

#if defined(_MSC_VER) && defined(_M_X64)
#include <mmintrin.h>
#endif

// Prefetch hint for next delta
typedef struct {
  uint64_t next_predicate;  // Next predicate to prefetch
  uint64_t next_offset;      // Next offset to prefetch
  uint64_t next_length;     // Next length to prefetch
  uint64_t confidence;      // Confidence score (0-100)
} knhk_prefetch_hint_t;

// Heatmap entry (time-windowed)
typedef struct {
  uint64_t predicate;       // Predicate ID
  uint64_t access_count;    // Access count in time window
  uint64_t last_access;     // Last access timestamp (ticks)
  uint64_t cache_line_addr; // Cache line address
} knhk_heatmap_entry_t;

// Heatmap (fixed size for hot predicates)
#define KNHK_HEATMAP_SIZE 64  // 64 entries (power of 2)

typedef struct {
  knhk_heatmap_entry_t entries[KNHK_HEATMAP_SIZE];
  uint64_t window_size;     // Time window size (ticks)
  uint64_t current_time;    // Current time (ticks)
} knhk_heatmap_t;

// Initialize heatmap
static inline void knhk_heatmap_init(knhk_heatmap_t *heatmap, uint64_t window_size) {
  if (!heatmap) return;
  
  for (size_t i = 0; i < KNHK_HEATMAP_SIZE; i++) {
    heatmap->entries[i].predicate = 0;
    heatmap->entries[i].access_count = 0;
    heatmap->entries[i].last_access = 0;
    heatmap->entries[i].cache_line_addr = 0;
  }
  
  heatmap->window_size = window_size;
  heatmap->current_time = 0;
}

// Update heatmap with access
static inline void knhk_heatmap_update(
    knhk_heatmap_t *heatmap,
    uint64_t predicate,
    uint64_t cache_line_addr,
    uint64_t current_time
) {
  if (!heatmap) return;
  
  heatmap->current_time = current_time;
  
  // Find or create entry for predicate
  size_t idx = predicate % KNHK_HEATMAP_SIZE;
  knhk_heatmap_entry_t *entry = &heatmap->entries[idx];
  
  // Check if entry is for this predicate
  if (entry->predicate == predicate) {
    // Update existing entry
    entry->access_count++;
    entry->last_access = current_time;
    entry->cache_line_addr = cache_line_addr;
  } else if (entry->predicate == 0) {
    // New entry
    entry->predicate = predicate;
    entry->access_count = 1;
    entry->last_access = current_time;
    entry->cache_line_addr = cache_line_addr;
  } else {
    // Collision: find next available slot
    for (size_t i = 1; i < KNHK_HEATMAP_SIZE; i++) {
      size_t probe_idx = (idx + i) % KNHK_HEATMAP_SIZE;
      knhk_heatmap_entry_t *probe_entry = &heatmap->entries[probe_idx];
      
      if (probe_entry->predicate == predicate) {
        // Update existing entry
        probe_entry->access_count++;
        probe_entry->last_access = current_time;
        probe_entry->cache_line_addr = cache_line_addr;
        return;
      } else if (probe_entry->predicate == 0) {
        // New entry
        probe_entry->predicate = predicate;
        probe_entry->access_count = 1;
        probe_entry->last_access = current_time;
        probe_entry->cache_line_addr = cache_line_addr;
        return;
      }
    }
  }
}

// Get prefetch hint from heatmap
// Returns next predicate to prefetch based on access patterns
static inline knhk_prefetch_hint_t knhk_heatmap_get_prefetch_hint(
    const knhk_heatmap_t *heatmap,
    uint64_t current_time
) {
  knhk_prefetch_hint_t hint = {0};
  
  if (!heatmap) return hint;
  
  // Find hottest predicate in time window
  uint64_t max_count = 0;
  uint64_t hottest_predicate = 0;
  
  for (size_t i = 0; i < KNHK_HEATMAP_SIZE; i++) {
    const knhk_heatmap_entry_t *entry = &heatmap->entries[i];
    
    if (entry->predicate == 0) continue;
    
    // Check if entry is within time window
    uint64_t age = current_time - entry->last_access;
    if (age > heatmap->window_size) {
      // Entry is too old, skip
      continue;
    }
    
    // Weight by recency (more recent = higher weight)
    uint64_t weight = entry->access_count * (heatmap->window_size - age);
    
    if (weight > max_count) {
      max_count = weight;
      hottest_predicate = entry->predicate;
    }
  }
  
  if (hottest_predicate != 0) {
    hint.next_predicate = hottest_predicate;
    hint.next_offset = 0; // Assume start of run
    hint.next_length = KNHK_NROWS; // Prefetch full run
    hint.confidence = (max_count * 100) / (heatmap->window_size * 10); // Normalize to 0-100
    if (hint.confidence > 100) hint.confidence = 100;
  }
  
  return hint;
}

// Prefetch cache line (architecture-specific)
// Uses __builtin_prefetch for GCC/Clang
static inline void knhk_prefetch_cache_line(const void *addr, int locality) {
  if (!addr) return;
  
#if defined(__GNUC__) || defined(__clang__)
  // Prefetch for read, with specified locality
  // locality: 0 = no temporal locality, 1 = low, 2 = moderate, 3 = high
  // Use switch to ensure compile-time constant for __builtin_prefetch
  switch (locality) {
    case 0:
      __builtin_prefetch(addr, 0, 0);
      break;
    case 1:
      __builtin_prefetch(addr, 0, 1);
      break;
    case 2:
      __builtin_prefetch(addr, 0, 2);
      break;
    case 3:
      __builtin_prefetch(addr, 0, 3);
      break;
    default:
      __builtin_prefetch(addr, 0, 0);
      break;
  }
#elif defined(_MSC_VER) && defined(_M_X64)
  // MSVC prefetch intrinsic (x64 only)
  _mm_prefetch((const char *)addr, _MM_HINT_T0);
#else
  // No-op for other architectures
  (void)addr;
  (void)locality;
#endif
}

// Predictive preload: prefetch next delta based on heatmap
static inline void knhk_predictive_preload(
    const knhk_heatmap_t *heatmap,
    const uint64_t *S,
    const uint64_t *P,
    const uint64_t *O,
    uint64_t current_time
) {
  if (!heatmap || !S || !P || !O) return;
  
  // Get prefetch hint
  knhk_prefetch_hint_t hint = knhk_heatmap_get_prefetch_hint(heatmap, current_time);
  
  if (hint.next_predicate == 0 || hint.confidence < 50) {
    // Low confidence, skip prefetch
    return;
  }
  
  // Prefetch S/P/O arrays for next predicate
  // Assume arrays are contiguous and aligned
  uint64_t prefetch_offset = hint.next_offset;
  uint64_t prefetch_length = hint.next_length;
  
  if (prefetch_length > KNHK_NROWS) {
    prefetch_length = KNHK_NROWS;
  }
  
  // Prefetch cache lines (64 bytes each)
  for (uint64_t i = 0; i < prefetch_length; i++) {
    const void *s_addr = &S[prefetch_offset + i];
    const void *p_addr = &P[prefetch_offset + i];
    const void *o_addr = &O[prefetch_offset + i];
    
    // Prefetch with high temporal locality (likely to be accessed soon)
    knhk_prefetch_cache_line(s_addr, 3); // High locality
    knhk_prefetch_cache_line(p_addr, 3);
    knhk_prefetch_cache_line(o_addr, 3);
  }
}

#endif // KNHK_PRELOAD_H

