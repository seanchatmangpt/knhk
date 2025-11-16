#!/usr/bin/env bash
#
# SHACL Shape Validation Script for YAWL Turtle Workflows
#
# DOCTRINE ALIGNMENT:
# - Covenant 2: Invariants Are Law
# - Validates Q invariants and workflow soundness
# - Hard failures (exit code 1) on violations
#
# USAGE:
#   ./validate-shapes.sh <workflow.ttl>
#   ./validate-shapes.sh <workflow.ttl> --shapes <custom-shapes.ttl>
#   ./validate-shapes.sh <workflow.ttl> --verbose
#
# EXIT CODES:
#   0 - All validations passed (no violations)
#   1 - Validation violations detected (hard invariants broken)
#   2 - Warnings detected (quality issues, but deployable)
#   3 - Script error (missing dependencies, invalid arguments)
#

set -euo pipefail

# =============================================================================
# CONFIGURATION
# =============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../../" && pwd)"
ONTOLOGY_DIR="$PROJECT_ROOT/ontology"
SHACL_DIR="$ONTOLOGY_DIR/shacl"

# Default SHACL shape files (all validations)
DEFAULT_SHAPES=(
    "$SHACL_DIR/q-invariants.ttl"
    "$SHACL_DIR/workflow-soundness.ttl"
    "$SHACL_DIR/soundness.ttl"
)

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Validation counters
VIOLATIONS=0
WARNINGS=0
INFOS=0

# =============================================================================
# HELPER FUNCTIONS
# =============================================================================

print_error() {
    echo -e "${RED}ERROR${NC}: $1" >&2
}

print_warning() {
    echo -e "${YELLOW}WARNING${NC}: $1"
}

print_info() {
    echo -e "${BLUE}INFO${NC}: $1"
}

print_success() {
    echo -e "${GREEN}SUCCESS${NC}: $1"
}

usage() {
    cat <<EOF
SHACL Shape Validation Script for YAWL Turtle Workflows

USAGE:
    $0 <workflow.ttl> [OPTIONS]

OPTIONS:
    --shapes <file.ttl>     Use custom SHACL shapes file (can be repeated)
    --no-q-invariants       Skip Q invariants validation
    --no-soundness          Skip workflow soundness validation
    --verbose               Show detailed validation output
    --help                  Show this help message

EXAMPLES:
    # Validate workflow with all default shapes
    $0 examples/autonomous-workflow.ttl

    # Validate with custom shapes only
    $0 workflow.ttl --shapes custom-shapes.ttl

    # Skip Q invariants, only check soundness
    $0 workflow.ttl --no-q-invariants

    # Verbose output with all details
    $0 workflow.ttl --verbose

EXIT CODES:
    0 - All validations passed
    1 - Violations detected (hard invariants broken)
    2 - Warnings detected (quality issues)
    3 - Script error

COVENANT 2: INVARIANTS ARE LAW
This script enforces hard quality constraints (Q). Violations block deployment.
Warnings indicate quality issues but don't prevent deployment.

EOF
    exit 0
}

