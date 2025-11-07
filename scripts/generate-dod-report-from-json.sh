#!/bin/bash
set -euo pipefail

# Generate Definition of Done v1 reports from validation JSON

cd "$(dirname "$0")/.."

JSON_REPORT="reports/dod-v1-validation.json"
MARKDOWN_REPORT="docs/V1-DOD-VALIDATION-REPORT.md"
PROGRESS_TRACKER="docs/V1-DOD-PROGRESS.md"

if [ ! -f "${JSON_REPORT}" ]; then
    echo "Error: JSON report not found. Run scripts/validate-dod-v1.sh first."
    exit 1
fi

# Extract summary using python (most reliable) or fallback to grep
if command -v python3 &> /dev/null; then
    TOTAL=$(python3 -c "import json; print(json.load(open('${JSON_REPORT}'))['summary']['total_criteria'])" 2>/dev/null || echo "0")
    PASSED=$(python3 -c "import json; print(json.load(open('${JSON_REPORT}'))['summary']['passed'])" 2>/dev/null || echo "0")
    FAILED=$(python3 -c "import json; print(json.load(open('${JSON_REPORT}'))['summary']['failed'])" 2>/dev/null || echo "0")
    WARNINGS=$(python3 -c "import json; print(json.load(open('${JSON_REPORT}'))['summary']['warnings'])" 2>/dev/null || echo "0")
    COMPLETION=$(python3 -c "import json; print(json.load(open('${JSON_REPORT}'))['summary']['completion_percentage'])" 2>/dev/null || echo "0.00")
    TIMESTAMP=$(python3 -c "import json; print(json.load(open('${JSON_REPORT}'))['timestamp'])" 2>/dev/null || echo "unknown")
elif command -v jq &> /dev/null && jq --version &> /dev/null; then
    TOTAL=$(jq -r '.summary.total_criteria' "${JSON_REPORT}" 2>/dev/null || echo "0")
    PASSED=$(jq -r '.summary.passed' "${JSON_REPORT}" 2>/dev/null || echo "0")
    FAILED=$(jq -r '.summary.failed' "${JSON_REPORT}" 2>/dev/null || echo "0")
    WARNINGS=$(jq -r '.summary.warnings' "${JSON_REPORT}" 2>/dev/null || echo "0")
    COMPLETION=$(jq -r '.summary.completion_percentage' "${JSON_REPORT}" 2>/dev/null || echo "0.00")
    TIMESTAMP=$(jq -r '.timestamp' "${JSON_REPORT}" 2>/dev/null || echo "unknown")
else
    # Fallback if neither available
    TOTAL=$(grep -o '"total_criteria":[0-9]*' "${JSON_REPORT}" | head -1 | grep -o '[0-9]*')
    PASSED=$(grep -o '"passed":[0-9]*' "${JSON_REPORT}" | head -1 | grep -o '[0-9]*')
    FAILED=$(grep -o '"failed":[0-9]*' "${JSON_REPORT}" | head -1 | grep -o '[0-9]*')
    WARNINGS=$(grep -o '"warnings":[0-9]*' "${JSON_REPORT}" | head -1 | grep -o '[0-9]*')
    COMPLETION=$(grep -o '"completion_percentage":[0-9.]*' "${JSON_REPORT}" | head -1 | grep -o '[0-9.]*')
    TIMESTAMP=$(grep -o '"timestamp":"[^"]*"' "${JSON_REPORT}" | head -1 | sed 's/"timestamp":"\(.*\)"/\1/')
fi

# Determine status
if [ "${FAILED}" -eq 0 ]; then
    STATUS_ICON="✅"
    STATUS_TEXT="PASSED"
else
    STATUS_ICON="❌"
    STATUS_TEXT="FAILED"
fi

# Generate Markdown Report
cat > "${MARKDOWN_REPORT}" <<EOF
# KNHK Definition of Done v1.0 Validation Report

**Generated:** ${TIMESTAMP}  
**Status:** ${STATUS_ICON} ${STATUS_TEXT}  
**Completion:** ${COMPLETION}%

---

## Executive Summary

- **Total Criteria:** ${TOTAL}
- **Passed:** ${PASSED} ✅
- **Failed:** ${FAILED} ❌
- **Warnings:** ${WARNINGS} ⚠️

---

## Core Team Standards (11 items)

EOF

