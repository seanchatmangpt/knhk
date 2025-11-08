# Chicago TDD Methodology

Chicago TDD (Test-Driven Development) methodology for KNHK.

## Overview

Chicago TDD uses:
- AAA pattern (Arrange-Act-Assert)
- Test-first development
- Behavior-focused tests
- Real collaborators over mocks

## AAA Pattern

```rust
#[test]
fn test_function() {
    // Arrange
    let scheduler = BeatScheduler::new();
    
    // Act
    let (tick, pulse) = scheduler.advance_beat();
    
    // Assert
    assert!(tick < 8);
    assert!(pulse == (tick == 0));
}
```

## Related Documentation

- [Chicago TDD](../chicago-tdd.md) - Overview
- [Test Organization](test-organization.md) - Test structure
- [AAA Pattern](aaa-pattern.md) - Pattern details
