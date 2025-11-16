#!/bin/bash
# Blue-Green Deployment Script for KNHK
# Zero-downtime deployment with automated rollback

set -e

# Configuration
NEW_VERSION=$1
NAMESPACE=${2:-knhk}
HEALTH_CHECK_RETRIES=30
SMOKE_TEST_TIMEOUT=300

if [ -z "$NEW_VERSION" ]; then
    echo "Usage: $0 <version> [namespace]"
    echo "Example: $0 v1.3.0 knhk"
    exit 1
fi

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Determine current active environment
log_info "Determining current active environment..."
CURRENT_COLOR=$(kubectl get vs knhk-routing -n $NAMESPACE -o json | jq -r '.spec.http[0].route[0].destination.subset' 2>/dev/null || echo "blue")

if [ "$CURRENT_COLOR" == "blue" ]; then
    NEW_COLOR="green"
    OLD_COLOR="blue"
else
    NEW_COLOR="blue"
    OLD_COLOR="green"
fi

log_info "Current environment: $OLD_COLOR"
log_info "Deploying to: $NEW_COLOR"
log_info "Version: $NEW_VERSION"

# Step 1: Deploy new version to inactive environment
log_info "Step 1: Deploying $NEW_VERSION to $NEW_COLOR environment..."
kubectl set image deployment/knhk-$NEW_COLOR \
    knhk-closed-loop=knhk-closed-loop:$NEW_VERSION \
    -n $NAMESPACE

# Step 2: Wait for rollout to complete
log_info "Step 2: Waiting for rollout to complete..."
kubectl rollout status deployment/knhk-$NEW_COLOR \
    -n $NAMESPACE \
    --timeout=5m

if [ $? -ne 0 ]; then
    log_error "Rollout failed - deployment did not become ready"
    exit 1
fi

# Step 3: Health check
log_info "Step 3: Running health checks on $NEW_COLOR environment..."
SERVICE_URL="http://knhk-$NEW_COLOR.$NAMESPACE.svc:8080"

for i in $(seq 1 $HEALTH_CHECK_RETRIES); do
    HEALTH_STATUS=$(kubectl run curl-test-$RANDOM \
        --image=curlimages/curl \
        --rm -it --restart=Never \
        -n $NAMESPACE \
        -- curl -s $SERVICE_URL/health | jq -r '.status' 2>/dev/null || echo "unhealthy")

    if [ "$HEALTH_STATUS" == "healthy" ]; then
        log_info "Health check passed (attempt $i/$HEALTH_CHECK_RETRIES)"
        break
    fi

    log_warn "Health check failed (attempt $i/$HEALTH_CHECK_RETRIES), retrying..."
    sleep 5

    if [ $i -eq $HEALTH_CHECK_RETRIES ]; then
        log_error "Health checks failed after $HEALTH_CHECK_RETRIES attempts"
        exit 1
    fi
done

# Step 4: Run smoke tests
log_info "Step 4: Running smoke tests on $NEW_COLOR environment..."
./deployment/scripts/smoke-test.sh $NEW_COLOR $NAMESPACE

if [ $? -ne 0 ]; then
    log_error "Smoke tests failed - aborting deployment"
    log_info "New version is deployed to $NEW_COLOR but traffic not switched"
    log_info "Manual investigation required or run rollback script"
    exit 1
fi

log_info "Smoke tests passed"

# Step 5: Shift traffic to new environment
log_info "Step 5: Shifting traffic from $OLD_COLOR to $NEW_COLOR..."

kubectl patch vs knhk-routing -n $NAMESPACE --type merge -p "
spec:
  http:
  - route:
    - destination:
        host: knhk-service
        subset: $NEW_COLOR
      weight: 100
"

log_info "Traffic switched to $NEW_COLOR"

# Step 6: Monitor for errors
log_info "Step 6: Monitoring deployment for 5 minutes..."
MONITOR_DURATION=300
MONITOR_INTERVAL=30
ELAPSED=0

