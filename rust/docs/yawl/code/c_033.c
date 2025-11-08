// 64-byte aligned allocation
ring->S = aligned_alloc(64, ring_size * sizeof(uint64_t));
ring->P = aligned_alloc(64, ring_size * sizeof(uint64_t));
ring->O = aligned_alloc(64, ring_size * sizeof(uint64_t));