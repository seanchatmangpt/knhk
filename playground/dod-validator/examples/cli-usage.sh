#!/bin/bash
# CLI Usage Examples
# Demonstrates various CLI usage patterns

set -e

echo "DoD Validator - CLI Usage Examples"
echo "==================================="
echo ""

# Build validator
echo "Building validator..."
cd rust
cargo build --release
cd ..

VALIDATOR="./rust/target/release/dod-validator"

# Create test file
TEST_FILE=$(mktemp /tmp/dod_validator_test.XXXXXX.rs)
cat > "$TEST_FILE" << 'EOF'
fn main() {
    let x: Option<i32> = Some(42);
    let value = x.unwrap(); // Violation
    println!("{}", value);
}
EOF

echo "Created test file: $TEST_FILE"
echo ""

# Example 1: Basic validation
echo "Example 1: Basic validation"
echo "---------------------------"
$VALIDATOR validate "$TEST_FILE"
echo ""

# Example 2: JSON output
echo "Example 2: JSON output"
echo "----------------------"
$VALIDATOR validate "$TEST_FILE" --format json | jq '.' 2>/dev/null || $VALIDATOR validate "$TEST_FILE" --format json
echo ""

# Example 3: Category-specific validation
echo "Example 3: Code quality category"
echo "---------------------------------"
$VALIDATOR category code-quality "$TEST_FILE"
echo ""

# Example 4: Directory validation
echo "Example 4: Directory validation"
echo "--------------------------------"
TEST_DIR=$(mktemp -d /tmp/dod_validator_test_dir.XXXXXX)
cp "$TEST_FILE" "$TEST_DIR/test.rs"
$VALIDATOR validate "$TEST_DIR" --format json > /tmp/validation_report.json
echo "Report saved to /tmp/validation_report.json"
echo ""

# Example 5: View report
echo "Example 5: View saved report"
echo "-----------------------------"
$VALIDATOR report /tmp/validation_report.json
echo ""

# Cleanup
rm -f "$TEST_FILE"
rm -rf "$TEST_DIR"
rm -f /tmp/validation_report.json

echo "✅ All examples completed!"

