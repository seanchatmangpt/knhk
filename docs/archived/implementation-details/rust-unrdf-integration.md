# Rust Integration Layer for unrdf Cold Path

## Overview

The Rust integration layer (`knhk-unrdf`) serves as the bridge between KNHK's C test layer and the unrdf knowledge hook engine. This architecture ensures that **Rust is the integration layer** between KNHK and unrdf, following the warm path architecture pattern.

## Architecture

```
┌─────────────────┐
│  C Test Layer   │  (Chicago TDD tests)
│  (tests/)       │
└────────┬────────┘
         │ FFI calls
         ▼
┌─────────────────┐
│  Rust Layer     │  (knhk-unrdf crate)
│  Integration    │  - FFI-safe C interface
│                 │  - Async runtime (Tokio)
│                 │  - Error handling
└────────┬────────┘
         │ Spawns Node.js processes
         ▼
┌─────────────────┐
│  unrdf Engine   │  (Node.js/JavaScript)
│  (vendors/)     │  - Knowledge hooks
│                 │  - SPARQL queries
│                 │  - Policy packs
└─────────────────┘
```

## Components

### 1. Rust Integration Crate (`rust/knhk-unrdf/`)

**Purpose**: FFI-safe interface for C to interact with unrdf

**Key Functions**:
- `knhk_unrdf_init()` - Initialize unrdf integration
- `knhk_unrdf_store_turtle()` - Store RDF data in unrdf
- `knhk_unrdf_query()` - Execute SPARQL queries via unrdf
- `knhk_unrdf_execute_hook()` - Execute knowledge hooks via unrdf

**Implementation Details**:
- Uses Tokio runtime for async operations
- Spawns Node.js processes to execute unrdf scripts
- Returns JSON results via C buffers
- Proper error handling with Result types

### 2. C Header (`c/include/knhk/unrdf.h`)

**Purpose**: C interface declarations for FFI

**Usage**:
```c
#include "knhk/unrdf.h"

// Initialize
knhk_unrdf_init("./vendors/unrdf");

// Store data
knhk_unrdf_store_turtle(turtle_data);

// Query
char result[4096];
knhk_unrdf_query(query, result, sizeof(result));
```

### 3. C Test (`tests/chicago_cold_path_unrdf_integration.c`)

**Purpose**: Chicago TDD test demonstrating cold path integration

**Tests**:
1. `test_cold_path_store_in_unrdf()` - Store data via Rust layer
2. `test_cold_path_query_unrdf()` - Query via Rust layer
3. `test_cold_path_end_to_end_unrdf()` - End-to-end integration
4. `test_cold_path_hooks_unrdf()` - Hook execution via Rust layer

## Build System

### Makefile Integration

The Makefile includes targets for building and testing:

```makefile
# Build Rust library
$(RUST_UNRDF_LIB):
	cd rust/knhk-unrdf && cargo build --release

# Build C test
$(TEST_COLD_PATH_UNRDF): tests/chicago_cold_path_unrdf_integration.c $(RUST_UNRDF_LIB) $(LIB)
	$(CC) ... -Lrust/knhk-unrdf/target/release -lknhk_unrdf ...

# Run test
test-cold-path-unrdf: $(TEST_COLD_PATH_UNRDF)
	./$(TEST_COLD_PATH_UNRDF)
```

### Build Command

```bash
cd c
make test-cold-path-unrdf
```

## Data Flow

### Store Operation
```
C Test → knhk_unrdf_store_turtle()
  → Rust: store_turtle_data()
    → Spawn Node.js process
      → Execute unrdf script
        → Parse Turtle
        → Store in unrdf N3 store
          → Return success
```

### Query Operation
```
C Test → knhk_unrdf_query()
  → Rust: query_sparql()
    → Spawn Node.js process
      → Execute unrdf query script
        → Query unrdf store
          → Return JSON results
            → C receives JSON buffer
```

### Hook Execution
```
C Test → knhk_unrdf_execute_hook()
  → Rust: execute_hook()
    → Spawn Node.js process
      → Define hook
      → Register hook
      → Evaluate hook
        → Return hook result JSON
          → C receives JSON buffer
```

## Chicago TDD Principles Applied

### ✅ Real Collaborators
- Uses real unrdf engine (not mocked)
- Uses real Rust integration layer
- Uses real Node.js processes

### ✅ State-Based Verification
- Verifies data stored in unrdf
- Verifies query results contain expected data
- Verifies hook execution results

### ✅ No Mocks
- All components are real implementations
- No test doubles or stubs

### ✅ Direct Assertions
- Asserts on actual results (JSON strings)
- Asserts on actual behavior (store/query/hook execution)

## Integration Points

### Cold Path → Rust → unrdf

The cold path architecture routes complex queries to unrdf:

1. **Simple queries** (≤8 ticks) → Hot path (C)
2. **Complex queries** (>8 ticks) → Cold path → Rust layer → unrdf

### Rust as Integration Layer

Rust provides:
- **Type safety** for FFI interfaces
- **Error handling** with Result types
- **Async runtime** for Node.js process management
- **Memory safety** for C buffer management

## Dependencies

### Rust (`knhk-unrdf` crate)
- `tokio` - Async runtime
- `serde` / `serde_json` - JSON serialization
- `anyhow` / `thiserror` - Error handling

### Build System
- Cargo (Rust build system)
- Make (C build system)
- Node.js (runtime for unrdf)

## Future Enhancements

1. **Persistent Store**: Maintain unrdf store across multiple operations
2. **Connection Pooling**: Reuse Node.js processes for better performance
3. **Direct FFI**: Use napi-rs or similar for direct Node.js FFI (no process spawning)
4. **Erlang Bridge**: Integrate Rust layer with Erlang cold path via NIF

## Files Created

- `rust/knhk-unrdf/Cargo.toml` - Rust crate configuration
- `rust/knhk-unrdf/src/lib.rs` - Rust integration implementation
- `c/include/knhk/unrdf.h` - C header for FFI
- `tests/chicago_cold_path_unrdf_integration.c` - Chicago TDD test
- `c/Makefile` - Build system integration (updated)

