// KNHK_ASSUME: Compiler hint pattern from simdjson
// Validates at ingress, trusts in hot path
#if defined(_MSC_VER)
  #define KNHK_ASSUME(COND) __assume(COND)
#elif defined(__GNUC__) || defined(__clang__)
  #define KNHK_ASSUME(COND) do { if (!(COND)) __builtin_unreachable(); } while (0)
#else
  #define KNHK_ASSUME(COND) assert(COND)
#endif

// Debug mode: use assertions that fire if violated
// Release mode: use compiler hints for optimization
#ifndef NDEBUG
  #define KNHK_DEBUG_ASSERT(COND) assert(COND)
#else
  #define KNHK_DEBUG_ASSERT(COND) KNHK_ASSUME(COND)
#endif