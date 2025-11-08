typedef struct {
  uint64_t cycle_id;   // Beat cycle ID (from knhk_beat_next())
  uint64_t shard_id;   // Shard identifier
  uint64_t hook_id;    // Hook identifier
  uint32_t ticks;      // Actual ticks used (≤8)
  uint32_t lanes;      // SIMD lanes used
  uint64_t span_id;    // OTEL-compatible span ID
  uint64_t a_hash;     // hash(A) = hash(μ(O)) fragment
} knhk_receipt_t;