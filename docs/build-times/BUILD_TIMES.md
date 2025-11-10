# KNHK Build Time Documentation

**Last Updated**: 2025-11-10 05:43:34 UTC  
**System**: Darwin (16 cores)  
**Rust**: rustc 1.90.0 (1159e78c4 2025-09-14)  
**Cargo**: cargo 1.90.0 (840b83a10 2025-07-30)

---

## Summary

| Metric | Time | Notes |
|--------|------|-------|
| **Total Debug Build** | 952.8s (15.8m) | Clean build, all crates |
| **Total Release Build** | 1729.3s (28.8m) | Clean build, all crates |
| **Total Test Time** | 655.8s (10.9m) | Unit tests only |
| **Total Check Time** | 799.1s (13.3m) | Type checking only |

**Note**: These are clean build times. Incremental builds are typically 5-10x faster.

---

## Per-Crate Build Times

### Debug Build Times

| Crate | LOC | Debug Build | Release Build | Test | Check | Efficiency (LOC/s) |
|-------|-----|-------------|---------------|------|-------|---------------------|