# Core criteria - using python if available, otherwise skip detailed breakdown
if command -v python3 &> /dev/null; then
    for criterion in core_compilation core_no_unwrap core_trait_compatibility core_backward_compatibility core_tests_pass core_no_linting core_error_handling core_async_sync core_no_false_positives core_performance core_otel_validation; do
        status=$(python3 -c "import json; d=json.load(open('${JSON_REPORT}')); print(d.get('criteria', {}).get('${criterion}', {}).get('status', 'unknown'))" 2>/dev/null || echo "unknown")
        message=$(python3 -c "import json; d=json.load(open('${JSON_REPORT}')); print(d.get('criteria', {}).get('${criterion}', {}).get('message', 'Not checked'))" 2>/dev/null || echo "Not checked")
        
        case "$status" in
            "passed")
                icon="🟢 ✅"
                ;;
            "failed")
                icon="🔴 ❌"
                ;;
            "warning")
                icon="🟡 ⚠️"
                ;;
            *)
                icon="⚪"
                ;;
        esac
        
        echo "- ${icon} **${criterion}**: ${message}" >> "${MARKDOWN_REPORT}"
    done
else
    # Fallback: extract from JSON manually
    echo "- ⚪ **Note**: Install jq for detailed criteria breakdown" >> "${MARKDOWN_REPORT}"
fi

cat >> "${MARKDOWN_REPORT}" <<EOF

---

## Extended Criteria (8 sections)

EOF

# Extended criteria
if command -v python3 &> /dev/null; then
    for criterion in ext_code_quality ext_documentation ext_performance ext_integration ext_security ext_testing ext_build_system ext_knhk_specific; do
        status=$(python3 -c "import json; d=json.load(open('${JSON_REPORT}')); print(d.get('criteria', {}).get('${criterion}', {}).get('status', 'unknown'))" 2>/dev/null || echo "unknown")
        message=$(python3 -c "import json; d=json.load(open('${JSON_REPORT}')); print(d.get('criteria', {}).get('${criterion}', {}).get('message', 'Not checked'))" 2>/dev/null || echo "Not checked")
        
        case "$status" in
            "passed")
                icon="🟢 ✅"
                ;;
            "failed")
                icon="🔴 ❌"
                ;;
            "warning")
                icon="🟡 ⚠️"
                ;;
            *)
                icon="⚪"
                ;;
        esac
        
        echo "- ${icon} **${criterion}**: ${message}" >> "${MARKDOWN_REPORT}"
    done
fi

cat >> "${MARKDOWN_REPORT}" <<EOF

---

## Blockers

EOF

# List blockers
if command -v python3 &> /dev/null && [ "${FAILED}" -gt 0 ]; then
    python3 <<EOF >> "${MARKDOWN_REPORT}" 2>/dev/null || echo "- None" >> "${MARKDOWN_REPORT}"
import json
with open('${JSON_REPORT}') as f:
    data = json.load(f)
    for key, value in data.get('criteria', {}).items():
        if value.get('status') == 'failed':
            print(f"- **{key}**: {value.get('message', 'Unknown error')}")
EOF
else
    if [ "${FAILED}" -gt 0 ]; then
        echo "- ${FAILED} criteria failed (see JSON report for details)" >> "${MARKDOWN_REPORT}"
    else
        echo "- None" >> "${MARKDOWN_REPORT}"
    fi
fi

cat >> "${MARKDOWN_REPORT}" <<EOF

---

## Remediation Steps

1. Fix all failed criteria (${FAILED} items)
2. Address warnings (${WARNINGS} items)
3. Re-run validation: \`./scripts/validate-dod-v1.sh\`
4. Verify Weaver live-check: \`weaver registry live-check --registry registry/\`

---

## Next Steps

- Review failed criteria and fix issues
- Address warnings for production readiness
- Run performance benchmarks
- Execute Weaver live-check during runtime

EOF

# Update Progress Tracker (simplified version)
cat > "${PROGRESS_TRACKER}" <<EOF
# KNHK Definition of Done v1.0 Progress Tracker

**Last Updated:** ${TIMESTAMP}  
**Overall Status:** ${STATUS_ICON} ${STATUS_TEXT}  
**Completion:** ${COMPLETION}%

---

## Overall Progress

- **Total Criteria:** ${TOTAL}
- **Completed:** ${PASSED} (${COMPLETION}%)
- **In Progress:** ${WARNINGS}
- **Blocked:** ${FAILED}

---

## How to Update

Run the validation script to update this tracker:

\`\`\`bash
./scripts/validate-dod-v1.sh
bash scripts/generate-dod-report-from-json.sh
\`\`\`

For detailed criteria breakdown, see: \`docs/V1-DOD-VALIDATION-REPORT.md\`

EOF

echo "✅ Generated reports:"
echo "  - ${MARKDOWN_REPORT}"
echo "  - ${PROGRESS_TRACKER}"

