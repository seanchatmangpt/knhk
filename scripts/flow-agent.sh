#!/bin/bash
# Single-Piece Flow: Process one task completely before starting next
# Implements Lean principle: "Stop starting, start finishing"

set -e

task="$1"

if [ -z "$task" ]; then
  echo "Usage: $0 <task-name>"
  echo ""
  echo "Flow principle: Finish one task before starting another"
  echo ""
  echo "Available tasks:"
  echo "  fix-tests      - Fix failing tests in workspace"
  echo "  fix-clippy     - Fix clippy warnings"
  echo "  optimize-perf  - Optimize performance (≤8 ticks)"
  echo "  fix-warnings   - Fix all compilation warnings"
  echo "  validate-weaver - Run Weaver schema validation"
  exit 1
fi

echo "🔄 Starting single-piece flow for: $task"
echo "   WIP Limit Check: Ensuring focused execution..."

# Step 1: Gate 0 validation (catch issues early)
echo ""
echo "→ Step 1/4: Gate 0 validation (shift-left quality)..."
if [ -f "./scripts/gate-0-validation.sh" ]; then
  ./scripts/gate-0-validation.sh || {
    echo "❌ Gate 0 validation failed - fix issues before proceeding"
    exit 1
  }
else
  echo "   ⚠️  Gate 0 script not found, skipping..."
fi

# Step 2: Execute the specific task
echo ""
echo "→ Step 2/4: Executing task: $task"
case "$task" in
  "fix-tests")
    echo "   Running cargo test..."
    cargo test --workspace || {
      echo "❌ Tests failed - investigate and fix"
      exit 1
    }
    ;;
  "fix-clippy")
    echo "   Auto-fixing clippy issues..."
    cargo clippy --workspace --fix --allow-dirty --allow-staged || true
    echo "   Validating clippy compliance..."
    cargo clippy --workspace -- -D warnings || {
      echo "❌ Clippy warnings remain - manual fixes needed"
      exit 1
    }
    ;;
  "optimize-perf")
    echo "   Running performance tests (≤8 ticks)..."
    make test-performance-v04 || {
      echo "❌ Performance tests failed"
      exit 1
    }
    ;;
  "fix-warnings")
    echo "   Checking for warnings..."
    cargo build --workspace 2>&1 | grep -i warning && {
      echo "❌ Warnings found - fix before proceeding"
      exit 1
    } || echo "   ✅ No warnings"
    ;;
  "validate-weaver")
    echo "   Running Weaver schema validation..."
    if command -v weaver &> /dev/null; then
      weaver registry check -r registry/ || {
        echo "❌ Weaver validation failed"
        exit 1
      }
    else
      echo "   ⚠️  Weaver not installed, skipping..."
    fi
    ;;
  *)
    echo "❌ Unknown task: $task"
    echo "   Run without args to see available tasks"
    exit 1
    ;;
esac

# Step 3: Validate completion
echo ""
echo "→ Step 3/4: Validating completion (ensure quality)..."
cargo check --workspace --quiet || {
  echo "❌ Workspace check failed"
  exit 1
}
echo "   ✅ Workspace check passed"

cargo test --workspace --quiet || {
  echo "❌ Tests failed after task completion"
  exit 1
}
echo "   ✅ Tests passed"

# Step 4: Record metrics
echo ""
echo "→ Step 4/4: Recording flow metrics..."
timestamp=$(date +%s)
echo "$timestamp,$task,completed" >> .flow-metrics.csv 2>/dev/null || true

echo ""
echo "✅ Task completed: $task"
echo "   Status: Ready for next task"
echo "   Flow: Single-piece flow maintained (WIP=1)"
echo ""
echo "Next: Pull next task from backlog (maintain WIP ≤ 2)"
