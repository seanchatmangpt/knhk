#!/usr/bin/env bash
# Chicago TDD Documentation Validation
# Validates documentation accuracy against actual code
# State-based verification: checks outputs and invariants, not implementation details

set -uo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

PASSED=0
FAILED=0
WARNINGS=0

# Test: Verify README files exist
test_readme_files_exist() {
    echo "[TEST] README Files Exist"
    
    local readmes=(
        "rust/knhk-etl/README.md"
        "rust/knhk-hot/README.md"
        "rust/knhk-lockchain/README.md"
        "rust/knhk-otel/README.md"
        "rust/knhk-validation/README.md"
        "rust/knhk-aot/README.md"
        "rust/knhk-etl/docs/README.md"
        "rust/knhk-hot/docs/README.md"
        "rust/knhk-lockchain/docs/README.md"
        "rust/knhk-otel/docs/README.md"
        "rust/knhk-validation/docs/README.md"
        "rust/knhk-aot/docs/README.md"
    )
    
    local failed=0
    for readme in "${readmes[@]}"; do
        if [[ ! -f "$readme" ]]; then
            echo "  ✗ Missing: $readme"
            ((failed++))
        fi
    done
    
    if [[ $failed -eq 0 ]]; then
        echo "  ✓ All README files exist"
        ((PASSED++))
        return 0
    else
        echo "  ✗ $failed README files missing"
        ((FAILED++))
        return 1
    fi
}

# Test: Verify README files are non-empty
test_readme_files_non_empty() {
    echo "[TEST] README Files Non-Empty"
    
    local readmes=(
        "rust/knhk-etl/README.md"
        "rust/knhk-hot/README.md"
        "rust/knhk-lockchain/README.md"
        "rust/knhk-otel/README.md"
        "rust/knhk-validation/README.md"
        "rust/knhk-aot/README.md"
    )
    
    local failed=0
    for readme in "${readmes[@]}"; do
        if [[ -f "$readme" ]] && [[ ! -s "$readme" ]]; then
            echo "  ✗ Empty: $readme"
            ((failed++))
        fi
    done
    
    if [[ $failed -eq 0 ]]; then
        echo "  ✓ All README files are non-empty"
        ((PASSED++))
        return 0
    else
        echo "  ✗ $failed README files are empty"
        ((FAILED++))
        return 1
    fi
}

# Test: Verify documentation links to detailed docs
test_readme_links_to_docs() {
    echo "[TEST] Root READMEs Link to Detailed Docs"
    
    local readmes=(
        "rust/knhk-etl/README.md:docs/README.md"
        "rust/knhk-hot/README.md:docs/README.md"
        "rust/knhk-lockchain/README.md:docs/README.md"
        "rust/knhk-otel/README.md:docs/README.md"
        "rust/knhk-validation/README.md:docs/README.md"
        "rust/knhk-aot/README.md:docs/README.md"
    )
    
    local failed=0
    for entry in "${readmes[@]}"; do
        local readme="${entry%%:*}"
        local link="${entry##*:}"
        
        if [[ -f "$readme" ]]; then
            if ! grep -q "$link" "$readme"; then
                echo "  ✗ Missing link to $link in $readme"
                ((failed++))
            fi
        fi
    done
    
    if [[ $failed -eq 0 ]]; then
        echo "  ✓ All root READMEs link to detailed docs"
        ((PASSED++))
        return 0
    else
        echo "  ✗ $failed READMEs missing links"
        ((FAILED++))
        return 1
    fi
}

# Test: Verify API references match actual code (knhk-validation)
test_validation_api_references() {
    echo "[TEST] knhk-validation API References Match Code"
    
    local doc_file="rust/knhk-validation/docs/README.md"
    local code_file="rust/knhk-validation/src/lib.rs"
    
    if [[ ! -f "$doc_file" ]] || [[ ! -f "$code_file" ]]; then
        echo "  ⚠ Files not found, skipping"
        ((WARNINGS++))
        return 0
    fi
    
    # Check that documented structs exist in code
    local apis=(
        "ValidationResult"
        "ValidationReport"
        "cli_validation"
        "network_validation"
        "property_validation"
        "performance_validation"
    )
    
    local failed=0
    for api in "${apis[@]}"; do
        if grep -q "$api" "$doc_file"; then
            if ! grep -q "$api" "$code_file"; then
                echo "  ✗ API '$api' documented but not found in code"
                ((failed++))
            fi
        fi
    done
    
    if [[ $failed -eq 0 ]]; then
        echo "  ✓ All documented APIs exist in code"
        ((PASSED++))
        return 0
    else
        echo "  ✗ $failed API references don't match code"
        ((FAILED++))
        return 1
    fi
}