check_dependencies() {
    local missing_deps=()

    # Check for rapper (RDF parser)
    if ! command -v rapper &> /dev/null; then
        missing_deps+=("rapper (install: sudo apt-get install raptor2-utils)")
    fi

    # Check for pyshacl or shacl-validate
    if ! command -v pyshacl &> /dev/null && ! command -v shacl &> /dev/null; then
        missing_deps+=("pyshacl (install: pip install pyshacl)")
    fi

    if [ ${#missing_deps[@]} -gt 0 ]; then
        print_error "Missing dependencies:"
        for dep in "${missing_deps[@]}"; do
            echo "  - $dep"
        done
        exit 3
    fi
}

validate_file_exists() {
    local file="$1"
    local description="$2"

    if [ ! -f "$file" ]; then
        print_error "$description not found: $file"
        exit 3
    fi
}

validate_turtle_syntax() {
    local file="$1"

    print_info "Validating Turtle syntax: $(basename "$file")"

    if rapper -q -i turtle -o ntriples "$file" > /dev/null 2>&1; then
        print_success "Turtle syntax valid"
        return 0
    else
        print_error "Turtle syntax invalid in $file"
        rapper -i turtle -o ntriples "$file" 2>&1 | head -20
        return 1
    fi
}

run_shacl_validation() {
    local workflow_file="$1"
    local shapes_file="$2"
    local verbose="$3"

    local shapes_name=$(basename "$shapes_file" .ttl)

    print_info "Running SHACL validation: $shapes_name"

    # Create temporary file for validation report
    local report_file=$(mktemp)

    # Run pyshacl validation
    local validation_output
    if command -v pyshacl &> /dev/null; then
        validation_output=$(pyshacl -s "$shapes_file" -f human "$workflow_file" 2>&1 || true)
    else
        # Fallback to shacl command
        validation_output=$(shacl validate -s "$shapes_file" -d "$workflow_file" 2>&1 || true)
    fi

    # Parse validation output
    if echo "$validation_output" | grep -q "Conforms: True"; then
        print_success "$shapes_name: No violations"
        return 0
    else
        # Count violations, warnings, info
        local violations=$(echo "$validation_output" | grep -c "sh:Violation" || echo "0")
        local warnings=$(echo "$validation_output" | grep -c "sh:Warning" || echo "0")
        local infos=$(echo "$validation_output" | grep -c "sh:Info" || echo "0")

        VIOLATIONS=$((VIOLATIONS + violations))
        WARNINGS=$((WARNINGS + warnings))
        INFOS=$((INFOS + infos))

        if [ "$violations" -gt 0 ]; then
            print_error "$shapes_name: $violations violation(s) detected"
        fi

        if [ "$warnings" -gt 0 ]; then
            print_warning "$shapes_name: $warnings warning(s) detected"
        fi

        if [ "$infos" -gt 0 ]; then
            print_info "$shapes_name: $infos info message(s)"
        fi

        # Show detailed output if verbose or violations detected
        if [ "$verbose" = "true" ] || [ "$violations" -gt 0 ]; then
            echo ""
            echo "$validation_output"
            echo ""
        fi

        return 1
    fi
}

# =============================================================================
# MAIN VALIDATION LOGIC
# =============================================================================

main() {
    local workflow_file=""
    local custom_shapes=()
    local skip_q_invariants=false
    local skip_soundness=false
    local verbose=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --shapes)
                custom_shapes+=("$2")
                shift 2
                ;;
            --no-q-invariants)
                skip_q_invariants=true
                shift
                ;;
            --no-soundness)
                skip_soundness=true
                shift
                ;;
            --verbose)
                verbose=true
                shift
                ;;
            --help)
                usage
                ;;
            -*)
                print_error "Unknown option: $1"
                usage
                ;;
            *)
                if [ -z "$workflow_file" ]; then
                    workflow_file="$1"
                else
                    print_error "Multiple workflow files specified"
                    usage
                fi
                shift
                ;;
        esac
    done

    # Validate arguments
    if [ -z "$workflow_file" ]; then
        print_error "No workflow file specified"
        usage
    fi

    validate_file_exists "$workflow_file" "Workflow file"

    # Check dependencies
    check_dependencies

    # Validate Turtle syntax first
    if ! validate_turtle_syntax "$workflow_file"; then
        exit 3
    fi

    echo ""
    echo "═══════════════════════════════════════════════════════════════════"
    echo "  SHACL VALIDATION: $(basename "$workflow_file")"
    echo "═══════════════════════════════════════════════════════════════════"
    echo ""

    # Determine which shapes to use
    local shapes_to_validate=()

    if [ ${#custom_shapes[@]} -gt 0 ]; then
        # Use custom shapes only
        shapes_to_validate=("${custom_shapes[@]}")
        print_info "Using custom SHACL shapes"
    else
        # Use default shapes based on flags
        if [ "$skip_q_invariants" = false ]; then
            shapes_to_validate+=("$SHACL_DIR/q-invariants.ttl")
        fi

        if [ "$skip_soundness" = false ]; then
            shapes_to_validate+=("$SHACL_DIR/workflow-soundness.ttl")
            shapes_to_validate+=("$SHACL_DIR/soundness.ttl")
        fi
    fi

    # Validate all shapes files exist
    for shapes_file in "${shapes_to_validate[@]}"; do
        validate_file_exists "$shapes_file" "SHACL shapes file"
    done

    # Run validation for each shapes file
    local all_passed=true
    for shapes_file in "${shapes_to_validate[@]}"; do
        if ! run_shacl_validation "$workflow_file" "$shapes_file" "$verbose"; then
            all_passed=false
        fi
        echo ""
    done

    # Print summary
    echo "═══════════════════════════════════════════════════════════════════"
    echo "  VALIDATION SUMMARY"
    echo "═══════════════════════════════════════════════════════════════════"
    echo ""

    if [ "$VIOLATIONS" -gt 0 ]; then
        print_error "Violations: $VIOLATIONS (HARD INVARIANTS BROKEN)"
        echo ""
        echo "Covenant 2: Invariants Are Law"
        echo "These violations BLOCK deployment. Fix all violations before proceeding."
        echo ""
        exit 1
    elif [ "$WARNINGS" -gt 0 ]; then
        print_warning "Warnings: $WARNINGS (QUALITY ISSUES)"
        echo ""
        echo "Workflow can deploy but has quality issues."
        echo "Consider fixing warnings for production use."
        echo ""
        exit 2
    else
        print_success "All validations passed! ✓"
        echo ""
        echo "Workflow satisfies all Q invariants and soundness constraints."
        echo "Ready for deployment."
        echo ""
        exit 0
    fi
}

# Run main function
main "$@"
