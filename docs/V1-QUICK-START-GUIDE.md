# KNHK v1.0 Release - Quick Start Guide

**For:** Release Engineers & System Architects
**Purpose:** Fast-track to v1.0 release validation
**Last Updated:** 2025-11-16

---

## TL;DR: 3-Step Path to v1.0

```bash
# 1. Install Weaver (SOURCE OF TRUTH validator)
bash scripts/install-weaver.sh

# 2. Run complete validation sequence
bash scripts/run-full-validation.sh

# 3. Review results
cat validation-evidence-*.md
```

**Expected Outcome:** All 6 phases pass, v1.0 ready to release.

---

## Current Blockers (Must Fix First)

| Blocker | Count | Priority | Estimated Fix Time |
|---------|-------|----------|-------------------|
| Compilation errors | Unknown | P0 | 1-2 hours |
| `panic!()` in production | 70+ | P0 | 8-16 hours |
| Weaver not installed | 1 | P0 | 30 minutes |

**Critical Path:** Fix compilation → Remediate panic!() → Run validation

---

## The Validation Hierarchy (Non-Negotiable)

```
┌─────────────────────────────────────────────────────┐
│ Level 1: Weaver Schema Validation                  │
│ ⚡ SOURCE OF TRUTH ⚡                               │
│ If this fails, nothing else matters.                │
└─────────────────────────────────────────────────────┘
                        ↑
┌─────────────────────────────────────────────────────┐
│ Level 2: Compilation & Code Quality                │
│ Baseline: Must compile with zero warnings          │
└─────────────────────────────────────────────────────┘
                        ↑
┌─────────────────────────────────────────────────────┐
│ Level 3: Traditional Tests                         │
│ Supporting Evidence: Can have false positives      │
└─────────────────────────────────────────────────────┘
```

**Key Principle:** Weaver validates actual runtime behavior, not test logic.

---

## The 6 Validation Phases

### Phase 0: Pre-Build Validation (30s)
- ✅ Syntax checks
- ✅ Weaver installed
- ✅ Code formatted

### Phase 1: Build & Code Quality (2-5m)
- ✅ C library builds
- ✅ Rust workspace builds (debug + release)
- ✅ Clippy zero warnings

### Phase 2: Unit Tests (1-3m)
- ✅ Rust unit tests 100% pass
- ✅ C tests 100% pass

### Phase 3: Integration Tests (2-5m)
- ✅ Chicago TDD tests pass
- ✅ C + Rust integration pass

### Phase 4: Performance Tests (1-2m)
- ✅ Hot path ≤8 ticks validated
- ✅ No regressions

### Phase 5: Weaver Validation (30-60s) ⚡ SOURCE OF TRUTH
- ✅ Static schema validation
- ✅ Live telemetry validation
- ✅ All 8 schemas valid

**Total Time:** 7-16 minutes

---

## Quick Commands Reference

### Validation Commands

```bash
# Full validation (all 6 phases)
bash scripts/run-full-validation.sh

# Individual phases
cargo check --workspace                    # Phase 0
make build                                 # Phase 1
cargo test --workspace --lib              # Phase 2
make test-chicago && make test-integration # Phase 3
make test-performance                      # Phase 4
weaver registry check -r registry/        # Phase 5 (SOURCE OF TRUTH)

# Checklist validation (detailed evidence)
bash scripts/validate-checklist.sh
```

### Diagnostic Commands

```bash
# Count panic!() calls in production code
grep -r "panic!" rust/*/src --include="*.rs" | \
    grep -v "test" | grep -v "/tests/" | wc -l

# Check compilation status
cargo build --workspace 2>&1 | grep -i "error"

# Verify Weaver installation
weaver --version

# List registry schemas
ls -la registry/*.yaml
```

### Quick Fixes

```bash
# Format all code
cargo fmt --all

# Fix obvious clippy issues
cargo clippy --workspace --fix

# Clean and rebuild
make clean && make build
```

---

## v1.0 Release Criteria (All Must Pass)

| Criterion | Command | Pass Criteria |
|-----------|---------|--------------|
| **Compilation** | `make build` | Exit 0, zero warnings |
| **Zero panic!()** | `grep -r "panic!" ...` | 0 matches |
| **Clippy** | `cargo clippy -- -D warnings` | Exit 0 |
| **All Tests** | `make test-all` | 100% pass rate |
| **Performance** | `make test-performance` | ≤8 ticks validated |
| **Weaver** ⚡ | `weaver registry check` | All schemas valid |

**If ANY criterion fails → NO v1.0 RELEASE**

---

## panic!() Remediation Pattern

**Problem:** 70+ panic!() calls in production code

**Solution:** Replace with proper error handling

