#!/bin/bash
# Automated Van der Aalst End-to-End Validation Framework
#
# Automates the complete van der Aalst validation framework:
# 1. Fitness validation
# 2. Precision validation
# 3. Generalization validation
# 4. Process mining analysis
# 5. Formal verification
#
# Usage:
#   ./scripts/automate_van_der_aalst_validation.sh [--spec-id <spec_id>] [--phase <phase>] [--format <format>] [--output-dir <dir>]

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
OUTPUT_DIR="${OUTPUT_DIR:-$PROJECT_ROOT/tmp/van_der_aalst_validation}"
SPEC_ID="${SPEC_ID:-}"
PHASE="${PHASE:-}"
FORMAT="${FORMAT:-markdown}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --spec-id)
            SPEC_ID="$2"
            shift 2
            ;;
        --phase)
            PHASE="$2"
            shift 2
            ;;
        --format)
            FORMAT="$2"
            shift 2
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--spec-id <spec_id>] [--phase <phase>] [--format <format>] [--output-dir <dir>]"
            exit 1
            ;;
    esac
done

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "=== Van der Aalst End-to-End Validation Framework ==="
echo ""
echo "📋 Configuration:"
echo "  Output Directory: $OUTPUT_DIR"
echo "  Format: $FORMAT"
if [ -n "$SPEC_ID" ]; then
    echo "  Spec ID: $SPEC_ID"
fi
if [ -n "$PHASE" ]; then
    echo "  Phase: $PHASE"
fi
echo ""

# Check if CLI is built
if ! command -v knhk &> /dev/null; then
    echo "⚠️  knhk CLI not found, building..."
    cd "$PROJECT_ROOT/rust/knhk-cli"
    cargo build --release
    export PATH="$PROJECT_ROOT/rust/knhk-cli/target/release:$PATH"
fi

# Run validation
echo "🔄 Running validation framework..."
echo ""

if [ -n "$SPEC_ID" ]; then
    # Run with specific spec ID
    if [ -n "$PHASE" ]; then
        # Run specific phase
        echo "Running phase: $PHASE"
        knhk workflow validate \
            --spec-id "$SPEC_ID" \
            --phase "$PHASE" \
            --output-dir "$OUTPUT_DIR" \
            --format "$FORMAT"
    else
        # Run complete validation
        echo "Running complete validation framework..."
        knhk workflow validate \
            --spec-id "$SPEC_ID" \
            --output-dir "$OUTPUT_DIR" \
            --format "$FORMAT"
    fi
else
    echo "⚠️  No spec ID provided. Running validation on all registered workflows..."
    echo "   (This requires workflows to be registered first)"
    
    # Try to find registered workflows or create a test workflow
    echo "   Creating test workflow for validation..."
    
    # This would require workflow registration - for now, just show usage
    echo ""
    echo "Usage:"
    echo "  $0 --spec-id <spec_id> [--phase <phase>] [--format <format>] [--output-dir <dir>]"
    echo ""
    echo "Phases:"
    echo "  - fitness"
    echo "  - precision"
    echo "  - generalization"
    echo "  - process_mining"
    echo "  - formal"
    echo ""
    echo "Formats:"
    echo "  - markdown (default)"
    echo "  - json"
    echo "  - html"
    exit 1
fi

# Check validation result
REPORT_FILE="$OUTPUT_DIR/validation_report.$([ "$FORMAT" = "json" ] && echo "json" || [ "$FORMAT" = "html" ] && echo "html" || echo "md")"

if [ -f "$REPORT_FILE" ]; then
    echo ""
    echo "✅ Validation complete!"
    echo "📋 Report: $REPORT_FILE"
    
    # Extract status from report
    if [ "$FORMAT" = "json" ]; then
        STATUS=$(jq -r '.summary.overall_status' "$REPORT_FILE" 2>/dev/null || echo "Unknown")
        PASSED=$(jq -r '.summary.passed_phases' "$REPORT_FILE" 2>/dev/null || echo "0")
        TOTAL=$(jq -r '.summary.total_phases' "$REPORT_FILE" 2>/dev/null || echo "0")
        
        echo "📊 Status: $STATUS"
        echo "✅ Passed: $PASSED / $TOTAL phases"
        
        if [ "$STATUS" != "Pass" ]; then
            echo ""
            echo "⚠️  Validation failed or has warnings"
            exit 1
        fi
    else
        echo "📊 Check report for validation status"
    fi
else
    echo ""
    echo "❌ Validation report not found: $REPORT_FILE"
    exit 1
fi

echo ""
echo "=== Validation Complete ==="

