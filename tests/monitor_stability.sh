#!/bin/bash
# Real-time stability monitoring script
# Usage: ./tests/monitor_stability.sh

set -e

INTERVAL=300  # 5 minutes
LOG_FILE="/Users/sac/knhk/docs/evidence/stability_monitor_$(date +%Y%m%d_%H%M%S).log"

echo "=== KNHK Stability Monitor ===" | tee -a "$LOG_FILE"
echo "Started: $(date)" | tee -a "$LOG_FILE"
echo "Check interval: ${INTERVAL}s" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

LAST_CYCLE=0

while true; do
    TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')

    # Check server is alive
    if ! ps aux | grep -q "[b]eat_server"; then
        echo "[$TIMESTAMP] ❌ SERVER CRASHED!" | tee -a "$LOG_FILE"
        exit 1
    fi

    # Query metrics with timeout
    CYCLE=$(timeout 5 curl -s http://127.0.0.1:8080/metrics/cycle 2>/dev/null || echo "TIMEOUT")

    if [ "$CYCLE" = "TIMEOUT" ]; then
        echo "[$TIMESTAMP] ⚠️  Server not responding (timeout)" | tee -a "$LOG_FILE"
        sleep 10
        continue
    fi

    PARK_RATE=$(curl -s http://127.0.0.1:8080/metrics/park_rate 2>/dev/null || echo 0)
    DELTAS=$(curl -s http://127.0.0.1:8080/metrics/deltas_processed 2>/dev/null || echo 0)

    # Calculate beat rate
    if [ "$LAST_CYCLE" -ne 0 ]; then
        DELTA_CYCLES=$((CYCLE - LAST_CYCLE))
        BEAT_RATE=$(echo "scale=2; $DELTA_CYCLES / $INTERVAL" | bc)
    else
        BEAT_RATE="N/A"
    fi

    # Status message
    STATUS="✅ OK"

    # Check for violations
    if [ "$PARK_RATE" -gt 20 ]; then
        STATUS="⚠️  PARK RATE VIOLATION"
        echo "[$TIMESTAMP] $STATUS: Park rate $PARK_RATE% exceeds 20%" | tee -a "$LOG_FILE"
    fi

    if [ "$LAST_CYCLE" -ne 0 ] && [ "$CYCLE" -le "$LAST_CYCLE" ]; then
        STATUS="❌ DRIFT DETECTED"
        echo "[$TIMESTAMP] $STATUS: Cycle $CYCLE ≤ $LAST_CYCLE" | tee -a "$LOG_FILE"
    fi

    # Log status
    echo "[$TIMESTAMP] $STATUS | Cycle: $CYCLE | Beat Rate: ${BEAT_RATE} Hz | Park: $PARK_RATE% | Deltas: $DELTAS" | tee -a "$LOG_FILE"

    LAST_CYCLE="$CYCLE"

    sleep "$INTERVAL"
done
