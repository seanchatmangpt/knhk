// ffi_wrappers.c
// Non-inline FFI wrapper functions for Rust-C linking
// These provide linkable symbols for functions that are defined as static inline in headers
// Production-ready with FFI-safe error handling

// Undefine inline macros to get non-inline versions
#define KNHK_EVAL_BOOL_INLINE
#define KNHK_EVAL_CONSTRUCT8_INLINE
#define KNHK_PIN_RUN_INLINE
#define KNHK_SELECT_KERNEL_INLINE

#include "knhk/types.h"
#include "knhk/eval_dispatch.h"
#include "knhk/eval.h"
#include "knhk/utils.h"
#include "knhk/kernels.h"
#include <stdint.h>

// Functions are now defined as non-inline in the headers when macros are set
// No need to redefine them here - they're already non-inline versions
// All functions include proper null pointer checks and error handling
