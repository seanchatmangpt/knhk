# Running Tests

## Test Structure

KNHK uses Chicago TDD methodology with comprehensive test coverage:

- **Unit Tests**: In `src/` files with `#[cfg(test)]`
- **Integration Tests**: In `tests/` directories
- **Chicago TDD Tests**: Organized by subsystem
- **Performance Tests**: Verify ≤8 tick budget

## Running Tests

### All Tests

```bash
# Run all tests across entire workspace
cd rust && cargo test --workspace
```

### Specific Crate Tests

```bash
# Run tests for specific crate
cd rust && cargo test -p knhk-etl
cd rust && cargo test -p knhk-unrdf --features native
```

### Chicago TDD Test Suite

```bash
# Run Chicago TDD tests for ETL
cd rust && cargo test -p knhk-etl --test chicago_tdd_beat_scheduler
cd rust/knhk-etl && cargo test --test chicago_tdd_pipeline
cd rust/knhk-etl && cargo test --test chicago_tdd_ring_conversion
```

### Hooks Engine Tests

```bash
# Run hooks engine tests
cd rust/knhk-unrdf && cargo test hooks_native::tests
```

### C Layer Tests

```bash
# Run C layer tests (from repo root)
make test-chicago-v04        # Chicago TDD C tests
make test-performance-v04    # Performance tests (verify ≤8 ticks)
make test-integration-v2     # Integration tests
```

## Code Quality Checks

### Linting

```bash
# Lint all code
cd rust && cargo clippy --workspace -- -D warnings
```

### Formatting

```bash
# Check formatting
cd rust && cargo fmt --all --check

# Auto-format
cd rust && cargo fmt --all
```

## Test Coverage

### Chicago TDD Test Suite (22 tests - all passing ✅)

Organized by subsystem following AAA pattern (Arrange-Act-Assert):

**8-Beat Epoch System** (`rust/knhk-etl/tests/`):
- `chicago_tdd_beat_scheduler.rs` - Beat advancement, tick rotation, pulse detection
- `chicago_tdd_pipeline.rs` - ETL pipeline stages with beat integration
- `chicago_tdd_ring_conversion.rs` - SoA ↔ RawTriple conversion
- `chicago_tdd_hook_registry.rs` - Hook registration and lookup
- `chicago_tdd_runtime_class.rs` - R1/W1/C1 runtime class management

**Weaver Integration Tests** (31 tests - all passing ✅):
- Error diagnostics with OTEL correlation (9 tests)
- Policy engine validation (10 tests)
- Streaming ingester pattern (12 tests)

## Performance Testing

### Hot Path Performance

Verify hot path operations meet ≤8 tick budget:

```bash
# Run performance tests
make test-performance-v04
```

### Benchmarks

```bash
# Run benchmarks (from crate directory)
cd rust/knhk-unrdf && cargo bench
```

## Next Steps

- [Chicago TDD](../development/chicago-tdd.md) - Testing methodology
- [Performance](../development/performance.md) - Performance considerations

