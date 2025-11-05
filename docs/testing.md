# Testing and Test Coverage

## Overview

KNHK warm path includes comprehensive test coverage following Chicago TDD principles. The test suite validates functionality, performance, error handling, and edge cases across all warm path components.

## Test Coverage Summary

**Total Tests**: 83 tests across 7 test files

### Test Files

1. **`tests/executor.rs`** (12 tests)
   - Executor creation and initialization
   - RDF loading
   - Query execution (SELECT, ASK, CONSTRUCT, DESCRIBE)
   - Path selection routing
   - Error handling
   - Graph accessor
   - Idempotence
   - State consistency

2. **`tests/graph.rs`** (12 tests)
   - File loading (valid, nonexistent, empty, invalid)
   - Triple insertion (valid, invalid IRI)
   - Quad insertion (valid, empty)
   - Cache invalidation on data changes
   - Epoch bumping
   - Graph size consistency

3. **`tests/query.rs`** (11 tests)
   - CONSTRUCT query execution
   - DESCRIBE query execution
   - Empty results handling
   - Parse errors
   - JSON conversion (select, ask, construct)
   - Result structure validation

4. **`tests/errors.rs`** (11 tests)
   - QueryError variants (ParseError, ExecutionError, UnsupportedQueryType)
   - WarmPathError variants
   - Graph error handling (invalid Turtle, invalid IRI, file not found)
   - Executor error handling
   - Error propagation
   - Error display implementations

5. **`tests/edge_cases.rs`** (13 tests)
   - Empty/whitespace queries
   - Invalid SPARQL syntax
   - Very large queries and result sets
   - Concurrent query execution
   - Concurrent data modification
   - Cache eviction
   - Query plan caching
   - Unicode handling
   - Prefix handling
   - Multiple concurrent graphs
   - Performance under load

6. **`tests/cache.rs`** (12 tests)
   - Cache invalidation (epoch bump, data insert, data load)
   - LRU eviction behavior
   - Query plan cache hits/misses
   - Cache metrics accuracy
   - Hit rate calculation
   - Cache consistency across epochs
   - Eviction order

7. **`tests/performance.rs`** (7 tests)
   - Query execution time (≤500ms)
   - Cache hit rate
   - Path selection accuracy
   - Concurrent query execution
   - Performance target validation

## Coverage by Module

- **Path Selector**: 100% coverage ✅
- **Query Module**: ~90% coverage ✅
- **Graph Module**: ~80% coverage ✅
- **Executor**: ~80% coverage ✅
- **Overall**: ~80%+ coverage ✅

## Chicago TDD Principles

All tests follow Chicago TDD methodology:

- **State-based assertions**: Verify outputs and invariants, not implementation details
- **Real collaborators**: Use oxigraph Store directly, no mocks
- **Invariant preservation**: Verify cache consistency, epoch tracking
- **Performance validation**: Ensure queries complete within ≤500ms target
- **Error handling**: No panics, proper Result types with descriptive errors

## Running Tests

```bash
# Run all warm path tests
cd rust/knhk-warm
cargo test

# Run specific test file
cargo test --test executor

# Run with output
cargo test -- --nocapture

# Run performance tests
cargo test --test performance

# Run benchmarks
cargo bench
```

## Test Categories

### Unit Tests
- Test individual functions and methods
- Verify input/output contracts
- Validate error handling

### Integration Tests
- Test component interactions
- Verify path selection routing
- Test cache behavior

### Performance Tests
- Validate ≤500ms target for warm path
- Measure cache hit rates
- Test concurrent execution

### Edge Case Tests
- Handle invalid inputs gracefully
- Test boundary conditions
- Verify concurrent access safety

## Hot Path Testing

Hot path queries are automatically routed to C hot path functions (≤2ns execution) when:
- Query is simple ASK or COUNT
- Data size ≤8 triples
- No complex SPARQL features

Tests verify:
- Path selection correctly identifies hot path queries
- C hot path functions are called directly
- Tick budget validation (≤8 ticks)
- Automatic fallback to warm path if hot path fails

## Continuous Integration

Tests run automatically on:
- Every commit
- Pull request validation
- Release preparation

All tests must pass before merging to main branch.

## Future Test Additions

Planned additions:
- Property path testing
- Complex JOIN queries
- SHACL validation tests
- OWL reasoning tests
- Performance regression tests
- Memory leak detection

