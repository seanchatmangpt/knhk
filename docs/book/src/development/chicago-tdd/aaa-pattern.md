# AAA Pattern

AAA (Arrange-Act-Assert) pattern for test organization.

## Overview

AAA pattern organizes tests into three phases:
- **Arrange**: Set up test data and dependencies
- **Act**: Execute the code under test
- **Assert**: Verify the results

## Example

```rust
#[test]
fn test_beat_scheduler() {
    // Arrange
    let mut scheduler = BeatScheduler::new();
    
    // Act
    let (tick, pulse) = scheduler.advance_beat();
    
    // Assert
    assert!(tick < 8);
    assert!(pulse == (tick == 0));
}
```

## Related Documentation

- [Chicago TDD](../chicago-tdd.md) - Overview
- [Methodology](methodology.md) - Methodology
