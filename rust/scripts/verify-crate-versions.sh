#!/bin/bash
set -e

# Use full jq path or fallback to PATH
JQ="${JQ:-$(which jq 2>/dev/null || echo '/opt/homebrew/bin/jq')}"

echo "🔍 Verifying internal crate dependency versions..."

# Get version from CLI (root crate)
CLI_VERSION=$(cargo metadata --no-deps --format-version 1 --manifest-path knhk-cli/Cargo.toml 2>/dev/null | $JQ -r '.packages[0].version')
echo "Root version (knhk-cli): $CLI_VERSION"

# Check each crate uses the same version
for crate in knhk-otel knhk-lockchain knhk-hot knhk-etl knhk-warm knhk-config knhk-connectors; do
    if [ -f "$crate/Cargo.toml" ]; then
        CRATE_VERSION=$(cargo metadata --no-deps --format-version 1 --manifest-path $crate/Cargo.toml 2>/dev/null | $JQ -r '.packages[0].version')
        echo "  $crate: $CRATE_VERSION"

        if [ "$CRATE_VERSION" != "$CLI_VERSION" ]; then
            echo "❌ Version mismatch: $crate ($CRATE_VERSION) != knhk-cli ($CLI_VERSION)"
            exit 1
        fi
    else
        echo "  ⚠️  $crate: not found (skipping)"
    fi
done

# Verify internal dependencies use correct versions
echo ""
echo "Checking internal dependency versions in Cargo.toml files..."

check_dependency() {
    local crate=$1
    local dep=$2

    if [ ! -f "$crate/Cargo.toml" ]; then
        return 0
    fi

    # Check if dependency exists and uses path dependency
    if grep -q "^$dep = { " $crate/Cargo.toml; then
        if grep -q "^$dep = { .*path = " $crate/Cargo.toml; then
            echo "  ✓ $crate → $dep uses path dependency"
            return 0
        else
            echo "  ❌ $crate → $dep should use path dependency"
            return 1
        fi
    fi
}

# Check knhk-cli dependencies
check_dependency "knhk-cli" "knhk-otel"
check_dependency "knhk-cli" "knhk-hot"
check_dependency "knhk-cli" "knhk-etl"
check_dependency "knhk-cli" "knhk-warm"
check_dependency "knhk-cli" "knhk-config"

# Check knhk-hot dependencies
check_dependency "knhk-hot" "knhk-otel"
check_dependency "knhk-hot" "knhk-lockchain"

# Check knhk-etl dependencies
check_dependency "knhk-etl" "knhk-otel"

echo ""
echo "✅ All version checks passed!"
