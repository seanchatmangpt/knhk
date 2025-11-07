# v1.0 Definition of Done Quick Reference

**One-page checklist for v1.0 DoD compliance**

## Quick Validation

```bash
# Run full DoD validation
./scripts/validate-v1.0-dod.sh

# Generate status report
./scripts/generate-dod-report.sh

# View status
cat docs/V1_DOD_STATUS.md
```

## P0 Criteria Checklist

### Code Quality
- [ ] No `unwrap()`/`expect()` in critical paths (hot path, fiber, beat scheduler)
- [ ] Proper error handling (`Result<T, E>`)
- [ ] Input validation implemented
- [ ] Guard constraints enforced

### Compilation
- [ ] All Rust crates compile: `cargo build --workspace`
- [ ] C library compiles: `make -C c libknhk.a`

### Testing
- [ ] All tests pass: `cargo test --workspace`
- [ ] Branchless tests pass: `./tests/chicago_branchless_test`
- [ ] Test coverage ≥90% (critical paths)

### Performance
- [ ] Hot path ≤8 ticks (verified via branchless tests)

### Integration
- [ ] C↔Rust FFI verified
- [ ] Beat scheduler integrated
- [ ] Lockchain integrated

## Common Issues & Fixes

### Issue: unwrap() found in production code
**Fix**: Check if in critical path (hot path, fiber, beat scheduler)
- If critical: Replace with proper error handling
- If non-critical: Document exception in `V1_DOD_EXCEPTIONS.md`

### Issue: TODOs found
**Fix**: Review and document as v1.1 if acceptable
- Update TODO comment: `// TODO(v1.1): Description`

### Issue: Tests failing
**Fix**: Run individual test suites to identify failures
```bash
cargo test --manifest-path rust/knhk-etl/Cargo.toml
make -C c test
```

## Validation Commands

```bash
# Full validation
./scripts/validate-v1.0-dod.sh

# Individual checks
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check --all
make -C c test
./tests/chicago_branchless_test
```

## Status Files

- `docs/V1_DOD_STATUS.md` - Live status dashboard
- `docs/V1_DOD_VALIDATION_REPORT.md` - Detailed validation report
- `docs/V1_DOD_EXCEPTIONS.md` - Documented exceptions

## Related Documents

- [Definition of Done](DEFINITION_OF_DONE.md) - Full DoD criteria
- [v1.0 Requirements](v1-requirements.md) - v1.0 specific requirements

