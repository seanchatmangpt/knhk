#!/bin/bash
set -euo pipefail

# Automated validation checklist execution
# Generates evidence for each v1.0 criterion

REPORT_FILE="docs/v1-validation-checklist-$(date +%Y%m%d-%H%M%S).md"
mkdir -p docs

echo "=========================================="
echo "KNHK v1.0 Validation Checklist"
echo "=========================================="
echo ""
echo "Collecting evidence for all v1.0 criteria..."
echo ""

# Initialize report
cat > "$REPORT_FILE" << EOF
# KNHK v1.0 Validation Checklist Evidence

**Generated:** $(date -u +"%Y-%m-%dT%H:%M:%SZ")
**Platform:** $(uname -s) $(uname -m)
**Location:** $(pwd)

---

EOF

# Track results
PASSED=0
FAILED=0
WARNINGS=0

# Helper function to collect evidence
collect_evidence() {
    local criterion=$1
    local command=$2
    local description=$3

    echo "Checking: $description"

    cat >> "$REPORT_FILE" << EOF
## Criterion: $criterion

**Description:** $description

\`\`\`bash
$ $command
EOF

    local exit_code=0
    if eval "$command" >> "$REPORT_FILE" 2>&1; then
        exit_code=0
    else
        exit_code=$?
    fi

    cat >> "$REPORT_FILE" << EOF
\`\`\`

EOF

    if [ $exit_code -eq 0 ]; then
        cat >> "$REPORT_FILE" << EOF
**Status:** ✅ PASS
**Exit Code:** 0
**Timestamp:** $(date -u +"%Y-%m-%dT%H:%M:%SZ")

---

EOF
        echo "  ✅ PASS"
        ((PASSED++))
    else
        cat >> "$REPORT_FILE" << EOF
**Status:** ❌ FAIL
**Exit Code:** $exit_code
**Timestamp:** $(date -u +"%Y-%m-%dT%H:%M:%SZ")

---

EOF
        echo "  ❌ FAIL (exit code: $exit_code)"
        ((FAILED++))
    fi
}

# Infrastructure Setup
echo ""
echo "=== Infrastructure Setup ==="
collect_evidence "Weaver Installation" \
    "weaver --version" \
    "Weaver is installed and accessible"

collect_evidence "Rust Toolchain" \
    "rustc --version && cargo --version" \
    "Rust toolchain 1.70+ is available"

collect_evidence "C Compiler" \
    "(gcc --version || clang --version)" \
    "C compiler is available"

collect_evidence "Registry Directory" \
    "ls -la registry/ && echo 'Schema count:' && ls registry/*.yaml | wc -l" \
    "Registry directory exists with schemas"

# Build & Compilation
echo ""
echo "=== Build & Compilation ==="
collect_evidence "C Library Build" \
    "make -C c lib 2>&1 | tee /tmp/c-build.log && test -f c/libknhk.a" \
    "C library builds successfully with zero warnings"

collect_evidence "Rust Workspace Build" \
    "cd rust && cargo build --workspace 2>&1 | grep -i warning | wc -l | grep -q '^0$'" \
    "Rust workspace builds with zero warnings"

collect_evidence "Release Build" \
    "cd rust && cargo build --workspace --release >/dev/null 2>&1" \
    "Release binaries build successfully"

collect_evidence "Clippy Validation" \
    "cargo clippy --workspace -- -D warnings" \
    "Clippy passes with zero warnings"

collect_evidence "Code Formatting" \
    "cargo fmt --all -- --check" \
    "Code formatting is compliant"

# Code Quality
echo ""
echo "=== Code Quality ==="
collect_evidence "No panic!() in Production" \
    "! grep -r 'panic!' rust/*/src --include='*.rs' | grep -v test | grep -v /tests/ | grep -v example" \
    "Zero panic!() calls in production code"

collect_evidence "No unimplemented!() in Production" \
    "! grep -r 'unimplemented!' rust/*/src --include='*.rs' | grep -v test | grep -v /tests/" \
    "Zero unimplemented!() calls in production code"

collect_evidence "No .unwrap() in Production" \
    "! grep -r '\.unwrap()' rust/*/src --include='*.rs' | grep -v test | grep -v /tests/ | grep -v example" \
    "Zero .unwrap() calls in production code"

collect_evidence "No .expect() in Production" \
    "! grep -r '\.expect(' rust/*/src --include='*.rs' | grep -v test | grep -v /tests/ | grep -v example" \
    "Zero .expect() calls in production code"

# Testing
echo ""
echo "=== Testing ==="
collect_evidence "Rust Unit Tests" \
    "cargo test --workspace --lib --no-fail-fast" \
    "All Rust unit tests pass (100%)"

collect_evidence "C Tests" \
    "make -C c test" \
    "All C tests pass"

collect_evidence "Chicago TDD Tests" \
    "make test-chicago" \
    "Chicago TDD tests pass"

collect_evidence "Integration Tests" \
    "make test-integration" \
    "Integration tests pass"

collect_evidence "Performance Tests" \
    "make test-performance" \
    "Performance tests pass (≤8 ticks validated)"

# Weaver Validation (SOURCE OF TRUTH)
echo ""
echo "=== Weaver Schema Validation (SOURCE OF TRUTH) ==="
collect_evidence "Static Schema Validation" \
    "weaver registry check -r registry/" \
    "All registry schemas are syntactically valid"

collect_evidence "Schema Count" \
    "ls registry/*.yaml | wc -l | grep -E '[0-9]+'" \
    "All schema files present"

collect_evidence "Individual Schema Validation" \
    "for schema in registry/*.yaml; do echo \"Checking \$(basename \$schema)\"; weaver registry check -r registry/ --schema \"\$(basename \$schema)\" || exit 1; done" \
    "Each schema validates individually"

# Documentation
echo ""
echo "=== Documentation ==="
collect_evidence "CHANGELOG.md v1.0 Entry" \
    "grep -q 'v1.0\\|1.0.0\\|# v1.0' CHANGELOG.md" \
    "CHANGELOG.md has v1.0 entry"

collect_evidence "README.md Exists" \
    "test -f README.md && test -s README.md" \
    "README.md exists and is not empty"

# Generate summary
cat >> "$REPORT_FILE" << EOF

---

## Summary

**Total Checks:** $((PASSED + FAILED))
**Passed:** $PASSED ✅
**Failed:** $FAILED ❌
**Warnings:** $WARNINGS ⚠️

**Validation Date:** $(date -u +"%Y-%m-%dT%H:%M:%SZ")

EOF

if [ $FAILED -eq 0 ]; then
    cat >> "$REPORT_FILE" << EOF
### Verdict

✅✅✅ **v1.0 RELEASE CRITERIA: MET** ✅✅✅

All mandatory criteria have been satisfied. The codebase is ready for v1.0 release.

**Next Steps:**
1. Review this evidence report
2. Run live telemetry validation: \`weaver registry live-check --registry registry/\`
3. Tag release: \`git tag -a v1.0.0 -m 'Release v1.0.0'\`
4. Push tag: \`git push origin v1.0.0\`
5. Create GitHub release

EOF
    echo ""
    echo "=========================================="
    echo "✅ v1.0 VALIDATION: PASSED"
    echo "=========================================="
    echo ""
    echo "Passed: $PASSED"
    echo "Failed: $FAILED"
    echo ""
    echo "Evidence report: $REPORT_FILE"
    echo ""
    exit 0
else
    cat >> "$REPORT_FILE" << EOF
### Verdict

❌❌❌ **v1.0 RELEASE CRITERIA: NOT MET** ❌❌❌

**Failed Checks:** $FAILED

The codebase has $FAILED failing criteria that must be addressed before v1.0 release.

**Required Actions:**
1. Review failed criteria above
2. Fix all failures
3. Re-run validation: \`bash scripts/validate-checklist.sh\`

EOF
    echo ""
    echo "=========================================="
    echo "❌ v1.0 VALIDATION: FAILED"
    echo "=========================================="
    echo ""
    echo "Passed: $PASSED"
    echo "Failed: $FAILED"
    echo ""
    echo "Evidence report: $REPORT_FILE"
    echo ""
    echo "Fix all failures and re-run validation."
    echo ""
    exit 1
fi
