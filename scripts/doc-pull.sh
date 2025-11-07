#!/bin/bash
# LEAN Pull System for Documentation
# Usage: ./scripts/doc-pull.sh <what-you-need>
#
# Principle: Pull documentation on-demand, don't push upfront
# Eliminates: Overproduction waste (10.0 hours, 21.1%)

set -e

DOCS_DIR="/Users/sac/knhk/docs"
REPORTS_DIR="/Users/sac/knhk/reports"

case "$1" in
  "status")
    echo "📊 Quick Status (30 seconds)"
    echo "=============================="
    echo ""
    if [ -f "$DOCS_DIR/V1-STATUS.md" ]; then
      cat "$DOCS_DIR/V1-STATUS.md"
    else
      echo "✅ Build Status:"
      cargo build --workspace --quiet 2>&1 | tail -1
      echo ""
      echo "✅ Test Status:"
      cargo test --workspace --quiet 2>&1 | tail -1
      echo ""
      echo "✅ Quality Status:"
      cargo clippy --workspace --quiet -- -D warnings 2>&1 | tail -1
      echo ""
      echo "💡 For detailed status, create docs/V1-STATUS.md (1 page max)"
    fi
    ;;

  "blockers")
    echo "🚧 Current Blockers (1 minute)"
    echo "=============================="
    echo ""
    if [ -d "$DOCS_DIR" ]; then
      # Search for blocker indicators
      BLOCKERS=$(grep -r "BLOCKER\|P0\|CRITICAL\|TODO.*urgent" "$DOCS_DIR" --include="*.md" 2>/dev/null || echo "")
      if [ -n "$BLOCKERS" ]; then
        echo "$BLOCKERS"
      else
        echo "✅ No documented blockers found"
      fi
    else
      echo "✅ No blockers documented"
    fi
    echo ""
    echo "💡 To add blockers, use inline comments with BLOCKER: prefix"
    ;;

  "metrics")
    echo "📈 Key Metrics (2 minutes)"
    echo "=========================="
    echo ""
    echo "🔨 Build:"
    if cargo build --workspace --quiet 2>&1; then
      echo "  ✅ PASS"
    else
      echo "  ❌ FAIL"
    fi
    echo ""
    echo "🧪 Tests:"
    if cargo test --workspace --quiet 2>&1; then
      echo "  ✅ PASS"
    else
      echo "  ❌ FAIL"
    fi
    echo ""
    echo "🔍 Code Quality:"
    if cargo clippy --workspace --quiet -- -D warnings 2>&1; then
      echo "  ✅ CLEAN"
    else
      echo "  ⚠️  WARNINGS"
    fi
    echo ""
    echo "📊 DoD Validation:"
    if [ -f "$REPORTS_DIR/dod-v1-validation.json" ]; then
      echo "  📄 $(jq -r '.summary.status' "$REPORTS_DIR/dod-v1-validation.json" 2>/dev/null || echo "UNKNOWN")"
    else
      echo "  ℹ️  Run ./scripts/validate-dod-v1.sh to generate"
    fi
    ;;

  "full-report")
    echo "⚠️  Full Analysis Report"
    echo "========================"
    echo ""
    echo "This generates extensive documentation (estimated 10+ minutes)"
    echo "This contradicts LEAN principles (create only what's requested)"
    echo ""
    echo "What specific analysis do you need?"
    echo "  - Architecture diagram? (specify component)"
    echo "  - Performance benchmark? (specify operation)"
    echo "  - Code quality report? (specify module)"
    echo "  - Integration analysis? (specify systems)"
    echo ""
    echo "Are you sure you need a full report? (y/N)"
    read -r confirm
    if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ]; then
      echo ""
      echo "Generating full report..."
      echo ""

      # Run comprehensive validation
      if [ -f "$DOCS_DIR/../scripts/validate-dod-v1.sh" ]; then
        bash "$DOCS_DIR/../scripts/validate-dod-v1.sh"
      fi

      # Generate summary
      echo "✅ Full validation complete"
      echo "📄 Report: $REPORTS_DIR/dod-v1-validation.json"
      echo ""
      echo "💡 Next time: Use specific pulls (status/blockers/metrics) instead"
    else
      echo ""
      echo "❌ Cancelled"
      echo "💡 Use: $0 {status|blockers|metrics} for quick info"
    fi
    ;;

  "help"|"--help"|"-h"|"")
    echo "LEAN Documentation Pull System"
    echo "=============================="
    echo ""
    echo "Usage: $0 <command>"
    echo ""
    echo "Commands:"
    echo "  status       Quick status (30s) - build/test/quality"
    echo "  blockers     Current blockers (1m) - P0/critical issues"
    echo "  metrics      Key metrics (2m) - validation status"
    echo "  full-report  Full analysis (10m+) - use sparingly"
    echo ""
    echo "LEAN Principle: Pull what you need, when you need it"
    echo ""
    echo "Examples:"
    echo "  $0 status    # Quick health check"
    echo "  $0 blockers  # What's blocking progress?"
    echo "  $0 metrics   # How are we doing?"
    echo ""
    exit 0
    ;;

  *)
    echo "❌ Unknown command: $1"
    echo ""
    echo "Usage: $0 {status|blockers|metrics|full-report|help}"
    echo ""
    echo "💡 Run '$0 help' for more information"
    exit 1
    ;;
esac
