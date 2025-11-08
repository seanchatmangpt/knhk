# Testing Guide

KNHK uses Chicago TDD methodology with comprehensive test coverage.

## Test Structure

### Unit Tests

Unit tests are in `src/` files with `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function() {
        // Test implementation
    }
}
```

### Integration Tests

Integration tests are in `tests/` directories:

```rust
#[test]
fn test_integration() {
    // Integration test implementation
}
```

### Chicago TDD Tests

Chicago TDD tests are organized by subsystem:

```rust
#[test]
fn test_beat_scheduler() {
    // Arrange
    let scheduler = BeatScheduler::new();
    
    // Act
    let (tick, pulse) = scheduler.advance_beat();
    
    // Assert
    assert!(tick < 8);
    assert!(pulse == (tick == 0));
}
```

## Running Tests

### All Tests

```bash
cd rust && cargo test --workspace
```

### Specific Crate

```bash
cd rust && cargo test -p knhk-etl
```

### Chicago TDD Tests

```bash
cd rust && cargo test -p knhk-etl --test chicago_tdd_beat_scheduler
```

## Test Coverage

### Chicago TDD Test Suite (22 tests - all passing ✅)

**8-Beat Epoch System**:
- `chicago_tdd_beat_scheduler.rs` - Beat advancement, tick rotation, pulse detection
- `chicago_tdd_pipeline.rs` - ETL pipeline stages with beat integration
- `chicago_tdd_ring_conversion.rs` - SoA ↔ RawTriple conversion
- `chicago_tdd_hook_registry.rs` - Hook registration and lookup
- `chicago_tdd_runtime_class.rs` - R1/W1/C1 runtime class management

**Weaver Integration Tests** (31 tests - all passing ✅):
- Error diagnostics with OTEL correlation (9 tests)
- Policy engine validation (10 tests)
- Streaming ingester pattern (12 tests)

## Related Documentation

- [Chicago TDD](chicago-tdd.md) - Testing methodology
- [Error Handling](error-handling.md) - Error handling guide
