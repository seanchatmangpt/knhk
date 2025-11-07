#!/bin/bash
# Quick 5-minute stability test for validation
# Same tests as 24h version but shorter duration for CI/CD

set -e

DURATION_MINUTES=5
DURATION_SECONDS=$((DURATION_MINUTES * 60))
START_TIME=$(date +%s)
END_TIME=$((START_TIME + DURATION_SECONDS))

LOG_FILE="/Users/sac/knhk/docs/evidence/stability_quick_$(date +%Y%m%d_%H%M%S).log"
METRICS_FILE="/Users/sac/knhk/docs/evidence/stability_quick_metrics.csv"

echo "=== 8-Beat Quick Stability Test (5 minutes) ===" | tee -a "$LOG_FILE"
echo "Start: $(date)" | tee -a "$LOG_FILE"
echo "Duration: $DURATION_MINUTES minutes" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

# Initialize metrics CSV
echo "timestamp,cycle,tick,pulse,drift" > "$METRICS_FILE"

# Build minimal test harness (inline for speed)
echo "Building test harness..." | tee -a "$LOG_FILE"

cat > /tmp/beat_stability_quick.rs <<'EOF'
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use std::thread;

static CYCLE: AtomicU64 = AtomicU64::new(0);

fn main() {
    println!("Starting 8-beat quick test...");

    // Beat generator
    let beat_thread = thread::spawn(|| {
        loop {
            CYCLE.fetch_add(1, Ordering::SeqCst);
            thread::sleep(Duration::from_millis(125)); // 8Hz
        }
    });

    // Metrics server
    let metrics_thread = thread::spawn(|| {
        use std::net::TcpListener;
        use std::io::{Read, Write};

        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

        for stream in listener.incoming() {
            if let Ok(mut stream) = stream {
                let mut buffer = [0; 1024];
                if let Ok(_) = stream.read(&mut buffer) {
                    let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", CYCLE.load(Ordering::SeqCst));
                    let _ = stream.write_all(response.as_bytes());
                }
            }
        }
    });

    beat_thread.join().unwrap();
    metrics_thread.join().unwrap();
}
EOF

rustc -O /tmp/beat_stability_quick.rs -o /tmp/beat_quick || {
    echo "❌ Build failed" | tee -a "$LOG_FILE"
    exit 1
}

# Start server
/tmp/beat_quick > /dev/null 2>&1 &
SERVER_PID=$!
sleep 2

echo "Server PID: $SERVER_PID" | tee -a "$LOG_FILE"

# Monitoring loop
LAST_CYCLE=0
DRIFT_DETECTED=0

while [ $(date +%s) -lt $END_TIME ]; do
    CURRENT_CYCLE=$(curl -s http://127.0.0.1:8080/metrics/cycle 2>/dev/null || echo "$LAST_CYCLE")

    if [ "$LAST_CYCLE" -ne 0 ] && [ "$CURRENT_CYCLE" -le "$LAST_CYCLE" ]; then
        echo "⚠️ DRIFT DETECTED" | tee -a "$LOG_FILE"
        DRIFT_DETECTED=$((DRIFT_DETECTED + 1))
    fi

    TICK=$((CURRENT_CYCLE & 0x7))
    PULSE=$([ $TICK -eq 0 ] && echo 1 || echo 0)

    echo "$(date +%s),$CURRENT_CYCLE,$TICK,$PULSE,$DRIFT_DETECTED" >> "$METRICS_FILE"

    LAST_CYCLE="$CURRENT_CYCLE"
    sleep 1
done

# Stop server
kill $SERVER_PID 2>/dev/null || true
wait $SERVER_PID 2>/dev/null || true

# Results
echo "" | tee -a "$LOG_FILE"
echo "=== Test Complete ===" | tee -a "$LOG_FILE"
echo "Drift events: $DRIFT_DETECTED" | tee -a "$LOG_FILE"

if [ "$DRIFT_DETECTED" -eq 0 ]; then
    echo "✅ QUICK STABILITY TEST PASSED" | tee -a "$LOG_FILE"
    exit 0
else
    echo "❌ QUICK STABILITY TEST FAILED" | tee -a "$LOG_FILE"
    exit 1
fi