while [ $ELAPSED -lt $MONITOR_DURATION ]; do
    # Check error rate
    ERROR_RATE=$(kubectl run prometheus-query-$RANDOM \
        --image=curlimages/curl \
        --rm -it --restart=Never \
        -n $NAMESPACE \
        -- curl -s 'http://prometheus:9090/api/v1/query?query=rate(knhk_requests_total{status="error"}[5m])' | \
        jq -r '.data.result[0].value[1]' 2>/dev/null || echo "0")

    # Check if error rate exceeds threshold (1%)
    if (( $(echo "$ERROR_RATE > 0.01" | bc -l) )); then
        log_error "Error rate too high: $ERROR_RATE (threshold: 0.01)"
        log_error "Initiating automatic rollback..."
        ./deployment/scripts/rollback.sh
        exit 1
    fi

    # Check p95 latency
    P95_LATENCY=$(kubectl run prometheus-query-$RANDOM \
        --image=curlimages/curl \
        --rm -it --restart=Never \
        -n $NAMESPACE \
        -- curl -s 'http://prometheus:9090/api/v1/query?query=histogram_quantile(0.95,rate(knhk_request_duration_ms_bucket[5m]))' | \
        jq -r '.data.result[0].value[1]' 2>/dev/null || echo "0")

    if (( $(echo "$P95_LATENCY > 200" | bc -l) )); then
        log_warn "High latency detected: ${P95_LATENCY}ms (threshold: 200ms)"
    fi

    log_info "Monitoring... ${ELAPSED}s/${MONITOR_DURATION}s (Error rate: $ERROR_RATE, p95: ${P95_LATENCY}ms)"

    sleep $MONITOR_INTERVAL
    ELAPSED=$((ELAPSED + MONITOR_INTERVAL))
done

# Step 7: Final validation
log_info "Step 7: Running final validation..."

# Verify Weaver validation
log_info "Running Weaver schema validation..."
weaver registry check -r registry/

if [ $? -ne 0 ]; then
    log_error "Weaver validation failed - telemetry schema mismatch"
    log_warn "Deployment succeeded but schema validation failed"
    log_warn "Manual review required"
fi

# Verify receipt chain integrity
log_info "Verifying receipt chain integrity..."
kubectl exec -it deploy/knhk-closed-loop -n $NAMESPACE -- \
    knhk receipts verify-chain --limit 100

if [ $? -ne 0 ]; then
    log_error "Receipt chain verification failed"
    log_error "CRITICAL: Data integrity issue detected"
    log_error "Escalate immediately to Engineering Manager"
fi

# Step 8: Create deployment record
log_info "Step 8: Recording deployment..."
kubectl annotate deployment/knhk-$NEW_COLOR \
    deployment.knhk.io/deployed-at="$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    deployment.knhk.io/deployed-version="$NEW_VERSION" \
    deployment.knhk.io/deployed-by="$USER" \
    -n $NAMESPACE

# Audit log
kubectl exec -it deploy/knhk-closed-loop -n $NAMESPACE -- \
    knhk audit log \
    --event-type="deployment" \
    --actor="$USER" \
    --action="blue_green_deploy" \
    --resource="deployment/$NEW_COLOR" \
    --metadata="{\"version\":\"$NEW_VERSION\",\"timestamp\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\"}"

log_info ""
log_info "========================================="
log_info "✅ Deployment Successful!"
log_info "========================================="
log_info "Version: $NEW_VERSION"
log_info "Active Environment: $NEW_COLOR"
log_info "Old Environment: $OLD_COLOR (still running, can rollback)"
log_info ""
log_info "Next steps:"
log_info "1. Monitor dashboards: http://grafana.company.com/d/knhk-overview"
log_info "2. Check alerts: http://prometheus.company.com/alerts"
log_info "3. If issues occur, run: ./deployment/scripts/rollback.sh"
log_info "4. After 24h stability, scale down $OLD_COLOR environment"
log_info "========================================="
