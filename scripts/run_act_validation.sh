#!/bin/bash
# Run act validation for GitHub Actions workflows
#
# Wrapper script that handles act version issues and provides clear output
#
# Usage:
#   ./scripts/run_act_validation.sh [--workflow <workflow>] [--list] [--run]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

WORKFLOW="${WORKFLOW:-}"
LIST_ONLY="${LIST_ONLY:-false}"
RUN_WORKFLOW="${RUN_WORKFLOW:-false}"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --workflow)
            WORKFLOW="$2"
            shift 2
            ;;
        --list)
            LIST_ONLY=true
            shift
            ;;
        --run)
            RUN_WORKFLOW=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--workflow <workflow>] [--list] [--run]"
            exit 1
            ;;
    esac
done

cd "$PROJECT_ROOT"

echo "=== Running act Validation ==="
echo ""

# Try to find act
ACT_CMD=""
if command -v act &> /dev/null; then
    ACT_CMD="act"
elif [ -f "/Users/sac/.asdf/shims/act" ]; then
    ACT_CMD="/Users/sac/.asdf/shims/act"
    # Try to set version
    asdf local act 0.2.60 2>&1 || true
else
    echo "❌ act not found"
    echo ""
    echo "Install act:"
    echo "  brew install act  # macOS"
    echo "  # or see https://github.com/nektos/act"
    exit 1
fi

echo "✅ Using act: $ACT_CMD"
echo ""

# Check Docker
if ! docker ps &> /dev/null; then
    echo "⚠️  Docker not running or not accessible"
    echo "   Start Docker Desktop or Docker daemon"
    echo ""
    echo "   For listing workflows, Docker is not required"
fi

# List workflows
if [ "$LIST_ONLY" = "true" ]; then
    echo "📋 Listing workflows..."
    if [ -n "$WORKFLOW" ]; then
        $ACT_CMD -W ".github/workflows/$WORKFLOW" --list 2>&1 || true
    else
        $ACT_CMD --list 2>&1 || true
    fi
    exit 0
fi

# Run workflow
if [ "$RUN_WORKFLOW" = "true" ] || [ -n "$WORKFLOW" ]; then
    if [ -z "$WORKFLOW" ]; then
        echo "❌ --workflow required for running"
        exit 1
    fi

    echo "🚀 Running workflow: $WORKFLOW"
    echo ""
    echo "⚠️  Note: Some steps may fail locally (GitHub API calls, artifacts)"
    echo "   Focus on: build steps, test execution, core logic"
    echo ""

    # Run with continue-on-error for GitHub-specific steps
    $ACT_CMD -W ".github/workflows/$WORKFLOW" \
        --container-architecture linux/amd64 \
        --artifact-server-path /tmp/artifacts 2>&1 || {
        echo ""
        echo "⚠️  Workflow execution completed with some failures"
        echo "   This is expected for GitHub-specific steps (API calls, artifacts)"
        echo "   Check build and test steps for actual validation"
    }
else
    # Default: list workflows
    echo "📋 Available workflows:"
    $ACT_CMD --list 2>&1 || true
    echo ""
    echo "💡 To run a workflow:"
    echo "   $0 --workflow <workflow> --run"
fi

echo ""
echo "=== Complete ==="