### Before (BAD):
```rust
match BeatScheduler::new() {
    Ok(s) => s,
    Err(e) => panic!("Failed: {:?}", e),  // ❌ WRONG
}
```

### After (GOOD):
```rust
BeatScheduler::new()
    .context("Failed to create beat scheduler")?  // ✅ CORRECT
```

**Estimated Time:** 8-16 hours for all 70+ instances

---

## Installation: OpenTelemetry Weaver

### Quick Install (Recommended):

```bash
bash scripts/install-weaver.sh
```

### Manual Install (If script fails):

**Method 1: Binary Download**
```bash
VERSION="0.10.0"
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
URL="https://github.com/open-telemetry/weaver/releases/download/v${VERSION}/weaver-${OS}-${ARCH}.tar.gz"

wget "$URL" -O /tmp/weaver.tar.gz
sudo tar -xzf /tmp/weaver.tar.gz -C /usr/local/bin
sudo chmod +x /usr/local/bin/weaver
weaver --version
```

**Method 2: Cargo**
```bash
cargo install weaver_forge weaver_checker
weaver --version
```

**Method 3: Docker**
```bash
docker pull ghcr.io/open-telemetry/weaver:latest
alias weaver='docker run --rm -v $(pwd):/workspace -w /workspace ghcr.io/open-telemetry/weaver:latest'
```

---

## Timeline to v1.0-Ready

### Optimistic Timeline: 3 Days

**Day 1:** Setup + Compilation (8 hours)
- Install Weaver (30min)
- Fix compilation (2-3 hours)
- Define error types (2 hours)
- Start panic!() fixes (2 hours)

**Day 2:** panic!() Remediation (8 hours)
- Replace safety-critical panics (4 hours)
- Replace validation panics (4 hours)

**Day 3:** Testing + Validation (8 hours)
- Complete panic!() fixes (2 hours)
- Run full test suite (2 hours)
- Weaver validation (2 hours)
- Documentation + release prep (2 hours)

### Realistic Timeline: 5 Days

Add buffer for:
- Unexpected test failures
- Weaver schema issues
- Performance regressions
- Documentation updates

---

## Success Metrics

**v1.0 is ready when:**

✅ All 6 validation phases pass
✅ Weaver validation succeeds (SOURCE OF TRUTH)
✅ Zero panic!() in production code
✅ Evidence collected and documented
✅ Team confident in release

**Not before.**

---

## Help & References

### Documentation
- **Complete Strategy:** [V1-RELEASE-VALIDATION-STRATEGY.md](./V1-RELEASE-VALIDATION-STRATEGY.md)
- **Project Rules:** [../CLAUDE.md](../CLAUDE.md)

### Support
- Weaver Docs: https://github.com/open-telemetry/weaver
- OTel Docs: https://opentelemetry.io/docs/
- KNHK Issues: https://github.com/seanchatmangpt/knhk/issues

### Scripts
- `scripts/install-weaver.sh` - Install Weaver
- `scripts/run-full-validation.sh` - Run all validation phases
- `scripts/validate-checklist.sh` - Detailed checklist validation
- `scripts/validate_v1.0.sh` - Legacy v1.0 validation

---

## Troubleshooting

### "Compilation failed"
```bash
# Check errors
cargo build --workspace 2>&1 | grep "error"

# Clean and rebuild
make clean && make build

# Check dependencies
cd rust && cargo tree
```

### "Weaver not found"
```bash
# Check installation
which weaver
weaver --version

# Reinstall
bash scripts/install-weaver.sh

# Use Docker fallback
alias weaver='docker run --rm -v $(pwd):/workspace -w /workspace ghcr.io/open-telemetry/weaver:latest'
```

### "Tests failing"
```bash
# Run specific test
cargo test --package knhk-etl --test integration_test -- --nocapture

# Check for flaky tests
cargo test --workspace --lib -- --test-threads=1

# Review test output
cargo test --workspace 2>&1 | less
```

### "Weaver validation fails"
```bash
# Check schema syntax
weaver registry check -r registry/

# Validate individual schema
weaver registry check -r registry/ --schema knhk-etl.yaml

# Review schema files
cat registry/knhk-etl.yaml
```

---

## Next Steps After v1.0

1. **Tag Release:**
   ```bash
   git tag -a v1.0.0 -m "Release v1.0.0"
   git push origin v1.0.0
   ```

2. **GitHub Release:**
   - Draft release notes
   - Attach binaries
   - Publish release

3. **Post-Release:**
   - Monitor for issues
   - Plan v1.1 features
   - Update documentation

---

**Remember:** Weaver validation is the SOURCE OF TRUTH. If Weaver fails, the feature doesn't work, regardless of test results.

**Good luck! 🚀**
