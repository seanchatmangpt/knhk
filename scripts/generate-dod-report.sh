#!/bin/bash
# KNHK v1.0 DoD Status Report Generator
# Generates human-readable DoD compliance report

set -euo pipefail

# Change to project root
cd "$(dirname "$0")/.." || exit 1

# Run validation script and capture output (strip ANSI codes for parsing)
VALIDATION_OUTPUT=$(./scripts/validate-v1.0-dod.sh 2>&1 | sed 's/\x1b\[[0-9;]*m//g' || true)

# Extract statistics
PASSED=$(echo "$VALIDATION_OUTPUT" | grep "Passed:" | sed 's/.*Passed: //' | sed 's/\x1b\[[0-9;]*m//g' | tr -d ' ')
FAILED=$(echo "$VALIDATION_OUTPUT" | grep "Failed:" | sed 's/.*Failed: //' | sed 's/\x1b\[[0-9;]*m//g' | tr -d ' ')
WARNINGS=$(echo "$VALIDATION_OUTPUT" | grep "Warnings:" | sed 's/.*Warnings: //' | sed 's/\x1b\[[0-9;]*m//g' | tr -d ' ')

# Calculate total and percentage
TOTAL=$((PASSED + FAILED + WARNINGS))
if [ "$TOTAL" -gt 0 ]; then
    PERCENTAGE=$((PASSED * 100 / TOTAL))
else
    PERCENTAGE=0
fi

# Determine overall status from validation output
if echo "$VALIDATION_OUTPUT" | grep -q "✅ All P0 DoD criteria met"; then
    if [ "$WARNINGS" -eq 0 ]; then
        STATUS="✅ COMPLIANT"
        STATUS_COLOR="🟢"
    else
        STATUS="⚠️  COMPLIANT WITH WARNINGS"
        STATUS_COLOR="🟡"
    fi
else
    STATUS="❌ NON-COMPLIANT"
    STATUS_COLOR="🔴"
fi

# Generate report
cat > docs/V1_DOD_STATUS.md <<EOF
# v1.0 Definition of Done Status

**Last Updated**: $(date -u +"%Y-%m-%d %H:%M:%S UTC")
**Overall Status**: $STATUS_COLOR $STATUS
**Compliance**: $PASSED/$TOTAL criteria met ($PERCENTAGE%)

## Summary

- **Passed**: $PASSED
- **Failed**: $FAILED
- **Warnings**: $WARNINGS

## Gate Status

### Gate 1: Code Quality $(if echo "$VALIDATION_OUTPUT" | grep -q "1.1.*✓\|1.1.*⚠" && echo "$VALIDATION_OUTPUT" | grep -q "1.2.*⚠\|1.2.*✓" && echo "$VALIDATION_OUTPUT" | grep -q "1.3.*⚠\|1.3.*✓"; then echo "⚠️"; elif echo "$VALIDATION_OUTPUT" | grep -q "1.1.*✗"; then echo "❌"; else echo "✅"; fi)

- [$(if echo "$VALIDATION_OUTPUT" | grep -q "1.1.*✓\|1.1.*⚠"; then echo "x"; else echo " "; fi)] No unwrap()/expect() in production code
- [$(if echo "$VALIDATION_OUTPUT" | grep -q "1.2.*✓\|1.2.*⚠"; then echo "x"; else echo " "; fi)] No TODOs/placeholders in production code
- [$(if echo "$VALIDATION_OUTPUT" | grep -q "1.3.*✓\|1.3.*⚠"; then echo "x"; else echo " "; fi)] Proper error handling

**Status**: $(if echo "$VALIDATION_OUTPUT" | grep -q "1.1.*✗"; then echo "❌ FAILED - unwrap()/expect() found"; elif echo "$VALIDATION_OUTPUT" | grep -q "1.1.*⚠\|1.2.*⚠\|1.3.*⚠"; then echo "⚠️  WARNINGS"; else echo "✅ PASS"; fi)

### Gate 2: Compilation $(if echo "$VALIDATION_OUTPUT" | grep -q "All Rust crates compile successfully" && echo "$VALIDATION_OUTPUT" | grep -q "C library compiles successfully"; then echo "✅"; else echo "❌"; fi)

- [$(if echo "$VALIDATION_OUTPUT" | grep -q "All Rust crates compile successfully"; then echo "x"; else echo " "; fi)] All Rust crates compile
- [$(if echo "$VALIDATION_OUTPUT" | grep -q "C library compiles successfully"; then echo "x"; else echo " "; fi)] C library compiles

**Status**: $(if echo "$VALIDATION_OUTPUT" | grep -q "All Rust crates compile successfully" && echo "$VALIDATION_OUTPUT" | grep -q "C library compiles successfully"; then echo "✅ PASS"; else echo "❌ FAILED"; fi)

### Gate 3: Testing $(if echo "$VALIDATION_OUTPUT" | grep -q "All Rust tests pass" && echo "$VALIDATION_OUTPUT" | grep -q "All C tests pass" && echo "$VALIDATION_OUTPUT" | grep -q "Branchless tests pass"; then echo "✅"; else echo "❌"; fi)

