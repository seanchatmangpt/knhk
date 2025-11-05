# knhk-warm Documentation

Warm path operations (≤500ms budget) using oxigraph for SPARQL query execution.

## Overview

The `knhk-warm` crate provides:
- SPARQL query execution (SELECT, ASK, CONSTRUCT, DESCRIBE)
- Query result caching (LRU cache, 1000 entries)
- Query plan caching (parsed SPARQL queries)
- Epoch-based cache invalidation
- Hot path integration (automatic routing to ≤2ns C hot path)
- OTEL metrics and observability

## Architecture

- **WarmPathGraph**: Main graph interface wrapping oxigraph Store
- **WarmPathExecutor**: Executor with automatic path selection
- **Hot Path Integration**: Routes simple queries to C hot path (≤2ns)
- **Query Module**: High-level query execution functions

## Key Features

- **Performance**: 10-14x faster than unrdf FFI (5-50ms vs 150-700ms)
- **Caching**: LRU cache for query results and query plans
- **Thread-safe**: Arc-based shared store for concurrent access
- **Path Selection**: Automatic routing to hot/warm/cold paths

## Test Coverage

83 Chicago TDD tests across 7 test files:
- `tests/executor.rs` - Executor tests (12 tests)
- `tests/graph.rs` - Graph operations (12 tests)
- `tests/query.rs` - Query module (11 tests)
- `tests/errors.rs` - Error handling (11 tests)
- `tests/edge_cases.rs` - Edge cases (13 tests)
- `tests/cache.rs` - Cache behavior (12 tests)
- `tests/performance.rs` - Performance tests (7 tests)

**Coverage**: ~80%+ overall

## Usage

See [../../docs/ggen-integration-guide.md](../../docs/ggen-integration-guide.md) for detailed usage guide.

## Related Documentation

- [Architecture](../../docs/architecture.md) - System architecture
- [Testing](../../docs/testing.md) - Testing documentation
- [Performance](../../docs/performance.md) - Performance guide

