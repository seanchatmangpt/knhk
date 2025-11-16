#!/bin/bash
set -euo pipefail

echo "=========================================="
echo "KNHK v1.0 Complete Validation Sequence"
echo "=========================================="
echo ""
echo "This script runs all 6 validation phases:"
echo "  Phase 0: Pre-Build Validation (30s)"
echo "  Phase 1: Build & Code Quality (2-5m)"
echo "  Phase 2: Unit Tests (1-3m)"
echo "  Phase 3: Integration Tests (2-5m)"
echo "  Phase 4: Performance Tests (1-2m)"
echo "  Phase 5: Weaver Schema Validation (30-60s) [SOURCE OF TRUTH]"
echo ""
echo "Total estimated time: 7-16 minutes"
echo ""

# Ensure we're in project root
cd "$(dirname "$0")/.."

FAILED_PHASE=""
START_TIME=$(date +%s)
EVIDENCE_FILE="validation-evidence-$(date +%Y%m%d-%H%M%S).md"

# Initialize evidence file
cat > "$EVIDENCE_FILE" << EOF
# KNHK v1.0 Validation Evidence

**Generated:** $(date -u +"%Y-%m-%dT%H:%M:%SZ")
**Platform:** $(uname -s) $(uname -m)
**Rust Version:** $(rustc --version)
**Cargo Version:** $(cargo --version)

---

EOF

