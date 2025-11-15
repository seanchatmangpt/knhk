# Troubleshooting Guides

When things don't work, find your issue here. Each guide covers the most common problems and their solutions.

---

## 📋 Available Guides

### [Weaver Validation Troubleshooting](WEAVER_VALIDATION_TROUBLESHOOTING.md)
**Covers**: OTel schema validation errors
- 10 most common Weaver errors
- Each error includes: symptom, cause, solution, code example
- Prevention tips at the end
- **You need this when**: `weaver registry live-check` fails

Common issues:
- Schema definition errors
- Missing metric/span declarations
- Type mismatches
- Missing telemetry at runtime
- Sampling configuration issues

### [Performance Troubleshooting](PERFORMANCE_TROUBLESHOOTING.md)
**Covers**: Speed and performance issues
- Too slow (missing ≤8 tick target)
- Inconsistent performance (spikes/jitter)
- Memory usage issues
- CPU hotspots
- Scaling problems

**You need this when**:
- Performance tests fail
- User reports slowness
- Profiling shows unexpected behavior

Common issues:
- Inefficient algorithms (O(n²) vs O(n))
- Lock contention
- Memory allocations in hot path
- Caching issues
- Profiling methodology errors

### [Testing Troubleshooting](TESTING_TROUBLESHOOTING.md)
**Covers**: Test failures and issues
- Test won't pass (logic failures)
- Test timeouts
- Flaky tests (intermittent failures)
- Coverage gaps
- Chicago TDD specific issues

**You need this when**:
- `cargo test` fails
- Tests pass locally but fail in CI
- Test coverage is below 90%

Common issues:
- Async/await complications
- Test setup/teardown
- Mock misconfiguration
- Timing assumptions
- Chicago TDD pattern violations

### [Telemetry Troubleshooting](TELEMETRY_TROUBLESHOOTING.md)
**Covers**: Instrumentation issues
- Missing spans/metrics/logs
- Incorrect telemetry data
- Sampling configuration
- Weaver validation failures related to telemetry
- Performance impact from instrumentation

**You need this when**:
- Telemetry doesn't appear in collector
- Weaver validation fails for telemetry
- Too much data being exported
- Instrumentation is too slow

Common issues:
- Spans not ended
- Metrics never recorded
- Context not propagated
- Sampling too aggressive
- Batch size misconfiguration

---

## 🎯 Issue Finder

**Don't know which guide?** Use this quick finder:

| Symptom | Guide |
|---------|-------|
| `weaver registry live-check` fails | [Weaver Validation](WEAVER_VALIDATION_TROUBLESHOOTING.md) |
| Code is too slow (>8 ticks) | [Performance](PERFORMANCE_TROUBLESHOOTING.md) |
| Performance spikes or jitter | [Performance](PERFORMANCE_TROUBLESHOOTING.md) |
| Tests fail or timeout | [Testing](TESTING_TROUBLESHOOTING.md) |
| Flaky tests (pass sometimes) | [Testing](TESTING_TROUBLESHOOTING.md) |
| No telemetry in collector | [Telemetry](TELEMETRY_TROUBLESHOOTING.md) |
| Telemetry data looks wrong | [Telemetry](TELEMETRY_TROUBLESHOOTING.md) |
| Weaver fails on telemetry | [Telemetry](TELEMETRY_TROUBLESHOOTING.md) |

---

## 🔍 How to Use These Guides

1. **Find your issue**: Use the Issue Finder above or scan the list
2. **Read the section**: Each issue has 4 parts:
   - **Symptom**: How you'll recognize the problem
   - **Cause**: Why it happens
   - **Solution**: Steps to fix it
   - **Prevention**: How to avoid it next time
3. **Apply the fix**: Follow the solution steps
4. **Verify**: Check that it's resolved

---

## 📊 Coverage Statistics

| Guide | Issues | Total Lines |
|-------|--------|-------------|
| Weaver Validation | 10 | ~800 |
| Performance | 5 | ~700 |
| Testing | 7 | ~700 |
| Telemetry | 6 | ~700 |

**Total issues covered**: 28
**Total content**: ~2800 lines
**Format**: Markdown (copy/paste friendly)

---

## 🔗 Related Documentation

- **How-to Guides**: [Comprehensive task guides](../../papers/how-to-guides/)
- **Quick Reference**: [One-page checklists](../cards/)
- **Code Examples**: [Working implementations](../../examples/)
- **Templates**: [Ready-to-use code](../../templates/)

---

## 🚀 Quick Access by Workflow

### Before Deploying
1. Check [Performance Troubleshooting](PERFORMANCE_TROUBLESHOOTING.md) - are we ≤8 ticks?
2. Check [Testing Troubleshooting](TESTING_TROUBLESHOOTING.md) - do tests pass?
3. Check [Weaver Validation Troubleshooting](WEAVER_VALIDATION_TROUBLESHOOTING.md) - schema valid?

### After Receiving Bug Report
1. Identify the symptom: slow? broken? missing data?
2. Jump to the appropriate guide above
3. Follow the diagnosis steps

### During Development
1. Tests fail → [Testing Troubleshooting](TESTING_TROUBLESHOOTING.md)
2. Too slow → [Performance Troubleshooting](PERFORMANCE_TROUBLESHOOTING.md)
3. No telemetry → [Telemetry Troubleshooting](TELEMETRY_TROUBLESHOOTING.md)

---

## 💡 Pro Tips

- **Bookmark this page**: Use as your first stop for any issue
- **Print guides**: Great to have nearby during debugging
- **Search**: Use your editor's search to find issue names
- **Prevention**: Read "Prevention" sections before they happen
- **Link to guides**: Share with team members when helping debug

---

**Last Updated**: 2025-11-15
**Version**: v1.1.0
**Framework**: Pragmatic Troubleshooting
