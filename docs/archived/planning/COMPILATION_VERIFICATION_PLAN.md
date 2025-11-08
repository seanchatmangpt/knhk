# Compilation Verification Plan

## Manual Verification Steps

Since automated verification is blocked by shell issues, follow these steps to verify all packages compile:

### Step 1: Verify Workspace Structure
```bash
cd /Users/sac/knhk/rust
grep -A 15 "members = \[" Cargo.toml
```

Expected: 13 packages listed (knhk-hot, knhk-otel, knhk-connectors, knhk-lockchain, knhk-unrdf, knhk-etl, knhk-warm, knhk-aot, knhk-validation, knhk-config, knhk-cli, knhk-integration-tests)

### Step 2: Verify Package Names Match Directories
```bash
cd /Users/sac/knhk/rust
for dir in knhk-* knhk-*; do
    if [ -d "$dir" ] && [ -f "$dir/Cargo.toml" ]; then
        pkg_name=$(grep "^name = " "$dir/Cargo.toml" | head -1 | sed 's/name = "\(.*\)"/\1/')
        dir_name=$(basename "$dir")
        if [ "$pkg_name" != "$dir_name" ]; then
            echo "MISMATCH: $dir has package name '$pkg_name'"
        fi
    fi
done
```

### Step 3: Check All Packages Compile
```bash
cd /Users/sac/knhk/rust
cargo check --workspace
```

### Step 4: Check Clippy Warnings
```bash
cd /Users/sac/knhk/rust
cargo clippy --workspace -- -D warnings
```

### Step 5: Check Each Package Individually
```bash
cd /Users/sac/knhk/rust
for pkg in knhk-hot knhk-otel knhk-connectors knhk-lockchain knhk-unrdf knhk-etl knhk-warm knhk-aot knhk-validation knhk-config knhk-cli knhk-integration-tests; do
    echo "=== Checking $pkg ==="
    cargo check -p $pkg || echo "FAILED: $pkg"
    echo ""
done
```

### Step 6: Verify C Code Compiles
```bash
cd /Users/sac/knhk/c
make clean
make lib
```

### Step 7: Verify C Test Targets Build
```bash
cd /Users/sac/knhk/c
make test-chicago-v04
```

### Step 8: Check Dependency Graph
```bash
cd /Users/sac/knhk/rust
cargo tree --workspace | head -100
```

## Common Issues to Check

### 1. Package Name Mismatches
- Check if package name in Cargo.toml matches directory name
- Check if workspace members list matches actual packages

### 2. Missing Dependencies
- Check if all imports have corresponding dependencies
- Check if optional dependencies are properly feature-gated

### 3. Type Mismatches
- Check for HashMap vs Map issues
- Check for serde_json::Value type mismatches

### 4. Feature Flag Issues
- Check if all feature-gated code has proper guards
- Check if fallback implementations exist

### 5. Version Mismatches
- Check if workspace dependencies match package versions
- Check for version conflicts

## Expected Results

- ✅ All 13 packages compile successfully
- ✅ Zero clippy warnings
- ✅ C code compiles successfully
- ✅ All test targets build
- ✅ Dependency graph is correct
- ✅ Package names match directories

