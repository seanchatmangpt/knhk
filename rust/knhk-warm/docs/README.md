# knhk-warm Documentation

Warm path operations (≤500ms budget) using oxigraph for SPARQL query execution.

## File Structure

```
rust/knhk-warm/
├── src/
│   ├── lib.rs              # Module exports and public API
│   ├── graph.rs            # WarmPathGraph (oxigraph wrapper with caching)
│   ├── query.rs            # SPARQL query execution (SELECT, ASK, CONSTRUCT, DESCRIBE)
│   ├── executor.rs         # WarmPathExecutor with path selection
│   ├── hot_path.rs         # Hot path integration (routes to C ≤2ns functions)
│   ├── warm_path.rs        # Warm path operations (CONSTRUCT8)
│   ├── ffi.rs              # FFI bindings
│   ├── construct8.rs      # CONSTRUCT8 operations
│   └── error.rs            # Error types
├── tests/
│   ├── executor.rs         # Executor tests (12 tests)
│   ├── graph.rs            # Graph operations tests (12 tests)
│   ├── query.rs            # Query module tests (11 tests)
│   ├── errors.rs           # Error handling tests (11 tests)
│   ├── edge_cases.rs       # Edge cases tests (13 tests)
│   ├── cache.rs            # Cache behavior tests (12 tests)
│   ├── performance.rs      # Performance tests (7 tests)
│   └── warm_path_test.rs   # Warm path tests (5 tests)
├── benches/
│   └── query_bench.rs      # Criterion benchmarks
├── examples/
│   └── warm_path_query.rs  # Usage examples
└── Cargo.toml
```

## Core Components

### WarmPathGraph (`src/graph.rs`)
- Wraps `oxigraph::store::Store`
- LRU cache for query results (1000 entries)
- Query plan cache (parsed SPARQL queries)
- Epoch-based cache invalidation
- OTEL metrics integration

### Query Module (`src/query.rs`)
- `execute_select()` - SELECT query execution
- `execute_ask()` - ASK query execution
- `execute_construct()` - CONSTRUCT query execution
- `execute_describe()` - DESCRIBE query execution
- JSON conversion functions

### Executor (`src/executor.rs`)
- `WarmPathExecutor` - Main executor with path selection
- Routes queries to hot/warm/cold paths automatically
- `execute_query()` - Unified query interface

### Hot Path Integration (`src/hot_path.rs`)
- `execute_hot_path_ask()` - Routes to C `knhk_eval_bool`
- `execute_hot_path_select()` - Routes to C hot path COUNT operations
- Validates tick budget (≤8 ticks)
- Automatic fallback to warm path

## Dependencies

- `oxigraph = "0.5"` - Rust-native RDF store and SPARQL engine
- `lru = "0.16"` - LRU cache implementation
- `knhk-hot` - Hot path FFI bindings
- `knhk-etl` - Path selection logic

## Test Coverage

83 Chicago TDD tests across 7 test files:
- **Path Selector**: 100% coverage
- **Query Module**: ~90% coverage  
- **Graph Module**: ~80% coverage
- **Executor**: ~80% coverage
- **Overall**: ~80%+ coverage

## Usage

See [../../docs/ggen-integration-guide.md](../../docs/ggen-integration-guide.md) for detailed usage guide.

## Related Documentation

- [Architecture Guide](../../../docs/ARCHITECTURE.md) - 🆕 Consolidated 80/20 guide (System architecture)
- [Performance Guide](../../../docs/PERFORMANCE.md) - 🆕 Consolidated 80/20 guide (Performance optimization)
- [Architecture Reference](../../../docs/architecture.md) - Detailed architecture reference
- [Performance Reference](../../../docs/performance.md) - Detailed performance reference
- [Testing](../../../docs/testing.md) - Testing documentation

