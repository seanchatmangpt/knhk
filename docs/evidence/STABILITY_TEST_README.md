# 24-Hour Stability Test Suite

## Overview

Validates 8-beat system stability under continuous load for 24 hours.

## Test Coverage

### Beat Stability
- **Zero drift**: Cycle counter must increment monotonically
- **No stalls**: Cycle advances at consistent rate (8Hz = 125ms per beat)
- **Deterministic**: Tick calculation (cycle & 0x7) always produces 0-7

### Receipt Continuity
- **Gap detection**: No missing receipt IDs in sequence
- **Lockchain roots**: Merkle roots committed every 8 beats (pulse boundary)
- **Deterministic replay**: μ(O) reconstruction from receipts

### Performance Compliance
- **Park rate**: ≤20% sustained (R1 compliance)
- **Tick budget**: ≤8 ticks per operation (Chatman Constant)
- **Memory stability**: No leaks or unbounded growth

## Test Scripts

### 1. Quick Test (5 minutes)
```bash
cd /Users/sac/knhk
chmod +x tests/stability_quick.sh
./tests/stability_quick.sh
```

**Use for:**
- CI/CD validation
- Rapid iteration during development
- Pre-commit checks

### 2. Full Test (24 hours)
```bash
cd /Users/sac/knhk
chmod +x tests/stability_24h.sh

# Run in background with nohup
nohup ./tests/stability_24h.sh &

# Monitor progress
tail -f docs/evidence/stability_24h_*.log

# Check PID
ps aux | grep stability_24h
```

**Use for:**
- Production validation
- Release certification
- Long-term stability proof

## Monitoring

### Real-time Progress
```bash
# Watch log file
tail -f docs/evidence/stability_24h_*.log

# Watch metrics (live update)
watch -n 5 'tail -10 docs/evidence/stability_24h_metrics.csv'

# Check drift count
grep "DRIFT" docs/evidence/stability_24h_*.log | wc -l
```

### Metrics Collected

CSV columns: `timestamp,cycle,tick,pulse,deltas_processed,actions_emitted,receipts,park_rate,drift`

- **timestamp**: Unix epoch seconds
- **cycle**: Current cycle counter
- **tick**: Current tick (0-7)
- **pulse**: 1 when tick==0, else 0
- **deltas_processed**: Total deltas ingested
- **actions_emitted**: Total actions executed
- **receipts**: Total receipts written
- **park_rate**: Percentage of operations parked (target: ≤20%)
- **drift**: Cumulative drift events detected

## Analysis

### Generate Report
```bash
# After test completion
python3 - <<'EOF'
import pandas as pd
import sys

# Load metrics
df = pd.read_csv('docs/evidence/stability_24h_metrics.csv')

# Analysis
print("=== 24-Hour Stability Analysis ===\n")
print(f"Total samples: {len(df)}")
print(f"Duration: {(df['timestamp'].max() - df['timestamp'].min()) / 3600:.1f} hours")
print(f"Cycles processed: {df['cycle'].max()}")
print(f"Drift events: {df['drift'].max()}")
print(f"Avg park rate: {df['park_rate'].mean():.2f}%")
print(f"Max park rate: {df['park_rate'].max()}%")
print(f"R1 compliance: {(df['park_rate'] <= 20).mean() * 100:.1f}%")

# Drift analysis
if df['drift'].max() > 0:
    drift_samples = df[df['drift'] > df['drift'].shift(1)]
    print(f"\nDrift detected at cycles: {drift_samples['cycle'].tolist()}")

# Park rate compliance
park_violations = df[df['park_rate'] > 20]
if len(park_violations) > 0:
    print(f"\nPark rate violations: {len(park_violations)} samples")
    print(f"Max violation: {park_violations['park_rate'].max()}% at cycle {park_violations['cycle'].iloc[0]}")
EOF
```

### Success Criteria

**✅ PASS if ALL true:**
- Drift events: 0
- Park rate max: ≤20%
- Park rate avg: ≤15%
- Memory stable (no leaks)
- No server crashes

**❌ FAIL if ANY true:**
- Drift events: >0
- Park rate max: >20%
- Server crashed
- Memory leak detected

## Troubleshooting

### Server won't start
```bash
# Check port availability
lsof -i :8080

# Kill existing server
pkill -f beat_server

# Check logs
cat docs/evidence/stability_*.log | grep "❌"
```

### High park rate
```bash
# Check which operations are parking
grep "park" docs/evidence/stability_24h_*.log

# Reduce load or increase tick budget
# Edit fiber.rs: increase tick_budget from 8
```

### Drift detected
```bash
# Find drift patterns
grep "DRIFT" docs/evidence/stability_24h_*.log

# Check for race conditions in beat scheduler
# Verify atomic operations in c/src/beat.c
```

## Integration with CI/CD

### GitHub Actions Example
```yaml
name: Stability Test

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly on Sunday

jobs:
  stability:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run 24h stability test
        run: |
          ./tests/stability_24h.sh
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: stability-results
          path: docs/evidence/stability_*
```

## Law Validation

This test validates:

1. **Beat stability**: No drift across 24h → cycle counter deterministic
2. **Receipt continuity**: 100% coverage → deterministic replay possible
3. **Performance**: Park rate ≤20% → R1 compliance (80% hot path)

**Weaver Validation**: After test completion:
```bash
# Verify telemetry schema
weaver registry check -r registry/

# Validate runtime telemetry
weaver registry live-check --registry registry/
```

## Evidence Storage

All test artifacts saved to `/Users/sac/knhk/docs/evidence/`:
- `stability_24h_YYYYMMDD_HHMMSS.log` - Full test log
- `stability_24h_metrics.csv` - Time-series metrics
- `stability_24h_report.md` - Analysis report (generated after test)

## Next Steps

After successful 24h test:
1. Generate final report: `./tests/generate_stability_report.sh`
2. Archive evidence: `tar czf stability_evidence.tar.gz docs/evidence/stability_24h_*`
3. Update release notes with stability proof
4. Run Weaver validation: `weaver registry live-check`
