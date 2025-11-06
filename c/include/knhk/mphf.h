// knhk/mphf.h
// Minimal Perfect Hash Function (MPHF) cache
// O(1) lookups without collisions for hot predicates and IDs
// Used for predicate and key resolution in warm path

#ifndef KNHK_MPHF_H
#define KNHK_MPHF_H

#include "types.h"
#include <stdint.h>
#include <stddef.h>

// MPHF cache entry
typedef struct {
  uint64_t key;        // Predicate or ID key
  uint64_t hash;       // MPHF hash value
  uint64_t value;      // Cached value (predicate run offset, etc.)
  uint8_t valid;       // 1 if entry is valid, 0 otherwise
} knhk_mphf_entry_t;

// MPHF cache (fixed size for hot predicates)
#define KNHK_MPHF_CACHE_SIZE 256  // 256 entries (power of 2 for fast modulo)

typedef struct {
  knhk_mphf_entry_t entries[KNHK_MPHF_CACHE_SIZE];
  uint64_t size;       // Number of valid entries
  uint64_t seed;       // MPHF seed for hash function
} knhk_mphf_cache_t;

// Initialize MPHF cache
static inline void knhk_mphf_init(knhk_mphf_cache_t *cache, uint64_t seed) {
  if (!cache) return;
  
  for (size_t i = 0; i < KNHK_MPHF_CACHE_SIZE; i++) {
    cache->entries[i].key = 0;
    cache->entries[i].hash = 0;
    cache->entries[i].value = 0;
    cache->entries[i].valid = 0;
  }
  
  cache->size = 0;
  cache->seed = seed;
}

// FNV-1a hash function (used for MPHF)
static inline uint64_t knhk_mphf_hash(uint64_t key, uint64_t seed) {
  const uint64_t FNV_OFFSET_BASIS = 14695981039346656037ULL;
  const uint64_t FNV_PRIME = 1099511628211ULL;
  
  uint64_t hash = FNV_OFFSET_BASIS ^ seed;
  
  // Hash the key (8 bytes)
  uint8_t *bytes = (uint8_t *)&key;
  for (size_t i = 0; i < 8; i++) {
    hash ^= bytes[i];
    hash *= FNV_PRIME;
  }
  
  return hash;
}

// Lookup in MPHF cache (O(1))
// Returns 1 if found, 0 otherwise
// Sets *value to cached value if found
static inline int knhk_mphf_lookup(
    const knhk_mphf_cache_t *cache,
    uint64_t key,
    uint64_t *value
) {
  if (!cache || !value) return 0;
  
  // Compute MPHF hash
  uint64_t hash = knhk_mphf_hash(key, cache->seed);
  
  // Modulo for cache index
  size_t idx = hash % KNHK_MPHF_CACHE_SIZE;
  
  // Check if entry is valid and key matches
  const knhk_mphf_entry_t *entry = &cache->entries[idx];
  if (entry->valid && entry->key == key) {
    *value = entry->value;
    return 1;
  }
  
  return 0;
}

// Insert into MPHF cache (O(1))
// Returns 1 if inserted, 0 if cache is full
static inline int knhk_mphf_insert(
    knhk_mphf_cache_t *cache,
    uint64_t key,
    uint64_t value
) {
  if (!cache) return 0;
  
  if (cache->size >= KNHK_MPHF_CACHE_SIZE) {
    // Cache is full
    return 0;
  }
  
  // Compute MPHF hash
  uint64_t hash = knhk_mphf_hash(key, cache->seed);
  
  // Modulo for cache index
  size_t idx = hash % KNHK_MPHF_CACHE_SIZE;
  
  // Insert entry
  knhk_mphf_entry_t *entry = &cache->entries[idx];
  if (!entry->valid) {
    // Slot is empty - insert
    entry->key = key;
    entry->hash = hash;
    entry->value = value;
    entry->valid = 1;
    cache->size++;
    return 1;
  }
  
  // Collision: MPHF requires perfect hash, but we handle collisions by chaining
  // Note: Perfect hash (CHD algorithm) planned for v1.0
  // For now, use linear probing
  for (size_t i = 1; i < KNHK_MPHF_CACHE_SIZE; i++) {
    size_t probe_idx = (idx + i) % KNHK_MPHF_CACHE_SIZE;
    knhk_mphf_entry_t *probe_entry = &cache->entries[probe_idx];
    
    if (!probe_entry->valid) {
      // Found empty slot
      probe_entry->key = key;
      probe_entry->hash = hash;
      probe_entry->value = value;
      probe_entry->valid = 1;
      cache->size++;
      return 1;
    }
  }
  
  // No empty slot found
  return 0;
}

#endif // KNHK_MPHF_H

