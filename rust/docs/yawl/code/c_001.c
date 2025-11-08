// Compile-time specialization
typedef struct {
  uint64_t s_const;      // Hardcoded subject (0 = variable)
  uint64_t p_const;      // Hardcoded predicate (0 = variable)
  uint64_t o_const;      // Hardcoded object (0 = variable)
  uint8_t s_is_var;      // 1 if subject is variable
  uint8_t p_is_var;      // 1 if predicate is variable
  uint8_t o_is_var;      // 1 if object is variable
} construct_template_t;

// AOT-generated specialized functions
static inline size_t knhk_construct8_emit_8_s_const_p_const_o_var(
  const uint64_t *S_base, uint64_t off, uint64_t len,
  uint64_t s_const, uint64_t p_const,
  const uint64_t *O_base,  // Object comes from WHERE clause
  uint64_t *out_S, uint64_t *out_P, uint64_t *out_O
) {
  // Hardcoded: s_const, p_const
  // Variable: O_base[off..off+len]
  // No need to broadcast s_const/p_const (compile-time constants)
  // ...
}