# API Reference

**Version**: v0.4.0 (Production-Ready)  
**Core Library**: v1.0.0 (API Stable)

## Public API

The KNHK API is organized into modular headers for better maintainability.

### Header Structure

```
include/
├── knhk.h              # Main umbrella header (includes all components)
└── knhk/
    ├── types.h          # Type definitions (enums, structs, constants)
    ├── eval.h           # Query evaluation functions (eval_bool, eval_construct8)
    ├── receipts.h       # Receipt operations (receipt_merge)
    └── utils.h          # Utility functions (init_ctx, load_rdf, clock utilities)
```

**Usage**: Include only `knhk.h` - it automatically includes all sub-modules:
```c
#include "knhk.h"  // Includes all API components
```

## See Also

- [C API](c-api.md) - C API reference
- [Rust API](rust-api.md) - Rust API reference
- [Erlang API](erlang-api.md) - Erlang API reference

