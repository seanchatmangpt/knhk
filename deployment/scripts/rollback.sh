#!/bin/bash
# Emergency Rollback Script for KNHK
# Switches traffic back to previous environment

set -e

NAMESPACE=${1:-knhk}

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# Determine current active environment
CURRENT_COLOR=$(kubectl get vs knhk-routing -n $NAMESPACE -o json | jq -r '.spec.http[0].route[0].destination.subset')

if [ "$CURRENT_COLOR" == "blue" ]; then
    ROLLBACK_COLOR="green"
else
    ROLLBACK_COLOR="blue"
fi

log_warn "========================================="
log_warn "⚠️  INITIATING EMERGENCY ROLLBACK"
log_warn "========================================="
log_warn "Current environment: $CURRENT_COLOR"
log_warn "Rolling back to: $ROLLBACK_COLOR"
log_warn ""
read -p "Are you sure? Type 'ROLLBACK' to confirm: " CONFIRMATION

if [ "$CONFIRMATION" != "ROLLBACK" ]; then
    log_info "Rollback cancelled"
    exit 0
fi

# Switch traffic immediately
log_info "Switching traffic to $ROLLBACK_COLOR..."
kubectl patch vs knhk-routing -n $NAMESPACE --type merge -p "
spec:
  http:
  - route:
    - destination:
        host: knhk-service
        subset: $ROLLBACK_COLOR
      weight: 100
"

log_info "Traffic switched to $ROLLBACK_COLOR"

# Verify health
log_info "Verifying health of $ROLLBACK_COLOR environment..."
sleep 5

SERVICE_URL="http://knhk-$ROLLBACK_COLOR.$NAMESPACE.svc:8080"
HEALTH_STATUS=$(kubectl run curl-test-$RANDOM \
    --image=curlimages/curl \
    --rm -it --restart=Never \
    -n $NAMESPACE \
    -- curl -s $SERVICE_URL/health | jq -r '.status' || echo "unhealthy")

if [ "$HEALTH_STATUS" != "healthy" ]; then
    log_error "Rollback environment is unhealthy!"
    log_error "Manual intervention required immediately"
    exit 1
fi

# Audit log
kubectl exec -it deploy/knhk-closed-loop -n $NAMESPACE -- \
    knhk audit log \
    --event-type="deployment" \
    --actor="$USER" \
    --action="rollback" \
    --resource="deployment/$ROLLBACK_COLOR" \
    --outcome="success" \
    --reason="Emergency rollback from $CURRENT_COLOR"

log_info ""
log_info "========================================="
log_info "✅ Rollback Successful"
log_info "========================================="
log_info "Active environment: $ROLLBACK_COLOR"
log_info "Failed environment: $CURRENT_COLOR (still deployed)"
log_info ""
log_info "Next steps:"
log_info "1. Investigate failure in $CURRENT_COLOR environment"
log_info "2. File post-incident review"
log_info "3. Fix issues before next deployment attempt"
log_info "========================================="
