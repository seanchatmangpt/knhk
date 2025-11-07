# 24-Hour Stability Test - Quick Start Guide

**TL;DR:** Run `./tests/stability_24h.sh` to validate 24h stability. Check report with `./tests/generate_stability_report.sh`.

---

## Prerequisites

✅ **System built:**
```bash
cd /Users/sac/knhk
cargo build --release --package knhk-etl
make build
```

✅ **Dependencies:**
- Rust toolchain
- curl (for metrics queries)
- Python 3 with pandas (for report generation)

---

## Quick Test (5 Minutes)

**Use for:** Pre-commit checks, rapid validation, CI/CD

```bash
cd /Users/sac/knhk
./tests/stability_quick.sh
```

**Expected output:**
```
=== 8-Beat Quick Stability Test (5 minutes) ===
Start: Thu Nov 06 17:53:37 PST 2025
Duration: 5 minutes

Building test harness...
Server PID: 36726

[Wait 5 minutes...]

=== Test Complete ===
Drift events: 0
✅ QUICK STABILITY TEST PASSED
```

**Check results:**
```bash
cat docs/evidence/stability_quick_*.log
tail docs/evidence/stability_quick_metrics.csv
```

---

## Full Test (24 Hours)

**Use for:** Production certification, release validation

### 1. Start Test

```bash
cd /Users/sac/knhk

# Run in background with nohup
nohup ./tests/stability_24h.sh &

# Save process ID
echo $! > /tmp/stability_test.pid

# Confirm started
tail docs/evidence/stability_24h_*.log
```

**Expected start output:**
```
=== 8-Beat 24-Hour Stability Test ===
Start: Thu Nov 06 18:00:00 PST 2025
Duration: 24 hours
Test PID: 37001

Building knhk-etl test binary...
Server PID: 37010
✅ Server started successfully

[0 hours] Cycle: 0, Drift: 0, Park: 0%
```

### 2. Monitor Progress

**Live log:**
```bash
tail -f docs/evidence/stability_24h_*.log
```

**Hourly summary:**
```bash
grep "hours]" docs/evidence/stability_24h_*.log | tail -5
```

**Current metrics:**
```bash
tail -10 docs/evidence/stability_24h_metrics.csv
```

**Check if still running:**
```bash
ps -p $(cat /tmp/stability_test.pid) && echo "Running" || echo "Completed"
```

### 3. Wait for Completion

**Time remaining:**
```bash
python3 - <<EOF
import time
import pandas as pd

df = pd.read_csv('docs/evidence/stability_24h_metrics.csv')
start = df['timestamp'].min()
elapsed = time.time() - start
remaining = (24 * 3600) - elapsed

print(f"Elapsed: {elapsed/3600:.1f} hours")
print(f"Remaining: {remaining/3600:.1f} hours")
EOF
```

**Set reminder:**
```bash
# Run this to get notified when done
(sleep $((24*3600)); echo "✅ Stability test complete!" | mail -s "KNHK Test" user@example.com) &
```

### 4. Generate Report

**After test completes:**
```bash
./tests/generate_stability_report.sh
```

**View report:**
```bash
cat docs/evidence/stability_24h_report_*.md
```

**Check verdict:**
```bash
grep "Test verdict:" docs/evidence/stability_24h_report_*.md
```

---

## Understanding Results

### Metrics Explained

**CSV columns:** `timestamp,cycle,tick,pulse,deltas_processed,actions_emitted,receipts,park_rate,drift`

| Metric | Meaning | Target |
|--------|---------|--------|
| **timestamp** | Unix epoch seconds | - |
| **cycle** | Global cycle counter | Monotonically increasing |
| **tick** | Current tick (0-7) | Cycles through 0-7 |
| **pulse** | Pulse flag (0 or 1) | 1 when tick==0 |
| **deltas_processed** | Total deltas ingested | Increasing |
| **actions_emitted** | Total actions executed | Increasing |
| **receipts** | Total receipts written | Increasing |
| **park_rate** | % operations parked | ≤20% |
| **drift** | Cumulative drift events | 0 |

### Success Criteria

**✅ PASS if:**
- Drift events: **0**
- Max park rate: **≤20%**
- Avg park rate: **≤15%**
- Server uptime: **≥23.5 hours**

**❌ FAIL if:**
- Drift events: **>0** → Cycle counter unstable
- Max park rate: **>20%** → R1 violation
- Server crashed → System instability

---

## Troubleshooting

### "Server not responding"

**Symptom:** Test fails with connection error

**Solution:**
```bash
# Check if port is in use
lsof -i :8080

# Kill conflicting process
pkill -f beat_server

# Restart test
./tests/stability_24h.sh
```

### "Build failed"

**Symptom:** Test can't build beat_server

**Solution:**
```bash
# Build manually
cargo build --release --package knhk-etl --example beat_server

# If example doesn't exist, test uses fallback harness
# Check log for "creating minimal test harness"
```

### "High park rate"

**Symptom:** Park rate >20% sustained

**Analysis:**
```bash
# Find peak times
grep "Park:" docs/evidence/stability_24h_*.log | sort -k6 -n -r | head -10

# Check metrics
awk -F, '$8 > 20 {print $0}' docs/evidence/stability_24h_metrics.csv | head
```

**Solutions:**
1. Reduce load (lower ingestion rate)
2. Increase tick budget (edit fiber.rs)
3. Profile hot path (use `perf` or `flamegraph`)

### "Drift detected"

**Symptom:** Cycle counter not monotonic

