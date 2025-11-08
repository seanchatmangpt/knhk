#!/bin/bash
# Validate all feature flag combinations
# Part of KNHK Build Validation Matrix
set -e

echo "=== Feature Matrix Validation ==="
echo "Systematically testing all feature combinations across workspace packages"
echo ""

total_builds=0
successful_builds=0
failed_builds=0

build_feature() {
  local package=$1
  local features=$2
  local extra_flags=$3

  total_builds=$((total_builds + 1))
  echo -n "[$total_builds] Testing $package with features: ${features:-default} ... "

  if [ -z "$features" ]; then
    if cargo build -p "$package" --release $extra_flags >/dev/null 2>&1; then
      echo "✅"
      successful_builds=$((successful_builds + 1))
    else
      echo "❌"
      failed_builds=$((failed_builds + 1))
    fi
  else
    if cargo build -p "$package" --release --features "$features" $extra_flags >/dev/null 2>&1; then
      echo "✅"
      successful_builds=$((successful_builds + 1))
    else
      echo "❌"
      failed_builds=$((failed_builds + 1))
    fi
  fi
}

# knhk-otel (3 combinations)
echo "## knhk-otel (3 combinations)"
build_feature "knhk-otel" ""
build_feature "knhk-otel" "" "--no-default-features"
build_feature "knhk-otel" "std" "--no-default-features"
echo ""

# knhk-connectors (4 combinations)
echo "## knhk-connectors (4 combinations)"
build_feature "knhk-connectors" ""
build_feature "knhk-connectors" "kafka"
build_feature "knhk-connectors" "salesforce"
build_feature "knhk-connectors" "kafka,salesforce"
echo ""

# knhk-unrdf (4 combinations)
echo "## knhk-unrdf (4 combinations)"
build_feature "knhk-unrdf" "" "--no-default-features"
build_feature "knhk-unrdf" "native" "--no-default-features"
build_feature "knhk-unrdf" "unrdf" "--no-default-features"
build_feature "knhk-unrdf" "native,unrdf" "--no-default-features"
echo ""

# knhk-etl (6 combinations)
echo "## knhk-etl (6 combinations)"
build_feature "knhk-etl" ""
build_feature "knhk-etl" "" "--no-default-features"
build_feature "knhk-etl" "std" "--no-default-features"
build_feature "knhk-etl" "grpc"
build_feature "knhk-etl" "tokio-runtime"
build_feature "knhk-etl" "grpc,tokio-runtime"
echo ""

# knhk-warm (4 combinations)
echo "## knhk-warm (4 combinations)"
build_feature "knhk-warm" ""
build_feature "knhk-warm" "otel"
build_feature "knhk-warm" "unrdf"
build_feature "knhk-warm" "otel,unrdf"
echo ""

# knhk-validation (7 combinations)
echo "## knhk-validation (7 combinations)"
build_feature "knhk-validation" ""
build_feature "knhk-validation" "" "--no-default-features"
build_feature "knhk-validation" "advisor"
build_feature "knhk-validation" "policy-engine"
build_feature "knhk-validation" "schema-resolution"
build_feature "knhk-validation" "streaming"
build_feature "knhk-validation" "advisor,policy-engine,schema-resolution,streaming"
echo ""

# knhk-patterns (2 combinations)
echo "## knhk-patterns (2 combinations)"
build_feature "knhk-patterns" ""
build_feature "knhk-patterns" "unrdf"
echo ""

# knhk-cli (3 combinations)
echo "## knhk-cli (3 combinations)"
build_feature "knhk-cli" ""
build_feature "knhk-cli" "" "--no-default-features"
build_feature "knhk-cli" "std" "--no-default-features"
echo ""

# Summary
echo "=== Feature Matrix Validation Summary ==="
echo "Total builds:      $total_builds"
echo "Successful:        $successful_builds ✅"
echo "Failed:            $failed_builds ❌"
echo ""

if [ $failed_builds -eq 0 ]; then
  echo "✅ All feature combinations validated successfully"
  exit 0
else
  echo "❌ Some feature combinations failed"
  exit 1
fi
