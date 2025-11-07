# KNHK 8-Beat v1.0 - OPA Policy Packs
# Open Policy Agent (OPA) rules for admission control
# Enforcement of DFLSS LAWs at runtime

package knhk.beat

# Import KNHK semantic conventions
import data.knhk.semconv

# ========================================
# LAW 1: Chatman Constant (τ ≤ 8 ticks)
# ========================================

# Check if operation complies with 8-tick hot path budget
budget_compliant {
    input.actual_ticks <= 8
}

# Deny admission if budget exceeded (park to W1)
deny_budget_violation[msg] {
    input.actual_ticks > 8
    msg := sprintf("Operation exceeds 8-tick budget: %d ticks (LAW violation)", [input.actual_ticks])
}

# ========================================
# LAW 2: Run Length ≤ 8 Rows (NROWS)
# ========================================

# Check if batch size fits in SIMD width
batch_size_compliant {
    input.batch_size <= 8
}

# Deny admission if batch too large
deny_batch_violation[msg] {
    input.batch_size > 8
    msg := sprintf("Batch size exceeds NROWS=8: %d rows (split required)", [input.batch_size])
}

# ========================================
# LAW 3: Park Rate ≤ 20%
# ========================================

# Check if park rate is within acceptable threshold
park_rate_threshold {
    input.park_rate <= 0.20
}

# Warning if park rate exceeds 15% (approaching limit)
warn_park_rate[msg] {
    input.park_rate > 0.15
    input.park_rate <= 0.20
    msg := sprintf("Park rate approaching limit: %.2f%% (threshold: 20%%)", [input.park_rate * 100])
}

# Deny if park rate exceeds 20%
deny_park_violation[msg] {
    input.park_rate > 0.20
    msg := sprintf("Park rate exceeds 20%% threshold: %.2f%%", [input.park_rate * 100])
}

# ========================================
# LAW 4: C1 Share < 2%
# ========================================

# Check if cold path usage is within budget
c1_threshold {
    input.c1_share < 0.02
}

# Warning if C1 exceeds 1.5%
warn_c1_share[msg] {
    input.c1_share >= 0.015
    input.c1_share < 0.02
    msg := sprintf("C1 cold path approaching limit: %.2f%% (threshold: 2%%)", [input.c1_share * 100])
}

# Deny if C1 exceeds 2%
deny_c1_violation[msg] {
    input.c1_share >= 0.02
    msg := sprintf("C1 cold path exceeds 2%% threshold: %.2f%%", [input.c1_share * 100])
}

# ========================================
# LAW 5: L1 Cache Ready
# ========================================

# Check if data is L1 cache resident
l1_ready {
    input.l1_ready == true
}

# Check if L1 hit rate meets 95% threshold
l1_hit_rate_compliant {
    input.l1_hit_rate >= 0.95
}

# Deny if L1 cache miss (park to W1)
deny_l1_miss[msg] {
    input.l1_ready == false
    msg := "L1 cache miss detected (park to W1)"
}

# ========================================
# LAW 6: 100% Receipt Coverage
# ========================================

# Check if receipt exists for operation
receipt_present {
    input.receipt != null
    input.receipt.span_id != ""
}

# Deny if receipt missing
deny_missing_receipt[msg] {
    not receipt_present
    msg := "Operation missing receipt (audit violation)"
}

# ========================================
# Admission Decision (R1 Hot Path)
# ========================================

# Admit to R1 hot path if all criteria met
admit_r1 {
    budget_compliant
    batch_size_compliant
    l1_ready
    l1_hit_rate_compliant
    receipt_present
    park_rate_threshold
    c1_threshold
}

# ========================================
# Park Decision (W1 Warm Path)
# ========================================

# Park to W1 if budget exceeded
park_to_w1 {
    not budget_compliant
}

# Park to W1 if L1 cache miss
park_to_w1 {
    not l1_ready
}

# Park to W1 if batch too large (split required)
park_to_w1 {
    not batch_size_compliant
}

# ========================================
# Escalate to C1 (Cold Path)
# ========================================

# Escalate to C1 if park rate budget exceeded
escalate_to_c1 {
    not park_rate_threshold
}

# Escalate to C1 if L1 hit rate persistently low
escalate_to_c1 {
    input.l1_hit_rate < 0.80
}

# ========================================
# CONSTRUCT8 Exception
# ========================================

# CONSTRUCT8 always parks to W1 (documented exception)
construct8_exception {
    input.operation == "K_CONSTRUCT8"
}

# Override: CONSTRUCT8 bypasses 8-tick budget check
admit_construct8 {
    construct8_exception
    receipt_present
}

# Route CONSTRUCT8 to W1 warm path
park_construct8 {
    construct8_exception
}

# ========================================
# Policy Enforcement
# ========================================

# Final admission decision
default allow = false

allow {
    admit_r1
}

allow {
    admit_construct8
}

# Collect all denials
deny[msg] {
    deny_budget_violation[msg]
}

deny[msg] {
    deny_batch_violation[msg]
}

deny[msg] {
    deny_park_violation[msg]
}

deny[msg] {
    deny_c1_violation[msg]
}

deny[msg] {
    deny_l1_miss[msg]
}

deny[msg] {
    deny_missing_receipt[msg]
}

# Collect all warnings
warnings[msg] {
    warn_park_rate[msg]
}

warnings[msg] {
    warn_c1_share[msg]
}

# ========================================
# Metrics for Dashboards
# ========================================

# Export policy decision metrics
metrics := {
    "admit_r1": admit_r1,
    "park_w1": park_to_w1,
    "escalate_c1": escalate_to_c1,
    "budget_compliant": budget_compliant,
    "park_rate": input.park_rate,
    "c1_share": input.c1_share,
    "l1_hit_rate": input.l1_hit_rate,
    "receipt_coverage": receipt_present
}

# ========================================
# Audit Queries
# ========================================

# Find operations violating Chatman Constant
audit_budget_violations[op] {
    op := input.operations[_]
    op.actual_ticks > 8
}

# Find operations missing receipts
audit_missing_receipts[op] {
    op := input.operations[_]
    not op.receipt
}

# Find operations parked to W1
audit_parked_operations[op] {
    op := input.operations[_]
    op.path == "W1"
}

# ========================================
# DFLSS Acceptance Criteria (Section 17)
# ========================================

dflss_acceptance := {
    "beat_stable": input.beat_drift == 0,
    "r1_p99_latency": input.r1_p99 <= 2.0,
    "park_rate_ok": input.park_rate <= 0.20,
    "c1_share_ok": input.c1_share < 0.02,
    "receipts_100": input.receipt_coverage == 1.0,
    "dashboards_green": input.dashboards_status == "green",
    "sre_signoff": input.sre_approved == true,
    "finance_signoff": input.finance_approved == true
}

# All criteria must pass for v1.0 release
v1_release_approved {
    dflss_acceptance.beat_stable
    dflss_acceptance.r1_p99_latency
    dflss_acceptance.park_rate_ok
    dflss_acceptance.c1_share_ok
    dflss_acceptance.receipts_100
    dflss_acceptance.dashboards_green
    dflss_acceptance.sre_signoff
    dflss_acceptance.finance_signoff
}
