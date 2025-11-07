# KNHK Stability Testing - Quick Reference

## Overview

This directory contains scripts for validating KNHK 8-beat system stability.

## Test Scripts

| Script | Duration | Purpose | Output |
|--------|----------|---------|--------|
| `stability_quick.sh` | 5 min | Fast CI/CD validation | Quick test results |
| `stability_24h.sh` | 24 hours | Production certification | Full stability metrics |
| `monitor_stability.sh` | Continuous | Real-time monitoring | Live status updates |
| `generate_stability_report.sh` | - | Analysis report | Markdown report |

## Quick Start

### 1. Run Quick Test (5 minutes)

```bash
cd /Users/sac/knhk
./tests/stability_quick.sh
```

**Expected output:**
```
✅ QUICK STABILITY TEST PASSED
```

### 2. Start 24-Hour Test

```bash
# Option A: Using tmux (recommended)
tmux new-session -d -s stability './tests/stability_24h.sh'
tmux attach -t stability  # To view

# Option B: Using nohup
nohup ./tests/stability_24h.sh > stability.out 2>&1 &
echo $! > stability.pid
```

### 3. Monitor Test Progress

```bash
# Real-time monitoring (checks every 5 minutes)
./tests/monitor_stability.sh

# Or manually check
tail -f docs/evidence/stability_24h_*.log
```

### 4. Generate Report

```bash
# After test completes (24 hours)
./tests/generate_stability_report.sh
cat docs/evidence/stability_24h_report_*.md
```

## Manual Metrics Check

```bash
# Check if server is running
ps aux | grep beat_server
lsof -i :8080

# Query current metrics
curl http://127.0.0.1:8080/metrics/cycle
curl http://127.0.0.1:8080/metrics/park_rate
curl http://127.0.0.1:8080/metrics/deltas_processed
```

## Success Criteria

✅ **Quick Test:**
- Duration: 5 minutes
- Drift events: 0
- Server uptime: 100%

✅ **24-Hour Test:**
- Cycles: >650,000
- Drift events: 0
- Park rate: ≤20% (≥80% of time)
- Server uptime: 24 hours

## Test Results Location

```
docs/evidence/
├── stability_quick_YYYYMMDD_HHMMSS.log
├── stability_quick_metrics.csv
├── stability_24h_YYYYMMDD_HHMMSS.log
├── stability_24h_metrics.csv
└── stability_24h_report_YYYYMMDD_HHMMSS.md
```

## Troubleshooting

### Test won't start

```bash
# Check port not in use
lsof -i :8080
kill <PID>  # If needed

# Verify rustc available
rustc --version
```

### Server not responding

```bash
# Check server process
ps aux | grep beat_server

# Check server logs
tail -50 docs/evidence/stability_24h_*.log

# Manual restart
killall beat_server
./tests/stability_24h.sh
```

### High park rate

**Expected:** Quick test harness simulates 10% park rate
**Action:** If using real KNHK, investigate receipt processing

## Expected Metrics

| Timepoint | Cycles | Park Rate | Drift |
|-----------|--------|-----------|-------|
| 5 min | ~2,400 | ≤20% | 0 |
| 1 hour | ~29,000 | ≤20% | 0 |
| 6 hours | ~173,000 | ≤20% | 0 |
| 24 hours | ~691,000 | ≤20% | 0 |

## Quick Test Results (Latest)

**Test Date:** 2025-11-06 19:11:26 PST
**Status:** ✅ **PASSED**

- Duration: 5 minutes (300 seconds)
- Cycles: 2,314
- Drift events: 0
- Server: Clean startup and shutdown
- Tick calculation: Correct (branchless)

## 24-Hour Test Status

**Status:** ⏳ READY (not yet executed)

To start the 24-hour production certification test:

```bash
tmux new-session -d -s stability 'cd /Users/sac/knhk && ./tests/stability_24h.sh'
./tests/monitor_stability.sh  # In separate terminal
```

## Production Certification

**Requirements:**
- [ ] Quick test passed (✅ DONE)
- [ ] 24-hour test passed (⏳ PENDING)
- [ ] Zero drift events
- [ ] Park rate ≤20%
- [ ] No crashes or hangs
- [ ] Memory stable

**Deliverable:** `docs/v1-stability-test-report.md` ✅ COMPLETE

---

For detailed information, see: `docs/v1-stability-test-report.md`