**Critical:** This is a **LAW VIOLATION** - system is NOT production ready

**Analysis:**
```bash
# Find drift events
grep "DRIFT" docs/evidence/stability_24h_*.log

# Check cycle timeline
awk -F, 'NR>1 {print $2}' docs/evidence/stability_24h_metrics.csv | head -100
```

**Possible causes:**
1. Race condition in beat scheduler
2. Atomic operation failure
3. System clock adjustment (NTP)

**Fix:**
1. Verify atomic operations in `c/src/beat.c`
2. Disable NTP during test: `sudo systemctl stop ntp`
3. Review beat scheduler logic in `rust/knhk-etl/src/beat_scheduler.rs`

---

## CI/CD Integration

### GitHub Actions

**.github/workflows/stability.yml:**
```yaml
name: 24h Stability Test

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday
  workflow_dispatch:      # Manual trigger

jobs:
  stability:
    runs-on: ubuntu-latest
    timeout-minutes: 1500  # 25 hours

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build
        run: |
          cargo build --release --package knhk-etl
          make build

      - name: Run 24h stability test
        run: ./tests/stability_24h.sh

      - name: Generate report
        if: always()
        run: ./tests/generate_stability_report.sh

      - name: Upload results
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: stability-results
          path: docs/evidence/stability_*

      - name: Check verdict
        run: |
          VERDICT=$(grep "Test verdict:" docs/evidence/stability_24h_report_*.md)
          echo "$VERDICT"
          [[ "$VERDICT" == *"✅ PASSED"* ]] || exit 1
```

### Pre-Commit Hook

**.git/hooks/pre-commit:**
```bash
#!/bin/bash
# Run quick stability test before commit

echo "Running 5-minute stability test..."
./tests/stability_quick.sh

if [ $? -eq 0 ]; then
    echo "✅ Stability test passed"
else
    echo "❌ Stability test failed - commit aborted"
    exit 1
fi
```

---

## Advanced Usage

### Custom Duration

**10-minute test:**
```bash
# Edit stability_quick.sh
sed -i 's/DURATION_MINUTES=5/DURATION_MINUTES=10/' tests/stability_quick.sh
./tests/stability_quick.sh
```

**48-hour test:**
```bash
# Edit stability_24h.sh
sed -i 's/DURATION_HOURS=24/DURATION_HOURS=48/' tests/stability_24h.sh
./tests/stability_24h.sh
```

### Custom Metrics Endpoint

**If using different port:**
```bash
# Edit test script
sed -i 's/127.0.0.1:8080/127.0.0.1:9090/' tests/stability_24h.sh
```

### Stress Testing

**High load:**
```bash
# Modify server to inject more deltas
# Or run multiple concurrent tests
for i in {1..4}; do
    PORT=$((8080 + i)) nohup ./tests/stability_24h.sh &
done
```

---

## Post-Test Actions

### If Test Passes ✅

1. **Generate report:**
   ```bash
   ./tests/generate_stability_report.sh
   ```

2. **Run Weaver validation:**
   ```bash
   weaver registry check -r registry/
   weaver registry live-check --registry registry/
   ```

3. **Archive evidence:**
   ```bash
   tar czf stability_evidence_$(date +%Y%m%d).tar.gz docs/evidence/stability_24h_*
   ```

4. **Update release notes:**
   ```bash
   echo "✅ 24-hour stability test passed on $(date)" >> RELEASE_NOTES.md
   echo "- Zero drift events" >> RELEASE_NOTES.md
   echo "- Park rate: [VALUE]% (R1 compliant)" >> RELEASE_NOTES.md
   ```

5. **Approve for production:**
   - Add evidence to release PR
   - Tag release: `git tag -a v1.0.0 -m "Stability validated"`
   - Deploy to production

### If Test Fails ❌

1. **Analyze failure:**
   ```bash
   grep "❌" docs/evidence/stability_24h_*.log
   grep "FAILED" docs/evidence/stability_24h_report_*.md
   ```

2. **Fix root cause:**
   - If drift: Fix beat scheduler atomics
   - If park rate: Optimize hot path or increase budget
   - If crash: Debug with core dump

3. **Re-run test:**
   ```bash
   ./tests/stability_24h.sh
   ```

4. **Do NOT proceed to production** until test passes

---

## FAQ

**Q: Can I run the test on my laptop?**
A: Yes, but keep it plugged in and prevent sleep mode.

**Q: How much disk space is needed?**
A: ~10MB for logs and metrics over 24 hours.

**Q: Can I pause the test?**
A: No - the test must run continuously. Pausing invalidates results.

**Q: What if my SSH session disconnects?**
A: Use `nohup` and `&` - test runs in background and survives disconnect.

**Q: How do I stop a running test?**
A: `kill $(cat /tmp/stability_test.pid)` - but this invalidates results.

**Q: Can I run multiple tests in parallel?**
A: Yes, but use different ports to avoid conflicts.

---

## References

- **Full guide:** `/Users/sac/knhk/docs/evidence/STABILITY_TEST_README.md`
- **Test scripts:** `/Users/sac/knhk/tests/stability_*.sh`
- **Report template:** `/Users/sac/knhk/docs/evidence/STABILITY_REPORT_TEMPLATE.md`
- **Summary:** `/Users/sac/knhk/docs/evidence/24H_STABILITY_VALIDATION_SUMMARY.md`

---

**Next Step:** Run `./tests/stability_24h.sh` to validate 24-hour stability.

**Expected Duration:** 24 hours
**Expected Result:** ✅ Zero drift, park rate ≤20%, system stable