# Test: Verify API references match actual code (knhk-aot)
test_aot_api_references() {
    echo "[TEST] knhk-aot API References Match Code"
    
    local doc_file="rust/knhk-aot/docs/README.md"
    local code_file="rust/knhk-aot/src/lib.rs"
    
    if [[ ! -f "$doc_file" ]] || [[ ! -f "$code_file" ]]; then
        echo "  ⚠ Files not found, skipping"
        ((WARNINGS++))
        return 0
    fi
    
    local apis=(
        "AotGuard"
        "ValidationResult"
        "PreboundIr"
        "Mphf"
    )
    
    local failed=0
    for api in "${apis[@]}"; do
        if grep -q "$api" "$doc_file"; then
            if ! grep -q "$api" "$code_file"; then
                echo "  ✗ API '$api' documented but not found in code"
                ((failed++))
            fi
        fi
    done
    
    if [[ $failed -eq 0 ]]; then
        echo "  ✓ All documented APIs exist in code"
        ((PASSED++))
        return 0
    else
        echo "  ✗ $failed API references don't match code"
        ((FAILED++))
        return 1
    fi
}

# Test: Verify API references match actual code (knhk-lockchain)
test_lockchain_api_references() {
    echo "[TEST] knhk-lockchain API References Match Code"
    
    local doc_file="rust/knhk-lockchain/docs/README.md"
    local code_file="rust/knhk-lockchain/src/lib.rs"
    
    if [[ ! -f "$doc_file" ]] || [[ ! -f "$code_file" ]]; then
        echo "  ⚠ Files not found, skipping"
        ((WARNINGS++))
        return 0
    fi
    
    local apis=(
        "Lockchain"
        "LockchainEntry"
        "ReceiptHash"
        "LockchainError"
    )
    
    local failed=0
    for api in "${apis[@]}"; do
        if grep -q "$api" "$doc_file"; then
            if ! grep -q "$api" "$code_file"; then
                echo "  ✗ API '$api' documented but not found in code"
                ((failed++))
            fi
        fi
    done
    
    if [[ $failed -eq 0 ]]; then
        echo "  ✓ All documented APIs exist in code"
        ((PASSED++))
        return 0
    else
        echo "  ✗ $failed API references don't match code"
        ((FAILED++))
        return 1
    fi
}

# Test: Verify API references match actual code (knhk-otel)
test_otel_api_references() {
    echo "[TEST] knhk-otel API References Match Code"
    
    local doc_file="rust/knhk-otel/docs/README.md"
    local code_file="rust/knhk-otel/src/lib.rs"
    
    if [[ ! -f "$doc_file" ]] || [[ ! -f "$code_file" ]]; then
        echo "  ⚠ Files not found, skipping"
        ((WARNINGS++))
        return 0
    fi
    
    local apis=(
        "Tracer"
        "Span"
        "SpanContext"
        "Metric"
        "OtlpExporter"
        "WeaverLiveCheck"
        "MetricsHelper"
    )
    
    local failed=0
    for api in "${apis[@]}"; do
        if grep -q "$api" "$doc_file"; then
            if ! grep -q "$api" "$code_file"; then
                echo "  ✗ API '$api' documented but not found in code"
                ((failed++))
            fi
        fi
    done
    
    if [[ $failed -eq 0 ]]; then
        echo "  ✓ All documented APIs exist in code"
        ((PASSED++))
        return 0
    else
        echo "  ✗ $failed API references don't match code"
        ((FAILED++))
        return 1
    fi
}

# Test: Verify no placeholder patterns in documentation
test_no_placeholders() {
    echo "[TEST] No Placeholder Patterns in Documentation"
    
    local readmes=(
        "rust/knhk-etl/docs/README.md"
        "rust/knhk-hot/docs/README.md"
        "rust/knhk-lockchain/docs/README.md"
        "rust/knhk-otel/docs/README.md"
        "rust/knhk-validation/docs/README.md"
        "rust/knhk-aot/docs/README.md"
    )
    
    local patterns=(
        "In production, this would"
        "^[[:space:]]*TODO:"
        "^[[:space:]]*FIXME:"
        "^[[:space:]]*XXX:"
        "placeholder.*would"
        "would.*placeholder"
    )
    
    local failed=0
    for readme in "${readmes[@]}"; do
        if [[ -f "$readme" ]]; then
            for pattern in "${patterns[@]}"; do
                if grep -qi "$pattern" "$readme"; then
                    echo "  ✗ Placeholder pattern '$pattern' found in $readme"
                    ((failed++))
                fi
            done
        fi
    done
    
    if [[ $failed -eq 0 ]]; then
        echo "  ✓ No placeholder patterns found"
        ((PASSED++))
        return 0
    else
        echo "  ✗ $failed placeholder patterns found"
        ((FAILED++))
        return 1
    fi
}

