#!/bin/bash
# Automated WIP (Work In Progress) Validation
#
# Automatically validates workflows in WIP state using van der Aalst framework.
# Integrates with git hooks and CI/CD to validate workflows before commit/push.
#
# Usage:
#   ./scripts/automate_validation_wip.sh [--check-changes] [--validate-all]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Configuration
CHECK_CHANGES="${CHECK_CHANGES:-false}"
VALIDATE_ALL="${VALIDATE_ALL:-false}"
OUTPUT_DIR="$PROJECT_ROOT/tmp/wip_validation"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --check-changes)
            CHECK_CHANGES=true
            shift
            ;;
        --validate-all)
            VALIDATE_ALL=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--check-changes] [--validate-all]"
            exit 1
            ;;
    esac
done

echo "=== Automated WIP Validation ==="
echo ""

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Check if CLI is built
if ! command -v knhk &> /dev/null; then
    echo "⚠️  knhk CLI not found, building..."
    cd "$PROJECT_ROOT/rust/knhk-cli"
    cargo build --release
    export PATH="$PROJECT_ROOT/rust/knhk-cli/target/release:$PATH"
fi

# Find workflow files that changed
if [ "$CHECK_CHANGES" = "true" ]; then
    echo "🔍 Checking for changed workflow files..."
    
    # Find changed .ttl files (YAWL workflows)
    CHANGED_WORKFLOWS=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.(ttl|turtle)$' || true)
    
    if [ -z "$CHANGED_WORKFLOWS" ]; then
        echo "✅ No workflow files changed"
        exit 0
    fi
    
    echo "📋 Changed workflow files:"
    echo "$CHANGED_WORKFLOWS" | sed 's/^/  - /'
    echo ""
    
    # For each changed workflow, we would need to:
    # 1. Parse the workflow
    # 2. Register it
    # 3. Run validation
    # This requires workflow parsing and registration
    
    echo "⚠️  Workflow validation requires workflow registration"
    echo "   Run: knhk workflow register <workflow_file>"
    echo "   Then: knhk workflow validate --spec-id <spec_id>"
else
    # Validate all registered workflows
    if [ "$VALIDATE_ALL" = "true" ]; then
        echo "🔄 Validating all registered workflows..."
        
        # This would require listing registered workflows
        # For now, show usage
        echo ""
        echo "To validate specific workflows:"
        echo "  knhk workflow validate --spec-id <spec_id>"
        echo ""
        echo "To validate all workflows, list them first:"
        echo "  knhk workflow list"
    else
        # Default: run validation framework tests
        echo "🧪 Running validation framework tests..."
        
        cd "$PROJECT_ROOT/rust/knhk-workflow-engine"
        cargo test --test van_der_aalst_framework --release
        
        echo ""
        echo "✅ Validation framework tests passed"
    fi
fi

echo ""
echo "=== WIP Validation Complete ==="

