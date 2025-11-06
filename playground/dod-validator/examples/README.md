# DoD Validator Examples

This directory contains working examples demonstrating how to use the KNHK DoD Validator.

## Examples

### Basic Validation
**File**: `basic.rs`

Demonstrates basic pattern detection for common violations:
- `.unwrap()` patterns
- `.expect()` patterns
- TODO comments
- Placeholder text

```bash
cargo run --example basic
```

### Advanced Pattern Detection
**File**: `advanced.rs`

Demonstrates advanced pattern detection:
- Closures with unwrap (`|x| x.unwrap()`)
- Macro definitions (`macro_rules!`)
- Async/await patterns (`.await.unwrap()`)

```bash
cargo run --example advanced
```

### Autonomous Validation
**File**: `autonomous.rs`

Demonstrates autonomous self-healing validation:
- Observe → Reflect → Act → Verify cycle
- Automatic fix generation
- Receipt validation

```bash
cargo run --example autonomous
```

### CLI Usage
**File**: `cli-usage.sh`

Shell script demonstrating CLI usage:
- File validation
- Directory validation
- JSON output
- Category-specific validation

```bash
chmod +x cli-usage.sh
./cli-usage.sh
```

## Quick Start

1. Build the validator:
```bash
cd rust
cargo build --release
```

2. Validate a file:
```bash
../target/release/dod-validator validate src/main.rs
```

3. Validate with JSON output:
```bash
../target/release/dod-validator validate src/ --format json > report.json
```

4. Validate specific category:
```bash
../target/release/dod-validator category code-quality src/
```

## Integration Examples

### Pre-commit Hook
```bash
cp .git/hooks/pre-commit /path/to/repo/.git/hooks/
chmod +x /path/to/repo/.git/hooks/pre-commit
```

### CI/CD Integration
See `.github/workflows/ci.yml` for GitHub Actions integration.

### Programmatic Usage
```rust
use dod_validator_core::{ValidationEngine, ValidationReport};
use std::path::PathBuf;

let mut engine = ValidationEngine::new()?;
let report = engine.validate_all(&PathBuf::from("src/"))?;

if !report.is_success() {
    for result in &report.results {
        if !result.passed {
            println!("Violation: {} at {:?}:{}", 
                result.message,
                result.file,
                result.line
            );
        }
    }
}
```

