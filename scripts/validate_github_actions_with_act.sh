#!/bin/bash
# Validate GitHub Actions workflows using act
#
# Uses act (https://github.com/nektos/act) to run GitHub Actions workflows locally
# for testing and validation before pushing to GitHub.
#
# Usage:
#   ./scripts/validate_github_actions_with_act.sh [--workflow <workflow>] [--list] [--dry-run]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Configuration
WORKFLOW="${WORKFLOW:-}"
LIST_ONLY="${LIST_ONLY:-false}"
DRY_RUN="${DRY_RUN:-false}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--workflow <workflow>] [--list] [--dry-run]"
            exit 1
            ;;
    esac
done

cd "$PROJECT_ROOT"

echo "=== GitHub Actions Validation with act ==="
echo ""

# Check if act is installed
if ! command -v act &> /dev/null; then
    echo "❌ act is not installed"
    echo ""
    echo "Install act:"
    echo "  macOS:   brew install act"
    echo "  Linux:   See https://github.com/nektos/act#installation"
    echo "  Windows: See https://github.com/nektos/act#installation"
    echo ""
    echo "Or use Docker:"
    echo "  docker pull nektos/act-environments-ubuntu:latest"
    exit 1
fi

echo "✅ act is installed"
act --version
echo ""

# List workflows
if [ "$LIST_ONLY" = "true" ]; then
    echo "📋 Available workflows:"
    act -l
    exit 0
fi

# Validate workflow file syntax
echo "🔍 Validating workflow file syntax..."
if [ -n "$WORKFLOW" ]; then
    WORKFLOW_FILE=".github/workflows/$WORKFLOW"
    if [ ! -f "$WORKFLOW_FILE" ]; then
        echo "❌ Workflow file not found: $WORKFLOW_FILE"
        exit 1
    fi
    
    echo "  Validating: $WORKFLOW_FILE"
    # Basic YAML syntax check
    if command -v yamllint &> /dev/null; then
        yamllint "$WORKFLOW_FILE" || echo "⚠️  yamllint warnings (may be acceptable)"
    else
        echo "  ⚠️  yamllint not installed, skipping YAML validation"
    fi
else
    # Validate all workflows
    echo "  Validating all workflow files..."
    for workflow_file in .github/workflows/*.yml .github/workflows/*.yaml; do
        if [ -f "$workflow_file" ]; then
            echo "  - $(basename "$workflow_file")"
            if command -v yamllint &> /dev/null; then
                yamllint "$workflow_file" || echo "    ⚠️  yamllint warnings"
            fi
        fi
    done
fi

echo ""

# Dry run mode
if [ "$DRY_RUN" = "true" ]; then
    echo "🔍 Dry run mode - listing what would run..."
    if [ -n "$WORKFLOW" ]; then
        act -W ".github/workflows/$WORKFLOW" --list
    else
        act --list
    fi
    exit 0
fi

# Run workflow with act
echo "🚀 Running workflow with act..."
echo ""

if [ -n "$WORKFLOW" ]; then
    echo "Running workflow: $WORKFLOW"
    echo ""
    
    # Run specific workflow
    if act -W ".github/workflows/$WORKFLOW" --container-architecture linux/amd64; then
        echo ""
        echo "✅ Workflow validation passed!"
    else
        echo ""
        echo "❌ Workflow validation failed"
        echo ""
        echo "Note: Some failures may be expected in local testing:"
        echo "  - GitHub API calls (PR comments, artifacts)"
        echo "  - Secrets and environment variables"
        echo "  - Platform-specific differences"
        exit 1
    fi
else
    echo "Running all workflows..."
    echo ""
    
    # Run all workflows
    if act --container-architecture linux/amd64; then
        echo ""
        echo "✅ All workflows validated!"
    else
        echo ""
        echo "⚠️  Some workflows may have failed (see above)"
        echo ""
        echo "Note: Some failures may be expected in local testing:"
        echo "  - GitHub API calls (PR comments, artifacts)"
        echo "  - Secrets and environment variables"
        echo "  - Platform-specific differences"
    fi
fi

echo ""
echo "=== Validation Complete ==="
echo ""
echo "💡 Tips:"
echo "  - Use --list to see available workflows"
echo "  - Use --dry-run to see what would run"
echo "  - Use --workflow <name> to run specific workflow"
echo "  - Some GitHub-specific features won't work locally (PR comments, artifacts)"




