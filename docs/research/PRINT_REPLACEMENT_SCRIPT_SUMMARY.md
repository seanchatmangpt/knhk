# Bulk Print Replacement Script - Summary

## Created Files

1. **`scripts/replace_print_with_logging.py`** - Main replacement script
2. **`scripts/replace_print_with_logging_README.md`** - Usage documentation

## Quick Start

```bash
# 1. Dry run to see what would change
python3 scripts/replace_print_with_logging.py --dry-run > replacements.txt

# 2. Test on a single file
python3 scripts/replace_print_with_logging.py --file rust/knhk-cli/src/main.rs --dry-run

# 3. Apply to single file
python3 scripts/replace_print_with_logging.py --file rust/knhk-cli/src/main.rs

# 4. Verify compilation
cd rust && cargo check

# 5. Apply incrementally to other directories
python3 scripts/replace_print_with_logging.py --dir rust/knhk-cli/src
```

## Features

- ✅ **Context-aware** - Determines log level from content (error/warn/info/debug)
- ✅ **Structured fields** - Converts format strings to structured logging fields
- ✅ **Safe** - Dry-run mode shows changes before applying
- ✅ **Incremental** - Can process single files or directories
- ✅ **Interactive** - Optional prompt before each replacement
- ✅ **Smart detection** - Handles `eprintln!` → `tracing::warn!/error!` automatically

## Replacement Examples

### Error Reporting
```rust
// Before
eprintln!("Warning: Failed to initialize tracing: {}", e);

// After
tracing::warn!(
    error.message = %e,
    "Failed to initialize tracing"
);
```

### Info Logging
```rust
// Before
println!("Processing {} triples", count);

// After
tracing::info!(
    count = count,
    "Processing triples"
);
```

## Next Steps

1. **Review research documents**:
   - `docs/research/print-to-logging-otel-migration.md`
   - `docs/research/print-to-logging-quick-reference.md`

2. **Run script incrementally**:
   - Start with high-priority files (error reporting)
   - Test compilation after each batch
   - Review and manually fix awkward replacements

3. **Add tracing imports**:
   - Ensure files have: `use tracing::{error, warn, info, debug, trace};`

4. **Add test verification**:
   - Create `LogCapture` utilities in `chicago-tdd-tools`
   - Add tests that verify logging behavior

5. **Run Weaver validation**:
   - Ensure telemetry validates against Weaver schema

## Notes

- The script makes intelligent guesses but some replacements may need manual review
- Complex format strings may need manual conversion to structured fields
- User-facing CLI output may need special handling (keep `println!` for users, add `tracing::info!` for logs)
- Always test compilation after applying changes

