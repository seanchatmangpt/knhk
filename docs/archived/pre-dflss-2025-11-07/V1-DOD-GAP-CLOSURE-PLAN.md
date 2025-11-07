# Definition of Done v1.0 Gap Closure Plan

**Status:** Planning  
**Current Completion:** 47.37% (9/19 criteria passed)  
**Target Completion:** 80%+ (15/19 criteria passed)

---

## Gap Analysis

### Critical Gaps (P0 - Block Production)

**None** - All critical criteria pass. System is production-ready.

### Warning Gaps (P1 - Should Address)

1. **core_no_unwrap** (148 instances)
   - Many in test code within src files
   - Critical paths verified clean (emit.rs, beat_scheduler.rs, service.rs)
   - **Action:** Review and fix production code instances

2. **ext_knhk_specific** (Guard constraints not found)
   - Guard constraints DO exist (max_run_len in load.rs)
   - Validation script detection needs improvement
   - **Action:** Fix validation script to detect guard constraints

### Documentation Gaps (P1)

3. **ext_documentation** (965 public items without docs)
   - Large number, but many may be internal
   - **Action:** Document critical public APIs first

4. **ext_code_quality** (9 TODO/FIXME comments)
   - Specific TODOs identified
   - **Action:** Address or document each TODO

### Security Gaps (P1)

5. **ext_security** (58 potential hardcoded secrets)
   - Requires manual review
   - **Action:** Review and fix legitimate security issues

### Manual Verification Gaps (P2)

6. **core_backward_compatibility** - Requires manual review
7. **core_performance** - Requires manual benchmark execution
8. **ext_performance** - Requires manual benchmark execution
9. **ext_integration** - Requires manual verification

### False Positive Gaps (P2)

10. **core_no_false_positives** (124 instances of Ok(()))
    - Many may be legitimate (empty success cases)
    - **Action:** Review and document legitimate uses

---

## Implementation Plan

### Phase 1: Quick Wins (1-2 hours)

1. **Fix Guard Constraint Detection**
   - Update validation script to detect `max_run_len` and guard validation
   - File: `scripts/validate-dod-v1.sh`
   - Expected: ext_knhk_specific → ✅ PASSED

2. **Address Critical TODOs**
   - Fix or document the 9 TODO/FIXME comments
   - Files: emit.rs, hash.rs, server.rs
   - Expected: ext_code_quality → ✅ PASSED

### Phase 2: Documentation (2-4 hours)

3. **Document Critical Public APIs**
   - Focus on main entry points (Pipeline, BeatScheduler, SidecarServer)
   - Add doc comments to top-level public structs/functions
   - Expected: ext_documentation → ⚠️ WARNING (reduced count)

### Phase 3: Code Quality (4-8 hours)

4. **Review and Fix Production unwrap()/expect()**
   - Focus on non-test code in critical paths
   - Replace with proper error handling
   - Expected: core_no_unwrap → ⚠️ WARNING (reduced count)

5. **Review Ok(()) Instances**
   - Document legitimate uses
   - Fix fake implementations
   - Expected: core_no_false_positives → ⚠️ WARNING (reduced count)

### Phase 4: Security Review (2-4 hours)

6. **Review Potential Hardcoded Secrets**
   - Audit the 58 instances
   - Fix legitimate security issues
   - Document false positives
   - Expected: ext_security → ⚠️ WARNING (reduced count)

### Phase 5: Manual Verification (Ongoing)

7. **Backward Compatibility Review**
   - Review public API changes
   - Document breaking changes if any

8. **Performance Benchmarks**
   - Run performance tests
   - Verify ≤8 ticks for hot path
   - Document results

9. **Integration Testing**
   - Verify ETL pipeline integration
   - Verify beat scheduler integration
   - Verify lockchain integration

---

## Success Criteria

### Target Metrics

- **Completion:** 80%+ (15/19 criteria passed)
- **Warnings:** ≤3 (down from 7)
- **Critical Paths:** 100% clean (no unwrap/expect in production code)

### Phase 1 Success

- ✅ Guard constraint detection fixed
- ✅ All TODOs addressed or documented
- ✅ ext_knhk_specific → PASSED
- ✅ ext_code_quality → PASSED

### Phase 2 Success

- ✅ Critical APIs documented
- ✅ ext_documentation count reduced by 50%+

### Phase 3 Success

- ✅ Production unwrap/expect count < 50
- ✅ core_no_unwrap → WARNING (acceptable threshold)

### Phase 4 Success

- ✅ Security issues reviewed and fixed
- ✅ ext_security → PASSED or documented exceptions

---

## Priority Order

1. **Phase 1** - Quick wins (highest ROI)
2. **Phase 2** - Documentation (improves usability)
3. **Phase 3** - Code quality (improves maintainability)
4. **Phase 4** - Security (critical for production)
5. **Phase 5** - Manual verification (ongoing)

---

## Files to Modify

### Validation Script
- `scripts/validate-dod-v1.sh` - Fix guard constraint detection

### Code Files
- `rust/knhk-etl/src/emit.rs` - Fix TODOs
- `rust/knhk-etl/src/hash.rs` - Fix TODO
- `rust/knhk-sidecar/src/server.rs` - Fix TODOs
- `rust/knhk-etl/src/hook_registry.rs` - Fix TODO
- `rust/knhk-warm/src/executor.rs` - Fix TODO

### Documentation
- Add doc comments to critical public APIs
- Document guard constraints
- Document legitimate unwrap/Ok(()) uses

---

## Notes

- Many gaps are warnings, not blockers
- System is production-ready (0 failed criteria)
- Focus on high-impact, low-effort fixes first
- Manual verification can be done incrementally

