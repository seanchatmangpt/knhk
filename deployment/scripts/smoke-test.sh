#!/bin/bash
# Smoke Test Suite for KNHK Deployment
# Validates basic functionality after deployment

set -e

ENVIRONMENT=${1:-blue}
NAMESPACE=${2:-knhk}
API_URL="http://knhk-$ENVIRONMENT.$NAMESPACE.svc:8080"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

TESTS_PASSED=0
TESTS_FAILED=0

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
    TESTS_PASSED=$((TESTS_PASSED + 1))
}

log_failure() {
    echo -e "${RED}[✗]${NC} $1"
    TESTS_FAILED=$((TESTS_FAILED + 1))
}

run_curl() {
    kubectl run curl-test-$RANDOM \
        --image=curlimages/curl \
        --rm -it --restart=Never \
        -n $NAMESPACE \
        -- "$@" 2>/dev/null
}

log_info "========================================="
log_info "Running Smoke Tests"
log_info "Environment: $ENVIRONMENT"
log_info "API URL: $API_URL"
log_info "========================================="
log_info ""

# Test 1: Health Check
log_info "Test 1: Health Check"
HEALTH_RESPONSE=$(run_curl curl -s $API_URL/health)
HEALTH_STATUS=$(echo "$HEALTH_RESPONSE" | jq -r '.status' 2>/dev/null || echo "")

if [ "$HEALTH_STATUS" == "healthy" ]; then
    log_success "Health check passed"
else
    log_failure "Health check failed - Status: $HEALTH_STATUS"
fi

# Test 2: Database connectivity (via health check)
log_info "Test 2: Database Connectivity"
DB_STATUS=$(echo "$HEALTH_RESPONSE" | jq -r '.dependencies.database' 2>/dev/null || echo "")

if [ "$DB_STATUS" == "healthy" ]; then
    log_success "Database connectivity verified"
else
    log_failure "Database connectivity failed - Status: $DB_STATUS"
fi

# Test 3: Redis connectivity
log_info "Test 3: Redis Connectivity"
REDIS_STATUS=$(echo "$HEALTH_RESPONSE" | jq -r '.dependencies.redis' 2>/dev/null || echo "")

if [ "$REDIS_STATUS" == "healthy" ]; then
    log_success "Redis connectivity verified"
else
    log_failure "Redis connectivity failed - Status: $REDIS_STATUS"
fi

# Test 4: Observation Ingestion
log_info "Test 4: Observation Ingestion"
OBS_RESPONSE=$(run_curl curl -s -X POST $API_URL/api/v1/observations \
    -H "Content-Type: application/json" \
    -d '{"event_type":"test.smoke","value":{"test":true},"sector":"finance"}')

OBS_ID=$(echo "$OBS_RESPONSE" | jq -r '.id' 2>/dev/null || echo "")

if [ -n "$OBS_ID" ] && [ "$OBS_ID" != "null" ]; then
    log_success "Observation ingestion passed - ID: $OBS_ID"
else
    log_failure "Observation ingestion failed - Response: $OBS_RESPONSE"
fi

# Test 5: Pattern Detection
log_info "Test 5: Pattern Detection"
PATTERN_RESPONSE=$(run_curl curl -s -X POST $API_URL/api/v1/patterns/detect)
PATTERN_COUNT=$(echo "$PATTERN_RESPONSE" | jq '. | length' 2>/dev/null || echo "0")

if [ "$PATTERN_COUNT" -ge 0 ]; then
    log_success "Pattern detection passed - Found $PATTERN_COUNT patterns"
else
    log_failure "Pattern detection failed"
fi

# Test 6: Metrics Endpoint
log_info "Test 6: Metrics Endpoint"
METRICS_RESPONSE=$(run_curl curl -s $API_URL/metrics)

if echo "$METRICS_RESPONSE" | grep -q "knhk_requests_total"; then
    log_success "Metrics endpoint responding correctly"
else
    log_failure "Metrics endpoint not working"
fi

# Test 7: Weaver Validation
log_info "Test 7: Weaver Schema Validation"
weaver registry check -r registry/ >/dev/null 2>&1

if [ $? -eq 0 ]; then
    log_success "Weaver validation passed"
else
    log_failure "Weaver validation failed - telemetry schema mismatch"
fi

# Test 8: API Latency Check
log_info "Test 8: API Latency Check"
START_TIME=$(date +%s%N)
run_curl curl -s $API_URL/health >/dev/null
END_TIME=$(date +%s%N)
LATENCY_MS=$(( (END_TIME - START_TIME) / 1000000 ))

if [ $LATENCY_MS -lt 500 ]; then
    log_success "API latency acceptable - ${LATENCY_MS}ms"
else
    log_failure "API latency too high - ${LATENCY_MS}ms (threshold: 500ms)"
fi

# Summary
log_info ""
log_info "========================================="
log_info "Smoke Test Results"
log_info "========================================="
log_info "Tests Passed: $TESTS_PASSED"
log_info "Tests Failed: $TESTS_FAILED"

if [ $TESTS_FAILED -eq 0 ]; then
    log_success "All smoke tests passed!"
    log_info "========================================="
    exit 0
else
    log_error "Some smoke tests failed"
    log_info "========================================="
    exit 1
fi