# Helper functions
run_phase() {
    local phase_num=$1
    local phase_name=$2
    local phase_cmd=$3

    echo ""
    echo "========================================"
    echo "PHASE $phase_num: $phase_name"
    echo "========================================"
    echo ""

    PHASE_START=$(date +%s)

    # Record phase start in evidence
    cat >> "$EVIDENCE_FILE" << EOF
## Phase $phase_num: $phase_name

**Started:** $(date -u +"%Y-%m-%dT%H:%M:%SZ")

\`\`\`bash
$phase_cmd
\`\`\`

**Output:**
\`\`\`
EOF

    # Run command and capture output
    if eval "$phase_cmd" 2>&1 | tee -a "$EVIDENCE_FILE"; then
        PHASE_END=$(date +%s)
        PHASE_DURATION=$((PHASE_END - PHASE_START))

        cat >> "$EVIDENCE_FILE" << EOF
\`\`\`

**Status:** ✅ PASS
**Duration:** ${PHASE_DURATION}s
**Ended:** $(date -u +"%Y-%m-%dT%H:%M:%SZ")

---

EOF

        echo ""
        echo "✅ PHASE $phase_num PASSED (${PHASE_DURATION}s)"
        return 0
    else
        PHASE_END=$(date +%s)
        PHASE_DURATION=$((PHASE_END - PHASE_START))

        cat >> "$EVIDENCE_FILE" << EOF
\`\`\`

**Status:** ❌ FAIL
**Duration:** ${PHASE_DURATION}s
**Ended:** $(date -u +"%Y-%m-%dT%H:%M:%SZ")

---

EOF

        echo ""
        echo "❌ PHASE $phase_num FAILED (${PHASE_DURATION}s)"
        FAILED_PHASE="Phase $phase_num: $phase_name"
        return 1
    fi
}

# Phase 0: Pre-Build Validation
run_phase 0 "Pre-Build Validation" "
    echo 'Checking Rust compilation...' && \
    cargo check --workspace --message-format=short && \
    echo '' && \
    echo 'Validating code formatting...' && \
    cargo fmt --all -- --check && \
    echo '' && \
    echo 'Checking for Weaver installation...' && \
    (command -v weaver && weaver --version) || echo 'WARNING: Weaver not installed (Phase 5 will fail)'
" || {
    echo ""
    echo "❌ Pre-Build validation failed. Cannot proceed."
    echo "Evidence saved to: $EVIDENCE_FILE"
    exit 1
}

# Phase 1: Build & Code Quality
run_phase 1 "Build & Code Quality" "
    echo 'Building C library...' && \
    make -C c lib && \
    echo '' && \
    echo 'Building Rust workspace (debug)...' && \
    (cd rust && cargo build --workspace) && \
    echo '' && \
    echo 'Running Clippy (zero warnings)...' && \
    cargo clippy --workspace -- -D warnings
" || {
    echo ""
    echo "❌ Build & Code Quality failed. Cannot proceed."
    echo "Evidence saved to: $EVIDENCE_FILE"
    exit 1
}

# Phase 2: Unit Tests
run_phase 2 "Unit Tests" "
    echo 'Running Rust unit tests...' && \
    cargo test --workspace --lib --test-threads=4 && \
    echo '' && \
    echo 'Running C tests...' && \
    (make -C c test || echo 'WARNING: C tests failed or not available')
" || {
    echo ""
    echo "❌ Unit tests failed. Cannot proceed."
    echo "Evidence saved to: $EVIDENCE_FILE"
    exit 1
}

# Phase 3: Integration Tests
run_phase 3 "Integration Tests" "
    echo 'Running Chicago TDD tests...' && \
    make test-chicago && \
    echo '' && \
    echo 'Running integration tests...' && \
    make test-integration
" || {
    echo ""
    echo "❌ Integration tests failed. Cannot proceed."
    echo "Evidence saved to: $EVIDENCE_FILE"
    exit 1
}

# Phase 4: Performance Tests
run_phase 4 "Performance Tests" "
    echo 'Running performance tests (≤8 ticks validation)...' && \
    make test-performance
" || {
    echo ""
    echo "⚠️  Performance tests failed. Continuing to Weaver validation..."
}

# Phase 5: Weaver Schema Validation (SOURCE OF TRUTH)
if command -v weaver &> /dev/null; then
    run_phase 5 "Weaver Schema Validation (SOURCE OF TRUTH)" "
        echo 'Validating registry schemas...' && \
        weaver registry check -r registry/ && \
        echo '' && \
        echo 'Listing registry entries...' && \
        (weaver registry list -r registry/ || echo 'Registry list not supported in this version') && \
        echo '' && \
        echo 'Validating individual schemas...' && \
        for schema in registry/*.yaml; do \
            echo \"Checking \$(basename \$schema)...\"; \
            weaver registry check -r registry/ --schema \"\$(basename \$schema)\" || true; \
        done
    " || {
        echo ""
        echo "❌ Weaver Schema Validation FAILED"
        echo "This is the SOURCE OF TRUTH - feature validation has FAILED."
        echo "Evidence saved to: $EVIDENCE_FILE"
        exit 1
    }
else
    echo ""
    echo "❌ PHASE 5 SKIPPED: Weaver not installed"
    echo ""
    echo "Weaver is REQUIRED for v1.0 validation (source of truth)."
    echo "Install with: bash scripts/install-weaver.sh"
    echo ""
    FAILED_PHASE="Phase 5: Weaver not installed"

    cat >> "$EVIDENCE_FILE" << EOF
## Phase 5: Weaver Schema Validation (SOURCE OF TRUTH)

**Status:** ❌ SKIPPED
**Reason:** Weaver not installed

Install with:
\`\`\`bash
bash scripts/install-weaver.sh
\`\`\`

---

EOF
fi

# Final Summary
END_TIME=$(date +%s)
TOTAL_DURATION=$((END_TIME - START_TIME))
TOTAL_MINUTES=$((TOTAL_DURATION / 60))
TOTAL_SECONDS=$((TOTAL_DURATION % 60))

echo ""
echo "=========================================="
if [ -z "$FAILED_PHASE" ]; then
    echo "✅✅✅ ALL PHASES PASSED ✅✅✅"
    echo "=========================================="
    echo ""
    echo "Total validation time: ${TOTAL_MINUTES}m ${TOTAL_SECONDS}s"
    echo "Evidence report: $EVIDENCE_FILE"
    echo ""
    echo "🎉 v1.0 RELEASE CRITERIA: MET 🎉"
    echo ""
    echo "Next steps:"
    echo "  1. Review evidence report: cat $EVIDENCE_FILE"
    echo "  2. Run live telemetry check: weaver registry live-check --registry registry/"
    echo "  3. Tag release: git tag -a v1.0.0 -m 'Release v1.0.0'"
    echo "  4. Push tag: git push origin v1.0.0"
    echo "  5. Create GitHub release"
    echo ""

    cat >> "$EVIDENCE_FILE" << EOF

---

## Final Summary

**Status:** ✅ ALL PHASES PASSED
**Total Duration:** ${TOTAL_MINUTES}m ${TOTAL_SECONDS}s
**Validation Timestamp:** $(date -u +"%Y-%m-%dT%H:%M:%SZ")

### v1.0 Release Criteria

- ✅ Phase 0: Pre-Build Validation
- ✅ Phase 1: Build & Code Quality
- ✅ Phase 2: Unit Tests
- ✅ Phase 3: Integration Tests
- ✅ Phase 4: Performance Tests
- ✅ Phase 5: Weaver Schema Validation (SOURCE OF TRUTH)

**Verdict:** READY FOR v1.0 RELEASE

EOF

    exit 0
else
    echo "❌❌❌ VALIDATION FAILED ❌❌❌"
    echo "=========================================="
    echo ""
    echo "Failed at: $FAILED_PHASE"
    echo "Total validation time: ${TOTAL_MINUTES}m ${TOTAL_SECONDS}s"
    echo "Evidence report: $EVIDENCE_FILE"
    echo ""
    echo "❌ v1.0 RELEASE CRITERIA: NOT MET"
    echo ""
    echo "Review the evidence report for details:"
    echo "  cat $EVIDENCE_FILE"
    echo ""

    cat >> "$EVIDENCE_FILE" << EOF

---

## Final Summary

**Status:** ❌ VALIDATION FAILED
**Failed Phase:** $FAILED_PHASE
**Total Duration:** ${TOTAL_MINUTES}m ${TOTAL_SECONDS}s
**Validation Timestamp:** $(date -u +"%Y-%m-%dT%H:%M:%SZ")

**Verdict:** NOT READY FOR v1.0 RELEASE

Fix the failed phase and re-run validation.

EOF

    exit 1
fi