# Test: Verify documentation has usage examples
test_usage_examples_exist() {
    echo "[TEST] Documentation Has Usage Examples"
    
    local readmes=(
        "rust/knhk-validation/docs/README.md"
        "rust/knhk-aot/docs/README.md"
        "rust/knhk-lockchain/docs/README.md"
        "rust/knhk-otel/docs/README.md"
    )
    
    local failed=0
    for readme in "${readmes[@]}"; do
        if [[ -f "$readme" ]]; then
            if ! grep -q '\`\`\`rust' "$readme"; then
                echo "  ✗ No Rust code examples in $readme"
                ((failed++))
            fi
        fi
    done
    
    if [[ $failed -eq 0 ]]; then
        echo "  ✓ All enhanced READMEs have usage examples"
        ((PASSED++))
        return 0
    else
        echo "  ✗ $failed READMEs missing usage examples"
        ((FAILED++))
        return 1
    fi
}

# Test: Verify DOCUMENTATION_GAPS.md reflects current state
test_documentation_gaps_accurate() {
    echo "[TEST] DOCUMENTATION_GAPS.md Reflects Current State"
    
    local gaps_file="docs/DOCUMENTATION_GAPS.md"
    
    if [[ ! -f "$gaps_file" ]]; then
        echo "  ✗ DOCUMENTATION_GAPS.md not found"
        ((FAILED++))
        return 1
    fi
    
    # Check that it mentions READMEs are complete
    if grep -q "Complete Documentation" "$gaps_file" && grep -q "Enhancement Needed" "$gaps_file"; then
        echo "  ✓ DOCUMENTATION_GAPS.md reflects current state"
        ((PASSED++))
        return 0
    else
        echo "  ✗ DOCUMENTATION_GAPS.md may not reflect current state"
        ((FAILED++))
        return 1
    fi
}

# Test: Verify INDEX.md links are accurate
test_index_links_accurate() {
    echo "[TEST] INDEX.md Links Are Accurate"
    
    local index_file="docs/INDEX.md"
    
    if [[ ! -f "$index_file" ]]; then
        echo "  ✗ INDEX.md not found"
        ((FAILED++))
        return 1
    fi
    
    # Extract markdown links and verify files exist
    local failed=0
    while IFS= read -r line; do
        # Extract markdown links [text](path)
        if [[ $line =~ \[.*\]\((.*)\) ]]; then
            local link="${BASH_REMATCH[1]}"
            # Skip external links and anchors
            if [[ ! $link =~ ^(http|#) ]] && [[ $link =~ \.md$ ]]; then
                # Resolve relative to docs/
                local resolved="docs/$link"
                if [[ ! -f "$resolved" ]] && [[ ! -f "$link" ]]; then
                    echo "  ⚠ Broken link: $link"
                    ((failed++))
                fi
            fi
        fi
    done < "$index_file"
    
    if [[ $failed -eq 0 ]]; then
        echo "  ✓ All INDEX.md links are valid"
        ((PASSED++))
        return 0
    else
        echo "  ⚠ $failed potentially broken links found"
        ((WARNINGS++))
        return 0  # Warnings don't fail the test
    fi
}

# Main test execution
main() {
    echo "=========================================="
    echo "Chicago TDD Documentation Validation"
    echo "=========================================="
    echo ""
    
    test_readme_files_exist
    test_readme_files_non_empty
    test_readme_links_to_docs
    test_validation_api_references
    test_aot_api_references
    test_lockchain_api_references
    test_otel_api_references
    test_no_placeholders
    test_usage_examples_exist
    test_documentation_gaps_accurate
    test_index_links_accurate
    
    echo ""
    echo "=========================================="
    echo "Results: $PASSED passed, $FAILED failed, $WARNINGS warnings"
    echo "=========================================="
    
    if [[ $FAILED -eq 0 ]]; then
        echo -e "${GREEN}✓ All tests passed${NC}"
        exit 0
    else
        echo -e "${RED}✗ $FAILED tests failed${NC}"
        exit 1
    fi
}

main "$@"

