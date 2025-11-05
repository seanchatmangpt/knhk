# DoD Validator Documentation

Definition of Done validator using KNHK's 2ns capabilities.

## File Structure

```
playground/dod-validator/
├── c/
│   ├── hot_validators.h    # C hot path pattern detection declarations
│   └── hot_validators.c    # C hot path pattern detection (SIMD, ≤2ns)
├── rust/
│   ├── dod-validator-core/
│   │   ├── src/
│   │   │   ├── lib.rs              # Core validation engine
│   │   │   └── pattern_extractor.rs # Pattern extraction logic
│   │   └── Cargo.toml
│   ├── dod-validator-cli/
│   │   ├── src/
│   │   │   └── main.rs             # CLI interface
│   │   └── Cargo.toml
│   ├── dod-validator-autonomous/
│   │   ├── src/
│   │   │   ├── lib.rs              # Autonomous validator (self-healing)
│   │   │   └── chicago_tdd_tests.rs # Chicago TDD tests
│   │   └── Cargo.toml
│   └── dod-validator-hot/
│       ├── src/
│       │   └── lib.rs              # Hot path FFI bindings
│       ├── build.rs                # Link to C library
│       └── Cargo.toml
├── tests/
│   └── chicago_autonomous_dod_validator.c # C integration tests
├── ARCHITECTURE.md         # System architecture
├── AUTONOMICS.md           # Autonomous system design
├── VISION_2027.md          # Strategic vision
├── PRESS_RELEASE.md        # Working backwards press release
└── README.md               # Project overview
```

## Core Components

### C Hot Path (`c/hot_validators.c`)
- `knhk_validate_unwrap_pattern()` - Detect unwrap() patterns
- `knhk_validate_todo_pattern()` - Detect TODO comments
- `knhk_validate_expect_pattern()` - Detect expect() patterns
- `knhk_validate_panic_pattern()` - Detect panic! patterns
- Uses SIMD intrinsics for ≤2ns pattern matching

### Core Validator (`rust/dod-validator-core/`)
- `DoDValidator` - Main validation engine
- `PatternType` enum - Violation types
- `ValidationReport` - Validation results
- Pattern extraction and validation

### Autonomous Validator (`rust/dod-validator-autonomous/`)
- `AutonomousValidator` - Self-healing validator
- `observe()` - Observe codebase
- `reflect()` - Analyze violations
- `act()` - Generate and apply fixes
- `verify()` - Verify fixes

### CLI (`rust/dod-validator-cli/`)
- Command-line interface
- `validate` - Validate codebase
- `category` - Category-specific validation

## Key Features

- **Pattern Detection**: Finds unwrap(), TODO, placeholders (≤2ns)
- **Performance**: Uses KNHK hot path for ≤2ns checks
- **Autonomous**: Automatic fix generation and application
- **Chicago TDD**: Comprehensive test coverage

## Related Documentation

- [Architecture](../ARCHITECTURE.md) - Validator architecture
- [Autonomics](../AUTONOMICS.md) - Autonomous system design
- [Vision 2027](../VISION_2027.md) - Strategic vision