- [$(if echo "$VALIDATION_OUTPUT" | grep -q "All Rust tests pass"; then echo "x"; else echo " "; fi)] All Rust tests pass
- [$(if echo "$VALIDATION_OUTPUT" | grep -q "All C tests pass"; then echo "x"; else echo " "; fi)] All C tests pass
- [$(if echo "$VALIDATION_OUTPUT" | grep -q "Branchless tests pass"; then echo "x"; else echo " "; fi)] Branchless tests pass

**Status**: $(if echo "$VALIDATION_OUTPUT" | grep -q "All Rust tests pass" && echo "$VALIDATION_OUTPUT" | grep -q "All C tests pass" && echo "$VALIDATION_OUTPUT" | grep -q "Branchless tests pass"; then echo "✅ PASS"; else echo "❌ FAILED"; fi)

### Gate 4: Linting $(if echo "$VALIDATION_OUTPUT" | grep -q "Clippy passes" && echo "$VALIDATION_OUTPUT" | grep -q "Code formatting correct"; then echo "✅"; else echo "❌"; fi)

- [$(if echo "$VALIDATION_OUTPUT" | grep -q "Clippy passes"; then echo "x"; else echo " "; fi)] Clippy passes
- [$(if echo "$VALIDATION_OUTPUT" | grep -q "Code formatting correct"; then echo "x"; else echo " "; fi)] Code formatting correct

**Status**: $(if echo "$VALIDATION_OUTPUT" | grep -q "Clippy passes" && echo "$VALIDATION_OUTPUT" | grep -q "Code formatting correct"; then echo "✅ PASS"; else echo "❌ FAILED"; fi)

### Gate 5: Performance $(if echo "$VALIDATION_OUTPUT" | grep -q "Hot path operations.*8 ticks"; then echo "✅"; else echo "❌"; fi)

- [$(if echo "$VALIDATION_OUTPUT" | grep -q "Hot path operations.*8 ticks"; then echo "x"; else echo " "; fi)] Hot path ≤8 ticks

**Status**: $(if echo "$VALIDATION_OUTPUT" | grep -q "Hot path operations.*8 ticks"; then echo "✅ PASS"; else echo "❌ FAILED"; fi)

### Gate 6: Integration $(if echo "$VALIDATION_OUTPUT" | grep -q "C↔Rust FFI integration verified" && echo "$VALIDATION_OUTPUT" | grep -q "Beat scheduler integration verified" && echo "$VALIDATION_OUTPUT" | grep -q "Lockchain integration verified"; then echo "✅"; else echo "❌"; fi)

- [$(if echo "$VALIDATION_OUTPUT" | grep -q "C↔Rust FFI integration verified"; then echo "x"; else echo " "; fi)] C↔Rust FFI verified
- [$(if echo "$VALIDATION_OUTPUT" | grep -q "Beat scheduler integration verified"; then echo "x"; else echo " "; fi)] Beat scheduler integrated
- [$(if echo "$VALIDATION_OUTPUT" | grep -q "Lockchain integration verified"; then echo "x"; else echo " "; fi)] Lockchain integrated

**Status**: $(if echo "$VALIDATION_OUTPUT" | grep -q "C↔Rust FFI integration verified" && echo "$VALIDATION_OUTPUT" | grep -q "Beat scheduler integration verified" && echo "$VALIDATION_OUTPUT" | grep -q "Lockchain integration verified"; then echo "✅ PASS"; else echo "❌ FAILED"; fi)

## Blockers

$(if [ "$FAILED" -gt 0 ]; then
    echo "### Critical Blockers (P0)"
    echo ""
    echo "$VALIDATION_OUTPUT" | grep "✗" | sed 's/.*✗/  - /' | head -10
    echo ""
else
    echo "None"
fi)

## Warnings

$(if [ "$WARNINGS" -gt 0 ]; then
    echo "### Non-Critical Issues (P1)"
    echo ""
    echo "$VALIDATION_OUTPUT" | grep "⚠" | sed 's/.*⚠/  - /' | head -10
    echo ""
else
    echo "None"
fi)

## Remediation Plan

$(if [ "$FAILED" -gt 0 ]; then
    cat <<REMEDIATION
1. **Fix unwrap()/expect() in production code**:
   - Review instances in core crates (knhk-etl, knhk-hot, knhk-lockchain)
   - Replace with proper error handling (Result types)
   - Document exceptions if legitimate

2. **Address TODOs/placeholders**:
   - Review and document legitimate TODOs (v1.1 items)
   - Fix or remove placeholders

REMEDIATION
else
    echo "All P0 criteria met. Proceed with v1.0 release."
fi)

## Next Steps

1. Run validation: \`./scripts/validate-v1.0-dod.sh\`
2. Review blockers and warnings
3. Fix critical issues
4. Re-run validation
5. Update this report

## Validation Command

\`\`\`bash
./scripts/validate-v1.0-dod.sh
\`\`\`

## Related Documents

- [Definition of Done](DEFINITION_OF_DONE.md)
- [v1.0 Requirements](v1-requirements.md)
- [Production Validation Report](V1-PRODUCTION-VALIDATION-REPORT.md)
EOF

echo "Generated DoD status report: docs/V1_DOD_STATUS.md"
echo "Status: $STATUS"
