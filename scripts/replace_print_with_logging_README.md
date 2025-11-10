# Print to Logging Replacement Script

## Overview

`scripts/replace_print_with_logging.py` is a Python script that automatically replaces Rust print statements (`println!`, `eprintln!`, `print!`, `eprint!`) with appropriate `tracing` macros.

## Usage

### Basic Usage

```bash
# Dry run - see what would be changed
python3 scripts/replace_print_with_logging.py --dry-run

# Apply changes to all Rust files
python3 scripts/replace_print_with_logging.py

# Process specific file
python3 scripts/replace_print_with_logging.py --file rust/knhk-cli/src/main.rs

# Process specific directory
python3 scripts/replace_print_with_logging.py --dir rust/knhk-cli/src

# Interactive mode - prompt before each replacement
python3 scripts/replace_print_with_logging.py --interactive
```

### Options

- `--dry-run` - Show changes without applying them (recommended first step)
- `--file <path>` - Process specific file only
- `--dir <path>` - Process directory (default: `rust/`)
- `--interactive` - Prompt before each replacement
- `--level <level>` - Default log level: `error`, `warn`, `info`, `debug`, `trace` (default: `info`)
- `--skip-tests` - Skip test files (`*_test.rs`, `tests/` directory)
- `--skip-examples` - Skip example files (`examples/` directory)

## Replacement Rules

### Error Output (`eprintln!`, `eprint!`)

**Default**: `tracing::warn!` or `tracing::error!` (based on content)

- Contains "error", "failed", "failure" → `tracing::error!`
- Contains "warning", "warn", "gap" → `tracing::warn!`
- Otherwise → `tracing::warn!`

**Example:**
```rust
// Before
eprintln!("Warning: Failed to initialize tracing: {}", e);

// After
tracing::warn!(
    error.message = %e,
    component = "tracing",
    "Failed to initialize tracing"
);
```

### Standard Output (`println!`, `print!`)

**Default**: `tracing::info!` (or based on content)

- Contains "error" → `tracing::error!`
- Contains "warning", "gap" → `tracing::warn!`
- Contains "debug", "test" → `tracing::debug!`
- Otherwise → `tracing::info!`

**Example:**
```rust
// Before
println!("Processing {} triples", count);

// After
tracing::info!(
    triple_count = count,
    "Processing triples"
);
```

## Workflow

### Step 1: Dry Run

```bash
# See what would be changed
python3 scripts/replace_print_with_logging.py --dry-run > replacements.txt
```

Review `replacements.txt` to see all proposed changes.

### Step 2: Test on Single File

```bash
# Test on one file first
python3 scripts/replace_print_with_logging.py --file rust/knhk-cli/src/main.rs --dry-run
```

### Step 3: Apply to Single File

```bash
# Apply changes
python3 scripts/replace_print_with_logging.py --file rust/knhk-cli/src/main.rs
```

### Step 4: Verify Changes

```bash
# Check compilation
cd rust && cargo check

# Run tests
cd rust && cargo test
```

### Step 5: Apply to All Files

```bash
# Apply to all files (start with high-priority directories)
python3 scripts/replace_print_with_logging.py --dir rust/knhk-cli/src

# Then expand to other directories
python3 scripts/replace_print_with_logging.py --dir rust/knhk-workflow-engine/src
```

### Step 6: Handle Test Files Separately

```bash
# Process test files with appropriate log levels
python3 scripts/replace_print_with_logging.py --dir rust/knhk-workflow-engine/tests --level debug
```

## Manual Review Required

The script makes intelligent guesses, but some replacements may need manual review:

1. **User-facing CLI output** - May need to keep `println!` for user output, add `tracing::info!` for logs
2. **Complex format strings** - May need manual conversion to structured fields
3. **Test output** - May need `tracing::debug!` or custom handling
4. **Error context** - May need additional structured fields

## Post-Migration Tasks

After running the script:

1. **Add tracing imports** - Ensure files have:
   ```rust
   use tracing::{error, warn, info, debug, trace};
   ```

2. **Review structured fields** - Convert format strings to structured fields where appropriate

3. **Add test verification** - Add tests that verify logging behavior:
   ```rust
   use chicago_tdd_tools::otel::LogCapture;
   
   #[test]
   fn test_error_logged() {
       let log_capture = LogCapture::new();
       execute_operation();
       log_capture.assert_log_contains("expected message");
   }
   ```

4. **Run Weaver validation** - Ensure telemetry validates against Weaver schema:
   ```bash
   weaver registry live-check --registry registry/
   ```

## Limitations

- **Format string parsing** - Complex format strings may need manual conversion
- **Multi-line statements** - Very long print statements may not be handled perfectly
- **Macro invocations** - Print statements inside macros may not be detected
- **Comments** - Print statements in comments are not handled

## Safety

- Always run with `--dry-run` first
- Review changes before applying
- Test compilation after each batch of changes
- Use version control (git) to track changes
- Can be run incrementally on specific files/directories

## Examples

### Example 1: Error Reporting

```bash
# Process CLI error handling
python3 scripts/replace_print_with_logging.py \
    --dir rust/knhk-cli/src \
    --skip-tests \
    --dry-run
```

### Example 2: Test Files

```bash
# Process test files with debug level
python3 scripts/replace_print_with_logging.py \
    --dir rust/knhk-workflow-engine/tests \
    --level debug \
    --dry-run
```

### Example 3: Interactive Review

```bash
# Review each replacement interactively
python3 scripts/replace_print_with_logging.py \
    --dir rust/knhk-cli/src \
    --interactive
```

## Troubleshooting

### Script doesn't find print statements

- Check file encoding (should be UTF-8)
- Verify file has `.rs` extension
- Check if file is excluded by `--skip-tests` or `--skip-examples`

### Replacement looks wrong

- Use `--interactive` mode to review each replacement
- Manually fix complex cases
- Report issues with specific file/line numbers

### Compilation errors after replacement

- Check that `tracing` crate is available
- Verify imports are correct
- Check for syntax errors in replacements

## Related Documentation

- `docs/research/print-to-logging-otel-migration.md` - Comprehensive migration guide
- `docs/research/print-to-logging-quick-reference.md` - Quick reference patterns
- `docs/research/PRINT_TO_LOGGING_RESEARCH_SUMMARY.md` - Research summary

